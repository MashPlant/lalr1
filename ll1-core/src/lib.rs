extern crate grammar_config;
extern crate bitset;
extern crate smallvec;
extern crate indexmap;

use grammar_config::AbstractGrammar;
use bitset::BitSet;
use std::collections::HashMap;
use smallvec::SmallVec;
use indexmap::IndexMap;

pub struct First {
  pub token_num: usize,
  pub eps: usize,
  pub nt_first: Vec<BitSet>,
}

impl First {
  pub fn new<'a>(g: &'a impl AbstractGrammar<'a>) -> First {
    let (token_num, nt_num, eps) = (g.token_num() as usize, g.nt_num() as usize, g.eps() as usize);
    assert!(nt_num <= eps && eps < token_num);
    let mut nt_first = vec![BitSet::new(token_num); nt_num];
    let mut tmp = BitSet::new(token_num);
    let inner_len = BitSet::calc_inner_len(token_num);
    unsafe {
      let mut changed = true;
      while changed {
        changed = false;
        for i in 0..nt_num {
          let lhs = nt_first[i].as_mut_ptr();
          let mut all_have_eps = true;
          for prod in g.get_prod(i as u32) {
            tmp.clear_all();
            for &ch in prod.0.as_ref() {
              let ch = ch as usize;
              if ch < nt_num {
                let rhs = nt_first[ch].as_ptr();
                BitSet::or_raw(tmp.as_mut_ptr(), rhs, inner_len);
                tmp.clear_unchecked(eps);
                if !BitSet::test_raw(rhs, eps) {
                  all_have_eps = false;
                  break;
                }
              } else {
                tmp.set_unchecked(ch);
                all_have_eps = false;
                break;
              }
            }
            if all_have_eps {
              tmp.set_unchecked(eps);
            }
            changed |= BitSet::or_raw(lhs, tmp.as_ptr(), inner_len);
          }
        }
      }
    }
    First { token_num, eps, nt_first }
  }

  pub fn first(&self, string: &[u32]) -> BitSet {
    let mut ret = BitSet::new(self.token_num);
    for &ch in string {
      let ch = ch as usize;
      if ch < self.nt_num() {
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
    // reach here, so string -> eps
    ret.set(self.eps);
    ret
  }

  pub fn nt_num(&self) -> usize {
    self.nt_first.len()
  }
}

pub struct Follow {
  pub nt_follow: Vec<BitSet>,
}

impl Follow {
  pub fn new<'a>(g: &'a impl AbstractGrammar<'a>, first: &First) -> Follow {
    let eof = g.eof() as usize;
    assert!(first.nt_num() <= eof && eof < first.token_num);
    let mut nt_follow = vec![BitSet::new(first.token_num); first.nt_num()];
    nt_follow[g.start().0 as usize].set(eof);
    let inner_len = BitSet::calc_inner_len(first.token_num);
    let mut first_cache = HashMap::new();
    unsafe {
      let mut changed = true;
      while changed {
        changed = false;
        for i in 0..first.nt_num() {
          for prod in g.get_prod(i as u32) {
            let lhs_follow = nt_follow[i].as_ptr();
            let prod = prod.0.as_ref();
            for (i, &ch) in prod.iter().enumerate() {
              let ch = ch as usize;
              if ch < first.nt_num() {
                let ch_follow = nt_follow[ch].as_mut_ptr();
                let remain = &prod[i + 1..];
                let remain_first = first_cache.entry(remain).or_insert_with(|| first.first(remain));
                changed |= BitSet::or_raw(ch_follow, remain_first.as_ptr(), inner_len);
                if remain_first.test(first.eps) {
                  changed |= BitSet::or_raw(ch_follow, lhs_follow, inner_len);
                }
              }
            }
          }
        }
      }
    }
    for follow in &mut nt_follow {
      follow.clear(first.eps);
    }
    Follow { nt_follow }
  }
}

pub struct LLCtx {
  pub first: First,
  pub follow: Follow,
  // u32: id of prod(it is easy to get prod by id, but not the reverse)
  // use IndexMap to solve conflict(who comes first has priority)
  pub ps: Vec<IndexMap<u32, BitSet>>,
  pub table: Vec<HashMap<u32, SmallVec<[u32; 1]>>>,
}

impl LLCtx {
  pub fn new<'a>(g: &'a impl AbstractGrammar<'a>) -> LLCtx {
    let first = First::new(g);
    let follow = Follow::new(g, &first);
    let mut ps = Vec::new();
    for i in 0..first.nt_num() {
      let mut psi = IndexMap::new();
      for prod in g.get_prod(i as u32) {
        let mut predict = first.first(prod.0.as_ref());
        if predict.test(first.eps) {
          predict.or(&follow.nt_follow[i]);
          predict.clear(first.eps);
        }
        psi.insert(prod.1, predict);
      }
      ps.push(psi);
    }
    let mut table = Vec::new();
    for ps in &ps {
      let mut tbi = HashMap::new();
      for (&prod, predict) in ps {
        for i in 0..first.token_num {
          if predict.test(i) {
            tbi.entry(i as u32).or_insert_with(|| SmallVec::new()).push(prod);
          }
        }
      }
      table.push(tbi);
    }
    LLCtx { first, follow, ps, table }
  }
}