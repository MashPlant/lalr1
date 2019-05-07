#![allow(unused)]

use std::hash::{Hash, Hasher};
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use crate::grammar::Grammar;
use crate::bitset::BitSet;
use crate::abstract_grammar::AbstractGrammar;
use std::cell::RefCell;
use std::collections::vec_deque::VecDeque;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct LRItem<'a> {
  pub prod: &'a [u32],
  pub prod_id: u32,
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

struct LRCtx {
  token_num: u32,
  nt_num: u32,
  eps: u32,
  //  first_cache: HashMap<&'a [u32], BitSet>,
  // non-terminal should occupy 0..nt_num
  nt_first: Vec<BitSet>,
}

impl LRCtx {
  fn new<'a>(g: &'a impl AbstractGrammar<'a>) -> LRCtx {
    let (token_num, nt_num, eps) = (g.token_num(), g.nt_num(), g.eps());
    let mut nt_first = vec![RefCell::new(BitSet::new(token_num)); nt_num as usize];
    let mut changed = true;
    while changed {
      changed = false;
      for i in 0..nt_num {
        for prod in g.get_prod(i) {
          let mut all_have_eps = true;
          for &ch in prod.0.as_ref() {
            if ch < nt_num {
              let rhs = &nt_first[ch as usize].borrow();
              if ch != i {
                changed |= nt_first[i as usize].borrow_mut().or(rhs);
              }
              if !rhs.test(eps) {
                all_have_eps = false;
                break;
              }
            } else {
              let mut borrow = nt_first[i as usize].borrow_mut();
              changed |= !borrow.test(ch);
              borrow.set(ch);
              all_have_eps = false;
              break;
            }
          }
          if all_have_eps {
            nt_first[i as usize].borrow_mut().set(eps);
          }
        }
      }
    }
    LRCtx {
      token_num,
      nt_num,
      eps,
//      first_cache: HashMap::new(),
      // oh what a waste...
      // pls someone tell me how to convert RefCell<T> to T
      nt_first: nt_first.into_iter().map(|x| x.borrow().clone()).collect(),
    }
  }

  // one beta, and many a
  fn first(&mut self, beta: &[u32], a: &BitSet) -> BitSet {
    let mut ret = BitSet::new(self.nt_num);
    for &ch in beta {
      if ch < self.nt_num {
        let rhs = &self.nt_first[ch as usize];
        ret.or(rhs);
        ret.clear(self.eps);
        if !rhs.test(self.eps) {
          return ret;
        }
      } else {
        ret.set(ch);
        return ret;
      }
    }
    // reach here, so beta -> eps(but ret doesn't contain eps)
    ret.or(a);
    ret
  }

  fn go<'a>(&mut self, state: &LRState<'a>, mov: u32, g: &'a impl AbstractGrammar<'a>) -> LRState<'a> {
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

  fn closure<'a>(&mut self, mut items: HashMap<LRItem<'a>, BitSet>, g: &'a impl AbstractGrammar<'a>) -> LRState<'a> {
//    println!("enter closure {:?}", items);
    let mut q = items.clone().into_iter().collect::<VecDeque<_>>();
    while let Some((item, look_ahead)) = q.pop_front() {
      if item.dot as usize >= item.prod.len() { // dot is after the last ch
        continue;
      }
      let b = item.prod[item.dot as usize];
      let beta = &item.prod[item.dot as usize + 1..];
//      println!("beta = {:?}", beta);
      if b < self.nt_num {
        let first = self.first(beta, &look_ahead);
//        println!("look_ahead = {:?}", look_ahead);
//        println!("first = {:?}", first);
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
  let mut ctx = LRCtx::new(g);
//  println!("{:?}", ctx.nt_first);
  let mut ss = HashMap::new();
  let init = ctx.closure({
                           let start = g.start();
                           let item = LRItem { prod: start.0.as_ref(), prod_id: start.1, dot: 0 };
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
  result
//  println!("{:?}", result);
}