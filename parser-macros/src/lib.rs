#![recursion_limit = "512"]
extern crate syn;
extern crate proc_macro;
extern crate quote;
extern crate lalr1_core;
extern crate re2dfa;
extern crate parser_gen;
extern crate grammar_config;
extern crate toml;
extern crate serde;
extern crate serde_derive;

use lalr1_core as lalr1;
use std::collections::HashMap;
use quote::ToTokens;
use grammar_config::{Assoc, AbstractGrammar, AbstractGrammarExt, RawPriorityRow, RawProduction, RawProductionRhs, RawGrammar};
use std::fmt::Write;
use serde::{Serialize, Deserialize};
use indexmap::IndexMap;

enum ArgInfo {
  Self_,
  Arg { name: Option<String>, ty: String },
}

fn parse_arg(arg: &syn::FnArg) -> ArgInfo {
  match arg {
    syn::FnArg::SelfRef(_) => ArgInfo::Self_,
    syn::FnArg::SelfValue(_) => ArgInfo::Self_,
    syn::FnArg::Captured(arg) => ArgInfo::Arg {
      name: Some(arg.pat.clone().into_token_stream().to_string()),
      ty: arg.ty.clone().into_token_stream().to_string(),
    },
    // what is this?
    syn::FnArg::Inferred(_) => unimplemented!("syn::FnArg::Inferred"),
    syn::FnArg::Ignored(ty) => ArgInfo::Arg { name: None, ty: ty.into_token_stream().to_string() }
  }
}

// is there a better way?
// I mean, if there is not such a api, since it is so frequently used, why do these libs not wrap them?
// and if there is, why is the document SO HARD to find?
fn parse_string(lit: &proc_macro2::Literal) -> String {
  let s = lit.to_string();
  s[s.find('\"').unwrap() + 1..s.rfind('\"').unwrap()].to_owned()
}

#[proc_macro_attribute]
pub fn lalr1(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let parser_impl = match syn::parse::<syn::ItemImpl>(input) {
    Ok(parser_impl) => parser_impl,
    Err(_) => panic!("Attribute `lalr1` can only be applied to an impl block."),
  };
  let parser_type = parser_impl.self_ty.as_ref();
  let parser_def = Some(parser_type.into_token_stream().to_string());

  #[derive(Debug, Deserialize, Serialize)]
  struct RawLexer {
    priority: Vec<RawPriorityRow>,
    lexical: IndexMap<String, String>,
  }
  let raw_lexer = match match parser_impl.attrs.iter().next() {
    Some(attr) if attr.path.is_ident("lex") => {
      if let Some(proc_macro2::TokenTree::Group(group)) = attr.tts.clone().into_iter().next() {
        let mut term_it = group.stream().into_iter();
        if let Some(proc_macro2::TokenTree::Literal(lit)) = term_it.next() {
          let path = parse_string(&lit);
          let cfg = std::fs::read_to_string(&path).expect(&format!("Fail to read {}.", path));
          // assume toml
          Some(toml::from_str::<RawLexer>(&cfg).expect(&format!("Fail to parse toml config of lexer in {}.", path)))
        } else { None }
      } else { None }
    }
    _ => None,
  } { Some(raw_lexer) => raw_lexer, None => panic!("Fail to parse lexer file path, expect `#[lex(PathToLexerConfig)]."), };
  let mut production = Vec::new();
  for item in &parser_impl.items {
    if let syn::ImplItem::Method(method) = item {
      let body = method.block.clone().into_token_stream().to_string();
      let attr = method.attrs.get(0).unwrap();
      let rule = attr.tts.to_string();
      let rule = rule[1..rule.len() - 1].trim();
      let mut rule_split = rule.split_whitespace();
      let lhs = match rule_split.next() { Some(lhs) => lhs.to_owned(), None => panic!("The rule `{}` method `{}` defined doesn't have a valid lhs.", rule, method.sig.ident), };
      let lhs_ty = match &method.sig.decl.output {
        // still may be unit here, but not checked...
        syn::ReturnType::Type(_, ty) => ty.into_token_stream().to_string(),
        _ => panic!("Semantic rule `{}` must return a value.", method.sig.ident),
      };
      match rule_split.next() { Some("->") => {} _ => panic!("The rule `{}` method `{}` defined doesn't have a `->`.", rule, method.sig.ident), };
      let rhs = rule_split.collect::<String>();
      let rhs_arg = method.sig.decl.inputs.iter().map(parse_arg).collect::<Vec<_>>();
      let skip_self = match rhs_arg.get(0) { Some(ArgInfo::Self_) => 1, _ => 0, };
      let rhs_arg = Some(rhs_arg.into_iter().skip(skip_self).map(|arg| match arg {
        ArgInfo::Self_ => panic!("Method `{}` takes self argument in illegal position.", method.sig.ident),
        ArgInfo::Arg { name, ty } => (name, ty)
      }).collect());
      let prec = if let Some(attr) = method.attrs.get(1) {
        match if attr.path.is_ident("prec") {
          match attr.tts.clone().into_iter().next() {
            Some(proc_macro2::TokenTree::Group(group)) => match group.stream().into_iter().next() {
              Some(proc_macro2::TokenTree::Ident(ident)) => Some(ident.to_string()),
              _ => None,
            }
            _ => None
          }
        } else { None } { Some(prec) => Some(prec), None => panic!("Fail to parse prec, expect `#[prec(Term)].") }
      } else { None };
      let act = method.block.clone().into_token_stream().to_string();
      production.push(RawProduction { lhs, type_: lhs_ty, rhs: vec![RawProductionRhs { rhs, rhs_arg, act, prec }] });
    } else { panic!("Impl block should only contain methods."); }
  }
  let raw = RawGrammar {
    include: "".into(),
    priority: raw_lexer.priority,
    lexical: raw_lexer.lexical,
    parser_field_ext: None,
    start: unimplemented!(),
    production,
    parser_def,
  };
  // TODO: macro -> RawGrammar -> error check by lalr1_core -> lalr1_core::Grammar -> String -> TokenStream
  unimplemented!()
}