/* sample text in data/db.bk */

use std::{cmp::Ordering, collections::HashMap, fs::File, io::Read, ops::RangeInclusive, rc::Rc};

use crate::tools::bracket::bk_error::{WARNING_ESCAPED, WARNING_FREE_TEXT};

use self::bk_error::{
    BracketsError, BASE_FORMAT_ERROR, EMPTY_STRING, FORMAT_ERROR, INVALID_CONFIG,
    WARNING_EMPTY_FREE_TEXT, WARNING_MASK,
};

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

const CONFIG_NAME: &str = "name";
const CONFIG_VERSION: &str = "version";
const CONFIG_TRIMMING: &str = "trimming";
const CONFIG_ALLOW_EMPTY_FT: &str = "allow empty free text";
const CONFIG_CLOSURE_MODE: &str = "closure mode";

#[derive(Debug, Clone, Default)]
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
pub struct BracketChunk {
    pub idx: usize,
    pub linked_idx: usize,
    pub typ: BracketType,
    pub warning_code: usize,
    pub is_first: bool
}

impl PartialEq for BracketChunk {
    fn eq(&self, other: &Self) -> bool {
        // impossible to have two with the same index
        self.idx == other.idx
    }
}

impl Eq for BracketChunk {}

impl PartialOrd for BracketChunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.idx.partial_cmp(&other.idx)
    }

    fn lt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Less))
    }

    fn le(&self, other: &Self) -> bool {
        matches!(
            self.partial_cmp(other),
            Some(Ordering::Less | Ordering::Equal)
        )
    }

    fn gt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Greater))
    }

    fn ge(&self, other: &Self) -> bool {
        matches!(
            self.partial_cmp(other),
            Some(Ordering::Greater | Ordering::Equal)
        )
    }
}

impl Ord for BracketChunk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

impl BracketChunk {
    pub fn is_escaped(&self, escaped_slices: &Vec<CharSlice>) -> bool {
        escaped_slices
            .iter()
            .any(|b| b.start + b.quantity == self.idx)
    }

    pub fn is_free_text(&self, free_text_ranges: &Vec<RangeInclusive<usize>>) -> bool {
        free_text_ranges.iter().any(|r| r.contains(&self.idx))
    }
}

#[derive(Debug, Clone)]
pub struct Brackets {
    buffer: String,
    all_open_bks: Vec<BracketChunk>,
    all_close_bks: Vec<BracketChunk>,
    open_bks: Vec<BracketChunk>,
    close_bks: Vec<BracketChunk>,
    flags: Vec<BracketFlag>,
    is_processing: bool,
    is_valid: Option<bool>,
    pub root: BracketValue,
    pub config: HashMap<String, String>,
}

impl std::fmt::Display for Brackets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "nb open: {}, nb close: {}, is valid: {}",
            self.open_bks.len(),
            self.close_bks.len(),
            self.is_valid.unwrap_or_default()
        )
    }
}

impl Default for Brackets {
    fn default() -> Self {
        Brackets {
            buffer: Default::default(),
            open_bks: Default::default(),
            close_bks: Default::default(),
            all_open_bks: Default::default(),
            all_close_bks: Default::default(),
            flags: Default::default(),
            root: BracketValue::Root(Rc::new(Default::default()), Rc::new(Default::default())),
            is_processing: false,
            is_valid: None,
            config: HashMap::new(),
        }
    }
}

impl Brackets {
    pub fn build_from_string(text: String) -> Result<Brackets, BracketsError> {
        let mut new: Brackets = Default::default();
        new.buffer = text;
        new.process_buffer()?;
        Ok(new)
    }

    pub fn build_from_file(file: &mut File) -> Result<Brackets, BracketsError> {
        let mut buf: String = Default::default();
        if let Ok(_) = file.read_to_string(&mut buf) {
            return Brackets::build_from_string(buf);
        }

        Ok(Default::default())
    }

    pub fn default_props() -> Vec<BracketValue> {
        return vec![
            BracketValue::Prop(
                Value {
                    index: 0,
                    string: "name".to_owned(),
                },
                Default::default(),
            ),
            BracketValue::Prop(
                Value {
                    index: 1,
                    string: "version".to_owned(),
                },
                Default::default(),
            ),
            BracketValue::Prop(
                Value {
                    index: 2,
                    string: "trimming".to_owned(),
                },
                Value {
                    index: 0,
                    string: "start".to_owned(),
                },
            ),
            BracketValue::Prop(
                Value {
                    index: 3,
                    string: "allow empty free text".to_owned(),
                },
                Value {
                    index: 0,
                    string: false.to_string(),
                },
            ),
            BracketValue::Prop(
                Value {
                    index: 4,
                    string: "closure mode".to_owned(),
                },
                Value {
                    index: 0,
                    string: "raw".to_owned(),
                },
            ),
        ];
    }

    pub fn get_nb_chunks(&self) -> usize {
        self.open_bks.len()
    }

    pub fn get_state(&self) -> String {
        let invalid: String = "Invalid".to_owned();
        let broken: String = "Broken".to_owned();
        let ok: String = "Ok".to_owned();

        if self.flags.contains(&BracketFlag::HasConfig)
            && !self.flags.contains(&BracketFlag::HasValidConfig)
        {
            return invalid;
        }

        if !self.flags.contains(&BracketFlag::HasBeginBracket)
            && !self.flags.contains(&BracketFlag::HasEndingBracket)
        {
            return invalid;
        }

        if !self.is_valid.unwrap_or_default() {
            return broken;
        }

        ok
    }

    fn process_buffer(&mut self) -> Result<(), BracketsError> {
        self.is_processing = true;
        self.reset();
        self.spot_bounds()?;
        self.link_bounds()?;
        Ok(())
    }

    fn reset(&mut self) {
        self.open_bks.clear();
        self.all_open_bks.clear();
        self.close_bks.clear();
        self.all_close_bks.clear();
        self.flags.clear();
        self.is_valid = None;
    }

    fn spot_bounds(&mut self) -> Result<(), BracketsError> {
        self.check_buffer()?;
        self.check_start();
        self.check_end();
        self.primal_validation()?;
        self.is_valid = Some(false);
        self.collect_open_bounds();
        self.collect_close_bounds();
        self.remove_non_bracket_bounds();
        self.is_valid = Some(self.open_bks.len() == self.close_bks.len());
        self.try_build_config()?;
        self.validate()?;
        self.is_valid = Some(true);
        Ok(())
    }

    fn link_bounds(&mut self) -> Result<(), BracketsError> {
        let mut scan: bool = true;
        let mut stack: Vec<usize> = Vec::new();

        let mut idx_map: HashMap<usize, usize> = HashMap::new();
        self.open_bks.iter().enumerate().for_each(|(x, o)| {
            idx_map.insert(x, o.idx);
        });
        
        let mut enm_o = self.open_bks.iter_mut().enumerate();
        let mut enm_c = self.close_bks.iter_mut().enumerate();
        
        let mut do_next_open = false;
        let mut do_next_close = false;

        let mut a = enm_o.next();
        let mut z = enm_c.next();
        
        while scan {
            a = if do_next_open { enm_o.next() } else { None };
            z = if do_next_close { enm_c.next() } else { None };
            do_next_open = false;
            do_next_close = false;
            if let Some((x, o)) = a {
                if let Some((y, c)) = z {
                    if o.idx < c.idx {
                        stack.push(x);
                        do_next_open = true;
                    }
                    else {
                        let open_index = stack.pop().unwrap();
                        c.linked_idx = *idx_map.get(&open_index).unwrap();
                        do_next_close = true;
                    }
                }
            }
        }
        Ok(())
    }

    fn try_build_config(&mut self) -> Result<(), BracketsError> {
        if self.flags.contains(&BracketFlag::HasConfig) {
            let key = Value {
                index: 0,
                string: "@".to_owned(),
            };
            let mut props: Vec<Rc<BracketValue>> = Brackets::default_props()
                .into_iter()
                .map(|v| Rc::new(v))
                .collect();
            if self.open_bks[1].idx > self.close_bks[0].idx {
                // empty config => remove flag
                self.open_bks[1].is_first = true;
                self.flags.retain(|f| *f != BracketFlag::HasConfig);
            } else {
                let (end_index, map) = bk_regex::extract_config(&self.buffer[self.open_bks[0].idx..]);
                if map.len() == 0 {
                    return Err(BracketsError::new(INVALID_CONFIG));
                }
                props = map
                    .iter()
                    .map(|pair| {
                        Rc::new(BracketValue::Prop(
                            Value { index: 0, string: pair.0.clone() },
                            Value { index: 0, string: pair.1.clone() },
                        ))
                    })
                    .collect();

                self.config = map;
                self.flags.push(BracketFlag::HasValidConfig);
                let firstb = self.open_bks.iter_mut().find(|o| o.idx > end_index);
                if let Some(b) = firstb {
                    b.is_first = true;
                }
            }
            self.root = BracketValue::Root(
                Rc::new(BracketValue::Obj(key, props)),
                Rc::new(Default::default()),
            );
        }
        else {
            self.open_bks[0].is_first = true;
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), BracketsError> {
        if !self.is_valid.unwrap() {
            let mask_found = self
                .open_bks
                .iter()
                .find(|b| b.warning_code == WARNING_MASK);
            if mask_found.is_none()
                || self.config.get(CONFIG_CLOSURE_MODE) == Some(&String::from("raw"))
            {
                // allow full parse
                return Err(BracketsError::new(
                    format!(
                        "{}{}",
                        FORMAT_ERROR,
                        match mask_found {
                            Some(b) => format!("Potential error was found at {}", b.idx),
                            _ => String::new(),
                        }
                    )
                    .as_str(),
                ));
            }
        }
        Ok(())
    }

    fn check_buffer(&self) -> Result<(), BracketsError> {
        if self.buffer.trim_end().len() == 0 {
            return Err(BracketsError::new(EMPTY_STRING));
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

    fn primal_validation(&self) -> Result<(), BracketsError> {
        let start_is_valid = self.flags.contains(&BracketFlag::HasConfig)
            || self.flags.contains(&BracketFlag::HasBeginBracket);
        let end_is_valid = self.flags.contains(&BracketFlag::HasEndingBracket);
        if start_is_valid && end_is_valid {
            return Ok(());
        }
        Err(BracketsError::new(BASE_FORMAT_ERROR))
    }

    fn collect_open_bounds(&mut self) {
        self.open_bks
            .extend(bk_regex::collect_bounds(&self.buffer, &RE_OPEN));
        self.open_bks.sort();
        self.all_open_bks = self.open_bks.iter().map(|o| o.clone()).collect();
    }

    fn collect_close_bounds(&mut self) {
        self.close_bks
            .extend(bk_regex::collect_bounds(&self.buffer, &RE_CLOSE));
        self.close_bks.sort();
        self.all_close_bks = self.close_bks.iter().map(|o| o.clone()).collect();
    }

    fn remove_non_bracket_bounds(&mut self) {
        let mut real_open_bks: Vec<BracketChunk> = vec![];
        let mut real_close_bks: Vec<BracketChunk> = vec![];

        let (escaped_slices, mut free_text_ranges) =
            self.identify_open_bks(&mut real_open_bks, &mut real_close_bks);
        self.identify_close_bks(&mut real_close_bks, &escaped_slices, &mut free_text_ranges);

        self.open_bks = real_open_bks;
        self.open_bks.sort();
        self.close_bks = real_close_bks;
        self.close_bks.sort();
    }

    fn identify_open_bks(
        &mut self,
        real_open_bks: &mut Vec<BracketChunk>,
        real_close_bks: &mut Vec<BracketChunk>,
    ) -> (Vec<CharSlice>, Vec<RangeInclusive<usize>>) {
        let mut x: usize = 0;
        let length = self.open_bks.len();
        let mut search = true;
        let escaped_slices = bk_regex::collect_escaped(&self.buffer);
        let mut warning: Option<usize> = None;
        let mut free_text_ranges: Vec<RangeInclusive<usize>> = vec![];
        while search {
            let mut enm = self.open_bks[x..].iter().enumerate();
            while x < length {
                warning = None;
                let (_, bk) = enm.next().unwrap();
                if bk.is_escaped(&escaped_slices) {
                    warning = Some(WARNING_ESCAPED)
                }
                if bk.is_free_text(&free_text_ranges) {
                    warning = Some(WARNING_FREE_TEXT)
                }
                if warning.is_some() {
                    break;
                }
                match bk.typ {
                    BracketType::FreeText(_) | BracketType::List(_) => break,
                    _ => {
                        real_open_bks.push(bk.clone());
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
                println!("warning {} broke on {}", code, index + 1);
                continue;
            }
            println!("broke on {}", index + 1);
            let close_bk =
                self.extract_free_text_range(index, real_open_bks, &mut free_text_ranges);
            if let Some(c) = close_bk {
                real_close_bks.push(c);
            }
        }

        (escaped_slices, free_text_ranges)
    }

    fn identify_close_bks(
        &mut self,
        real_close_bks: &mut Vec<BracketChunk>,
        escaped_slices: &Vec<CharSlice>,
        free_text_ranges: &Vec<RangeInclusive<usize>>,
    ) {
        let nyet_idfied: Vec<&BracketChunk> = self
            .close_bks
            .iter()
            .filter(|c| !real_close_bks.contains(&c))
            .collect();

        real_close_bks.extend(
            nyet_idfied
                .into_iter()
                .filter(|c| !c.is_free_text(free_text_ranges))
                .filter(|c| !c.is_escaped(escaped_slices))
                .map(|c| c.clone()),
        );
    }

    fn extract_free_text_range(
        &mut self,
        index: usize,
        real_open_bk: &mut Vec<BracketChunk>,
        free_text_ranges: &mut Vec<RangeInclusive<usize>>,
    ) -> Option<BracketChunk> {
        let found = self.find_close_bk(&self.open_bks[index]);
        if let Some(cbk) = found {
            let start_idx = self.open_bks[index].idx;
            if cbk == self.open_bks[index] || cbk.idx == start_idx + 2 {
                let mut obk = self.open_bks[index].clone();
                obk.typ = Default::default();
                real_open_bk.push(obk);
            } else {
                let open_slice = match self.open_bks[index].typ {
                    BracketType::FreeText(slc) => slc,
                    BracketType::List(size) => CharSlice {
                        start: start_idx + 1,
                        quantity: size,
                        character: COMMA_CHAR,
                    },
                    _ => unreachable!(),
                };
                if self.open_bks.len() < index + 1 {
                    let open_alike = self.open_bks[index + 1..]
                        .iter()
                        .filter(|o| o.idx < cbk.idx)
                        .find(|o| match o.typ {
                            BracketType::FreeText(slc) => {
                                open_slice.character == slc.character
                                    && open_slice.quantity == slc.quantity
                            }
                            BracketType::List(size) => {
                                open_slice.character == COMMA_CHAR && open_slice.quantity == size
                            }
                            _ => false,
                        });
                    if let Some(o) = open_alike {
                        if self
                            .close_bks
                            .iter()
                            .any(|cb| (start_idx..=o.idx).contains(&cb.idx))
                        {
                            self.open_bks[index].warning_code = WARNING_MASK;
                        }
                    }
                }
                let start = open_slice.start + open_slice.quantity;
                let end = match cbk.typ {
                    BracketType::FreeText(_) | BracketType::List(_) => cbk.idx - 1,
                    _ => unreachable!(),
                };
                free_text_ranges.push(start..=end);
                real_open_bk.push(self.open_bks[index].clone());
            }

            if cbk != self.open_bks[index] {
                return Some(cbk);
            }
        }
        None
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
                ft_char = Some(COMMA_CHAR);
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
        let o = self.close_bks.iter().filter(|c| c.idx > start).find(|c| {
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
                    cbk.warning_code = WARNING_EMPTY_FREE_TEXT;
                }
            }
            return Some(cbk);
        }
        None
    }
}

#[cfg(test)]
mod tests_brackets {
    use std::borrow::BorrowMut;

    use regex::Regex;

    use super::*;

    #[test]
    fn search_open_bk() {
        let re = Regex::new(RE_OPEN).unwrap();
        let haystack = "@[version[@int:1]]";
        let c = re.captures_iter(haystack).count();
        assert_eq!(c, 2);

        re.captures_iter(haystack).for_each(|c| {
            assert!(
                c.get(0)
                    .unwrap()
                    .as_str()
                    .chars()
                    .next()
                    .unwrap_or_default()
                    == OPEN
                    || c.get(0).unwrap().as_str() == TOKEN_INT
            )
        });
    }

    #[test]
    fn empty_text() {
        let b = Brackets::build_from_string(String::new());
        if let Err(m) = b {
            assert_eq!(m.message(), EMPTY_STRING);
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
            is_first: true,
            linked_idx: 0
        };
        let chk2 = BracketChunk {
            idx: 0,
            typ: BracketType::Date,
            warning_code: 10,
            is_first: false,
            linked_idx: 0
        };
        assert!(chk1 == chk2);
        assert!(chk1.eq(&chk2));
        chk1.idx = 3;
        assert!(chk1 != chk2);
        assert!(chk1.ne(&chk2));
    }

    #[test]
    fn sample_file() {
        let file = File::open("/home/soul/dev/rust/calculator/src/data/db.bk");
        if let Ok(mut f) = file {
            let b = Brackets::build_from_file(f.borrow_mut()).unwrap();
            println!("{}", b);
            assert_eq!(b.get_nb_chunks(), 33);
            assert_eq!(b.get_state(), "Broken");
        }
    }

    #[test]
    fn config_valid() {
        let file = File::open("/home/soul/dev/rust/calculator/src/data/db.bk");
        if let Ok(mut f) = file {
            let b = Brackets::build_from_file(f.borrow_mut()).unwrap();
            assert!(b.config.len() > 0);
        }
    }
}

pub mod bk_error;
pub mod bk_macro;
pub mod bk_regex;
