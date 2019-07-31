use crate::{Lr0Item, Lr1Closure, Lr1Item};
use hashbrown::HashMap;
use grammar_config::AbstractGrammar;
use std::collections::vec_deque::VecDeque;
use ll1_core::First;
use bitset::BitSet;
use std::ops::Deref;

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

  // `go` was used by lr1 before, now not used
  pub fn go<'a>(&mut self, state: &Lr1Closure<'a>, mov: u32, g: &'a impl AbstractGrammar<'a>) -> Lr1Closure<'a> {
    let mut new_items = HashMap::new();
    for Lr1Item { item, lookahead } in state {
      if item.dot as usize >= item.prod.len() { // dot is after the last ch
        continue;
      }
      if item.prod[item.dot as usize] == mov {
        let new_item = Lr0Item { prod: item.prod, prod_id: item.prod_id, dot: item.dot + 1 };
        match new_items.get_mut(&new_item) {
          None => { new_items.insert(new_item, lookahead.clone()); }
          Some(old_lookahead) => { old_lookahead.or(lookahead); }
        }
      }
    }
    self.closure(new_items, g)
  }

  pub fn closure<'a>(&mut self, mut items: HashMap<Lr0Item<'a>, BitSet>, g: &'a impl AbstractGrammar<'a>) -> Lr1Closure<'a> {
    let mut q = items.clone().into_iter().collect::<VecDeque<_>>();
    while let Some((item, lookahead)) = q.pop_front() {
      if item.dot as usize >= item.prod.len() { // dot is after the last ch
        continue;
      }
      let b = item.prod[item.dot as usize];
      let beta = &item.prod[item.dot as usize + 1..];
      if b < (self.0.nt_num() as u32) {
        let first = self.first(beta, &lookahead);
        for new_prod in g.get_prod(b) {
          let new_item = Lr0Item { prod: new_prod.0.as_ref(), prod_id: new_prod.1, dot: 0 };
          match items.get_mut(&new_item) {
            None => {
              items.insert(new_item, first.clone());
              q.push_back((new_item, first.clone()));
            }
            Some(old_lookahead) => {
              // if look ahead changed, also need to reenter queue
              if old_lookahead.or(&first) {
                q.push_back((new_item, first.clone()));
              }
            }
          }
        }
      }
    }
    let mut closure = items.into_iter().map(|(item, lookahead)| Lr1Item { item, lookahead }).collect::<Vec<_>>();
    // sort it, so that vec's equal implies state's equal
    closure.sort_unstable_by(|l, r| l.item.cmp(&r.item));
    closure
  }
}