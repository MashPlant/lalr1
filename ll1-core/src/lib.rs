extern crate grammar_config;
extern crate bitset;

use grammar_config::AbstractGrammar;
use bitset::BitSet;

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
    let inner_len = BitSet::calc_inner_len(token_num);
    unsafe {
      let mut changed = true;
      while changed {
        changed = false;
        for i in 0..nt_num {
          for prod in g.get_prod(i as u32) {
            let lhs = nt_first[i].as_mut_ptr();
            if prod.0.as_ref().is_empty() {
              changed |= !BitSet::test_raw(lhs, eps);
              BitSet::set_raw(lhs, eps);
            }
            let mut all_have_eps = true;
            for &ch in prod.0.as_ref() {
              let ch = ch as usize;
              if ch < nt_num {
                let rhs = nt_first[ch].as_ptr();
                changed |= BitSet::or_raw(lhs, rhs, inner_len);
                if !BitSet::test_raw(rhs, eps) {
                  all_have_eps = false;
                  break;
                }
              } else {
                changed |= !BitSet::test_raw(lhs, ch);
                BitSet::set_raw(lhs, ch);
                all_have_eps = false;
                break;
              }
            }
            if all_have_eps {
              BitSet::set_raw(lhs, eps);
            }
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
        let rhs = &self.nt_first[ch];
        ret.or(rhs);
        if !rhs.test(self.eps) {
          break;
        }
      } else {
        ret.set(ch);
        break;
      }
    }
    ret
  }

  pub fn nt_num(&self) -> usize {
    self.nt_first.len()
  }
}

pub struct Follow {}

pub struct LLCtx {
  first: First,
  follow: Follow,
}