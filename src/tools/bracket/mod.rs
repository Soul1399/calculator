/* sample text in data/db.bk */

use std::{error::Error, fs::File, io::Read, rc::Rc, str::ParseBoolError};

pub const OPEN: char = '[';
pub const CLOSE: char = ']';
const ESCAPE_CHAR: char = '\\';
const COMMA_CHAR: char = ',';
const TOKEN_PIPE: &str = "[|";
const TOKEN_PIPE_END: &str = "|]";
const TOKEN_COLON: &str = "[:";
const TOKEN_COLON_END: &str = ":]";
const TOKEN_COMMA: &str = "[,";
const RE_OPEN_CONFIG: &str = r"^\s*(@\[)";
const RE_OPEN_START: &str = r"^\s*\[";
const RE_END: &str = r"]\s*$";
const TOKEN_INT: &str = "[@int:";
const TOKEN_DATE: &str = "[@date:";
const TOKEN_REAL: &str = "[@real:";
const RE_OPEN: &str = r"(\[(?:\|+|:+|,+|@int:|@date:|@real:)|\[)";
const RE_CLOSE: &str = r"((?:\|+|:+|,+)]|])";

#[derive(Debug, Clone)]
pub struct Value {
    pub index: i32,
    pub string: String,
}

#[derive(Debug, Clone)]
pub enum BracketValue {
    Root(Rc<BracketValue>, Rc<BracketValue>),
    Obj(Value, Vec<Rc<BracketValue>>),
    Val(Value, Rc<BracketValue>),
    Array(Vec<Rc<BracketValue>>),
    Prop(Value, Value),
    NoVal,
}

impl Default for BracketValue {
    fn default() -> Self {
        BracketValue::NoVal
    }
}

pub struct BracketConfig {
    pub name: String,
    pub version: String,
    pub allow_empty_free_text: bool,
}

impl BracketConfig {
    pub fn build() -> BracketConfig {
        BracketConfig {
            name: Default::default(),
            version: Default::default(),
            allow_empty_free_text: false,
        }
    }

    pub fn build_from_props(value: &BracketValue) -> BracketConfig {
        let mut config = BracketConfig::build();
        if let BracketValue::Obj(val, props) = value {
            if val.string == "@" {
                props.into_iter().for_each(|p| {
                    if let BracketValue::Prop(name, value) = p.as_ref() {
                        match name.string.as_str() {
                            "version" => config.version = value.string.clone(),
                            "name" => config.name = value.string.clone(),
                            "allow empty free text" => {
                                let b: Result<bool, ParseBoolError> = value.string.parse();
                                config.allow_empty_free_text = b.unwrap_or_default();
                            }
                            _ => {}
                        }
                    }
                })
            }
        }
        config
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum BracketFlag {
    HasConfig = 1,
    HasBeginBracket = 2,
    HasEndingBracket = 3,
    HasValidConfig = 4,
}

#[derive(Debug, Clone)]
pub enum BracketType {
    Simple,
    FreeText(CharSlice),
    Int,
    Date,
    Real,
    List(usize),
}

impl Default for BracketType {
    fn default() -> Self {
        BracketType::Simple
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CharSlice {
    pub start: usize,
    pub quantity: usize,
    pub character: char,
}

#[derive(Debug, Clone)]
pub struct Brackets {
    buffer: String,
    open_bks: Vec<BracketChunk>,
    close_bks: Vec<BracketChunk>,
    flags: Vec<BracketFlag>,
    is_processing: bool,
    is_valid: Option<bool>,
    pub root: BracketValue,
}

#[derive(Debug, Clone)]
pub struct BracketChunk {
    pub idx: usize,
    pub typ: BracketType,
    pub warning_code: usize,
}

impl PartialEq for BracketChunk {
    fn eq(&self, other: &Self) -> bool {
        // impossible to have two with the same index
        self.idx == other.idx
    }
}

impl Default for Brackets {
    fn default() -> Self {
        Brackets {
            buffer: Default::default(),
            open_bks: vec![],
            close_bks: vec![],
            flags: vec![],
            root: BracketValue::Root(Rc::new(Default::default()), Rc::new(Default::default())),
            is_processing: false,
            is_valid: None,
        }
    }
}

impl Brackets {
    pub fn build_from_string(text: String) -> Result<Brackets, Box<dyn Error>> {
        let mut new: Brackets = Default::default();
        new.buffer = text;
        new.process_buffer()?;
        Ok(new)
    }

    pub fn build_from_file(file: &mut File) -> Result<Brackets, Box<dyn Error>> {
        let mut buf: String = Default::default();
        if let Ok(_) = file.read_to_string(&mut buf) {
            return Brackets::build_from_string(buf);
        }

        Ok(Default::default())
    }

    pub fn get_config(&self) -> BracketConfig {
        if let BracketValue::Root(cfg, _) = &self.root {
            if let BracketValue::Obj(_, _) = cfg.as_ref() {
                return BracketConfig::build_from_props(cfg);
            }
        }
        BracketConfig::build()
    }

    fn process_buffer(&mut self) -> Result<(), &'static str> {
        self.is_processing = true;
        self.reset();
        self.spot_bounds()?;
        self.link_bounds()?;
        Ok(())
    }

    fn reset(&mut self) {
        self.open_bks.clear();
        self.close_bks.clear();
        self.flags.clear();
        self.is_valid = None;
    }

    fn spot_bounds(&mut self) -> Result<(), &'static str> {
        self.check_buffer()?;
        self.check_start();
        self.check_end();
        self.primal_validation()?;
        self.is_valid = Some(false);
        self.collect_open_bounds();
        self.collect_close_bounds();
        self.remove_non_bracket_bounds();
        Ok(())
    }

    fn link_bounds(&mut self) -> Result<(), &'static str> {
        todo!()
    }

    fn check_buffer(&self) -> Result<(), &'static str> {
        if self.buffer.trim_end().len() == 0 {
            return Err("Empty string");
        }
        Ok(())
    }

    fn check_start(&mut self) {
        self.identify_config();
        if !self.flags.contains(&BracketFlag::HasConfig) {
            if bk_regex::match_simple_start(&self.buffer) {
                self.flags.push(BracketFlag::HasBeginBracket);
            }
        }
    }

    fn identify_config(&mut self) {
        if bk_regex::match_start(&self.buffer) {
            self.flags.push(BracketFlag::HasConfig);
        }
    }

    fn check_end(&mut self) {
        if bk_regex::match_end(&self.buffer) {
            self.flags.push(BracketFlag::HasEndingBracket);
        }
    }

    fn primal_validation(&self) -> Result<(), &'static str> {
        let start_is_valid = self.flags.contains(&BracketFlag::HasConfig)
            || self.flags.contains(&BracketFlag::HasBeginBracket);
        let end_is_valid = self.flags.contains(&BracketFlag::HasEndingBracket);
        if start_is_valid && end_is_valid {
            return Ok(());
        }

        // let mut error: String = String::new();
        // if !start_is_valid {
        //     error.push_str("");
        // }
        // if !end_is_valid {
        //     error.push_str("");
        // }
        // let s=error.as_str();
        Err("Start and/or end of the text is malformed")
    }

    fn collect_open_bounds(&mut self) {
        self.open_bks
            .extend(bk_regex::collect_bounds(&self.buffer, &RE_OPEN));
        self.open_bks
            .sort_by(|a, b| a.idx.partial_cmp(&b.idx).unwrap());
    }

    fn collect_close_bounds(&mut self) {
        self.close_bks
            .extend(bk_regex::collect_bounds(&self.buffer, &RE_CLOSE));
        self.close_bks
            .sort_by(|a, b| a.idx.partial_cmp(&b.idx).unwrap());
    }

    fn remove_non_bracket_bounds(&mut self) {
        let mut x: usize = 0;
        let length = self.open_bks.len();
        let mut search = true;
        let mut real_open_bk: Vec<BracketChunk> = vec![];
        let escaped_bounds = bk_regex::collect_escaped(&self.buffer);
        let mut warning: Option<usize> = None;
        let mut free_text_ranges: Vec<(usize, usize)> = vec![];
        while search {
            let mut enm = self.open_bks[x..length - 1].iter().enumerate();
            while x < length {
                warning = None;
                let (_, bk) = enm.next().unwrap();
                if escaped_bounds
                    .iter()
                    .any(|b| b.start + b.quantity == bk.idx) { warning = Some(2) }
                if free_text_ranges
                    .iter()
                    .any(|r| r.0 >= bk.idx || r.1 <= bk.idx) { warning = Some(3) }
                if warning.is_some() { break }
                match bk.typ {
                    BracketType::FreeText(_) | BracketType::List(_) => { break }
                    _ => {
                        real_open_bk.push(bk.clone());
                    }
                }
                x += 1;
            }
            if x == length {
                search = false;
                continue;
            }
            enm.last();
            let index = x;
            x += 1;
            if let Some(code) = warning {
                self.open_bks[index].warning_code = code;
                continue;
            }
            self.extract_free_text_range(index, &mut real_open_bk, &mut free_text_ranges);
        }
    }

    fn extract_free_text_range(
        &mut self,
        index: usize,
        real_open_bk: &mut Vec<BracketChunk>,
        free_text_ranges: &mut Vec<(usize, usize)>,
    ) {
        let found = self.find_close_bk(&self.open_bks[index]);
        if let Some(cbk) = found {
            if cbk == self.open_bks[index] || cbk.idx == self.open_bks[index].idx + 2 {
                let mut obk = self.open_bks[index].clone();
                obk.typ = Default::default();
                real_open_bk.push(obk);
            } else {
                let start = match self.open_bks[index].typ {
                    BracketType::FreeText(slc) => slc.start + slc.quantity,
                    BracketType::List(size) => self.open_bks[index].idx + 1 + size,
                    _ => unreachable!(),
                };
                let end = match cbk.typ {
                    BracketType::FreeText(_) | BracketType::List(_) => cbk.idx - 1,
                    _ => unreachable!(),
                };
                free_text_ranges.push((start, end));
            }
        }
    }

    fn find_close_bk(&self, open_bk: &BracketChunk) -> Option<BracketChunk> {
        let mut ft_char: Option<char> = None;
        let mut nb_ft_char: usize = 0;
        match open_bk.typ {
            BracketType::FreeText(slc) => {
                ft_char = Some(slc.character);
                nb_ft_char = slc.quantity;
            }
            BracketType::List(size) => {
                ft_char = Some(',');
                nb_ft_char = size;
            }
            _ => {
                if !self.is_valid.unwrap_or(false) {
                    return None;
                }
            }
        }
        if let Some(chr) = ft_char {
            let b = self.find_free_text_end(open_bk.idx, chr, nb_ft_char);
            if b.is_none() {
                return Some(open_bk.clone());
            }
            return b;
        }

        todo!()
    }

    fn find_free_text_end(&self, start: usize, chr: char, size: usize) -> Option<BracketChunk> {
        let end = self.close_bks.len() - 1;
        let o = self.close_bks[start..end].into_iter().find(|c| {
            if let BracketType::FreeText(slc) = c.typ {
                return size == slc.quantity && chr == slc.character;
            }
            return false;
        });
        if !o.is_none() {
            let mut cbk: BracketChunk = o.unwrap().clone();
            let nb_free_text_char = match cbk.typ {
                BracketType::FreeText(slc) => slc.quantity,
                _ => unreachable!(),
            };
            if start == cbk.idx + 1 {
                if nb_free_text_char == 1 {
                    cbk.typ = BracketType::Simple;
                    cbk.idx += 1;
                    return Some(cbk);
                }
                if nb_free_text_char % 2 == 0 {
                    cbk.warning_code = 1;
                }
            }
            return Some(cbk);
        }
        None
    }
}

#[cfg(test)]
mod tests_brackets {
    use regex::Regex;

    use super::*;

    #[test]
    fn search_open_bk() {
        let re = Regex::new(RE_OPEN).unwrap();
        let haystack = "@[version[@int:1]]";
        let c = re.captures_iter(haystack).count();
        assert_eq!(c, 2);

        re.captures_iter(haystack).for_each(|c| {
            assert!(c.get(0).unwrap().as_str() == "[" || c.get(0).unwrap().as_str() == "[@int:")
        });
    }

    #[test]
    fn empty_text() {
        let b = Brackets::build_from_string(String::new());
        if let Err(m) = b {
            assert_eq!(m.to_string(), "Empty string");
        }
    }

    #[test]
    fn invalid_text() {
        let b = Brackets::build_from_string(String::from("(dddd) //lo"));
        if let Err(m) = b {
            assert_eq!(m.to_string(), "Start and/or end of the text is malformed");
        }
    }

    #[test]
    fn simple_text() {
        let b = Brackets::build_from_string(String::from("[]"));
        assert!(b.is_ok());
    }

    #[test]
    fn simple_full_text() {
        let b = Brackets::build_from_string(String::from("\n\n@[]\n[]\n"));
        assert!(b.is_ok());
    }

    #[test]
    fn test_equal_chunks() {
        let mut chk1 = BracketChunk {
            idx: 0,
            typ: BracketType::Simple,
            warning_code: 0,
        };
        let chk2 = BracketChunk {
            idx: 0,
            typ: BracketType::Date,
            warning_code: 10,
        };
        assert!(chk1 == chk2);
        assert!(chk1.eq(&chk2));
        chk1.idx = 3;
        assert!(chk1 != chk2);
        assert!(chk1.ne(&chk2));
    }
}

pub mod bk_regex;
