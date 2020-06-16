#![feature(proc_macro_diagnostic)]
extern crate proc_macro;

use quote::ToTokens;
use proc_macro::{Diagnostic, Level, TokenStream};
use std::fs;
use lalr1_core::*;
use parser_gen::*;
use common::{grammar::*, IndexMap};
use syn::{FnArg, NestedMeta, ItemImpl, ImplItem, Attribute, ReturnType, Error};
use darling::FromMeta;
use std::fmt::{self, Display};
use ll1_core::LLCtx;
use re2dfa::Dfa;

fn parse_arg(arg: &FnArg) -> Option<(String, String)> {
  match arg {
    FnArg::Receiver(_) => None,
    FnArg::Typed(pat) => Some((pat.to_token_stream().to_string(), pat.ty.to_token_stream().to_string(), ))
  }
}

#[derive(FromMeta)]
struct Config {
  lex: String,
  #[darling(default)] verbose: Option<String>,
  #[darling(default)] show_fsm: Option<String>,
  #[darling(default)] show_dfa: Option<String>,
  #[darling(default)] log_token: bool,
  #[darling(default)] log_reduce: bool,
  #[darling(default)] use_unsafe: bool,
  #[darling(default)] expand: bool,
  #[darling(skip)] code: String,
}

impl Config {
  fn codegen(&self) -> RustCodegen {
    RustCodegen { log_token: self.log_token, log_reduce: self.log_reduce, use_unsafe: self.use_unsafe, show_token_prod: self.verbose.is_some() }
  }
}

impl Codegen for Config {
  fn dfa_ec(&mut self, dfa_ec: &(Dfa, [u8; 256])) {
    if let Some(path) = self.show_dfa.as_ref() {
      fs::write(path, dfa_ec.0.print_dot()).unwrap_or_else(|e| panic!("failed to write dfa into \"{}\": {}", path, e));
    }
  }

  fn ll(&mut self, g: &Grammar, ll: LLCtx, dfa_ec: &(Dfa, [u8; 256])) {
    if let Some(path) = self.verbose.as_ref() {
      fs::write(path, show_ll::table(&ll, g)).unwrap_or_else(|e| panic!("failed to write ll1 table into \"{}\": {}", path, e));
    }
    for c in show_ll::conflict(&ll.table, g) { Diagnostic::new(Level::Warning, c).emit(); }
    self.code = self.codegen().gen_ll1(&g, &ll, &dfa_ec.0, &dfa_ec.1).unwrap_or_else(|| panic!(INVALID_DFA));
  }

  // won't be called
  fn lr0(&mut self, _g: &Grammar, _lr0: Lr0Fsm) {}

  fn lr1(&mut self, g: &Grammar, lr1: &Lr1Fsm, dfa_ec: &(Dfa, [u8; 256]), orig_table: Table, table: Table, conflict: Vec<Conflict>) {
    if let Some(path) = self.verbose.as_ref() {
      fs::write(path, show_lr::table(&orig_table, &table, g)).unwrap_or_else(|e| panic!("failed to write lr1 information into \"{}\": {}", path, e));
    }
    if let Some(path) = self.show_fsm.as_ref() {
      fs::write(path, show_lr::lr1_dot(g, &lr1)).unwrap_or_else(|e| panic!("failed to write lr1 fsm into \"{}\": {}", path, e));
    }
    for c in show_lr::conflict(g, &conflict) { Diagnostic::new(Level::Warning, c).emit(); }
    if conflict.iter().any(Conflict::is_many) { panic!(">= 3 conflicts on one token, give up solving conflicts"); }
    self.code = self.codegen().gen_lalr1(&g, &table, &dfa_ec.0, &dfa_ec.1).unwrap_or_else(|| panic!(INVALID_DFA));
  }
}

// part of RawGrammar
#[derive(serde::Deserialize)]
struct RawLexer {
  priority: Vec<RawPriorityRow>,
  lexical: IndexMap<String, String>,
}

#[derive(FromMeta)]
struct Rule {
  rule: String,
  #[darling(default)] prec: Option<String>,
}

struct PrettyError(Error);

impl Display for PrettyError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let lc = self.0.span().start();
    write!(f, "{} at {}:{}", self.0, lc.line, lc.column)
  }
}

fn parse_attrs(attrs: &[Attribute]) -> Vec<NestedMeta> {
  attrs.iter().map(|x| NestedMeta::Meta(x.parse_meta()
    .unwrap_or_else(|e| panic!("failed to parse meta: {}", PrettyError(e))))).collect()
}

fn work(attr: TokenStream, input: TokenStream, algo: PGAlgo) -> TokenStream {
  let parser = syn::parse::<ItemImpl>(input).unwrap_or_else(|e| panic!("failed to parse impl block: {}", PrettyError(e)));
  let start = attr.to_string();
  let parser_def = Some(parser.self_ty.to_token_stream().to_string());

  let mut cfg = Config::from_list(&parse_attrs(&parser.attrs)).unwrap_or_else(|e| panic!("failed to read attributes: {}", e));
  let RawLexer { priority, lexical } =
    toml::from_str::<RawLexer>(&cfg.lex).unwrap_or_else(|e| panic!("fail to parse lexer toml: {}", e));

  let mut production = Vec::new();
  for item in &parser.items {
    if let ImplItem::Method(method) = item {
      let Rule { rule, prec } = Rule::from_list(&parse_attrs(&method.attrs)).unwrap_or_else(|e| panic!("failed to parse rule: {}", e));
      let mut sp = rule.split_whitespace();
      let lhs = match sp.next() { Some(lhs) => lhs.to_owned(), None => panic!("rule \"{}\" of method `{}` has no lhs", rule, method.sig.ident), };
      let lhs_ty = match &method.sig.output {
        ReturnType::Default => "()".to_owned(),
        ReturnType::Type(_, ty) => ty.to_token_stream().to_string(),
      };
      match sp.next() { Some("->") => {} _ => panic!("rule \"{}\" of method `{}` has no \"->\"", rule, method.sig.ident), };
      let rhs = sp.map(|s| s.to_owned()).collect();
      let rhs_arg = method.sig.inputs.iter().map(parse_arg).collect::<Vec<_>>();
      let skip_self = match rhs_arg.get(0) { Some(None) => 1, _ => 0, };
      let rhs_arg = Some(rhs_arg.into_iter().skip(skip_self).map(|arg| match arg {
        None => panic!("method `{}` takes `self` at illegal position", method.sig.ident),
        Some((pat, name)) => (Some(pat), name),
      }).collect());
      let act = method.block.to_token_stream().to_string();
      production.push(RawProduction { lhs, type_: lhs_ty, rhs: vec![RawProductionRhs { rhs, rhs_arg, act, prec }] });
    } else { panic!("only support method impl, found {:?}", item); }
  }

  parser_gen::work(RawGrammar { include: String::new(), priority, lexical, parser_field: None, start, production, parser_def }, algo, &mut cfg);
  if cfg.expand { println!("{}", cfg.code); }
  cfg.code.parse().unwrap()
}

#[proc_macro_attribute]
pub fn lalr1(attr: TokenStream, input: TokenStream) -> TokenStream {
  work(attr, input, PGAlgo::LALR1)
}

#[proc_macro_attribute]
pub fn ll1(attr: TokenStream, input: TokenStream) -> TokenStream {
  work(attr, input, PGAlgo::LL1)
}