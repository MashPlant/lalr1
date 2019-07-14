#![feature(proc_macro_hygiene)]
extern crate parser_macros;
extern crate lazy_static;
extern crate hashbrown;

mod lalr1;
mod ll1;
mod lifetime;
mod test;