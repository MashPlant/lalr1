use crate::lr1::*;
use crate::abstract_grammar::AbstractGrammarExt;
use std::collections::{HashMap, HashSet};
use crate::bitset::BitSet;
use smallvec::SmallVec;
use std::cmp::Ordering;
use crate::raw_grammar::Assoc;

#[derive(Debug, Copy, Clone)]
pub enum ParserAct {
  Acc,
  Shift(u32),
  Reduce(u32),
  // goto is for non-terminal, others are for terminal
  // so they can be together in one table
  Goto(u32),
}

#[derive(Debug)]
pub enum ConflictType {
  RR { r1: u32, r2: u32 },
  SR { s: u32, r: u32 },
}

#[derive(Debug)]
pub struct ConflictInfo {
  pub ty: ConflictType,
  pub state: u32,
  pub ch: u32,
}

#[derive(Debug)]
pub struct ParseTable<'a> {
  // in most cases there is no conflict, so use a small vec of inline capacity = 1
  pub action: Vec<(Vec<&'a LRItem<'a>>, HashMap<u32, SmallVec<[ParserAct; 1]>>)>,
  pub conflict: Vec<ConflictInfo>,
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

// Reference: https://docs.oracle.com/cd/E19504-01/802-5880/6i9k05dh3/index.html
// A precedence and associativity is associated with each grammar rule.
// It is the precedence and associativity of the **final token or literal** in the body of the rule.
// If the %prec construction is used, it overrides this default value.
// Some grammar rules may have no precedence and associativity associated with them.
//
// When there is a reduce-reduce or shift-reduce conflict, and **either** the input symbol or the grammar rule has no precedence and associativity,
// then the two default disambiguating rules given in the preceding section are used, and the **conflicts are reported**.
//   In a shift-reduce conflict, the default is to shift.
//   In a reduce-reduce conflict, the default is to reduce by the earlier grammar rule (in the yacc specification).
// If there is a shift-reduce conflict and both the grammar rule and the input character have precedence and associativity associated with them,
// then the conflict is resolved in favor of the action -- shift or reduce -- associated with the higher precedence.
// If precedences are equal, then associativity is used.
// Left associative implies reduce; right associative implies shift; nonassociating implies error.

fn solve_sr<'a>(state: u32, ch: u32, s: u32, r: u32, acts: &mut SmallVec<[ParserAct; 1]>, reports: &mut Vec<ConflictInfo>, g: &'a impl AbstractGrammarExt<'a>) -> bool {
  match (g.prod_pri_assoc(r), g.term_pri_assoc(ch)) {
    (Some((prod_pri, prod_assoc)), Some((ch_pri, ch_assoc))) => {
      match prod_pri.cmp(&ch_pri) {
        Ordering::Less => {
          *acts = smallvec![ParserAct::Shift(s)];
          true
        }
        Ordering::Greater => {
          *acts = smallvec![ParserAct::Reduce(r)];
          true
        }
        Ordering::Equal => {
          assert_eq!(prod_assoc, ch_assoc); // should I assert here? or if I don't assert, which to use?
          match prod_assoc {
            Assoc::Left => {
              *acts = smallvec![ParserAct::Reduce(r)];
              true
            }
            Assoc::Right => {
              *acts = smallvec![ParserAct::Shift(s)];
              true
            }
            Assoc::NoAssoc => false, // not retained
            Assoc::Token => unimplemented!("Why will you have a conflict here???")
          }
        }
      }
    }
    _ => {
      *acts = smallvec![ParserAct::Shift(s)];
      reports.push(ConflictInfo { ty: ConflictType::SR { s, r }, state, ch });
      true
    }
  }
}


fn try_solve_conflict<'a>(t: &mut Vec<(Vec<&'a LRItem<'a>>, HashMap<u32, SmallVec<[ParserAct; 1]>>)>, g: &'a impl AbstractGrammarExt<'a>) -> Vec<ConflictInfo> {
  let mut reports = Vec::new();
  for (state_id, state) in t.iter_mut().enumerate() {
    state.1.retain(|&ch, acts| {
      match acts.len() {
        1 => true,
        2 => {
          let (s, r) = (acts[0], acts[1]);
          match (s, r) {
            (ParserAct::Reduce(r1), ParserAct::Reduce(r2)) => {
              let used = r1.min(r2);
              *acts = smallvec![ParserAct::Reduce(used)];
              reports.push(ConflictInfo { ty: ConflictType::RR { r1, r2 }, state: state_id as u32, ch });
              true
            }
            (ParserAct::Reduce(r), ParserAct::Shift(s)) => solve_sr(state_id as u32, ch, s, r, acts, &mut reports, g),
            (ParserAct::Shift(s), ParserAct::Reduce(r)) => solve_sr(state_id as u32, ch, s, r, acts, &mut reports, g),
            _ => unreachable!("There should be a bug in lr process"),
          }
        }
        _ => unimplemented!("Why so many conflict???")
      }
    });
//    state.1.
  }
  reports
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
  let conflict = try_solve_conflict(&mut action, g);
  ParseTable { action, conflict }
}