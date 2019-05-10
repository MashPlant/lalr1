// "Compilers: Principles, Techniques and Tools" Algorithm 4.63

use crate::bitset::BitSet;
use crate::lr1::LRCtx;
use crate::lr0::LRItem;
use crate::abstract_grammar::AbstractGrammarExt;
use crate::lalr1_common::*;
use std::collections::HashMap;
use smallvec::SmallVec;

pub fn work<'a>(lr0: &'a Vec<(Vec<LRItem<'a>>, HashMap<u32, u32>)>, g: &'a impl AbstractGrammarExt<'a>) -> ParseTable<'a> {
  let mut ctx = LRCtx::new(g);
  let mut lalr1 = lr0.iter().map(|lr0| {
    lr0.0.iter().map(|item| (item, BitSet::new(ctx.token_num))).collect::<Vec<_>>()
  }).collect::<Vec<_>>();
  let mut clo_cache = HashMap::new();
  let mut prop = Vec::new();

  for (i, state) in lr0.iter().enumerate() {
    for (item_id, &item) in state.0.iter().enumerate() {
      // ctx.closure is really slow, so add a cache here
      let clo = clo_cache.entry(item.unique_id()).or_insert_with(||
        ctx.closure({
                      let mut look_ahead = BitSet::new(ctx.token_num);
                      look_ahead.set(ctx.token_num - 1);
                      let mut init = HashMap::new();
                      init.insert(item, look_ahead);
                      init
                    }, g));
      let from = lalr1[i][item_id].1.as_ptr();
      for (&edge, &to_id) in &state.1 {
        let to = &mut lalr1[to_id as usize];
        for (clo_item, look_ahead) in &clo.items {
          if (item.dot as usize) < item.prod.len() && item.prod[item.dot as usize] == edge {
            let id = item.unique_id() + 1; // dot + 1
            let to_item = to.iter_mut().enumerate().find(|item| (item.1).0.unique_id() == id).unwrap();
            for i in 0..ctx.token_num - 1 {
              if look_ahead.test(i) {
                (to_item.1).1.set(i);
              }
            }
            if look_ahead.test(ctx.token_num - 1) {
              prop.push((from, (to_item.1).1.as_mut_ptr()));
            }
          }
        }
      }
    }
  }

  let mut changed = true;
  let len = lalr1[0][0].1.inner_len();
  while changed {
    changed = false;
    unsafe {
      for &(from, to) in &prop {
        changed |= BitSet::or_raw(to, from, len);
      }
    }
  }

  let mut action = Vec::with_capacity(lalr1.len());
  let eof = g.eof();
  let start_id = g.start().1;
  let token_num = g.token_num();
  for (i, state) in lalr1.iter().enumerate() {
    let mut act = HashMap::new();
    for (&k, &v) in &lr0[i].1 {
      if k < ctx.nt_num {
        act.insert(k, smallvec![ParserAct::Goto(v)]);
      } else {
        act.insert(k, smallvec![ParserAct::Shift(v)]);
      }
    }
    for (item, look_ahead) in state {
      if item.dot == item.prod.len() as u32 {
        if look_ahead.test(g.eof()) && item.prod_id == start_id {
          act.insert(eof, smallvec![ParserAct::Acc]);
        } else {
          for i in 0..token_num {
            if look_ahead.test(i) {
              // maybe conflict here
              act.entry(i).or_insert_with(|| SmallVec::new()).push(ParserAct::Reduce(item.prod_id));
            }
          }
        }
      }
    }
    action.push((state.iter().map(|&(item, _)| item).collect(), act));
  }

  let conflict = try_solve_conflict(&mut action, g);
  ParseTable { action, conflict }
}
