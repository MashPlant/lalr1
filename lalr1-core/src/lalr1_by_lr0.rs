// "Compilers: Principles, Techniques and Tools" Algorithm 4.63

use crate::bitset::BitSet;
use crate::lr1::{LRCtx, LRState, LRResult};
use crate::lr0::LRItem;
use grammar_config::AbstractGrammarExt;
use crate::lalr1_common::*;
use std::collections::HashMap;
use smallvec::SmallVec;

// inner version, the return value doesn't contain `link`
fn _lalr1_only<'a>(lr0: &'a Vec<(Vec<LRItem<'a>>, HashMap<u32, u32>)>, g: &'a impl AbstractGrammarExt<'a>) -> Vec<LRState<'a>> {
  let mut ctx = LRCtx::new(g);
  let mut look_ahead = lr0.iter()
    .map(|(items, _)| vec![BitSet::new(ctx.token_num as usize); items.len()]).collect::<Vec<_>>();
  let mut clo_cache = HashMap::new();
  let mut prop = Vec::new();
  let start_prod = g.start().0.as_ref();

  for (i, item) in lr0[0].0.iter().enumerate() {
    if item.prod == start_prod {
      look_ahead[0][i].set(g.eof() as usize);
      break;
    }
  }

  for (i, (state, link)) in lr0.iter().enumerate() {
    for (item_id, &item) in state.iter().enumerate() {
      // only consider lr0 core item
      if item.prod == start_prod || item.dot != 0 {
        // ctx.closure is really slow, so add a cache here
        let clo = clo_cache.entry(item.unique_id()).or_insert_with(||
          ctx.closure({
                        let mut look_ahead = BitSet::new(ctx.token_num as usize);
                        look_ahead.set(ctx.token_num as usize - 1);
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
          if clo_item_look_ahead.test(ctx.token_num  as usize - 1) {
            prop.push((from, goto_look_ahead.as_mut_ptr()));
          }
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

  lr0.clone().into_iter().zip(look_ahead.into_iter()).map(|((state, _), look_ahead_s)| {
    ctx.closure(state.into_iter().zip(look_ahead_s.into_iter()).collect(), g)
  }).collect::<Vec<_>>()
}

pub fn work<'a>(lr0: &'a Vec<(Vec<LRItem<'a>>, HashMap<u32, u32>)>, g: &'a impl AbstractGrammarExt<'a>) -> ParseTable<'a> {
  let result = _lalr1_only(lr0, g);

  let mut action = Vec::with_capacity(lr0.len());
  let eof = g.eof();
  let start_id = g.start().1;
  let token_num = g.token_num();
  for (i, (state, link)) in lr0.iter().enumerate() {
    let mut act = HashMap::new();
    for (&k, &v) in link {
      if k < g.nt_num() {
        act.insert(k, smallvec![ParserAct::Goto(v)]);
      } else {
        act.insert(k, smallvec![ParserAct::Shift(v)]);
      }
    }
    for (item, (_, look_ahead)) in state.iter().zip(result[i].items.iter()) {
      if item.dot == item.prod.len() as u32 {
        if look_ahead.test(g.eof()  as usize) && item.prod_id == start_id {
          act.insert(eof, smallvec![ParserAct::Acc]);
        } else {
          for i in 0..token_num {
            if look_ahead.test(i  as usize) {
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

// the return type is the same with lr1::work
pub fn lalr1_only<'a>(lr0: &'a Vec<(Vec<LRItem<'a>>, HashMap<u32, u32>)>, g: &'a impl AbstractGrammarExt<'a>) -> Vec<LRResult<'a>> {
  let result = _lalr1_only(lr0, g);
  result.into_iter().zip(lr0.clone().into_iter()).map(|(state, (_, link))| (state, link)).collect()
}