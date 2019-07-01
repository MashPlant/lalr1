extern crate toml;
extern crate grammar_config;
extern crate lalr1_core;

use grammar_config::{RawGrammar, extend_grammar, AbstractGrammar};

fn main() {
  let decaf = include_str!("../../examples/decaf.toml");
  let mut raw = toml::from_str::<RawGrammar>(decaf).unwrap();
  let g = extend_grammar(&mut raw).unwrap();
  println!("{}", g.err());
  let lr0 = lalr1_core::lr0::work(&g);
  let _lalr1 = lalr1_core::lalr1_by_lr0::work(&lr0, &g);
//  let lr1 = lalr1_core::lr1::work(&g);
//  let _lalr1 = lalr1_core::lalr1_by_lr1::work(&lr1, &g);
}