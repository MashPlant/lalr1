use crate::lr1::*;
use crate::lr0::LRItem;
use grammar_config::AbstractGrammarExt;
use std::collections::HashMap;
use crate::bitset::BitSet;
use smallvec::SmallVec;
use std::hash::{Hash, Hasher};
use crate::lalr1_common::*;

struct LALR1State<'a> {
  items: Vec<(&'a LRItem<'a>, BitSet)>,
}

struct LRCore<'a> {
  state: &'a LRState<'a>,
}

impl Hash for LRCore<'_> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    for (item, _) in &self.state.items {
      item.hash(state);
    }
  }
}

impl PartialEq for LRCore<'_> {
  fn eq(&self, other: &LRCore) -> bool {
    self.state.items.len() == other.state.items.len() &&
      self.state.items.iter().zip(other.state.items.iter()).all(|(l, r)| l.0 == r.0)
  }
}

impl Eq for LRCore<'_> {}

fn get_lalr1_table<'a>(lr: &'a Vec<LRResult<'a>>) -> Vec<(LALR1State<'a>, HashMap<u32, u32>)> {
  // lalr1 state -> id(in lalr1 states) + corresponding lr1 states
  let mut states = HashMap::new();
  let mut rev_states = HashMap::new();
  for (state, link) in lr {
    let id = states.len() as u32;
    let v = states.entry(LRCore { state }).or_insert_with(|| (id, Vec::new()));
    v.1.push((state, link));
    rev_states.insert(state as *const LRState, v.0);
  }
  let mut states = states.into_iter().collect::<Vec<_>>();
  states.sort_unstable_by(|l, r| (l.1).0.cmp(&(r.1).0));

  let mut result = Vec::new();
  for (_state, (_id, old_states)) in states {
    let mut new_state = LALR1State { items: old_states[0].0.items.iter().map(|(state, look_ahead)| (state, look_ahead.clone())).collect() };
    for &(old_state, _) in &old_states[1..] {
      for (it1, it2) in new_state.items.iter_mut().zip(old_state.items.iter()) {
        it1.1.or(&it2.1);
      }
    }
    let mut new_link = HashMap::new();
    for (_, old_link) in old_states {
      for (&k, &v) in old_link {
        let old_to = &lr[v as usize].0;
        let new_to = rev_states[&(old_to as *const _)];
        new_link.insert(k, new_to);
      }
    }
    result.push((new_state, new_link));
  }
  result
}

// merge lr1 states, and try to solve conflict using g's information
pub fn work<'a>(lr: &'a Vec<LRResult<'a>>, g: &'a impl AbstractGrammarExt<'a>) -> ParseTable<'a> {
  let lalr1_table = get_lalr1_table(lr);
  let mut action = Vec::with_capacity(lalr1_table.len());
  let (nt_num, token_num, eof) = (g.nt_num(), g.token_num(), g.eof());
  for (_, (state, link)) in lalr1_table.iter().enumerate() {
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
  let conflict = try_solve_conflict(&mut action, g);
  ParseTable { action, conflict }
}