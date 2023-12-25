use regex::Regex;

use super::{RE_OPEN_START, RE_OPEN_CONFIG, RE_END, TOKEN_INT, BracketType, TOKEN_DATE, TOKEN_REAL, OPEN, TOKEN_PIPE, TOKEN_COLON, TOKEN_COMMA, TOKEN_COLON_END, TOKEN_PIPE_END, BracketChunk, CharSlice, ESCAPE_CHAR, CLOSE};

pub fn match_simple_start(s: &str) -> bool {
    let re_start = Regex::new(RE_OPEN_START).unwrap();
    re_start.is_match(s)
}

pub fn match_start(s: &str) -> bool {
    let re_start = Regex::new(RE_OPEN_CONFIG).unwrap();
    re_start.is_match(s)
}

pub fn match_end(s: &str) -> bool {
    let re_start = Regex::new(RE_END).unwrap();
    re_start.is_match(s)
}

pub fn collect_bounds(s: &str, pattern: &str) -> Vec<BracketChunk> {
    let re = Regex::new(pattern).unwrap();
    re.captures_iter(&s).map(|cp| {
        let m = cp.get(0).unwrap();
        BracketChunk { idx: m.start(), typ: guess_type(&m), warning_code: Default::default() }
    }).collect()
}

pub fn collect_escaped(s: &str) -> Vec<CharSlice> {
    let re = Regex::new(format!(
        "{}{}{}{}{}{}{}", 
        r"(\", ESCAPE_CHAR, r"+)[\", OPEN , r"\", CLOSE, "]").as_str()).unwrap();
    re.captures_iter(&s).filter(|cp| {
        let m = cp.get(1).unwrap();
        m.len() % 2 != 0
    })
    .map(|cp| {
        let m = cp.get(1).unwrap();
        CharSlice { start: m.start(), quantity: m.len(), character: ESCAPE_CHAR }
    })
    .collect()
}

pub fn guess_type(m: &regex::Match<'_>) -> BracketType {
    let s = m.as_str();
    if s.len() == 1 {
        return BracketType::Simple;
    }
    if s.chars().next().unwrap() == CLOSE {
        return guess_open_type(&m);
    }
    guess_close_type(&m)
}

pub fn guess_open_type(m: &regex::Match<'_>) -> BracketType {
    let s = m.as_str();
    if s == TOKEN_INT {
        return BracketType::Int;
    }
    if s == TOKEN_DATE {
        return BracketType::Date;
    }
    if s == TOKEN_REAL {
        return BracketType::Real;
    }
    if s.starts_with(TOKEN_PIPE) || s.starts_with(TOKEN_COLON) {
        return BracketType::FreeText(CharSlice { start: m.start() + 1, quantity: s.len() - 1, character: s.chars().last().unwrap() });
    }
    if s.starts_with(TOKEN_COMMA) {
        return BracketType::List(s.len() - 1);
    }
    
    BracketType::Simple
}

pub fn guess_close_type(m: &regex::Match<'_>) -> BracketType {
    let s = m.as_str();
    if s.ends_with(TOKEN_PIPE_END) || s.ends_with(TOKEN_COLON_END) {
        return BracketType::FreeText(CharSlice { start: m.start(), quantity: s.len() - 1, character: s.chars().next().unwrap() });
    }
    
    BracketType::Simple
}