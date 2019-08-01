use crate::{ParserAct, ConflictKind, Conflict, RawTable};
use std::cmp::Ordering;
use grammar_config::{Assoc, AbstractGrammarExt};
use smallvec::smallvec;

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

// `solve` will modify t in these ways:
// for conflicts solved based on precedence and/or associativity, other choices are removed
// for conflicts solved based on location or "shift better than reduced", other choices are NOT removed
// in both cases, the selected choice is placed at [0]
pub fn solve<'a>(t: &mut RawTable<'a>, g: &'a impl AbstractGrammarExt<'a>) -> Vec<Conflict> {
  use ParserAct::{Reduce, Shift};
  let mut reports = Vec::new();
  for (idx, t) in t.iter_mut().enumerate() {
    for (&ch, acts) in &mut t.act {
      match acts.len() {
        1 => {}
        2 => match (acts[0], acts[1]) {
          (Reduce(r1), Reduce(r2)) =>
            *acts = match (g.prod_pri(r1), g.prod_pri(r2)) {
              (Some(p1), Some(p2)) if p1 != p2 => smallvec![Reduce(if p1 < p2 { r2 } else { r1 })],
              _ => {
                reports.push(Conflict { kind: ConflictKind::RR { r1, r2 }, state: idx as u32, ch });
                smallvec![Reduce(r1.min(r2)), Reduce(r1.max(r2))]
              }
            },
          (Reduce(r), Shift(s)) | (Shift(s), Reduce(r)) =>
            *acts = match (g.prod_pri(r), g.term_pri_assoc(ch)) {
              (Some(pp), Some((cp, ca))) => match pp.cmp(&cp) {
                Ordering::Less => smallvec![Shift(s)],
                Ordering::Greater => smallvec![Reduce(r)],
                Ordering::Equal => match ca {
                  Assoc::Left => smallvec![Reduce(r)],
                  Assoc::Right => smallvec![Shift(s)],
                  Assoc::NoAssoc => smallvec![],
                }
              },
              _ => {
                reports.push(Conflict { kind: ConflictKind::SR { s, r }, state: idx as u32, ch });
                smallvec![Shift(s), Reduce(r)]
              }
            },
          _ => unreachable!("There should be a bug in lr."),
        }
        _ => reports.push(Conflict { kind: ConflictKind::Many(acts.clone()), state: idx as u32, ch }),
      }
    }
  }
  reports
}