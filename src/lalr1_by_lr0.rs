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
//  let mut lalr1 = lr0.iter().map(|lr0| {
//    lr0.0.iter().map(|item| (item, BitSet::new(ctx.token_num))).collect::<Vec<_>>()
//  }).collect::<Vec<_>>();
  let mut look_ahead = lr0.iter().map(|(items, _)| vec![BitSet::new(ctx.token_num); items.len()]).collect::<Vec<_>>();
  let mut clo_cache = HashMap::new();
  let mut prop = Vec::new();

  for (i, item) in lr0[0].0.iter().enumerate() {
    if item.prod == g.start().0.as_ref() {
      look_ahead[0][i].set(g.eof());
      break;
    }
  }
  for (i, (state, link)) in lr0.iter().enumerate() {
    for (item_id, &item) in state.iter().enumerate() {
      // ctx.closure is really slow, so add a cache here
      let clo = clo_cache.entry(item.unique_id()).or_insert_with(||
        ctx.closure({
                      let mut look_ahead = BitSet::new(ctx.token_num);
                      look_ahead.set(ctx.token_num - 1);
                      let mut init = HashMap::new();
                      init.insert(item, look_ahead);
                      init
                    }, g));
      let from = look_ahead[i][item_id].as_ptr();
      for (clo_item, clo_item_look_ahead) in &clo.items {
        if clo_item.dot as usize >= clo_item.prod.len() {
          continue;
        }
        let goto_state = link[&clo_item.prod[clo_item.dot as usize]];
        let goto_item_id = clo_item.unique_id() + 1; // dot + 1
        let goto_item_idx = lr0[goto_state as usize].0.iter().enumerate().find(|item| item.1.unique_id() == goto_item_id).unwrap().0;
        let goto_look_ahead = &mut look_ahead[goto_state as usize][goto_item_idx];
        goto_look_ahead.or(&clo_item_look_ahead);
//        println!("{:?}", to_look_ahead);
        if clo_item_look_ahead.test(ctx.token_num - 1) {
          prop.push((from, goto_look_ahead.as_mut_ptr()));
        }
      }
    }
  }

  let mut changed = true;
  let len = look_ahead[0][0].inner_len();
  while changed {
    changed = false;
    unsafe {
      for &(from, to) in &prop {
        changed |= BitSet::or_raw(to, from, len);
      }
    }
  }

  let result = lr0.clone().into_iter().zip(look_ahead.into_iter()).map(|((state, _), look_ahead_s)| {
    ctx.closure(state.into_iter().zip(look_ahead_s.into_iter()).collect(), g)
  }).collect::<Vec<_>>();

  let mut action = Vec::with_capacity(lr0.len());
  let eof = g.eof();
  let start_id = g.start().1;
  let token_num = g.token_num();
  for (i, (state, link)) in lr0.iter().enumerate() {
    let mut act = HashMap::new();
    for (&k, &v) in link {
      if k < ctx.nt_num {
        act.insert(k, smallvec![ParserAct::Goto(v)]);
      } else {
        act.insert(k, smallvec![ParserAct::Shift(v)]);
      }
    }
    for (item, (_, look_ahead)) in state.iter().zip(result[i].items.iter()) {
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
    action.push((state.iter().map(|item| item).collect(), act));
  }

  let conflict = try_solve_conflict(&mut action, g);
  ParseTable { action, conflict }
}
