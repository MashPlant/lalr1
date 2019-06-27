use std::cmp::Ordering;
use std::collections::HashMap;
use crate::raw_grammar::Assoc;
use crate::lr0::LRItem;
use crate::abstract_grammar::AbstractGrammarExt;
use smallvec::SmallVec;

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

pub fn solve_sr<'a>(state: u32, ch: u32, s: u32, r: u32, acts: &mut SmallVec<[ParserAct; 1]>, reports: &mut Vec<ConflictInfo>, g: &'a impl AbstractGrammarExt<'a>) -> bool {
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

pub fn try_solve_conflict<'a>(t: &mut Vec<(Vec<&'a LRItem<'a>>, HashMap<u32, SmallVec<[ParserAct; 1]>>)>, g: &'a impl AbstractGrammarExt<'a>) -> Vec<ConflictInfo> {
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
  }
  reports
}