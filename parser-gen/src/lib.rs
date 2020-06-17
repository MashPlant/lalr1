pub mod rs;
pub mod show_lr;
pub mod show_ll;
pub mod workflow;

pub use rs::*;

// parse a "lhs -> rhs1 rhs2 ..." string
pub fn parse_arrow_prod(s: &str) -> Option<(String, Vec<String>)> {
  let mut sp = s.split_whitespace();
  let lhs = sp.next()?.to_owned();
  match sp.next() { Some("->") => {} _ => return None };
  let rhs = sp.map(|s| s.to_owned()).collect();
  Some((lhs, rhs))
}

pub(crate) fn min_u(x: usize) -> &'static str {
  // I don't think any number beyond `u32` is really possible
  match x { 0..=255 => "u8", 256..=65535 => "u16", _ => "u32" }
}

pub const INVALID_DFA: &str = "final dfa is not suitable for a lexer, i.e., it doesn't accept anything, or it accepts empty string";