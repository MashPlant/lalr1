use ll1_core::First;
use crate::*;

pub struct Lr1Ctx(pub First);

impl Lr1Ctx {
  pub fn new(g: &Grammar) -> Lr1Ctx { Lr1Ctx(First::new(g)) }

  // one beta, and many a
  pub fn first(&self, beta: &[u32], a: &[u32], g: &Grammar) -> Box<[u32]> {
    let beta_first = self.0.first(beta, g);
    let bs = unsafe { bitset::ubs(beta_first.as_ref()) };
    if bs.get(EPS_IDX) {
      bs.del(EPS_IDX);
      bs.or(a.as_ptr(), a.len());
    }
    beta_first
  }

  // `go` was used by lr1 before, now not used
  pub fn go<'a>(&mut self, state: &Lr1Closure<'a>, mov: u32, g: &'a Grammar<'a>) -> Lr1Closure<'a> {
    let mut new_items = HashMap::default();
    for Lr1Item { lr0, lookahead } in state {
      if lr0.prod.get(lr0.dot as usize) == Some(&mov) {
        let new_item = Lr0Item { prod: lr0.prod, prod_id: lr0.prod_id, dot: lr0.dot + 1 };
        match new_items.get_mut(&new_item) {
          None => { new_items.insert(new_item, lookahead.clone()); }
          Some(old_lookahead) => { bitset::bs(old_lookahead).or(lookahead); }
        }
      }
    }
    self.closure(new_items, g)
  }

  pub fn closure<'a>(&mut self, mut items: HashMap<Lr0Item<'a>, Box<[u32]>>, g: &'a Grammar<'a>) -> Lr1Closure<'a> {
    let mut q = items.clone().into_iter().collect::<VecDeque<_>>();
    while let Some((item, lookahead)) = q.pop_front() {
      // if the token after dot is a non-terminal
      if let Some(ch) = item.prod.get(item.dot as usize).and_then(|&ch| g.as_nt(ch)) {
        let beta = &item.prod[item.dot as usize + 1..];
        let first = self.first(beta, &lookahead, g);
        for new_prod in g.get_prod(ch) {
          let new_item = Lr0Item { prod: &new_prod.rhs, prod_id: new_prod.id, dot: 0 };
          match items.get_mut(&new_item) {
            None => {
              items.insert(new_item, first.clone());
              q.push_back((new_item, first.clone()));
            }
            Some(old_lookahead) => if bitset::bs(old_lookahead).or(&first) {
              // if look ahead changed, also need to reenter queue
              q.push_back((new_item, first.clone()));
            }
          }
        }
      }
    }
    let mut closure = items.into_iter().map(|(lr0, lookahead)| Lr1Item { lr0, lookahead }).collect::<Vec<_>>();
    // sort it, so that vec's equal implies state's equal
    closure.sort_unstable_by(|l, r| l.lr0.cmp(&r.lr0));
    closure
  }
}

// I think it is only for `simple_grammar.rs`'s use now...
pub fn work<'a>(g: &'a Grammar) -> crate::Lr1Fsm<'a> {
  let mut ctx = Lr1Ctx(First::new(g));
  let mut ss = HashMap::default();
  let init = ctx.closure({
    let start = g.start().1;
    let item = Lr0Item { prod: &start.rhs, prod_id: start.id, dot: 0 };
    let mut lookahead = bitset::bsmake(g.token_num());
    bitset::bs(&mut lookahead).set(EOF_IDX);
    let mut init = HashMap::default();
    init.insert(item, lookahead);
    init
  }, g);
  let mut q = VecDeque::new();
  let mut result = Vec::new();
  ss.insert(init.clone(), 0);
  q.push_back(init);
  while let Some(cur) = q.pop_front() {
    let mut link = HashMap::default();
    for mov in 0..g.token_num() as u32 {
      let ns = ctx.go(&cur, mov, g);
      if !ns.is_empty() {
        let new_id = ss.len() as u32;
        let id = *ss.entry(ns.clone()).or_insert_with(|| (q.push_back(ns), new_id).1);
        link.insert(mov, id);
      }
    }
    result.push(crate::Lr1Node { closure: cur, link });
  }
  result
}