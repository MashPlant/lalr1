use crate::{Lr0Item, Lr0Fsm, Lr0Node};
use grammar_config::AbstractGrammar;
use hashbrown::{HashMap, HashSet};
use std::collections::vec_deque::VecDeque;

struct Ctx {
  token_num: u32,
  nt_num: u32,
}

impl Ctx {
  fn go<'a>(&self, items: &Vec<Lr0Item<'a>>, mov: u32, g: &'a impl AbstractGrammar<'a>) -> Vec<Lr0Item<'a>> {
    let mut new_items = HashSet::new();
    for item in items {
      if item.dot as usize >= item.prod.len() { // dot is after the last ch
        continue;
      }
      if item.prod[item.dot as usize] == mov {
        new_items.insert(Lr0Item { prod: item.prod, prod_id: item.prod_id, dot: item.dot + 1 });
      }
    }
    self.closure(new_items, g)
  }

  fn closure<'a>(&self, mut items: HashSet<Lr0Item<'a>>, g: &'a impl AbstractGrammar<'a>) -> Vec<Lr0Item<'a>> {
    let mut q = items.clone().into_iter().collect::<VecDeque<_>>();
    while let Some(item) = q.pop_front() {
      if item.dot as usize >= item.prod.len() { // dot is after the last ch
        continue;
      }
      let b = item.prod[item.dot as usize];
      if b < self.nt_num {
        for new_prod in g.get_prod(b) {
          let new_item = Lr0Item { prod: new_prod.0.as_ref(), prod_id: new_prod.1, dot: 0 };
          if items.insert(new_item) {
            q.push_back(new_item);
          }
        }
      }
    }
    let mut items = items.into_iter().collect::<Vec<_>>();
    // why sort_unstable_by_key(|x| &x.0) won't work here?
    items.sort_unstable_by(|l, r| l.cmp(&r));
    items
  }
}

pub fn work<'a>(g: &'a impl AbstractGrammar<'a>) -> Lr0Fsm<'a> {
  let ctx = Ctx { token_num: g.token_num(), nt_num: g.nt_num() };
  let mut ss = HashMap::new();
  let init = ctx.closure({
                           let start = g.start().1;
                           let mut init = HashSet::new();
                           init.insert(Lr0Item { prod: start.0.as_ref(), prod_id: start.1, dot: 0 });
                           init
                         }, g);
  ss.insert(init.clone(), 0);
  let mut q = VecDeque::new();
  let mut result = Vec::new();
  q.push_back(init);
  while let Some(cur) = q.pop_front() {
    let mut link = HashMap::new();
    for mov in 0..ctx.token_num {
      let ns = ctx.go(&cur, mov, g);
      if !ns.is_empty() {
        let id = match ss.get(&ns) {
          None => {
            let id = ss.len() as u32;
            ss.insert(ns.clone(), id);
            q.push_back(ns);
            id
          }
          Some(id) => *id,
        };
        link.insert(mov, id);
      }
    }
    result.push(Lr0Node { closure: cur, link });
  }
  result
}