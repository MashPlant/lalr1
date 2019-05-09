use crate::grammar::Grammar;
use crate::printer::IndentPrinter;
use crate::raw_grammar::RawLexerFieldExt;
use crate::lalr1::ParseTable;
use std::collections::HashSet;
use crate::abstract_grammar::AbstractGrammar;

pub trait Codegen {
  fn gen(&self, g: &Grammar, table: &ParseTable) -> String;
}

pub struct RustCodegen;

impl Codegen for RustCodegen {
  fn gen(&self, g: &Grammar, table: &ParseTable) -> String {
    let mut p = IndentPrinter::new();
    p.ln(r#"#![allow(unused)]
#![allow(unused_mut)]

use regex::Regex;
use std::collections::HashMap;"#).ln("");

    p.lns(r#"#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType {"#).inc();
    for &(nt, _) in &g.nt {
      p.ln(format!("{},", nt));
    }
    for &(t, _) in &g.terminal {
      p.ln(format!("{},", t));
    }
    p.dec().ln("}\n");

    p.lns(r#"#[derive(Debug, Clone, Copy)]
pub enum LexerState {"#).inc();
    for (i, &state) in g.lex_state.iter().enumerate() {
      p.ln(format!("{} = {},", state, i));
    }
    p.dec().ln("}\n");

    p.lns(r#"macro_rules! map (
  { $($key:expr => $value:expr),+ } => {{
    let mut m = ::std::collections::HashMap::new();
    $( m.insert($key, $value); )+
    m
  }};
);"#).ln("");

    p.ln("lazy_static! {").inc();
    p.ln(format!("static ref LEX_RULES: [Vec<(Regex, fn(&mut Lexer) -> TokenType)>; {}] = [", g.lex.len())).inc();
    {
      let mut cnt = 0;
      for lex_state_rules in &g.lex {
        p.ln("vec![").inc();
        for (re, _, _) in lex_state_rules {
          // add enough # to prevent the re contains `#"`
          let raw = "#".repeat(re.matches('#').count() + 1);
          p.ln(format!(r#"(Regex::new(r{}"^{}"{}).unwrap(), lex_act{}),"#, raw, &re, raw, cnt));
          cnt += 1;
        }
        p.dec().ln("],");
      }
    }
    p.dec().ln("];\n"); // LEX_RULES
    p.ln(format!("static ref TABLE: [HashMap<u32, Act>; {}] = [", table.action.len())).inc();
    for act in &table.action {
      let mut map = "map! { ".to_owned();
      // manually join...
      // rust's join seems still unstable now?
      for (i, (&link, act)) in act.1.iter().enumerate() {
        if i == 0 {
          map += &format!("{} => Act::{:?}", link, act[0]);
        } else {
          map += &format!(", {} => Act::{:?}", link, act[0]);
        }
      }
      map += " },";
      p.ln(map);
    }
    p.dec().ln("];"); // TABLE
    p.dec().ln("}\n"); // lazy_static

    p.ln(format!("static PRODUCTION_ACT: [fn(&mut Parser); {}] = [", g.prod_extra.len())).inc();
    for i in 0..g.prod_extra.len() {
      p.ln(format!("parser_act{}, ", i));
    }
    p.dec().ln("];"); // TABLE

    p.lns(r#"#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
  pub ty: TokenType,
  pub piece: &'a str,
  pub line: u32,
  pub col: u32,
}
"#).ln("");

    p.lns(r#"pub struct Lexer<'a> {
  pub string: &'a str,
  pub states: Vec<LexerState>,
  pub cur_line: u32,
  pub cur_col: u32,
  pub piece: &'a str,"#).inc();
    if let Some(ext) = &g.raw.lexer_field_ext {
      for RawLexerFieldExt { field, type_, init: _ } in ext {
        p.ln(format!("pub {}: {},", field, type_));
      }
    }
    p.dec().ln("}\n");

    p.ln("impl Lexer<'_> {").inc();
    p.lns(r#"pub fn new(string: &str) -> Lexer {"#).inc();
    p.lns(r#"Lexer {
  string,
  states: vec![LexerState::_Initial],
  cur_line: 1,
  cur_col: 0,
  piece: "","#).inc();
    if let Some(ext) = &g.raw.lexer_field_ext {
      for RawLexerFieldExt { field, type_: _, init } in ext {
        p.ln(format!("{}: {},", field, init));
      }
    }
    p.dec().ln("}"); // Lexer
    p.dec().ln("}\n"); // new

    p.lns(r#"pub fn next(&mut self) -> Option<Token> {
  loop {
    if self.string.is_empty() {
      return Some(Token { ty: TokenType::_Eof, piece: "", line: self.cur_line, col: self.cur_col });
    }
    let mut max: Option<(&str, fn(&mut Lexer) -> TokenType)> = None;
    for (re, act) in &LEX_RULES[*self.states.last()? as usize] {
      match re.find(self.string) {
        None => {}
        Some(n) => {
          let n = n.as_str();
          if match max {
            None => true,
            Some((o, _)) => o.len() < n.len(),
          } { max = Some((n, *act)); }
        }
      }
    }
    let (piece, act) = max?;
    self.piece = piece;
    let ty = act(self);
    self.string = &self.string[piece.len()..];
    let (line, col) = (self.cur_line, self.cur_col);
    for (i, l) in piece.split('\n').enumerate() {
      self.cur_line += 1;
      if i == 0 {
        self.cur_col += l.len() as u32;
      } else {
        self.cur_col = l.len() as u32;
      }
    }
    if ty != TokenType::_Eps {
      break Some(Token { ty, piece, line, col });
    }
  }
}"#);
    p.dec().ln("}\n"); // impl

    p.lns("enum Act {
  Acc,
  Shift(u32),
  Reduce(u32),
  Goto(u32),
}").ln("");

    p.ln("enum StackItem {").inc();
    p.ln("_Token(Token)");
    let types = g.nt.iter().map(|(_, ty)| ty).collect::<HashSet<_>>();
    for ty in types {
      p.ln(format!("{}({}),", ty, ty));
    }
    p.dec().ln("}\n");

//    p.ln(format!("type ParseResult = {};\n", g.nt[AbstractGrammar::start(g).1 as usize].1));

    p.lns(r#"pub struct Parser<'a> {
  value_stk: Vec<StackItem>,
  state_stk: Vec<u32>,
  lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
  pub fn new(string: &'a str) -> Parser {
    Parser {
      value_stk: Vec::new(),
      state_stk: vec![0],
      lexer: Lexer::new(string),
    }
  }

  pub fn parse(&mut self) -> TResult {
    let mut token = self.lexer.next();
    let mut shifted_token = token;

    loop {
      let state = *self.state_stk.last().unwrap();
      let column = token.ty;

      if !TABLE[state].contains_key(&column) {
        self.unexpected_token(&token);
        break;
      }

      let entry = &TABLE[state][&column];

      match entry {

        // Shift a token, go to state.
        &Act::Shift(next_state) => {
          // Push token.
          self.value_stk.push(SV::_0(token));

          // Push next state number: "s5" -> 5
          self.state_stk.push(next_state as usize);

          shifted_token = token;
          token = self.tokenizer.get_next_token();
        }

        // Reduce by production.
        &Act::Reduce(production_number) => {
          let production = PRODUCTIONS[production_number];

          self.tokenizer.yytext = shifted_token.value;
          self.tokenizer.yyleng = shifted_token.value.len();

          let mut rhs_length = production[1];
          while rhs_length > 0 {
            self.state_stk.pop();
            rhs_length = rhs_length - 1;
          }

          // Call the handler, push result onto the stack.
          let result_value = self.handlers[production_number](self);

          let previous_state = *self.state_stk.last().unwrap();
          let symbol_to_reduce_with = production[0];

          // Then push LHS onto the stack.
          self.value_stk.push(result_value);

          let next_state = match &TABLE[previous_state][&symbol_to_reduce_with] {
            &Act::Goto(next_state) => next_state,
            _ => unreachable!(),
          };

          self.state_stk.push(next_state);
        }

        // Accept the string.
        &Act::Acc => {
          // Pop state number.
          self.state_stk.pop();

          // Pop the parsed value.
          let parsed = self.value_stk.pop().unwrap();

          if self.state_stk.len() != 1 ||
            self.state_stk.pop().unwrap() != 0 ||
            self.tokenizer.has_more_tokens() {
            self.unexpected_token(&token);
          }

          let result = get_result!(parsed, {{{RESULT_TYPE}}});
          return result;
        }

        _ => unreachable!(),
      }
    }
    unreachable!();
  }

  fn unexpected_token(&self, _token: &Token) {
    panic!("unexpected_token");
  }
}"#).ln("");

    {
      let mut cnt = 0;
      for lex_state_rules in &g.lex {
        for &(_, act, term) in lex_state_rules {
          p.ln(format!("fn lex_act{}(_l: &mut Lexer) -> TokenType {{", cnt)).inc();
          if !act.is_empty() { // just to make it prettier...
            p.lns(act);
          }
          p.ln(format!("TokenType::{}", term));
          p.dec().ln("}\n");
          cnt += 1;
        }
      }
    }

    for (i, &(act, (lhs, rhs), _)) in g.prod_extra.iter().enumerate() {
      p.ln(format!("fn parser_act{}(_p: &mut Parser) {{", i)).inc();
      let rhs = &g.prod[lhs as usize][rhs as usize];
      for (j, &x) in rhs.0.iter().enumerate().rev() {
        let j = j + 1;
        if x < AbstractGrammar::nt_num(g) {
          let ty = g.nt[x as usize].1;
          p.ln(format!("let mut _{} = match _p.value_stk.pop() {{ Some(StackItem::{}(x)) => x, _ => unreachable!() }};", j, ty));
        } else {
          p.ln(format!("let mut _{} = match _p.value_stk.pop() {{ Some(StackItem::_Token(x)) => x, _ => unreachable!() }};", j));
        }
      }
      if !act.is_empty() { // just to make it prettier...
        p.lns(act);
      }
      let ty = g.nt[lhs as usize].1;
      p.lns(format!("_p.value_stk.push(StackItem::{}(_0));", ty));
      p.dec().ln("}\n");
    }
    p.finish()
  }
}