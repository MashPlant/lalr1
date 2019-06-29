extern crate syn;
extern crate proc_macro;
extern crate lalr1_core;
extern crate re2dfa;
extern crate parser_gen;
extern crate grammar_config;
extern crate toml;
extern crate serde;
extern crate serde_derive;
extern crate quote;

use quote::ToTokens;
use grammar_config::{RawPriorityRow, RawProduction, RawProductionRhs, RawGrammar};
use serde::{Serialize, Deserialize};
use indexmap::IndexMap;
use parser_gen::RustCodegen;

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
  let start = match attr.clone().into_iter().next() {
    Some(proc_macro::TokenTree::Ident(ident)) => ident.to_string(),
    _ => panic!("Fail to parse start non-term, expect `#[lalr1(StartName)]."),
  };
  let parser_type = parser_impl.self_ty.as_ref();
  let parser_def = Some(parser_type.into_token_stream().to_string());

  // part of RawGrammar
  #[derive(Debug, Deserialize, Serialize)]
  struct RawLexer {
    priority: Vec<RawPriorityRow>,
    lexical: IndexMap<String, String>,
  }
  let raw_lexer = (match parser_impl.attrs.iter().next() {
    Some(attr) if attr.path.is_ident("lex") => {
      if let Some(proc_macro2::TokenTree::Group(group)) = attr.tts.clone().into_iter().next() {
        let mut term_it = group.stream().into_iter();
        if let Some(proc_macro2::TokenTree::Literal(lit)) = term_it.next() {
          let cfg = parse_string(&lit);
          // assume toml
          Some(toml::from_str::<RawLexer>(&cfg).unwrap_or_else(|err| panic!("Fail to parse toml config of lexer, reason: `{}`.", err)))
        } else { None }
      } else { None }
    }
    _ => None,
  }).unwrap_or_else(|| panic!("Fail to parse lexer file path, expect `#[lex(PathToLexerConfig)]."));
  let mut production = Vec::new();
  for item in &parser_impl.items {
    if let syn::ImplItem::Method(method) = item {
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
      // is there a better way to get the remain part(with spaces) of this iterator?
      let rhs = rule_split.map(|s| {
        let mut s = s.to_owned();
        s.push(' ');
        s
      }).collect::<String>();
      let rhs_arg = method.sig.decl.inputs.iter().map(parse_arg).collect::<Vec<_>>();
      let skip_self = match rhs_arg.get(0) { Some(ArgInfo::Self_) => 1, _ => 0, };
      let rhs_arg = Some(rhs_arg.into_iter().skip(skip_self).map(|arg| match arg {
        ArgInfo::Self_ => panic!("Method `{}` takes self argument at illegal position.", method.sig.ident),
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
  let mut raw = RawGrammar {
    include: "".into(),
    priority: raw_lexer.priority,
    lexical: raw_lexer.lexical,
    parser_field: None,
    start: Some(start),
    production,
    parser_def,
  };
  let (dfa, ec) = re2dfa::re2dfa(raw.lexical.iter().map(|(k, _)| k))
    .unwrap_or_else(|(idx, reason)| panic!("Invalid regex {}, reason: {}.", raw.lexical.get_index(idx).unwrap().0, reason));
  let g = grammar_config::extend_grammar(&mut raw)
    .unwrap_or_else(|err| panic!("Grammar is invalid, reason: `{}`.", err));
  let lr0 = lalr1_core::lr0::work(&g);
  let table = lalr1_core::lalr1_by_lr0::work(&lr0, &g);
  for _conflict in &table.conflict {
    unimplemented!()
  }
  let code = RustCodegen { log_token: false, log_reduce: false }.gen_lalr1(&g, &table, &dfa, &ec);
  code.parse().unwrap()
}