#![feature(vec_resize_with)]
#![allow(unused)]
extern crate toml;
extern crate serde;
extern crate serde_derive;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate fixedbitset;

mod printer;
mod parser;
mod raw_grammar;
mod grammar;
mod lalr1;
mod bitset;

use std::fs::read_to_string;

fn main() {
//  let prog = read_to_string("test.decaf").unwrap();
//  let mut lex = parser::Lexer::new(&prog);
//  while let Some(tk) = lex.next() {
//    println!("{:?}", tk);
//  }
//  let s = read_to_string("decaf.toml").unwrap();
//  let g: RawGrammar = toml::from_str(&s).unwrap();
//  let g = g.to_grammar().unwrap();
//  println!("{}", g.gen());
}