use std::collections::HashMap;

use regex::Regex;

use super::{RE_OPEN_START, RE_OPEN_CONFIG, RE_END, TOKEN_INT, BracketType, TOKEN_DATE, TOKEN_REAL, OPEN, TOKEN_PIPE, TOKEN_COLON, TOKEN_COMMA, TOKEN_COLON_END, TOKEN_PIPE_END, BracketChunk, CharSlice, ESCAPE_CHAR, CLOSE, CONFIG_NAME, CONFIG_VERSION, CONFIG_ALLOW_EMPTY_FT, CONFIG_CLOSURE_MODE, CONFIG_TRIMMING, RE_OPEN, CONFIG_CACHE, TOKEN_COMMA_END};

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
        BracketChunk {
            idx: m.start(), 
            typ: guess_type(&m), 
            warning_code: Default::default(), 
            is_open_first: if pattern == RE_OPEN { Some(false) } else { None }, 
            linked_idx: 0
        }
    }).collect()
}

pub fn collect_escaped(s: &str) -> Vec<CharSlice> {
    let re = Regex::new(format!(
        "{}{}{}{}{}{}{}", 
        "(\\", ESCAPE_CHAR, "+)[\\", OPEN , "\\", CLOSE, "]").as_str()).unwrap();
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
    if s.chars().last().unwrap() == CLOSE {
        return guess_close_type(&m);
    }
    guess_open_type(&m)
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
    if s == TOKEN_COMMA {
        return BracketType::List;
    }
    
    BracketType::Simple
}

pub fn guess_close_type(m: &regex::Match<'_>) -> BracketType {
    let s = m.as_str();
    if s.ends_with(TOKEN_PIPE_END) || s.ends_with(TOKEN_COLON_END) {
        return BracketType::FreeText(CharSlice { start: m.start(), quantity: s.len() - 1, character: s.chars().next().unwrap() });
    }
    if s == TOKEN_COMMA_END {
        return BracketType::List;
    }
    
    BracketType::Simple
}

pub fn extract_config(s: &str) -> (usize, HashMap<String, String>) {
    let props = vec![CONFIG_NAME, CONFIG_VERSION, CONFIG_ALLOW_EMPTY_FT, CONFIG_CLOSURE_MODE, CONFIG_CACHE, CONFIG_TRIMMING];
    let patterns: Vec<String> = props
        .iter()
        .map(|s| format!(r"({0}\[(?<{1}>[\w\s]*)\])?", s, s.replace(" ", "_")))
        .collect();

    let pattern = patterns.join(r"\s*");

    let mut map = HashMap::new();
    let re = Regex::new(format!("{}{}{}", r"^\[\s*", pattern, r"\s*(?<closure>\])").as_str()).unwrap();
    
    let mut closure_index: usize = 0;
    let result = re.captures(&s);
    if let Some(captures) = result {
        props.into_iter().for_each(|p| {
            if let Some(val) = captures.name(p.replace(" ", "_").as_str()) {
                map.insert(p.to_owned(), val.as_str().to_owned());
            }
            else {
                map.insert(p.to_owned(), String::new());
            }
            if let Some(m) = captures.name("closure") {
                closure_index = m.start();
            }
        })
    }

    (closure_index, map)
}