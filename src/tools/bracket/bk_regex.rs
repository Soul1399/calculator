use std::ops::RangeInclusive;
use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

use super::{BracketChunk, BracketType, CharSlice, ConfigProps, CLOSE, ESCAPE_CHAR, OPEN, RE_END, RE_OPEN, RE_FREETEXT_START, RE_FREETEXT_END, RE_CLOSE, RE_OPEN_CONFIG_START, RE_OPEN_START, TOKEN_COLON, TOKEN_COLON_END, TOKEN_COMMA, TOKEN_COMMA_END, TOKEN_COMMENT_END, TOKEN_COMMENT_START, TOKEN_DATE, TOKEN_INT, TOKEN_PIPE, TOKEN_PIPE_END, TOKEN_REAL};

lazy_static! {
    static ref RGX_START: Regex = RegexBuilder::new(RE_OPEN_START)
        .unicode(true).multi_line(true)
        .build().unwrap();
    static ref RGX_COMMENTS_START: Regex = RegexBuilder::new(&format!("{}{}", r"^\s*", regex::escape(TOKEN_COMMENT_START)))
        .unicode(true).multi_line(true)
        .build().unwrap();
    static ref RGX_OPEN_ANY: Regex = RegexBuilder::new(&regex::escape(&OPEN.to_string()))
        .unicode(true).multi_line(true)
        .build().unwrap();
    static ref RGX_OPEN: Regex = RegexBuilder::new(&RE_OPEN)
        .unicode(true).multi_line(true)
        .build().unwrap();
    static ref RGX_CLOSE: Regex = RegexBuilder::new(&RE_CLOSE)
        .unicode(true).multi_line(true)
        .build().unwrap();
    static ref RGX_COMMENTS: Regex = RegexBuilder::new(format!(
        "({}|{})",
        regex::escape(TOKEN_COMMENT_START),
        regex::escape(TOKEN_COMMENT_END)).as_str()).unicode(true).multi_line(true).build().unwrap();
    static ref RGX_WHITE_SPACES: Regex = RegexBuilder::new(r"^\s*$")
        .unicode(true).multi_line(true)
        .build().unwrap();
    static ref RGX_ESCAPED: Regex = RegexBuilder::new(&format!(
        "({}+)[{}{}]", 
        regex::escape(&ESCAPE_CHAR.to_string()), 
        regex::escape(&OPEN.to_string()), 
        regex::escape(&CLOSE.to_string())))
        .unicode(true).multi_line(true)
        .build().unwrap();
    static ref RGX_END: Regex = RegexBuilder::new(RE_END)
        .unicode(true).multi_line(true)
        .build().unwrap();
    static ref RGX_TOKEN_FREETEXT: Regex = RegexBuilder::new(&RE_FREETEXT_START)
        .unicode(true).multi_line(true)
        .build().unwrap();
    static ref RGX_TOKEN_END_FREETEXT: Regex = RegexBuilder::new(&RE_FREETEXT_END)
    .unicode(true).multi_line(true)
    .build().unwrap();
}

pub fn match_simple_start(s: &str, ranges_of_comments: Vec<RangeInclusive<usize>>) -> bool {
    if ranges_of_comments.is_empty() || !TOKEN_COMMENT_START.chars().any(|c| c == OPEN) {
        return RGX_START.is_match(s);
    }

    let mut first_bk = RGX_OPEN_ANY.captures_iter(s).next();
    if first_bk.is_none() {
        return false;
    }
    let c = first_bk.unwrap();
    let mut start = c.get(0).unwrap().start();
    let ln = TOKEN_COMMENT_START.len();
    let start_len = start as i8 - ln as i8 + 1i8;
    if start_len <= -1 {
        return RGX_START.is_match(s);
    }
    else {
        start = start_len as usize;
        if &s[start..start + ln] != TOKEN_COMMENT_START {
            return RGX_START.is_match(s);
        }
        if !RGX_COMMENTS_START.is_match(s) {
            return false;
        }
    }
    
    let mut ranges_iter = ranges_of_comments.iter().peekable();
    let mut end: usize;
    while ranges_iter.peek().is_some() {
        let current_rg = ranges_iter.next().unwrap();
        let next_rg = ranges_iter.peek();
        start = *current_rg.end() + TOKEN_COMMENT_END.len();
        first_bk = RGX_START.captures_iter(&s[start..]).next();
        if first_bk.is_some() {
            return true;
        }
        if next_rg.is_none() {
            return false;
        }
        end = *next_rg.unwrap().start();
        if !RGX_WHITE_SPACES.is_match(&s[start..end]) {
            return false;
        }
    }
    
    false

}

pub fn match_start(s: &str) -> bool {
    let re_start = Regex::new(RE_OPEN_CONFIG_START).unwrap();
    re_start.is_match(s)
}

pub fn match_end(s: &str) -> bool {
    let re_start = Regex::new(RE_END).unwrap();
    re_start.is_match(s)
}

pub fn collect_bounds(s: &str, pattern: &str) -> Vec<BracketChunk> {
    let default_rx: Regex;
    let re: &Regex = match pattern {
        o if o == RE_OPEN.as_str() => &RGX_OPEN,
        c if c == RE_CLOSE.as_str() => &RGX_CLOSE,
        _ => {
            default_rx = Regex::new(pattern).unwrap();
            &default_rx
        }
    };
    
    re.captures_iter(s).map(|cp| {
        let m = cp.get(0).unwrap();
        BracketChunk {
            idx: m.start(), 
            typ: guess_type(&m), 
            warning_code: Default::default(), 
            is_open_first: if pattern == RE_OPEN.as_str() { Some(false) } else { None }, 
            linked_idx: 0
        }
    }).collect()
}

pub fn collect_escaped(s: &str) -> Vec<CharSlice> {
    RGX_ESCAPED.captures_iter(s).filter(|cp| {
        let m = cp.get(1).unwrap();
        m.len() % 2 != 0
    })
    .map(|cp| {
        let m = cp.get(1).unwrap();
        CharSlice { start: m.start(), quantity: m.len(), character: ESCAPE_CHAR }
    })
    .collect()
}

pub fn collect_comments(s: &str) -> Vec<(usize, char)> {
    RGX_COMMENTS.captures_iter(&s).map(|cp| {
        let m = cp.get(1).unwrap();
        let c = m.as_str().chars().next().unwrap();
        (m.start(), c)
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

fn guess_open_type(m: &regex::Match<'_>) -> BracketType {
    let s = m.as_str();
    if RGX_TOKEN_FREETEXT.is_match(s) {
        return BracketType::FreeText(CharSlice {
            start: m.start() + 1, 
            quantity: s.len() - 1, 
            character: s.chars().last().unwrap()
        });
    }
    
    match s {
        int if int == TOKEN_INT.as_str() => BracketType::Int,
        dt if dt == TOKEN_DATE.as_str() => BracketType::Date,
        rl if rl == TOKEN_REAL.as_str() => BracketType::Real,
        cmm if cmm == TOKEN_COMMA.as_str() => BracketType::List,
        _ => BracketType::Simple
    }
}

fn guess_close_type(m: &regex::Match<'_>) -> BracketType {
    let s = m.as_str();
    if s == TOKEN_COMMA_END.as_str() {
        return BracketType::List;
    }
    else if RGX_TOKEN_END_FREETEXT.is_match(s) {
        return BracketType::FreeText(CharSlice {
            start: m.start(),
            quantity: s.len() - 1,
            character: s.chars().next().unwrap()
        });
    }
    
    BracketType::Simple
}

pub fn extract_config(s: &str) -> (usize, HashMap<String, String>) {
    let patterns: Vec<String> = ConfigProps
        .iter()
        .map(|p| format!(r"({0}\[(?<{1}>[\w\s]*)\])?", p.replace("_", " "), p))
        .collect();

    let pattern = patterns.join(r"\s*");

    let mut map = HashMap::new();
    let re = Regex::new(format!("{}{}{}", r"^\[\s*", pattern, r"\s*(?<closure>\])").as_str()).unwrap();
    
    let mut closure_index: usize = 0;
    let result = re.captures(&s);
    if let Some(captures) = result {
        ConfigProps.iter().for_each(|p| {
            if let Some(val) = captures.name(p.as_str()) {
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

#[cfg(test)]
mod tests_my_rgex {
    use super::*;

    #[test]
    fn get_comments() {
        let bounds = collect_comments("this is a test\n**[ comment text ]**\n end");
        assert!(bounds.len() == 2);
        assert!(bounds[0].0 == 15);
        assert!(bounds[1].0 == 32);
    }

    #[test]
    fn match_comment_start_invalid() {
        let comments = "**[ comment text ]**";
        let texts = vec![
            format!("this is a test\n{}\n end. invalid BK", comments),
            format!("this is a test\n{}\n end. invalid BK []", comments),
            format!("this is a \n{}\n test\n{}\n end. \n{}\n invalid BK []", comments, comments, comments),
            format!("{}\n\n{}", comments, comments)
        ];
        let mut texts_iter = texts.iter().peekable();
        while texts_iter.peek().is_some() {
            let text = texts_iter.next().unwrap();
            let mut start = text.find(comments);
            let mut rg_comments: Vec<RangeInclusive<usize>> = vec![];
            let mut z: usize = 0;
            while start.is_some() {
                let a = start.unwrap() + z;
                z = a + comments.len() - 2;
                rg_comments.push(a..=z-1);
                z += 2;
                start = (&text[z..]).find(comments);
            }
            assert!(!match_simple_start(&text, rg_comments));
        }
    }

    #[test]
    fn match_comment_start_valid() {
        let comments = "**[ comment text ]**";
        let texts = vec![
            format!("  {}\n[]", comments),
            format!("{}\n []", comments),
            format!("{}\n{}\n{}\n[]", comments, comments, comments),
            format!("[]\n{}\n ", comments)
        ];
        let mut texts_iter = texts.iter().peekable();
        while texts_iter.peek().is_some() {
            let text = texts_iter.next().unwrap();
            let mut start = text.find(comments);
            let mut rg_comments: Vec<RangeInclusive<usize>> = vec![];
            let mut z: usize = 0;
            while start.is_some() {
                let a = start.unwrap() + z;
                z = a + comments.len() - 2;
                rg_comments.push(a..=z-1);
                z += 2;
                start = (&text[z..]).find(comments);
            }
            assert!(match_simple_start(&text, rg_comments));
        }
    }
}