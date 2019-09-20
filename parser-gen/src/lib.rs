use grammar_config::RawGrammar;

pub mod rs;
pub mod show_lr;
pub mod show_ll;

pub use rs::*;

pub(crate) fn min_u(x: u32) -> &'static str {
  match x { 0..=255 => "u8", 256..=65535 => "u16", _ => "u32", }
}

pub const INVALID_DFA: &str = "The merged dfa is not suitable for a lexer, i.e., it doesn't accept anything, or it accept empty string.";