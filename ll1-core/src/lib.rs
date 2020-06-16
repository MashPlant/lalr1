use common::{grammar::{Grammar, EPS_IDX, EOF_IDX}, *};

// first.len() == follow.len() == g.nt.len() (calculating the first/follow set of terminal is meaningless)
pub struct First(pub Vec<BitSet>);

pub struct Follow(pub Vec<BitSet>);

impl First {
  pub fn new(g: &Grammar) -> First {
    let token_num = g.token_num();
    let mut first = vec![BitSet::new(token_num); g.nt.len()];
    let mut tmp = BitSet::new(token_num);
    let inner_len = BitSet::calc_inner_len(token_num);
    unsafe {
      let mut changed = true;
      while changed {
        changed = false;
        for i in 0..g.nt.len() {
          let lhs = first[i].as_mut_ptr();
          for prod in g.get_prod(i) {
            let mut all_have_eps = true;
            tmp.clear_all();
            for &ch in &prod.rhs {
              if let Some(ch) = g.as_nt(ch) {
                let rhs = first[ch as usize].as_ptr();
                BitSet::or_raw(tmp.as_mut_ptr(), rhs, inner_len);
                tmp.clear_unchecked(EPS_IDX);
                if !BitSet::test_raw(rhs, EPS_IDX) {
                  all_have_eps = false;
                  break;
                }
              } else {
                tmp.set_unchecked(ch as usize);
                all_have_eps = false;
                break;
              }
            }
            if all_have_eps { tmp.set_unchecked(EPS_IDX); }
            changed |= BitSet::or_raw(lhs, tmp.as_ptr(), inner_len);
          }
        }
      }
    }
    First(first)
  }

  pub fn first(&self, string: &[u32], g: &Grammar) -> BitSet {
    let mut ret = BitSet::new(g.token_num());
    for &ch in string {
      if let Some(ch) = g.as_nt(ch) {
        let rhs = &self.0[ch as usize];
        ret.or(rhs);
        ret.clear(EPS_IDX);
        if !rhs.test(EPS_IDX) { return ret; }
      } else { return (ret.set(ch as usize), ret).1; }
    }
    // reach here, so string -> eps
    ret.set(EPS_IDX);
    ret
  }
}

impl Follow {
  pub fn new(g: &Grammar, first: &First) -> Follow {
    let mut follow = vec![BitSet::new(g.token_num()); g.nt.len()];
    follow[g.start().0 as usize].set(EOF_IDX);
    let inner_len = BitSet::calc_inner_len(g.token_num());
    let mut first_cache = HashMap::new();
    unsafe {
      let mut changed = true;
      while changed {
        changed = false;
        for i in 0..g.nt.len() {
          for prod in g.get_prod(i as u32) {
            let lhs_follow = follow[i].as_ptr();
            for (i, &ch) in prod.rhs.iter().enumerate() {
              if let Some(ch) = g.as_nt(ch) {
                let ch_follow = follow[ch as usize].as_mut_ptr();
                let remain = &prod.rhs[i + 1..];
                let remain_first = first_cache.entry(remain).or_insert_with(|| first.first(remain, g));
                changed |= BitSet::or_raw(ch_follow, remain_first.as_ptr(), inner_len);
                if remain_first.test(EPS_IDX) {
                  changed |= BitSet::or_raw(ch_follow, lhs_follow, inner_len);
                }
              }
            }
          }
        }
      }
    }
    for f in &mut follow { f.clear(EPS_IDX); }
    Follow(follow)
  }
}

pub type LLTable = Vec<HashMap<u32, SmallVec<[u32; 2]>>>;

// first set and ps set are useless for parser generating, if you need them, `LLCtx::new` have these 2 local variables
// update: now first set is added back into LLCtx, but only for printing
pub struct LLCtx {
  pub first: First,
  pub follow: Follow,
  // u32: id of prod(it is easy to get prod by id, but not the reverse)
  // use IndexMap to solve conflict(who comes first has priority)
//  pub ps: Vec<IndexMap<u32, BitSet>>,
  pub table: LLTable,
}

impl LLCtx {
  pub fn new(g: &Grammar) -> LLCtx {
    let first = First::new(g);
    let follow = Follow::new(g, &first);
    let mut ps = Vec::new();
    for i in 0..g.nt.len() {
      let mut psi = IndexMap::default();
      for prod in g.get_prod(i) {
        let mut predict = first.first(&prod.rhs, g);
        if predict.test(EPS_IDX) {
          predict.or(&follow.0[i]);
          predict.clear(EPS_IDX);
        }
        psi.insert(prod.id, predict);
      }
      ps.push(psi);
    }
    let mut table = Vec::new();
    for ps in &ps {
      let mut tbi = HashMap::new();
      for (&prod, predict) in ps {
        for i in 0..g.token_num() {
          if predict.test(i) {
            tbi.entry(i as u32).or_insert_with(|| SmallVec::new()).push(prod);
          }
        }
      }
      table.push(tbi);
    }
    LLCtx { first, follow, table }
  }
}