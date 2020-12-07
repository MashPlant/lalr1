use common::{grammar::{Grammar, EPS_IDX, EOF_IDX}, *};

// both First and Follow are equivalent to vec![BitSet(g.token_num()); g.nt.len()]
// (calculating the first/follow set of terminal is meaningless)
pub struct First { bitsets: Box<[u32]>, elem_len: usize }

pub struct Follow { bitsets: Box<[u32]>, elem_len: usize }

impl First {
  pub fn new(g: &Grammar) -> First {
    let (token_num, nt_num) = (g.token_num(), g.nt.len());
    let elem_len = bitset::bslen(token_num);
    let first = vec![0; elem_len * nt_num].into_boxed_slice();
    let mut tmp = vec![0; elem_len];
    unsafe {
      let tmp_bs = bitset::ubs(tmp.as_ref());
      loop {
        let mut changed = false;
        for i in 0..nt_num {
          for prod in g.get_prod(i) {
            let mut all_have_eps = true;
            bitset::bs(&mut tmp).clear();
            for &ch in &prod.rhs {
              if let Some(ch) = g.as_nt(ch) {
                let rhs = first.as_ptr().add(ch as usize * elem_len);
                tmp_bs.or(rhs, elem_len);
                tmp_bs.del(EPS_IDX);
                if !bitset::ubs1(rhs).get(EPS_IDX) {
                  all_have_eps = false;
                  break;
                }
              } else {
                tmp_bs.set(ch as usize);
                all_have_eps = false;
                break;
              }
            }
            if all_have_eps { tmp_bs.set(EPS_IDX); }
            changed |= bitset::ubs1(first.as_ptr().add(i * elem_len)).or(tmp.as_ptr(), elem_len);
          }
        }
        if !changed { break; }
      }
    }
    First { bitsets: first, elem_len }
  }

  pub fn first(&self, string: &[u32], g: &Grammar) -> Box<[u32]> {
    let ret = vec![0; self.elem_len].into_boxed_slice();
    unsafe {
      let ret_bs = bitset::ubs(ret.as_ref());
      for &ch in string {
        if let Some(ch) = g.as_nt(ch) {
          let rhs = self.bitsets.as_ptr().add(ch as usize * self.elem_len);
          ret_bs.or(rhs, self.elem_len);
          ret_bs.del(EPS_IDX);
          if !bitset::ubs1(rhs).get(EPS_IDX) { return ret; }
        } else {
          ret_bs.set(ch as usize);
          return ret;
        }
      }
      // reach here, so string -> eps
      ret_bs.set(EPS_IDX);
    }
    ret
  }

  pub fn get(&self, i: usize) -> &[u32] { &self.bitsets[i * self.elem_len..(i + 1) * self.elem_len] }
}

impl Follow {
  pub fn new(g: &Grammar, first: &First) -> Follow {
    let (token_num, nt_num) = (g.token_num(), g.nt.len());
    let elem_len = bitset::bslen(token_num);
    let follow = vec![0; elem_len * nt_num].into_boxed_slice();
    let mut first_cache = HashMap::default();
    unsafe {
      bitset::ubs1(follow.as_ptr().add(g.start().0 as usize * elem_len)).set(EOF_IDX);
      loop {
        let mut changed = false;
        for i in 0..nt_num {
          for prod in g.get_prod(i) {
            let lhs_follow = follow.as_ptr().add(i * elem_len);
            for (i, &ch) in prod.rhs.iter().enumerate() {
              if let Some(ch) = g.as_nt(ch) {
                let ch_follow = bitset::ubs1(follow.as_ptr().add(ch as usize * elem_len));
                let remain = prod.rhs.get_unchecked(i + 1..);
                let remain_first = first_cache.entry(remain).or_insert_with(|| first.first(remain, g));
                changed |= ch_follow.or(remain_first.as_ptr(), elem_len);
                if bitset::ubs(remain_first.as_ref()).get(EPS_IDX) {
                  changed |= ch_follow.or(lhs_follow, elem_len);
                }
              }
            }
          }
        }
        if !changed { break; }
      }
      for i in 0..nt_num {
        bitset::ubs1(follow.as_ptr().add(i * elem_len)).del(EPS_IDX);
      }
    }
    Follow { bitsets: follow, elem_len }
  }

  pub fn get(&self, i: usize) -> &[u32] { &self.bitsets[i * self.elem_len..(i + 1) * self.elem_len] }
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
        let mut predict_bs = bitset::bs(&mut predict);
        if predict_bs.as_imm().get(EPS_IDX) {
          predict_bs.or(follow.get(i));
          predict_bs.del(EPS_IDX);
        }
        psi.insert(prod.id, predict);
      }
      ps.push(psi);
    }
    let mut table = Vec::new();
    for ps in &ps {
      let mut tbi = HashMap::default();
      for (&prod, predict) in ps {
        bitset::ibs(predict).ones(|i| {
          tbi.entry(i as u32).or_insert_with(SmallVec::new).push(prod);
        });
      }
      table.push(tbi);
    }
    LLCtx { first, follow, table }
  }
}