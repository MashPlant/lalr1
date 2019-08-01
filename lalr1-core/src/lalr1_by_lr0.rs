// "Compilers: Principles, Techniques and Tools" Algorithm 4.63

use crate::{lr1::LRCtx, Lr1Item, Lr1Closure, ParserAct, LrFsm, LrNode, TableItem, RawTable};
use grammar_config::AbstractGrammarExt;
use hashbrown::HashMap;
use smallvec::{SmallVec, smallvec};
use bitset::BitSet;

// inner version, the return value doesn't contain `link`
fn lalr1_by_lr0<'a>(lr0: &'a LrFsm<'a>, g: &'a impl AbstractGrammarExt<'a>) -> Vec<Lr1Closure<'a>> {
  let mut ctx = LRCtx::new(g);
  let mut lookahead = lr0.iter()
    .map(|LrNode { items, .. }| vec![BitSet::new(ctx.token_num); items.len()]).collect::<Vec<_>>();
  let mut prop = Vec::new();
  let start_prod = (g.start().1).0.as_ref();

  for (i, item) in lr0[0].items.iter().enumerate() {
    if item.prod == start_prod {
      lookahead[0][i].set(g.eof() as usize);
      break;
    }
  }

  let special_term = g.err() as usize;
  for (i, LrNode { items: state, link }) in lr0.iter().enumerate() {
    for (item_id, &item) in state.iter().enumerate() {
      // only consider lr0 core item
      if item.prod == start_prod || item.dot != 0 {
        let cl = ctx.closure({
                               let mut lookahead = BitSet::new(ctx.token_num);
                               lookahead.set(special_term);
                               let mut init = HashMap::new();
                               init.insert(item, lookahead);
                               init
                             }, g);
        let from = lookahead[i][item_id].as_ptr();
        for Lr1Item { item: cl_item, lookahead: cl_lookahead } in &cl {
          if cl_item.dot as usize >= cl_item.prod.len() {
            continue;
          }
          let goto_state = link[&cl_item.prod[cl_item.dot as usize]];
          let goto_item_id = cl_item.unique_id() + 1; // dot + 1
          let goto_item_idx = lr0[goto_state as usize].items.iter().enumerate().find(|item| item.1.unique_id() == goto_item_id).unwrap().0;
          let goto_lookahead = &mut lookahead[goto_state as usize][goto_item_idx];
          goto_lookahead.or(&cl_lookahead);
          if cl_lookahead.test(special_term) {
            prop.push((from, goto_lookahead.as_mut_ptr()));
          }
        }
      }
    }
  }

  let mut changed = true;
  let len = lookahead[0][0].inner_len();
  while changed {
    changed = false;
    unsafe {
      for &(from, to) in &prop {
        changed |= BitSet::or_raw(to, from, len);
      }
    }
  }

  lookahead.iter_mut().for_each(|l| l.iter_mut().for_each(|l| l.clear(special_term)));

  lr0.iter().zip(lookahead.into_iter()).map(|(node, lookahead_s)| {
    ctx.closure(node.items.clone().into_iter().zip(lookahead_s.into_iter()).collect(), g)
  }).collect::<Vec<_>>()
}

pub fn work<'a>(lr0: &'a LrFsm<'a>, g: &'a impl AbstractGrammarExt<'a>) -> RawTable<'a> {
  let result = lalr1_by_lr0(lr0, g);
  let mut table = Vec::with_capacity(lr0.len());
  let eof = g.eof();
  let start_id = (g.start().1).1;
  let token_num = g.token_num();
  for (i, LrNode { items: state, link }) in lr0.iter().enumerate() {
    let mut act = HashMap::new();
    for (&k, &v) in link {
      if k < g.nt_num() {
        act.insert(k, smallvec![ParserAct::Goto(v)]);
      } else {
        act.insert(k, smallvec![ParserAct::Shift(v)]);
      }
    }
    for (item, Lr1Item { lookahead, .. }) in state.iter().zip(result[i].iter()) {
      if item.dot == item.prod.len() as u32 {
        if lookahead.test(eof as usize) && item.prod_id == start_id {
          act.insert(eof, smallvec![ParserAct::Acc]);
        } else {
          for i in 0..token_num {
            if lookahead.test(i as usize) {
              // maybe conflict here
              act.entry(i).or_insert_with(|| SmallVec::new()).push(ParserAct::Reduce(item.prod_id));
            }
          }
        }
      }
    }
    table.push(TableItem { items: state, act });
  }

  table
}
