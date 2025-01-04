/* sample text in data/db.bk */
use std::{cell::RefCell, cmp::Ordering, collections::HashMap, fs::File, io::Read, ops::RangeInclusive, path::Path, rc::Rc};
use bk_error::{BAD_ENDING_COMMENT, BAD_START_COMMENT};
use lazy_static::lazy_static;
use memmap2::{Mmap, MmapOptions};
use uuid::Uuid;

use crate::{bk_config, date::DayDate, tools::bracket::bk_error::{WARNING_ESCAPED, WARNING_FREE_TEXT}};
use self::bk_error::{
    BracketsError, BASE_FORMAT_ERROR, EMPTY_STRING, FORMAT_ERROR, INVALID_CONFIG,
    WARNING_EMPTY_FREE_TEXT, WARNING_MASK,
};

pub const OPEN: char = '[';
pub const CLOSE: char = ']';

const ESCAPE_CHAR: char = '\\';
const COMMA_CHAR: char = ',';
const PIPE_CHAR: char = '|';
const COLON_CHAR: char = ':';
const AT_CHAR: char = '@';
const LEFT_BRACE_CHAR: char = '{';
const RIGHT_BRACE_CHAR: char = '}';

const TOKEN_COMMENT_START: &str = "**[";
const TOKEN_COMMENT_END: &str = "]**";

const RE_OPEN_CONFIG_START: &str = r"^\s*(@\[)";
const RE_OPEN_START: &str = r"^\s*\[[^@]";
const RE_END: &str = r"](?:\*{2})?\s*$";

const CLOSURE_MODE_SMART: &str = "smart";
const CLOSURE_MODE_RAW: &str = "raw";
const CACHE_MODE_OFF: &str = "off";
const CACHE_MODE_ON: &str = "on";
const TRIM_MODE_START: &str = "start";
const TRIM_MODE_WRAP: &str = "space wrapped";
const TRIM_MODE_END: &str = "end";
const TRIM_MODE_FULL: &str = "full";
const TRIM_MODE_OFF: &str = "off";

bk_config!(name, version, allow_empty_free_text, closure_mode, cache, trimming);

lazy_static! {
    static ref TOKEN_PIPE: String = format!("{}{}", OPEN, PIPE_CHAR);
    static ref TOKEN_PIPE_END: String = format!("{}{}", PIPE_CHAR, CLOSE);
    static ref TOKEN_COMMA: String = format!("{}{}", OPEN, COMMA_CHAR);
    static ref TOKEN_COMMA_END: String = format!("{}{}", COMMA_CHAR, CLOSE);
    static ref TOKEN_COLON: String = format!("{}{}", OPEN, COLON_CHAR);
    static ref TOKEN_COLON_END: String = format!("{}{}", COLON_CHAR, CLOSE);
    static ref TOKEN_NAME: String = format!("{}{}", OPEN, AT_CHAR);
    static ref TOKEN_NAME_END: String = format!("{}{}", AT_CHAR, CLOSE);
    
    static ref TOKEN_DATE: String = format!("{}=date{}", OPEN, LEFT_BRACE_CHAR);
    static ref TOKEN_INT: String = format!("{}=int{}", OPEN, LEFT_BRACE_CHAR);
    static ref TOKEN_REAL: String = format!("{}=real{}", OPEN, LEFT_BRACE_CHAR);
    static ref TOKEN_TEXT: String = format!("{}=text{}", OPEN, LEFT_BRACE_CHAR);
    static ref TOKEN_TYPE_END: String = format!("{}{}", RIGHT_BRACE_CHAR, CLOSE);

    static ref RE_OPEN: String = format!(
        r"({}(?:{}+|{}+|[{}{}]|{}|{}|{}|{})?)",
        regex::escape(&OPEN.to_string()),
        regex::escape(&PIPE_CHAR.to_string()),
        regex::escape(&COLON_CHAR.to_string()),
        regex::escape(&COMMA_CHAR.to_string()),
        regex::escape(&AT_CHAR.to_string()),
        regex::escape(&TOKEN_INT[1..]),
        regex::escape(&TOKEN_DATE[1..]),
        regex::escape(&TOKEN_REAL[1..]),
        regex::escape(&TOKEN_TEXT[1..])
    );

    static ref RE_CLOSE: String = format!(
        r"((?:{}+|{}+|[{}{}{}])?{})",
        regex::escape(&PIPE_CHAR.to_string()),
        regex::escape(&COLON_CHAR.to_string()),
        regex::escape(&COMMA_CHAR.to_string()),
        regex::escape(&AT_CHAR.to_string()),
        regex::escape(&RIGHT_BRACE_CHAR.to_string()),
        regex::escape(&CLOSE.to_string())
    );

    static ref RE_FREETEXT_START: String = format!(
        "^({}|{}|{}|{})",
        regex::escape(&TOKEN_PIPE),
        regex::escape(&TOKEN_COLON),
        regex::escape(&TOKEN_COMMA),
        regex::escape(&TOKEN_NAME)
    );

    static ref RE_FREETEXT_END: String = format!(
        "({}|{}|{}|{})$",
        regex::escape(&TOKEN_PIPE_END),
        regex::escape(&TOKEN_COLON_END),
        regex::escape(&TOKEN_COMMA_END),
        regex::escape(&TOKEN_NAME_END)
    );
}

type RefStr = Rc<RefCell<String>>;

#[derive(Debug, Clone, Default)]
pub struct BracketValue {
    pub start: usize,
    pub end: usize,
    pub value: RefStr,
    pub btyp: BracketType,
    pub is_empty: bool
}

impl BracketValue {
    pub fn new(start: usize, end: usize, btype: BracketType) -> BracketValue {
        let mut v: BracketValue = Default::default();
        v.start = start;
        v.end = end;
        v.btyp = btype;
        v
    }
    
    pub fn new_value(name: &str) -> BracketValue {
        let mut v: BracketValue = Default::default();
        v.value = Rc::new(RefCell::new(name.to_owned()));
        v
    }

    pub fn get_length(&self, source_buffer: &str, trim_mode: &str) -> usize {
        let l = self.value.borrow().len();
        if l > 0 {
            return l;
        }
        self.extract_string_from(&source_buffer, trim_mode).len()
    }

    pub fn extract_string_from<'a>(&self, source_buffer: &'a str, trim_mode: &str) -> &'a str {
        let slice: &'a str = &source_buffer[self.start..=self.end];
        if trim_mode == TRIM_MODE_OFF {
            return slice
        }
        if trim_mode == TRIM_MODE_END {
            return slice.trim_end()
        }
        if trim_mode == TRIM_MODE_FULL {
            return slice.trim()
        }
        
        slice.trim_start()
    }

    pub fn extract_int_from(&self, buffer: &str, trim_mode: &str) -> Option<isize> {
        match self.btyp {
            BracketType::Int => {},
            _ => { return None }
        }
        let text = self.extract_string_from(buffer, trim_mode);
        match str::parse::<isize>(text) {
            Err(_) => None,
            Ok(i) => Some(i)
        }
    }

    pub fn extract_real_from(&self, buffer: &str, trim_mode: &str) -> Option<f64> {
        match self.btyp {
            BracketType::Real => {},
            _ => { return None }
        }
        let text = self.extract_string_from(buffer, trim_mode);
        match str::parse::<f64>(text) {
            Err(_) => None,
            Ok(f) => Some(f)
        }
    }

    pub fn extract_list_from(&self, buffer: &str, trim_mode: &str) -> Option<Vec<String>> {
        match self.btyp {
            BracketType::List => {},
            _ => { return None }
        }
        let text = self.extract_string_from(buffer, trim_mode);
        Some(text.split(COMMA_CHAR).into_iter().map(|s| s.trim().to_owned()).collect())
    }

    pub fn extract_date_from(&self, buffer: &str, trim_mode: &str) -> Option<DayDate> {
        match self.btyp {
            BracketType::Date => {},
            _ => { return None }
        }
        let text = self.extract_string_from(buffer, trim_mode);
        match DayDate::parse(text) {
            Err(_) => None,
            Ok(d) => Some(d)
        }
    }
}

#[derive(Debug, Clone)]
pub struct BracketArray {
    pub array: Vec<Rc<BracketSection>>,
    pub is_pure: bool,
    pub name: BracketValue,
    id: Uuid
}

impl Default for BracketArray {
    fn default() -> Self {
        BracketArray {
            array: Default::default(),
            is_pure: false,
            name: BracketValue{ btyp: BracketType::Name, ..Default::default() },
            id: Uuid::new_v4()
        }
    }
}

impl BracketArray {
    pub fn as_section(s: &BracketValue) -> BracketSection {
        let bk_array = Rc::new(RefCell::new(BracketArray::default()));
        bk_array.borrow_mut().name = BracketValue {
            start: s.start, end: s.end, is_empty: s.is_empty, btyp: BracketType::Name, value: Default::default()
        };
        BracketSection::Array(bk_array)
    }
}

type BkSection = Rc<RefCell<BracketSection>>;
type BkValue = Rc<RefCell<BracketValue>>;
type BkArray = Rc<RefCell<BracketArray>>;
//type MutBkSection = Arc<Mutex<BracketSection>>;

#[derive(Debug, Clone)]
pub enum BracketSection {
    Array(BkArray),
    Str(BkValue),
    Int(BkValue),
    Real(BkValue),
    NoVal
}

impl Default for BracketSection {
    fn default() -> Self {
        BracketSection::NoVal
    }
}

impl BracketSection {
    pub fn init_single_value(&mut self, typ: &BracketType) {
        let new_value = match typ {
            BracketType::List => { BracketSection::Array(Default::default()) },
            BracketType::Int => { BracketSection::Int(Default::default()) },
            BracketType::Real => { BracketSection::Real(Default::default()) },
            _ => { BracketSection::Str(Default::default()) }
        };
        match self {
            BracketSection::NoVal => {
                *self = new_value;
            },
            BracketSection::Array(ref vc) => {
                vc.borrow_mut().array.push(Rc::new(new_value));
            },
            _ => unreachable!()
        };
    }

    pub fn set_single_value(&self, value: &BracketValue) {
        match self {
            BracketSection::Str(s) => {
                *s.borrow_mut() = value.clone();
            },
            BracketSection::Int(i) => {
                let mut v = value.clone();
                v.btyp = BracketType::Int;
                *i.borrow_mut() = v;
            },
            BracketSection::Real(r) => {
                let mut v = value.clone();
                v.btyp = BracketType::Real;
                *r.borrow_mut() = v;
            },
            BracketSection::Array(vc) => {
                if vc.borrow().array.len() > 0 {
                    vc.borrow_mut().array[0].set_single_value(value);
                }
            }
            _ => unreachable!()
        };
    }

    pub fn set_noval(&mut self) {
        match self {
            BracketSection::Array(vc) => {
                vc.borrow_mut().array.clear();
            }
            _ => {}
        }
    }

    fn add_child(&mut self, value: BracketSection) {
        match self {
            BracketSection::Array(a) => {
                a.borrow_mut().array.push(Rc::new(value))
            },
            _ => {}
        }
    }

    fn add_children(&mut self, value: BkValue, section: BkSection, result: &BuildValueResult) {
        if let BracketSection::NoVal = self {
            *self = BracketArray::as_section(&value.borrow_mut());
        }
        if let BuildValueResult::Empty = result {
            self.add_child(BracketSection::NoVal);
            return;
        }
        match &*section.borrow() {
            BracketSection::Array(a) => {
                self.add_child(BracketSection::Array(a.clone()));
            }
            _  => unreachable!()
        }
    }

    fn adapt(&mut self, value_result: &BuildValueResult) {
        let mut new_value: Option<BracketSection> = None;
        if let BuildValueResult::Empty = value_result {
            new_value = Some(Default::default())
        }
        else {
            match self {
                BracketSection::Array(ref mut a) => {
                    match value_result {
                        BuildValueResult::Single => {
                            let obj = Rc::clone(&a.borrow().array.iter().next().unwrap());
                            new_value = Some(obj.as_ref().clone());
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        if new_value.is_none() {return;}
        *self = new_value.unwrap();
    }
}

#[repr(isize)]
#[derive(Debug, Clone, Default)]
enum BuildValueResult {
    Tuple = 1,
    Single  = 2,
    #[default]
    Multiple = 3,
    Empty = 4,
    DoubleSingle = 5
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum BracketFlag {
    HasConfig = 1,
    HasBeginBracket = 2,
    HasEndingBracket = 3,
    HasValidConfig = 4,
}

#[derive(Debug, Clone, Default)]
pub enum BracketType {
    #[default]
    Simple,
    FreeText(CharSlice),
    Int,
    Date,
    Real,
    List,
    Name,
    Comment
}

#[derive(Debug, Clone, Copy)]
pub struct CharSlice {
    pub start: usize,
    pub quantity: usize,
    pub character: char,
}

#[derive(Debug, Clone, Default)]
pub struct BracketChunk {
    pub idx: usize,
    pub linked_idx: usize,
    pub typ: BracketType,
    pub warning_code: usize,
    pub is_open_first: Option<bool>
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

    pub fn is_in_chunk(&self, chk: &BracketChunk) -> bool {
        if chk.idx == self.idx { return false; }
        (chk.idx..=chk.linked_idx).contains(&self.idx)
    }

    pub fn is_free_text(&self, free_text_ranges: &Vec<RangeInclusive<usize>>) -> bool {
        free_text_ranges.iter().any(|r| r.contains(&self.idx))
    }

    fn get_inside_value_index(&self) -> usize {
        if self.is_open_first.is_some() {
            return match self.typ {
                BracketType::FreeText(s) => s.start + s.quantity,
                BracketType::List => self.idx + 2,
                BracketType::Date => self.idx + TOKEN_DATE.len(),
                BracketType::Int => self.idx + TOKEN_INT.len(),
                BracketType::Real => self.idx + TOKEN_REAL.len(),
                _ => self.idx + 1
            }
        }
        
        self.idx - 1
    }

    fn get_outside_value_index(&self) -> usize {
        if self.is_open_first.is_none() {
            return match self.typ {
                BracketType::FreeText(s) => s.start + s.quantity + 1,
                BracketType::List => self.idx + 2,
                BracketType::Date => self.idx + TOKEN_DATE.len(),
                BracketType::Int => self.idx + TOKEN_INT.len(),
                BracketType::Real => self.idx + TOKEN_REAL.len(),
                _ => self.idx + 1
            }
        }
        
        self.idx - 1
    }
}

#[derive(Debug)]
pub struct Brackets {
    buffer: String,
    buffer_map: Option<Mmap>,
    file_map: Option<Rc<File>>,
    all_open_bks: Vec<BracketChunk>,
    all_close_bks: Vec<BracketChunk>,
    open_bks: Vec<BracketChunk>,
    close_bks: Vec<BracketChunk>,
    open_bks_hash: HashMap<usize, usize>,
    close_bks_hash: HashMap<usize, usize>,
    flags: Vec<BracketFlag>,
    is_processing: bool,
    is_valid: Option<bool>,
    start_index: Option<usize>,
    pub file_name: Option<String>,
    pub root: BkSection,
    pub config: BracketConfig
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
            buffer_map: None,
            file_map: None,
            open_bks: Default::default(),
            open_bks_hash: HashMap::new(),
            close_bks: Default::default(),
            close_bks_hash: HashMap::new(),
            all_open_bks: Default::default(),
            all_close_bks: Default::default(),
            flags: Default::default(),
            root: Default::default(),
            file_name: Default::default(),
            is_processing: false,
            is_valid: None,
            start_index: None,
            config: Default::default()
        }
    }
}

impl Drop for Brackets {
    fn drop(&mut self) {
        if self.file_map.is_some() {
            self.buffer_map = None;
            self.file_map = None;
        }
    }
}

impl Brackets {
    pub fn build_from_string(text: String) -> Result<Brackets, BracketsError> {
        let mut new = Brackets::default();
        new.buffer = text;
        new.process_buffer()?;
        Ok(new)
    }

    pub fn build_from_file_string(file: &mut File, file_name: &str) -> Result<Brackets, BracketsError> {
        let mut buf: String = Default::default();
        let result = file.read_to_string(&mut buf);
        if let Ok(_) = result {
            let mut bk = Brackets::build_from_string(buf)?;
            bk.file_name = Some(file_name.to_string());
            return Ok(bk)
        }

        Err(BracketsError::new(format!("Could not read file: {}", result.err().unwrap()).as_str()))
    }

    pub fn build_from_file_map(filename: &str) -> Result<Brackets, BracketsError> {
        let result = File::open(filename);
        let file: File;
        if let Err(e) = result {
            return Err(BracketsError::new(&format!("Cannot open file {} {}", filename, e)))
        }
        file = result.unwrap();
        let m = unsafe { MmapOptions::new().map(&file) };
        if let Err(e) = m {
            return Err(BracketsError::new(&format!("OS error {} occured", e.raw_os_error().unwrap())));
        }
        
        let mut new = Brackets::default();
        new.buffer_map = Some(m.unwrap());
        new.file_map = Some(Rc::new(file));
        new.file_name = Some(Path::new(filename).file_name().unwrap().to_str().unwrap().to_string());
        new.process_buffer()?;
        Ok(new)
    }

    pub fn get_nb_chunks(&self) -> usize {
        self.open_bks.len()
    }

    pub fn is_cache_off(&self) -> bool {
        self.config.cache != CACHE_MODE_ON
    }

    pub fn get_trim_mode(&self) -> &str {
        if self.config.trimming.len() > 0 {
            return self.config.trimming.as_str();
        }
        TRIM_MODE_START
    }

    pub fn get_state(&self) -> &str {
        let invalid = "Invalid";
        let broken = "Broken";
        let ok = "Ok";

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

    pub fn get_start_index(&self) -> usize {
        if self.start_index.is_some() { return self.start_index.unwrap() }
        let mut start_index: usize = 0;
        let mut enm = self.open_bks.iter().peekable();
        while let Some(bk) = enm.peek() {
            if let Some(false) | None = bk.is_open_first { enm.next(); }
            else {
                start_index = bk.idx;
                break;
            }
        }
        start_index
    }

    fn get_buffer(&self) -> &str {
        if let Some(ref map) = self.buffer_map {
            let result = std::str::from_utf8(&map[..]);
            return &result.unwrap();
        }
        return &self.buffer;
    }

    fn process_buffer(&mut self) -> Result<(), BracketsError> {
        self.is_processing = true;
        self.reset();
        self.spot_bounds()?;
        self.try_link_bounds()?;
        self.start_index = Some(self.get_start_index());
        self.build_values(None, None)?;
        self.is_processing = false;
        Ok(())
    }

    fn reset(&mut self) {
        self.open_bks.clear();
        self.all_open_bks.clear();
        self.close_bks.clear();
        self.all_close_bks.clear();
        self.open_bks_hash.clear();
        self.close_bks_hash.clear();
        self.flags.clear();
        self.is_valid = None;
    }

    fn preset(&mut self) {
        self.reset();
        self.is_valid = Some(true);
        self.start_index = Some(0);
        self.config = Default::default(); 
    }

    fn spot_bounds(&mut self) -> Result<(), BracketsError> {
        self.check_buffer()?;
        self.collect_comments()?;
        self.check_start();
        self.check_end();
        self.primal_validation()?;
        self.is_valid = Some(false);
        self.collect_open_bounds();
        self.collect_close_bounds();
        self.remove_non_bracket_bounds();
        self.is_valid = Some(self.open_bks.len() == self.close_bks.len());
        self.init_root()?;
        self.validate()?;
        Ok(())
    }

    fn try_link_bounds(&mut self) -> Result<(), BracketsError> {
        if let Err(e) = self.link_bounds() {
            self.is_valid = Some(false);
            self.is_processing = false;
            return Err(e)
        }
        Ok(())
    }

    fn link_bounds(&mut self) -> Result<(), BracketsError> {
        let mut scan = true;
        let mut stack_idx: Vec<usize> = Vec::new();
        let start_index = self.get_start_index();
        let all_bks = self.get_bounds_mut();
        let mut enm = all_bks.into_iter();
        
        while scan {
            if let Some(bk) = enm.next() {
                if bk.idx < start_index { continue; }
                if bk.is_open_first.is_some() { stack_idx.push(bk.idx) }
                else {
                    if let Some(x) = stack_idx.pop() { bk.linked_idx = x }
                    else { return Err(BracketsError::error_close(bk)) }
                }
            }
            else {
                scan = false
            }
        }
        
        let mut map_clos: HashMap<usize, usize> = HashMap::new();
        self.close_bks.iter().enumerate().filter(|(_,c)| c.idx > start_index).for_each(|(u,c)| {
            map_clos.insert(c.linked_idx, c.idx);
            self.close_bks_hash.insert(c.idx, u);
        });
        self.open_bks.iter_mut().enumerate().filter(|(_,o)| o.idx >= start_index).for_each(|(u,o)| {
            o.linked_idx = *map_clos.get(&o.idx).unwrap();
            self.open_bks_hash.insert(o.idx, u);
        });

        self.is_valid = Some(true);
        Ok(())
    }

    fn build_values(&mut self, p: Option<BkSection>, s: Option<usize>) -> Result<BuildValueResult, BracketsError> {
        if p.is_none() {
            let root_start_index = self.get_start_index();
            let doc_value = Rc::clone(&self.root);
            let result = self.build_values(Some(doc_value), Some(root_start_index))?;
            return Ok(result);
        }

        let parent = p.unwrap();
        let p_start = s.unwrap();
        let open_parent = &self.open_bks[*self.open_bks_hash.get(&p_start).unwrap()];
        let found = self.open_bks.iter().any(|o| o.is_in_chunk(open_parent));
        let start = open_parent.get_inside_value_index();
        let result: BuildValueResult;
        if !found {
            let close_parent = &self.close_bks[*self.close_bks_hash.get(&open_parent.linked_idx).unwrap()];
            let end = close_parent.get_inside_value_index();
            result = self.build_single_value(p_start, parent.clone(), start, end);
        }
        else {
            result = self.build_child_values(open_parent.idx+1..open_parent.linked_idx, start, parent.clone())?;
        }
        //parent.borrow_mut().adapt(&result);
        Ok(result)
    }

    fn build_child_values(&mut self, parent_range: std::ops::Range<usize>, start: usize, parent: BkSection) -> Result<BuildValueResult, BracketsError> {
        let mut children_iter = self.open_bks.iter()
            .filter(|o| parent_range.contains(&o.idx))
            .peekable();
        let mut bypass_ranges: Vec<RangeInclusive<usize>> = Vec::new();
        let mut all_vals_empty = true;
        let mut children_map: Vec<(usize, BkValue)> = Vec::new();
        let trim_mode = self.get_trim_mode();
        while children_iter.peek().is_some() {
            let child = children_iter.next().unwrap();
            if bypass_ranges.iter().any(|rg| rg.contains(&child.idx)) { continue; }

            let child_close = &self.close_bks[*self.close_bks_hash.get(&child.linked_idx).unwrap()];
            let key_end = child.get_outside_value_index();
            let mut key_start = start;
            if bypass_ranges.len() > 0 {
                key_start = *bypass_ranges.last().unwrap().end();
            }
            bypass_ranges.push(child.get_inside_value_index()..=child_close.get_outside_value_index());
            bypass_ranges.sort_by_key(|x| *x.start());

            let mut val = BracketValue::new(key_start, key_end, Default::default());
            if val.get_length(self.get_buffer(), trim_mode) > 0 {
                all_vals_empty = false;
                if !self.is_cache_off() {
                    *val.value.borrow_mut() = val.extract_string_from(self.get_buffer(), trim_mode).to_owned();
                }
            }
            else {
                val.is_empty = true;
            }
            
            children_map.push((child.idx, Rc::new(RefCell::new(val))));
        }

        let len = children_map.len();
        let mut last_result: BuildValueResult = BuildValueResult::Empty;
        
        for (size, v) in children_map {
            let array_section = BracketArray::as_section(&v.borrow_mut());
            let s = Rc::new(RefCell::new(array_section));
            last_result = self.build_values(Some(s.clone()), Some(size))?;
            parent.borrow_mut().add_children(v, s.clone(), &last_result);
        }

        if len == 1 {
            return Ok(
                if let BuildValueResult::Single | BuildValueResult::Empty = last_result {
                    BuildValueResult::DoubleSingle
                }
                else { BuildValueResult::Single }
            );
        }

        Ok(if all_vals_empty { BuildValueResult::Tuple } else { BuildValueResult::Multiple })
    }

    fn build_single_value(&mut self, p_start : usize, parent: BkSection, start: usize, end: usize) -> BuildValueResult {
        let trim_mode = self.get_trim_mode();
        let typ = self.define_single_value_type(parent.clone(), p_start);
        let val = BracketValue::new(start, end, typ);
        if !self.is_cache_off() {
            *val.value.borrow_mut() = val.extract_string_from(self.get_buffer(), trim_mode).to_owned();
        }
        if val.get_length(self.get_buffer(), trim_mode) == 0 {
            parent.borrow_mut().set_noval();
            return BuildValueResult::Empty;
        }
        
        parent.borrow_mut().set_single_value(&val);
        BuildValueResult::Single
    }

    fn init_root(&mut self) -> Result<(), BracketsError> {
        self.root = Default::default();
        if self.flags.contains(&BracketFlag::HasConfig) {
            if self.open_bks[1].idx > self.close_bks[0].idx {
                // empty config => remove flag
                self.open_bks[1].is_open_first = Some(true);
                self.flags.retain(|f| *f != BracketFlag::HasConfig);
            } else {
                let (end_index, map) = bk_regex::extract_config(&self.get_buffer()[self.open_bks[0].idx..]);
                if map.len() == 0 {
                    return Err(BracketsError::new(INVALID_CONFIG));
                }
                //self.root.config = Rc::new(map);
                for (k, v) in map {
                    self.config.set_config(k.as_str(), v.as_str());
                }
                self.flags.push(BracketFlag::HasValidConfig);
                let firstb = self.open_bks.iter_mut().find(|o| o.idx > end_index);
                if let Some(b) = firstb {
                    b.is_open_first = Some(true);
                }
            }
        }
        else {
            self.open_bks[0].is_open_first = Some(true);
        }
        
        Ok(())
    }

    fn validate(&self) -> Result<(), BracketsError> {
        if !self.is_valid.unwrap() {
            let mask_found = self
                .open_bks
                .iter()
                .find(|b| b.warning_code == WARNING_MASK);
            if mask_found.is_none() || self.config.closure_mode == CLOSURE_MODE_RAW {
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
        if self.get_buffer().trim_end().len() == 0 {
            return Err(BracketsError::new(EMPTY_STRING));
        }
        Ok(())
    }

    fn collect_comments(&mut self) -> Result<(), BracketsError> {
        let bf = self.get_buffer();
        if bf.contains(TOKEN_COMMENT_START) && bf.contains(TOKEN_COMMENT_END) {
            return self.identify_comment_ranges();
        }
        let bf = self.get_buffer();
        if bf.contains(TOKEN_COMMENT_START) {
            return Err(BracketsError::new(BAD_ENDING_COMMENT));
        }
        if bf.contains(TOKEN_COMMENT_END) {
            return Err(BracketsError::new(BAD_START_COMMENT));
        }
        
        Ok(())
    }

    fn identify_comment_ranges(&mut self) -> Result<(), BracketsError> {
        let mut comments = bk_regex::collect_comments(self.get_buffer());
        comments.sort();
        let mut comments_iter = comments.iter();
        let mut ok = true;
        let mut odd = true;
        let mut open_idx: usize = 0;
        while ok {
            let o = comments_iter.next();
            if o.is_none() { ok = false; }
            else {
                odd = !odd;
                let bound = o.unwrap();
                if odd && bound.1 == TOKEN_COMMENT_START.chars().next().unwrap() {
                    return Err(BracketsError::new(BAD_ENDING_COMMENT));
                }
                if !odd && bound.1 == TOKEN_COMMENT_END.chars().next().unwrap() {
                    return Err(BracketsError::new(BAD_START_COMMENT));
                }
                if odd {
                    self.all_close_bks.push(BracketChunk { idx: bound.0, linked_idx: open_idx, typ: BracketType::Comment, is_open_first: None, warning_code: 0 });
                    let x = self.all_open_bks.iter().position(|c| c.idx == open_idx).unwrap();
                    let mut open = self.all_open_bks[x].clone();
                    open.linked_idx = bound.0;
                    self.all_open_bks[x] = open;
                }
                else {
                    self.all_open_bks.push(BracketChunk { idx: bound.0, linked_idx: 0, typ: BracketType::Comment, is_open_first: None, warning_code: 0 });
                    open_idx = bound.0;
                }
            }
        }
        Ok(())
    }

    fn get_comment_ranges(&self) -> Vec<RangeInclusive<usize>> {
        self.all_open_bks.iter()
            .filter(|b| match b.typ { BracketType::Comment => true, _ => false })
            .map(|b| b.idx..=b.linked_idx)
            .collect()
    }

    fn check_start(&mut self) {
        self.identify_config();
        if !self.flags.contains(&BracketFlag::HasConfig) {
            let comment_ranges = self.get_comment_ranges();
            if bk_regex::match_simple_start(self.get_buffer(), comment_ranges) {
                self.flags.push(BracketFlag::HasBeginBracket);
            }
        }
    }

    fn identify_config(&mut self) {
        if bk_regex::match_start(self.get_buffer()) {
            self.flags.push(BracketFlag::HasConfig);
        }
    }

    fn check_end(&mut self) {
        if bk_regex::match_end(self.get_buffer()) {
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
            .extend(bk_regex::collect_bounds(self.get_buffer(), &RE_OPEN));
        let comment_ranges = self.get_comment_ranges();
        self.open_bks.retain(|b| !comment_ranges.iter().any(|r| r.contains(&b.idx)));
        self.open_bks.sort();
        self.all_open_bks.extend(self.open_bks.iter().map(|o| o.clone()));
    }

    fn collect_close_bounds(&mut self) {
        self.close_bks
            .extend(bk_regex::collect_bounds(self.get_buffer(), &RE_CLOSE));
        let comment_ranges = self.get_comment_ranges();
        self.close_bks.retain(|b| !comment_ranges.iter().any(|r| r.contains(&b.idx)));
        self.close_bks.sort();
        self.all_close_bks.extend(self.close_bks.iter().map(|o| o.clone()));
    }

    fn remove_non_bracket_bounds(&mut self) {
        let mut real_open_bks: Vec<BracketChunk> = vec![];
        let mut real_close_bks: Vec<BracketChunk> = vec![];

        let (escaped_slices, mut free_text_ranges) =
            self.identify_open_bks(&mut real_open_bks, &mut real_close_bks);
        self.identify_close_bks(&mut real_close_bks, &escaped_slices, &mut free_text_ranges);

        real_open_bks.sort();
        real_close_bks.sort();
        self.open_bks = real_open_bks;
        self.close_bks = real_close_bks;
    }

    fn identify_open_bks(
        &mut self,
        real_open_bks: &mut Vec<BracketChunk>,
        real_close_bks: &mut Vec<BracketChunk>,
    ) -> (Vec<CharSlice>, Vec<RangeInclusive<usize>>) {
        let mut x: usize = 0;
        let length = self.open_bks.len();
        let mut search = true;
        let escaped_slices = bk_regex::collect_escaped(self.get_buffer());
        let mut warning: Option<usize> = None;
        let mut free_text_ranges: Vec<RangeInclusive<usize>> = vec![];
        while search {
            let mut open_iter = self.open_bks[x..].iter();
            while x < length {
                warning = None;
                let bk = open_iter.next().unwrap();
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
                    BracketType::FreeText(_) | BracketType::List => break,
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
            let index = x;
            x += 1;
            if let Some(code) = warning {
                self.open_bks[index].warning_code = code;
                continue;
            }
            let close_bk = self.extract_free_text_range(
                index,
                real_open_bks,
                &mut free_text_ranges);
            
            if let Some(c) = close_bk {
                let chr = match c.typ {
                    BracketType::FreeText(s) => s.character,
                    _ => Default::default()
                };
                if chr != AT_CHAR {
                    real_close_bks.push(c);
                }
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
        let notyet_identified: Vec<&BracketChunk> = self
            .close_bks
            .iter()
            .filter(|c| !real_close_bks.contains(&c))
            .collect();

        real_close_bks.extend(
            notyet_identified
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
        let found = self.find_free_text_close_bk(&self.open_bks[index]);
        if found.is_none() { return found; }
        let cbk = found.unwrap();
        let start_idx = self.open_bks[index].idx;
        if cbk.idx == self.open_bks[index].idx || cbk.idx == start_idx + 2 {
            let mut obk = self.open_bks[index].clone();
            obk.typ = Default::default();
            real_open_bk.push(obk);
        } else {
            let open_slice = match self.open_bks[index].typ {
                BracketType::FreeText(slc) => slc,
                BracketType::List => CharSlice {
                    start: start_idx + 1,
                    quantity: 1,
                    character: COMMA_CHAR,
                },
                _ => unreachable!(),
            };
            if self.open_bks.len() < index + 1 {
                self.find_open_alike(index, &cbk, start_idx, open_slice);
            }
            let start = open_slice.start + open_slice.quantity;
            let end = match cbk.typ {
                BracketType::FreeText(_) | BracketType::List => cbk.idx - 1,
                _ => unreachable!(),
            };
            if open_slice.character != AT_CHAR {
                free_text_ranges.push(start..=end);
                real_open_bk.push(self.open_bks[index].clone());
            }
            else {
                free_text_ranges.push(start-1..=end+1);
            }
        }

        if cbk != self.open_bks[index] {
            return Some(cbk);
        }
        
        None
    }

    fn find_open_alike(&mut self, index: usize, cbk: &BracketChunk, start_idx: usize, open_slice: CharSlice) {
        let open_alike = self.open_bks[index + 1..]
            .iter()
            .filter(|o| o.idx < cbk.idx)
            .find(|o| match o.typ {
                BracketType::FreeText(slc) => {
                    open_slice.character == slc.character
                        && open_slice.quantity == slc.quantity
                }
                BracketType::List => {
                    open_slice.character == COMMA_CHAR && open_slice.quantity == 1
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
    
    fn find_free_text_close_bk(&self, open_bk: &BracketChunk) -> Option<BracketChunk> {
        let mut ft_char: Option<char> = None;
        let mut nb_ft_char: usize = 0;
        match open_bk.typ {
            BracketType::FreeText(slc) => {
                ft_char = Some(slc.character);
                nb_ft_char = slc.quantity;
            }
            BracketType::List => {
                ft_char = Some(COMMA_CHAR);
                nb_ft_char = 1;
            }
            _ => { }
        }

        if let Some(chr) = ft_char {
            let b = self.find_free_text_end(open_bk.idx, chr, nb_ft_char);
            if b.is_none() {
                return Some(open_bk.clone());
            }
            return b;
        }

        None
    }

    fn find_free_text_end(&self, start: usize, chr: char, size: usize) -> Option<BracketChunk> {
        let o = self.close_bks.iter().filter(|c| c.idx > start).find(|c| {
            if let BracketType::FreeText(slc) = c.typ {
                return size == slc.quantity && chr == slc.character;
            }
            return false;
        });
        if o.is_none() {
            return None;
        }
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
        Some(cbk)
    }

    fn get_bounds_mut<'a>(&'a mut self) -> Vec<&'a mut BracketChunk> {
        let mut list: Vec<&'a mut BracketChunk> = Vec::new();
        list.extend(self.open_bks.iter_mut());
        list.extend(self.close_bks.iter_mut());
        list.sort();
        list
    }

    fn define_single_value_type(&self, p: BkSection, idx: usize) -> BracketType {
        let o = &self.open_bks[*self.open_bks_hash.get(&idx).unwrap()];
        p.borrow_mut().init_single_value(&o.typ);
        o.typ.clone()
    }
}

#[cfg(test)]
mod tests_brackets {
    use std::borrow::BorrowMut;

    use super::bk_regex::RGX_OPEN;

    use super::*;

    #[test]
    fn search_open_bk() {
        let re = &RGX_OPEN;
        let haystack = "@[version[=int{1}]";
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
                    || c.get(0).unwrap().as_str() == TOKEN_INT.as_str()
            )
        });
    }

    #[test]
    fn invalid_empty_text() {
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
    fn empty_text() {
        let b = Brackets::build_from_string(String::from("[]"));
        assert!(b.is_ok());
    }

    #[test]
    fn simple_text() {
        let b = Brackets::build_from_string(String::from("[ \"test\" ]"));
        assert!(b.is_ok());
    }

    #[test]
    fn empty_single() {
        let b = Brackets::build_from_string(String::from("[noval[]]"));
        assert!(b.is_ok());
        let brackets = b.unwrap();
        let root = brackets.root.clone();
        let mut array: BkArray = Default::default();
        assert!(match &*root.borrow() {
            BracketSection::Array(a) => {
                array = a.clone();
                a.borrow().array.len() == 1
            },
            _ => false
        });
        
        assert!(match &*array.borrow().array[0] {
            BracketSection::NoVal => true,
            _ => false
        });
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
            is_open_first: Some(true),
            linked_idx: 0
        };
        let chk2 = BracketChunk {
            idx: 0,
            typ: BracketType::Date,
            warning_code: 10,
            is_open_first: Some(false),
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
            let b = Brackets::build_from_file_string(f.borrow_mut(), "db").unwrap();
            assert_eq!(b.get_state(), "Ok");
            assert_eq!(b.get_nb_chunks(), 33);
        }
    }

    #[test]
    fn config_valid() {
        let file = File::open("/home/soul/dev/rust/calculator/src/data/db.bk");
        if let Ok(mut f) = file {
            let b = Brackets::build_from_file_string(f.borrow_mut(), "db").unwrap();
            assert_eq!(b.config.name, "trees");
            assert_eq!(b.config.version, "1");
            assert_eq!(b.config.trimming, "start");
        }
    }

    #[test]
    fn obj_doc() {
        let file = File::open("/home/soul/dev/rust/calculator/src/data/obj.bk");
        if let Ok(mut f) = file {
            let b = Brackets::build_from_file_string(f.borrow_mut(), "obj").unwrap();
            assert!(b.is_valid.unwrap_or_default());
            assert_eq!(b.get_state(), "Ok");
        }
    }

    #[test]
    fn obj_doc_map() {
        let b = Brackets::build_from_file_map("/home/soul/dev/rust/calculator/src/data/obj.bk").unwrap();
        assert!(b.is_valid.unwrap_or_default());
        assert_eq!(b.get_state(), "Ok");
    }
}

pub mod bk_error;
pub mod bk_macro;
pub mod bk_regex;
pub mod bk_query;
pub mod bk_json;
