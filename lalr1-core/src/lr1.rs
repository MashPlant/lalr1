use hashbrown::HashMap;
use grammar_config::AbstractGrammar;
use std::collections::vec_deque::VecDeque;
use ll1_core::First;
use crate::lr0::LRItem;
use crate::bitset::BitSet;
use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LRState<'a> {
  // item -> look_ahead, which only have [token_num..nt_num] possible to be 1
  // when calculation, use HashMap; after calculation, convert it to Vec, and sort it
  pub items: Vec<(LRItem<'a>, BitSet)>,
  // link is the map value in ss
//  pub link: HashMap<u32, u32>,
}

pub struct LRCtx(First);

impl Deref for LRCtx {
  type Target = First;

  fn deref(&self) -> &Self::Target { &self.0 }
}

impl LRCtx {
  pub fn new<'a>(g: &'a impl AbstractGrammar<'a>) -> LRCtx { LRCtx(First::new(g)) }

  // one beta, and many a
  pub fn first(&self, beta: &[u32], a: &BitSet) -> BitSet {
    let mut beta_first = self.0.first(beta);
    if beta_first.test(self.0.eps) {
      beta_first.clear(self.0.eps);
      beta_first.or(a);
    }
    beta_first
  }

  pub fn go<'a>(&mut self, state: &LRState<'a>, mov: u32, g: &'a impl AbstractGrammar<'a>) -> LRState<'a> {
    let mut new_items = HashMap::new();
    for (item, look_ahead) in &state.items {
      if item.dot as usize >= item.prod.len() { // dot is after the last ch
        continue;
      }
      if item.prod[item.dot as usize] == mov {
        let new_item = LRItem { prod: item.prod, prod_id: item.prod_id, dot: item.dot + 1 };
        match new_items.get_mut(&new_item) {
          None => { new_items.insert(new_item, look_ahead.clone()); }
          Some(old_look_ahead) => { old_look_ahead.or(look_ahead); }
        }
      }
    }
    self.closure(new_items, g)
  }

  pub fn closure<'a>(&mut self, mut items: HashMap<LRItem<'a>, BitSet>, g: &'a impl AbstractGrammar<'a>) -> LRState<'a> {
    let mut q = items.clone().into_iter().collect::<VecDeque<_>>();
    while let Some((item, look_ahead)) = q.pop_front() {
      if item.dot as usize >= item.prod.len() { // dot is after the last ch
        continue;
      }
      let b = item.prod[item.dot as usize];
      let beta = &item.prod[item.dot as usize + 1..];
      if b < (self.0.nt_num() as u32) {
        let first = self.first(beta, &look_ahead);
        for new_prod in g.get_prod(b) {
          let new_item = LRItem { prod: new_prod.0.as_ref(), prod_id: new_prod.1, dot: 0 };
          match items.get_mut(&new_item) {
            None => {
              items.insert(new_item, first.clone());
              q.push_back((new_item, first.clone()));
            }
            Some(old_look_ahead) => {
              // if look ahead changed, also need to reenter queue
              if old_look_ahead.or(&first) {
                q.push_back((new_item, first.clone()));
              }
            }
          }
        }
      }
    }
    let mut items = items.into_iter().map(|(k, v)| (k, v)).collect::<Vec<_>>();
    // why sort_unstable_by_key(|x| &x.0) won't work here?
    items.sort_unstable_by(|l, r| l.0.cmp(&r.0));

    LRState { items }
  }
}

pub type LRResult<'a> = (LRState<'a>, HashMap<u32, u32>);

pub fn work<'a>(g: &'a impl AbstractGrammar<'a>) -> Vec<LRResult<'a>> {
  let mut ctx = LRCtx(First::new(g));
  let mut ss = HashMap::new();
  let init = ctx.closure({
                           let start = g.start().1;
                           let item = LRItem { prod: start.0.as_ref(), prod_id: start.1, dot: 0 };
                           let mut look_ahead = BitSet::new(g.token_num() as usize);
                           look_ahead.set(g.eof() as usize);
                           let mut init = HashMap::new();
                           init.insert(item, look_ahead);
                           init
                         }, g);
  ss.insert(init.items.clone(), 0);
  let mut q = VecDeque::new();
  let mut result = Vec::new();
  q.push_back(init);
  while let Some(cur) = q.pop_front() {
    let mut link = HashMap::new();
    for mov in 0..ctx.0.token_num as u32 {
      let ns = ctx.go(&cur, mov, g);
      if !ns.items.is_empty() {
        let id = match ss.get(&ns.items) {
          None => {
            let id = ss.len() as u32;
            ss.insert(ns.items.clone(), id);
            q.push_back(ns);
            id
          }
          Some(id) => *id,
        };
        link.insert(mov, id);
      }
    }

    result.push((cur, link));
  }
  result
}