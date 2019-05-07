#![allow(unused)]

use std::hash::{Hash, Hasher};
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use crate::grammar::Grammar;
use crate::bitset::BitSet;
use std::cell::RefCell;
use std::collections::vec_deque::VecDeque;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct LRItem<'a> {
  pub prod: &'a [u32],
  // prod[dot] = the token after dot
  pub dot: u32,
  // look_ahead is the map value in LRState
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct LRState<'a> {
  // item -> look_ahead, which only have [token_num..nt_num] possible to be 1
  // when calculation, use HashMap; after calculation, convert it to Vec, and sort it
  pub items: Vec<(LRItem<'a>, BitSet)>,
  // link is the map value in ss
//  pub link: HashMap<u32, u32>,
}

struct LRCtx<'a> {
  token_num: u32,
  nt_num: u32,
  first_cache: HashMap<&'a [u32], BitSet>,
  // non-terminal should occupy 0..nt_num
  nt_first: Vec<BitSet>,
}

impl LRCtx<'_> {
  fn new<'a>(g: &'a Grammar) -> LRCtx<'a> {
    let (token_num, nt_num) = (g.token_num(), g.nt_num());
    let mut nt_first = vec![RefCell::new(BitSet::new(token_num)); nt_num as usize];
    let mut changed = true;
    while changed {
      changed = false;
      for i in 0..nt_num {
        for prod in g.get_prod(i) {
          let prod = &prod[1..];
          let mut all_have_eps = true;
          for &ch in prod {
            if ch < nt_num { // should be a nt
              if ch != i {
                let rhs = &nt_first[ch as usize].borrow();
                changed |= nt_first[i as usize].borrow_mut().or(rhs);
                if !rhs.test(0) {
                  all_have_eps = false;
                  break;
                }
              }
            } else {
              let mut borrow = nt_first[i as usize].borrow_mut();
              changed |= !borrow.test(ch);
              borrow.set(ch);
              break;
            }
          }
          if all_have_eps {
            nt_first[i as usize].borrow_mut().set(0);
          }
        }
      }
    }
    LRCtx {
      token_num,
      nt_num,
      first_cache: HashMap::new(),
      // oh what a waste...
      // pls someone tell me how to convert RefCell<T> to T
      nt_first: nt_first.into_iter().map(|x| x.borrow().clone()).collect(),
    }
  }

  // ont beta, and many a
  fn first(&mut self, beta: &[u32], a: &BitSet) -> BitSet {
    let mut ret = BitSet::new(self.nt_num);
    for &ch in beta {
      if ch < self.nt_num {
        let rhs = &self.nt_first[ch as usize];
        ret.or(rhs);
        ret.clear(0);
        if !rhs.test(0) {
          return ret;
        }
      } else {
        return ret;
      }
    }
    // reach here, so beta -> eps(but ret doesn't contain eps)
    ret.or(a);
    ret
  }

  fn go<'a>(&mut self, state: &LRState<'a>, mov: u32, g: &Grammar) -> LRState<'a> {
    let mut new_items = HashMap::new();
    for (item, look_ahead) in &state.items {
      if item.prod[item.dot as usize] == mov {
        match new_items.get_mut(item) {
          None => { new_items.insert(item.clone(), look_ahead.clone()); }
          Some(old_look_ahead) => { old_look_ahead.or(look_ahead); }
        }
      }
    }
    self.closure(new_items, g)
  }

  fn closure<'a>(&mut self, mut items: HashMap<LRItem<'a>, BitSet>, g: &Grammar) -> LRState<'a> {
    let mut q = items.clone().into_iter().collect::<VecDeque<_>>();
    while let Some((item, look_ahead)) = q.pop_front() {
      let b = item.prod[item.dot as usize];
      let beta = &item.prod[item.dot as usize + 1..];
      if b < self.nt_num {
        let first = self.first(beta, &look_ahead);
        match items.get_mut(&item) {
          None => {
            items.insert(item.clone(), first.clone());
            q.push_back((item, first));
          }
          Some(old_look_ahead) => { old_look_ahead.or(&first); }
        }
      }
    }
    let mut items = items.into_iter().map(|(k, v)| (k, v)).collect::<Vec<_>>();
    items.sort_unstable_by(|l, r| l.0.cmp(&r.0));
    LRState { items }
  }
}

pub fn work<'a>(g: &'a Grammar, start: &'a [u32]) -> Vec<LRState<'a>> {
  let mut ctx = LRCtx::new(g);
  let mut ss = HashMap::new();
  let mut init = HashMap::new();
  init.insert(LRItem { prod: start, dot: 1 }, g.eof());
  let init = ctx.closure({
                           let item = LRItem { prod: start, dot: 1 };
                           let mut look_ahead = BitSet::new(g.token_num());
                           look_ahead.set(g.eof());
                           let mut init = HashMap::new();
                           init.insert(item, look_ahead);
                           init
                         }, g);
  ss.insert(init.items.clone(), 0);
  let mut q = VecDeque::new();
  let mut result = Vec::new();
  q.push_back(init);
  while let Some(mut cur) = q.pop_front() {
    let mut link = HashMap::new();
    for mov in 0..ctx.token_num {
      let mut ns = ctx.go(&cur, mov, g);
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
  unimplemented!()
}
//impl LRItem<'_> {}
//
//impl Hash for LRItem<'_> {
//  fn hash<H: Hasher>(&self, h: &mut H) {
//    h.write_u32(self.hash);
//  }
//}