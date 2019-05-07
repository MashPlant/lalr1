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
  pub look_ahead: Vec<u32>,
}

// only consider lr1 core(prod & dot) in hash & eq & ord
// assume prods may never be duplicate, so compare their pointers is just ok
// and this is 100% safe rust :)
//impl Hash for LRItem<'_> {
//  fn hash<H: Hasher>(&self, state: &mut H) {
//    (&self.prod[0] as *const u32).hash(state);
//    self.dot.hash(state);
//  }
//}
//
//impl PartialEq for LRItem<'_> {
//  fn eq(&self, other: &LRItem) -> bool {
//    self.prod as *const _ == other.prod as *const _ && self.dot == other.dot
//  }
//}
//
//impl Eq for LRItem<'_> {}
//
//impl PartialOrd for LRItem<'_> {
//  fn partial_cmp(&self, other: &LRItem) -> Option<Ordering> {
//    Some(self.cmp(other))
//  }
//}
//
//impl Ord for LRItem<'_> {
//  fn cmp(&self, other: &LRItem) -> Ordering {
//    match (&self.prod[0] as *const u32).cmp(&(&other.prod[0] as *const _)) {
//      Ordering::Equal => self.dot.cmp(&other.dot),
//      t => t,
//    }
//  }
//}

#[derive(Debug, Clone)]
pub struct LRState<'a> {
  // should be sorted in ordered to be compared
  pub items: Vec<LRItem<'a>>,
  pub link: HashMap<u32, u32>,
}

//pub struct LRCoreState<'a> {
//  pub items: Vec<LRCore<'a>>,
//  pub link: Vec<u32>,
//  pub hash: u64,
//}

//impl PartialEq for LRState<'_> {
//  fn eq(&self, other: &LRState) -> bool {
//    self.items == other.items
//  }
//}
//
//impl Eq for LRState<'_> {}
//
//impl Hash for LRState<'_> {
//  fn hash<H: Hasher>(&self, state: &mut H) {
//    self.items.hash(state);
//  }
//}

struct LRCtx<'a> {
  token_num: u32,
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
        for prod in g.get_prod(i).unwrap() {
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
    unimplemented!()
  }

  fn first(&mut self) {

  }
//  fn first(&mut self, string: &[u32], g: &Grammar) -> &FixedBitSet {
//    match self.first_cache.get(string) {
//      Some(first) => first,
//      None => {
//        let string = string.into_vec();
//        let mut result = FixedBitSet::with_capacity(self.token_num as usize);
//        for &ch in &string {
//          match g.get_prod(ch) {
//            None => {}
//            Some(prods) => {
//              for prod in prods {
//                let prod = &prod[1..];
//
//              }
//            }
//          }
//        }
////        for (i,ch) in string.iter.en {
////
////        }
////        for i in 0..string.len() {
////          let prefix
////        }
//      }
//    }
////    self.first_cache.entry(beta_a).or_insert_with(|| {
//////      let mut result = FixedBitSet::with_capacity(self.token_num as usize);
////      let mut result = self.first1(beta_a.0, g).clone();
////      if result[0] {
//////        result |= self.first1(&[beta_a])
////      }
//////      for &ch in beta_a.0 {
//////        // only to save a copy
//////        g.add_first(ch, &mut result);
//////        if !result[0] {
//////          return result;
//////        }
//////        // in lr1 algorithm, first(beta a) can never contain eps, because a is not eps
//////        result.set(0, false);
//////      }
//////      g.add_first(beta_a.1, &mut result);
////      result
////    })
//  }

//  fn first1<'b: 'a>(&mut self, string: &[u32], g: &Grammar) -> &FixedBitSet {
//    unimplemented!()
//  }

  fn go<'a>(&mut self, state: &LRState<'a>, mov: u32, g: &Grammar) -> LRState<'a> {
    let mut new_items = HashSet::new();
    for item in &state.items {
      if item.prod[item.dot as usize] == mov {
        new_items.insert(LRItem { prod: item.prod, dot: item.dot + 1, look_ahead: item.look_ahead.clone() });
      }
    }
    self.closure(new_items, g)
  }

  fn closure<'a>(&mut self, mut items: HashSet<LRItem<'a>>, g: &Grammar) -> LRState<'a> {
    let mut q = items.clone().into_iter().collect::<VecDeque<_>>();
    while let Some(cur) = q.pop_front() {
      let b = cur.prod[cur.dot as usize];
      let beta = &cur.prod[cur.dot as usize + 1..];
//      let a  = cur.prod[cur.dot as usize];
      match g.get_prod(b) {
        None => {}
        Some(prod) => {
//          let first =
//          let n =
        }
      }
    }
    LRState {
      items: items.into_iter().collect(),
      link: HashMap::new(),
    }
  }
}

pub fn work<'a>(g: &'a Grammar, start: &'a [u32], token_num: u32) -> Vec<LRState<'a>> {
  let mut ctx = LRCtx::new(g);
//  let mut ctx = LRCtx { token_num, first_cache: HashMap::new(), first_cache1: HashMap::new() };
  let mut ss = HashMap::new();
  let mut init = HashSet::new();
  init.insert(LRItem { prod: start, dot: 1, look_ahead: Vec::new() });
  let init = ctx.closure(init, g);
  ss.insert(init.items.clone(), 0);
  let mut q = VecDeque::new();
  let mut result = Vec::new();
  q.push_back(init);
  while let Some(mut cur) = q.pop_front() {
    for mov in 0..token_num {
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
        cur.link.insert(mov, id);
      }
    }
    result.push(cur);
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