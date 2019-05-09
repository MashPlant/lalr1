#![allow(unused)]
#![allow(unused_mut)]

use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType {
  Expr,
  _Expr,
  _Eps,
  _Eof,
  Or,
  And,
  BOr,
  BXor,
  BAnd,
  Eq,
  Ne,
  Le,
  Ge,
  Lt,
  Gt,
  Repeat,
  Shl,
  Shr,
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  UMinus,
  Not,
  Inc,
  Dec,
  LBracket,
  Dot,
  Default,
  RParenthesis,
  Empty,
  Else,
  Void,
  Int,
  Bool,
  String,
  New,
  Null,
  True,
  False,
  Class,
  Extends,
  This,
  While,
  Foreach,
  For,
  If,
  Return,
  Break,
  Print,
  ReadInteger,
  ReadLine,
  Static,
  InstanceOf,
  SCopy,
  Sealed,
  Var,
  In,
  GuardSplit,
  Comma,
  Semicolon,
  LParenthesis,
  RBracket,
  LBrace,
  RBrace,
  Colon,
  Identifier,
}

#[derive(Debug, Clone, Copy)]
pub enum LexerState {
  _Initial = 0,
  S = 1,
}

macro_rules! map (
  { $($key:expr => $value:expr),+ } => {{
    let mut m = ::std::collections::HashMap::new();
    $( m.insert($key, $value); )+
    m
  }};
);

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
  pub ty: TokenType,
  pub piece: &'a str,
  pub line: u32,
  pub col: u32,
}


pub struct Lexer<'a> {
  pub string: &'a str,
  pub states: Vec<LexerState>,
  pub cur_line: u32,
  pub cur_col: u32,
  pub piece: &'a str,
  pub string_builder: (String, u32, u32),
  pub errors: Vec<String>,
}

impl Lexer<'_> {
  pub fn new(string: &str) -> Lexer {
    Lexer {
      string,
      states: vec![LexerState::_Initial],
      cur_line: 1,
      cur_col: 0,
      piece: "",
      string_builder: (String::new(), 0, 0),
      errors: Vec::new(),
    }
  }

  pub fn next(&mut self) -> Option<Token> {
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
            } {
              max = Some((n, *act));
            }
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
  }
}

lazy_static! {
  static ref LEX_RULES: [Vec<(Regex, fn(&mut Lexer) -> TokenType)>; 2] = [
    vec![
      (Regex::new(r#"^void"#).unwrap(), lex_act0),
      (Regex::new(r#"^int"#).unwrap(), lex_act1),
      (Regex::new(r#"^bool"#).unwrap(), lex_act2),
      (Regex::new(r#"^string"#).unwrap(), lex_act3),
      (Regex::new(r#"^new"#).unwrap(), lex_act4),
      (Regex::new(r#"^null"#).unwrap(), lex_act5),
      (Regex::new(r#"^true"#).unwrap(), lex_act6),
      (Regex::new(r#"^false"#).unwrap(), lex_act7),
      (Regex::new(r#"^class"#).unwrap(), lex_act8),
      (Regex::new(r#"^extends"#).unwrap(), lex_act9),
      (Regex::new(r#"^this"#).unwrap(), lex_act10),
      (Regex::new(r#"^while"#).unwrap(), lex_act11),
      (Regex::new(r#"^foreach"#).unwrap(), lex_act12),
      (Regex::new(r#"^for"#).unwrap(), lex_act13),
      (Regex::new(r#"^if"#).unwrap(), lex_act14),
      (Regex::new(r#"^else"#).unwrap(), lex_act15),
      (Regex::new(r#"^return"#).unwrap(), lex_act16),
      (Regex::new(r#"^break"#).unwrap(), lex_act17),
      (Regex::new(r#"^Print"#).unwrap(), lex_act18),
      (Regex::new(r#"^ReadInteger"#).unwrap(), lex_act19),
      (Regex::new(r#"^ReadLine"#).unwrap(), lex_act20),
      (Regex::new(r#"^static"#).unwrap(), lex_act21),
      (Regex::new(r#"^instanceof"#).unwrap(), lex_act22),
      (Regex::new(r#"^scopy"#).unwrap(), lex_act23),
      (Regex::new(r#"^sealed"#).unwrap(), lex_act24),
      (Regex::new(r#"^var"#).unwrap(), lex_act25),
      (Regex::new(r#"^default"#).unwrap(), lex_act26),
      (Regex::new(r#"^in"#).unwrap(), lex_act27),
      (Regex::new(r#"^\|\|\|"#).unwrap(), lex_act28),
      (Regex::new(r#"^<="#).unwrap(), lex_act29),
      (Regex::new(r#"^>="#).unwrap(), lex_act30),
      (Regex::new(r#"^=="#).unwrap(), lex_act31),
      (Regex::new(r#"^!="#).unwrap(), lex_act32),
      (Regex::new(r#"^\&\&"#).unwrap(), lex_act33),
      (Regex::new(r#"^\|\|"#).unwrap(), lex_act34),
      (Regex::new(r#"^%%"#).unwrap(), lex_act35),
      (Regex::new(r#"^\+\+"#).unwrap(), lex_act36),
      (Regex::new(r#"^\-\-"#).unwrap(), lex_act37),
      (Regex::new(r#"^<<"#).unwrap(), lex_act38),
      (Regex::new(r#"^>>"#).unwrap(), lex_act39),
      (Regex::new(r#"^\+"#).unwrap(), lex_act40),
      (Regex::new(r#"^\-"#).unwrap(), lex_act41),
      (Regex::new(r#"^\*"#).unwrap(), lex_act42),
      (Regex::new(r#"^/"#).unwrap(), lex_act43),
      (Regex::new(r#"^%"#).unwrap(), lex_act44),
      (Regex::new(r#"^\&"#).unwrap(), lex_act45),
      (Regex::new(r#"^\|"#).unwrap(), lex_act46),
      (Regex::new(r#"^\^"#).unwrap(), lex_act47),
      (Regex::new(r#"^="#).unwrap(), lex_act48),
      (Regex::new(r#"^<"#).unwrap(), lex_act49),
      (Regex::new(r#"^>"#).unwrap(), lex_act50),
      (Regex::new(r#"^\."#).unwrap(), lex_act51),
      (Regex::new(r#"^,"#).unwrap(), lex_act52),
      (Regex::new(r#"^;"#).unwrap(), lex_act53),
      (Regex::new(r#"^!"#).unwrap(), lex_act54),
      (Regex::new(r#"^\("#).unwrap(), lex_act55),
      (Regex::new(r#"^\)"#).unwrap(), lex_act56),
      (Regex::new(r#"^\["#).unwrap(), lex_act57),
      (Regex::new(r#"^\]"#).unwrap(), lex_act58),
      (Regex::new(r#"^\{"#).unwrap(), lex_act59),
      (Regex::new(r#"^\}"#).unwrap(), lex_act60),
      (Regex::new(r#"^:"#).unwrap(), lex_act61),
      (Regex::new(r#"^\s+"#).unwrap(), lex_act62),
      (Regex::new(r#"^\d+"#).unwrap(), lex_act63),
      (Regex::new(r#"^[A-Za-z][_0-9A-Za-z]*"#).unwrap(), lex_act64),
      (Regex::new(r#"^""#).unwrap(), lex_act65),
      (Regex::new(r#"^//[^\n]*"#).unwrap(), lex_act66),
    ],
    vec![
      (Regex::new(r#"^\n"#).unwrap(), lex_act67),
      (Regex::new(r#"^\\r"#).unwrap(), lex_act68),
      (Regex::new(r#"^$"#).unwrap(), lex_act69),
      (Regex::new(r#"^""#).unwrap(), lex_act70),
      (Regex::new(r#"^\\n"#).unwrap(), lex_act71),
      (Regex::new(r#"^\\t"#).unwrap(), lex_act72),
      (Regex::new(r#"^\\""#).unwrap(), lex_act73),
      (Regex::new(r#"^\\"#).unwrap(), lex_act74),
      (Regex::new(r#"^."#).unwrap(), lex_act75),
    ],
  ];
}

enum Act {
  Acc,
  Shift(u32),
  Reduce(u32),
  Goto(u32),
}

enum StackItem {
  Expr(Expr),
}

lazy_static! {
  static ref TABLE: [HashMap<u32, Act>; 15] = [
    map! { 19 => Act::Shift(5), 34 => Act::Shift(5), 0 => Act::Goto(5) },
    map! { 0 => Act::Goto(8), 34 => Act::Shift(8), 19 => Act::Shift(8) },
    map! { 34 => Act::Shift(4), 0 => Act::Goto(4), 19 => Act::Shift(4) },
    map! { 18 => Act::Reduce(5), 19 => Act::Reduce(5), 20 => Act::Reduce(5), 3 => Act::Reduce(5) },
    map! { 18 => Act::Reduce(4), 20 => Act::Reduce(4), 3 => Act::Reduce(4), 21 => Act::Shift(14), 22 => Act::Shift(14), 19 => Act::Reduce(4) },
    map! { 34 => Act::Shift(2), 19 => Act::Shift(2), 0 => Act::Goto(2) },
    map! { 19 => Act::Reduce(3), 20 => Act::Reduce(3), 3 => Act::Reduce(3), 21 => Act::Shift(13), 22 => Act::Shift(13), 18 => Act::Reduce(3) },
    map! { 22 => Act::Shift(1), 19 => Act::Shift(1), 3 => Act::Acc, 18 => Act::Shift(1), 20 => Act::Shift(1), 21 => Act::Shift(1) },
    map! { 3 => Act::Reduce(0), 18 => Act::Shift(10), 19 => Act::Shift(10), 22 => Act::Shift(10), 20 => Act::Shift(10), 21 => Act::Shift(10) },
    map! { 18 => Act::Reduce(2), 20 => Act::Reduce(2), 22 => Act::Shift(12), 3 => Act::Reduce(2), 21 => Act::Shift(12), 19 => Act::Reduce(2) },
    map! { 0 => Act::Goto(0), 34 => Act::Shift(0), 19 => Act::Shift(0) },
    map! { 0 => Act::Goto(6), 34 => Act::Shift(6), 19 => Act::Shift(6) },
    map! { 34 => Act::Shift(7), 0 => Act::Goto(7), 19 => Act::Shift(7) },
    map! { 22 => Act::Shift(11), 20 => Act::Shift(11), 21 => Act::Shift(11), 18 => Act::Shift(11), 19 => Act::Shift(11), 3 => Act::Reduce(1) },
    map! { 20 => Act::Reduce(6), 3 => Act::Reduce(6), 21 => Act::Reduce(6), 18 => Act::Reduce(6), 19 => Act::Reduce(6), 22 => Act::Reduce(6) },
  ];
}

pub struct Parser<'a> {
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
      let state = *self.states_stack.last().unwrap();
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
          self.values_stack.push(SV::_0(token));

          // Push next state number: "s5" -> 5
          self.states_stack.push(next_state as usize);

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
            self.states_stack.pop();
            rhs_length = rhs_length - 1;
          }

          // Call the handler, push result onto the stack.
          let result_value = self.handlers[production_number](self);

          let previous_state = *self.states_stack.last().unwrap();
          let symbol_to_reduce_with = production[0];

          // Then push LHS onto the stack.
          self.values_stack.push(result_value);

          let next_state = match &TABLE[previous_state][&symbol_to_reduce_with] {
            &Act::Goto(next_state) => next_state,
            _ => unreachable!(),
          };

          self.states_stack.push(next_state);
        }

        // Accept the string.
        &Act::Acc => {
          // Pop state number.
          self.states_stack.pop();

          // Pop the parsed value.
          let parsed = self.values_stack.pop().unwrap();

          if self.states_stack.len() != 1 ||
            self.states_stack.pop().unwrap() != 0 ||
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
}

fn lex_act0(_l: &mut Lexer) -> TokenType {
  TokenType::Void
}

fn lex_act1(_l: &mut Lexer) -> TokenType {
  TokenType::Int
}

fn lex_act2(_l: &mut Lexer) -> TokenType {
  TokenType::Bool
}

fn lex_act3(_l: &mut Lexer) -> TokenType {
  TokenType::String
}

fn lex_act4(_l: &mut Lexer) -> TokenType {
  TokenType::New
}

fn lex_act5(_l: &mut Lexer) -> TokenType {
  TokenType::Null
}

fn lex_act6(_l: &mut Lexer) -> TokenType {
  TokenType::True
}

fn lex_act7(_l: &mut Lexer) -> TokenType {
  TokenType::False
}

fn lex_act8(_l: &mut Lexer) -> TokenType {
  TokenType::Class
}

fn lex_act9(_l: &mut Lexer) -> TokenType {
  TokenType::Extends
}

fn lex_act10(_l: &mut Lexer) -> TokenType {
  TokenType::This
}

fn lex_act11(_l: &mut Lexer) -> TokenType {
  TokenType::While
}

fn lex_act12(_l: &mut Lexer) -> TokenType {
  TokenType::Foreach
}

fn lex_act13(_l: &mut Lexer) -> TokenType {
  TokenType::For
}

fn lex_act14(_l: &mut Lexer) -> TokenType {
  TokenType::If
}

fn lex_act15(_l: &mut Lexer) -> TokenType {
  TokenType::Else
}

fn lex_act16(_l: &mut Lexer) -> TokenType {
  TokenType::Return
}

fn lex_act17(_l: &mut Lexer) -> TokenType {
  TokenType::Break
}

fn lex_act18(_l: &mut Lexer) -> TokenType {
  TokenType::Print
}

fn lex_act19(_l: &mut Lexer) -> TokenType {
  TokenType::ReadInteger
}

fn lex_act20(_l: &mut Lexer) -> TokenType {
  TokenType::ReadLine
}

fn lex_act21(_l: &mut Lexer) -> TokenType {
  TokenType::Static
}

fn lex_act22(_l: &mut Lexer) -> TokenType {
  TokenType::InstanceOf
}

fn lex_act23(_l: &mut Lexer) -> TokenType {
  TokenType::SCopy
}

fn lex_act24(_l: &mut Lexer) -> TokenType {
  TokenType::Sealed
}

fn lex_act25(_l: &mut Lexer) -> TokenType {
  TokenType::Var
}

fn lex_act26(_l: &mut Lexer) -> TokenType {
  TokenType::Default
}

fn lex_act27(_l: &mut Lexer) -> TokenType {
  TokenType::In
}

fn lex_act28(_l: &mut Lexer) -> TokenType {
  TokenType::GuardSplit
}

fn lex_act29(_l: &mut Lexer) -> TokenType {
  TokenType::Le
}

fn lex_act30(_l: &mut Lexer) -> TokenType {
  TokenType::Ge
}

fn lex_act31(_l: &mut Lexer) -> TokenType {
  TokenType::Eq
}

fn lex_act32(_l: &mut Lexer) -> TokenType {
  TokenType::Ne
}

fn lex_act33(_l: &mut Lexer) -> TokenType {
  TokenType::And
}

fn lex_act34(_l: &mut Lexer) -> TokenType {
  TokenType::Or
}

fn lex_act35(_l: &mut Lexer) -> TokenType {
  TokenType::Repeat
}

fn lex_act36(_l: &mut Lexer) -> TokenType {
  TokenType::Inc
}

fn lex_act37(_l: &mut Lexer) -> TokenType {
  TokenType::Dec
}

fn lex_act38(_l: &mut Lexer) -> TokenType {
  TokenType::Shl
}

fn lex_act39(_l: &mut Lexer) -> TokenType {
  TokenType::Shr
}

fn lex_act40(_l: &mut Lexer) -> TokenType {
  TokenType::Add
}

fn lex_act41(_l: &mut Lexer) -> TokenType {
  TokenType::Sub
}

fn lex_act42(_l: &mut Lexer) -> TokenType {
  TokenType::Mul
}

fn lex_act43(_l: &mut Lexer) -> TokenType {
  TokenType::Div
}

fn lex_act44(_l: &mut Lexer) -> TokenType {
  TokenType::Mod
}

fn lex_act45(_l: &mut Lexer) -> TokenType {
  TokenType::BAnd
}

fn lex_act46(_l: &mut Lexer) -> TokenType {
  TokenType::BOr
}

fn lex_act47(_l: &mut Lexer) -> TokenType {
  TokenType::BXor
}

fn lex_act48(_l: &mut Lexer) -> TokenType {
  TokenType::Eq
}

fn lex_act49(_l: &mut Lexer) -> TokenType {
  TokenType::Lt
}

fn lex_act50(_l: &mut Lexer) -> TokenType {
  TokenType::Gt
}

fn lex_act51(_l: &mut Lexer) -> TokenType {
  TokenType::Dot
}

fn lex_act52(_l: &mut Lexer) -> TokenType {
  TokenType::Comma
}

fn lex_act53(_l: &mut Lexer) -> TokenType {
  TokenType::Semicolon
}

fn lex_act54(_l: &mut Lexer) -> TokenType {
  TokenType::Not
}

fn lex_act55(_l: &mut Lexer) -> TokenType {
  TokenType::LParenthesis
}

fn lex_act56(_l: &mut Lexer) -> TokenType {
  TokenType::RParenthesis
}

fn lex_act57(_l: &mut Lexer) -> TokenType {
  TokenType::LBracket
}

fn lex_act58(_l: &mut Lexer) -> TokenType {
  TokenType::RBracket
}

fn lex_act59(_l: &mut Lexer) -> TokenType {
  TokenType::LBrace
}

fn lex_act60(_l: &mut Lexer) -> TokenType {
  TokenType::RBrace
}

fn lex_act61(_l: &mut Lexer) -> TokenType {
  TokenType::Colon
}

fn lex_act62(_l: &mut Lexer) -> TokenType {
  TokenType::_Eps
}

fn lex_act63(_l: &mut Lexer) -> TokenType {
  TokenType::Int
}

fn lex_act64(_l: &mut Lexer) -> TokenType {
  TokenType::Identifier
}

fn lex_act65(_l: &mut Lexer) -> TokenType {
  _l.states.push(LexerState::S);
  _l.string_builder.0.clear();
  _l.string_builder.1 = _l.cur_line;
  _l.string_builder.2 = _l.cur_col + 1;
  TokenType::_Eps
}

fn lex_act66(_l: &mut Lexer) -> TokenType {
  TokenType::_Eps
}

fn lex_act67(_l: &mut Lexer) -> TokenType {
//  let loc = Loc(_l.string_builder.1, _l.string_builder.2);
//  let string = print::quote(&_l.string_builder.0.clone());
//  _l.report_error(Error::new(loc, NewlineInStr{ string }));
  TokenType::_Eps
}

fn lex_act68(_l: &mut Lexer) -> TokenType {
  TokenType::_Eps
}

fn lex_act69(_l: &mut Lexer) -> TokenType {
//  let loc = Loc(_l.string_builder.1, _l.string_builder.2);
//  let string = print::quote(&_l.string_builder.0.clone());
//  _l.report_error(Error::new(loc, UnterminatedStr{ string }));
  TokenType::_Eps
}

fn lex_act70(_l: &mut Lexer) -> TokenType {
  _l.states.pop();
  TokenType::String
}

fn lex_act71(_l: &mut Lexer) -> TokenType {
  _l.string_builder.0.push('\n');
  TokenType::_Eps
}

fn lex_act72(_l: &mut Lexer) -> TokenType {
  _l.string_builder.0.push('\t');
  TokenType::_Eps
}

fn lex_act73(_l: &mut Lexer) -> TokenType {
  _l.string_builder.0.push('\"');
  TokenType::_Eps
}

fn lex_act74(_l: &mut Lexer) -> TokenType {
  _l.string_builder.0.push('\\');
  TokenType::_Eps
}

fn lex_act75(_l: &mut Lexer) -> TokenType {
  _l.string_builder.0.push_str(_l.piece);
  TokenType::_Eps
}


