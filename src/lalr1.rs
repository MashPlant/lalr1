use crate::lr1::*;
use crate::abstract_grammar::AbstractGrammarExt;
use std::collections::{HashMap, HashSet};
use crate::bitset::BitSet;
use smallvec::SmallVec;

#[derive(Debug)]
pub enum ParserAct {
  Acc,
  Shift(u32),
  Reduce(u32),
  // goto is for non-terminal, others are for terminal
  // so they can be together in one table
  Goto(u32),
}

#[derive(Debug)]
pub struct ParseTable<'a> {
  // in most cases there is no conflict, so use a small vec of inline capacity = 1
  pub action: Vec<(Vec<&'a LRItem<'a>>, HashMap<u32, SmallVec<[ParserAct; 1]>>)>,
  pub conflict: Option<Vec<u32>>,
}

// the difference with LR1State is that LRItem is a reference
struct LALR1State<'a> {
  items: Vec<(&'a LRItem<'a>, BitSet)>,
}

fn get_lalr1_table<'a>(lr: &'a Vec<LRResult<'a>>, g: &'a impl AbstractGrammarExt<'a>) -> Vec<(LALR1State<'a>, HashMap<u32, u32>)> {
  // lalr1 state -> id(in lalr1 states) + corresponding lr1 states
  let mut states = HashMap::new();
  let mut rev_states = HashMap::new();
  for (state, link) in lr {
    let id = states.len() as u32;
    let v = states.entry(state as *const LRState).or_insert_with(|| (id, Vec::new()));
    v.1.push((state, link));
    rev_states.insert(state as *const LRState, v.0);
  }
  let mut result = Vec::new();
  for (state, (id, old_states)) in states {
    let mut new_state = LALR1State { items: old_states[0].0.items.iter().map(|(state, look_ahead)| (state, look_ahead.clone())).collect() };
    for &(old_state, _) in &old_states[1..] {
      for (it1, it2) in new_state.items.iter_mut().zip(old_state.items.iter()) {
        it1.1.or(&it2.1);
      }
    }
    let mut new_link = HashMap::new();
    for (old_state, old_link) in old_states {
      for (&k, &v) in old_link {
        new_link.insert(k, *rev_states.get(&(old_state as *const _)).unwrap());
      }
    }
    result.push((new_state, new_link));
  }
  result
}

// return whether success in solving conflict
fn try_solve_conflict<'a>(t: &mut Vec<(Vec<&'a LRItem<'a>>, HashMap<u32, SmallVec<[ParserAct; 1]>>)>, g: &'a impl AbstractGrammarExt<'a>) -> bool {
  for state_acts in t {}
//  unimplemented!()
  true
}

// merge lr1 states, and try to solve conflict using g's information
pub fn work<'a>(lr: &'a Vec<LRResult<'a>>, g: &'a impl AbstractGrammarExt<'a>) -> ParseTable<'a> {
  let lalr1_table = get_lalr1_table(lr, g);
  let mut action = Vec::with_capacity(lalr1_table.len());
  let (nt_num, token_num, eof) = (g.nt_num(), g.token_num(), g.eof());
  for (i, (state, link)) in lalr1_table.iter().enumerate() {
    let mut act = HashMap::new();
    for (&k, &v) in link {
      if k < nt_num {
        act.insert(k, smallvec![ParserAct::Goto(v)]);
      } else {
        act.insert(k, smallvec![ParserAct::Shift(v)]);
      }
    }
    let start_id = g.start().1;
    for (item, look_ahead) in &state.items {
      if item.dot == item.prod.len() as u32 {
        if look_ahead.test(eof) && item.prod_id == start_id {
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
    action.push((state.items.iter().map(|item| item.0).collect(), act));
  }
  try_solve_conflict(&mut action, g);
  ParseTable { action, conflict: None }
}