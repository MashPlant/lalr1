use crate::{Lr0Item, Acts};
use std::cmp::Ordering;
use hashbrown::HashMap;
use grammar_config::{Assoc, AbstractGrammarExt};
use smallvec::smallvec;

#[derive(Debug, Copy, Clone)]
pub enum ParserAct {
  Acc,
  Shift(u32),
  Reduce(u32),
  // goto is for non-terminal, others are for terminal
  // so they can be together in one table
  Goto(u32),
}

pub enum ConflictType {
  RR { r1: u32, r2: u32 },
  SR { s: u32, r: u32 },
  Many(Acts),
}

pub struct ConflictInfo {
  pub ty: ConflictType,
  pub state: u32,
  pub ch: u32,
}

pub struct LRTable<'a> {
  pub action: Vec<(Vec<&'a Lr0Item<'a>>, HashMap<u32, Acts>)>,
  pub conflict: Vec<ConflictInfo>,
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

pub fn solve_sr<'a>(state: u32, ch: u32, s: u32, r: u32, acts: &mut Acts, reports: &mut Vec<ConflictInfo>, g: &'a impl AbstractGrammarExt<'a>) -> bool {
  *acts = match (g.prod_pri(r), g.term_pri_assoc(ch)) {
    (Some(prod_pri), Some((ch_pri, ch_assoc))) => {
      match prod_pri.cmp(&ch_pri) {
        Ordering::Less => smallvec![ParserAct::Shift(s)],
        Ordering::Greater => smallvec![ParserAct::Reduce(r)],
        Ordering::Equal => match ch_assoc {
          Assoc::Left => smallvec![ParserAct::Reduce(r)],
          Assoc::Right => smallvec![ParserAct::Shift(s)],
          Assoc::NoAssoc => return false,
        }
      }
    }
    _ => {
      reports.push(ConflictInfo { ty: ConflictType::SR { s, r }, state, ch });
      smallvec![ParserAct::Shift(s)]
    }
  };
  true
}

pub fn try_solve_conflict<'a>(t: &mut Vec<(Vec<&'a Lr0Item<'a>>, HashMap<u32, Acts>)>, g: &'a impl AbstractGrammarExt<'a>) -> Vec<ConflictInfo> {
  let mut reports = Vec::new();
  for (state_id, state) in t.iter_mut().enumerate() {
    state.1.retain(|&ch, acts| {
      match acts.len() {
        1 => true,
        2 => {
          match (acts[0], acts[1]) {
            (ParserAct::Reduce(r1), ParserAct::Reduce(r2)) => {
              let used = r1.min(r2);
              *acts = smallvec![ParserAct::Reduce(used)];
              reports.push(ConflictInfo { ty: ConflictType::RR { r1, r2 }, state: state_id as u32, ch });
              true
            }
            (ParserAct::Reduce(r), ParserAct::Shift(s)) | (ParserAct::Shift(s), ParserAct::Reduce(r)) =>
              solve_sr(state_id as u32, ch, s, r, acts, &mut reports, g),
            _ => unreachable!("There should be a bug in lr."),
          }
        }
        _ => {
          reports.push(ConflictInfo { ty: ConflictType::Many(acts.clone()), state: state_id as u32, ch });
          false // it doesn't matter whether this edge is retained, because no code can be generated
        }
      }
    });
  }
  reports
}