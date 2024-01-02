/* sample text in data/db.bk */

use std::{cmp::Ordering, collections::HashMap, fs::File, io::Read, ops::RangeInclusive, rc::Rc, cell::RefCell};
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
const TOKEN_COMMA_END: &str = ":]";
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
const CONFIG_ALLOW_EMPTY_FT: &str = "allow empty free text";
const CONFIG_CLOSURE_MODE: &str = "closure mode";
const CLOSURE_MODE_RAW: &str = "raw";
const CONFIG_CACHE: &str = "cache";
const CACHE_MODE_OFF: &str = "off";
const CACHE_MODE_ON: &str = "on";
const CONFIG_TRIMMING: &str = "trimming";
const TRIM_MODE_START: &str = "start";
const TRIM_MODE_END: &str = "end";
const TRIM_MODE_FULL: &str = "full";
const TRIM_MODE_OFF: &str = "off";

#[derive(Debug, Clone, Default)]
pub struct BracketId {
    pub start: usize,
    pub end: usize,
    pub id_value: Option<String>,
    pub btyp: BracketType
}

impl BracketId {
    pub fn new_id(start: usize, end: usize) -> BracketId {
        let mut id: BracketId = Default::default();
        id.start = start;
        id.end = end;
        id
    }
    
    pub fn new_value(name: &str) -> BracketId {
        let mut id: BracketId = Default::default();
        id.id_value = Some(name.to_owned());
        id
    }

    pub fn get_length(&self, buffer: &str, trim_mode: &str) -> usize {
        if let Some(s) = &self.id_value {
            return s.len()
        }
        self.extract_string_from(buffer, trim_mode).len()
    }

    pub fn extract_string_from<'a>(&self, buffer: &'a str, trim_mode: &str) -> &'a str {
        let slice = &buffer[self.start..=self.end];
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
}

#[derive(Debug, Clone, Default)]
pub struct BkDoc {
    pub name: String,
    pub value: Rc<RefCell<BracketValue>>
}

#[derive(Debug, Clone, Default)]
pub struct BkArray {
    pub value: Vec<Rc<BracketValue>>
}

#[derive(Debug, Clone)]
pub enum BracketValue {
    Root(Rc<RefCell<BracketValue>>, Rc<RefCell<BkDoc>>),
    Array(Rc<RefCell<BkArray>>),
    Obj(Rc<RefCell<BracketId>>, Rc<RefCell<BracketValue>>),
    Prop(Rc<RefCell<BracketId>>, Rc<BracketValue>),
    Str(Rc<RefCell<BracketId>>),
    Int(Rc<RefCell<BracketId>>),
    Real(Rc<RefCell<BracketId>>),
    NoVal
}

impl Default for BracketValue {
    fn default() -> Self {
        BracketValue::NoVal
    }
}

impl BracketValue {
    pub fn init_single_value(&mut self, typ: &BracketType) {
        let new_value = match typ {
            BracketType::List(_) => { todo!() },
            BracketType::Int => { BracketValue::Int(Default::default()) },
            BracketType::Real => { BracketValue::Real(Default::default()) },
            BracketType::Simple | BracketType::Date | BracketType::FreeText(_) => { BracketValue::Str(Default::default()) }
        };
        match self {
            BracketValue::Array(ref vc) => {
                vc.borrow_mut().value.push(Rc::new(new_value));
            },
            BracketValue::Prop(_, ref mut v) => {
                *v = Rc::new(new_value);
            },
            _ => { }
        };
    }

    pub fn set_str_value(&self, value: &BracketId) {
        match self {
            BracketValue::Prop(_, v) => {
                match v.as_ref() {
                    BracketValue::Str(_) | BracketValue::Int(_) | BracketValue::Real(_) => {
                        v.set_str_value(value);
                    },
                    _ => unreachable!()
                }
            },
            BracketValue::Str(s) => {
                *s.borrow_mut() = value.clone();
            },
            BracketValue::Int(i) => {
                let mut v = value.clone();
                v.btyp = BracketType::Int;
                *i.borrow_mut() = v;
            },
            BracketValue::Real(r) => {
                let mut v = value.clone();
                v.btyp = BracketType::Real;
                *r.borrow_mut() = v;
            },
            BracketValue::Array(vc) => {
                vc.borrow_mut().value[0].set_str_value(value);
            }
            _ => unreachable!()
        };

        //true
    }

    pub fn set_id_value(&self, start: usize, end: usize) {

    }

    pub fn set_noval(&mut self, is_doc: bool) {
        match self {
            BracketValue::Prop(_, v) => {
                *v = Default::default();
            },
            BracketValue::Array(vc) => {
                if !is_doc { vc.borrow_mut().value.clear() }
            }
            _ => {}
        }

        if is_doc { *self = Default::default() }
    }

    pub fn init_obj(&self, id_val: BracketId) -> Rc<BracketValue> {
        let obj = BracketValue::Obj(
            Rc::new(RefCell::new(id_val)),
            Rc::new(RefCell::new(BracketValue::Array(Rc::new(RefCell::new(BkArray { value: Default::default() })))))
        );
        let obj_rc = Rc::new(obj);
        let rc = Rc::clone(&obj_rc);
        match self {
            BracketValue::Array(a) => {
                a.borrow_mut().value.push(obj_rc)
            }
            BracketValue::Obj(_, vc) => {
                if let BracketValue::Array(ref a) = *vc.borrow() {
                    a.borrow_mut().value.push(obj_rc)
                }
            },
            _ => {}
        }

        rc
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

    pub fn is_free_text(&self, free_text_ranges: &Vec<RangeInclusive<usize>>) -> bool {
        free_text_ranges.iter().any(|r| r.contains(&self.idx))
    }

    fn get_inside_value_index(&self) -> usize {
        if self.is_open_first.is_some() {
            return match self.typ {
                BracketType::FreeText(s) => s.start + s.quantity,
                BracketType::List(s) => self.idx + s + 1,
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
                BracketType::List(s) => self.idx + s + 1,
                BracketType::Date => self.idx + TOKEN_DATE.len(),
                BracketType::Int => self.idx + TOKEN_INT.len(),
                BracketType::Real => self.idx + TOKEN_REAL.len(),
                _ => self.idx + 1
            }
        }
        
        self.idx - 1
    }
}

#[derive(Debug, Clone)]
pub struct Brackets {
    buffer: String,
    all_open_bks: Vec<BracketChunk>,
    all_close_bks: Vec<BracketChunk>,
    open_bks: Vec<BracketChunk>,
    close_bks: Vec<BracketChunk>,
    open_bks_hash: HashMap<usize, usize>,
    close_bks_hash: HashMap<usize, usize>,
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
            open_bks_hash: HashMap::new(),
            close_bks: Default::default(),
            close_bks_hash: HashMap::new(),
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
        let result = file.read_to_string(&mut buf);
        if let Ok(_) = result {
            return Brackets::build_from_string(buf);
        }

        Err(BracketsError::new(format!("Could not read file: {}", result.err().unwrap()).as_str()))
    }

    pub fn default_props() -> Vec<BracketValue> {
        return vec![
            BracketValue::Prop(
                Rc::new(RefCell::new(BracketId::new_value(CONFIG_NAME))),
                Default::default(),
            ),
            BracketValue::Prop(
                Rc::new(RefCell::new(BracketId::new_value(CONFIG_VERSION))),
                Default::default(),
            ),
            BracketValue::Prop(
                Rc::new(RefCell::new(BracketId::new_value(CONFIG_TRIMMING))),
                Rc::new(BracketValue::Str(Rc::new(RefCell::new(BracketId::new_value(TRIM_MODE_START)))))
            ),
            BracketValue::Prop(
                Rc::new(RefCell::new(BracketId::new_value(CONFIG_ALLOW_EMPTY_FT))),
                Rc::new(BracketValue::Str(Rc::new(RefCell::new(BracketId::new_value(false.to_string().as_str())))))
            ),
            BracketValue::Prop(
                Rc::new(RefCell::new(BracketId::new_value(CONFIG_CLOSURE_MODE))),
                Rc::new(BracketValue::Str(Rc::new(RefCell::new(BracketId::new_value(CLOSURE_MODE_RAW)))))
            ),
            BracketValue::Prop(
                Rc::new(RefCell::new(BracketId::new_value(CONFIG_CACHE))),
                Rc::new(BracketValue::Str(Rc::new(RefCell::new(BracketId::new_value(CACHE_MODE_ON)))))
            )
        ];
    }

    pub fn get_nb_chunks(&self) -> usize {
        self.open_bks.len()
    }

    pub fn is_cache_off(&self) -> bool {
        self.config.get(CONFIG_CACHE).unwrap_or(&String::new()) != CACHE_MODE_ON
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

    pub fn get_start_index(&self) -> usize {
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

    fn process_buffer(&mut self) -> Result<(), BracketsError> {
        self.is_processing = true;
        self.reset();
        self.spot_bounds()?;
        if let Err(e) = self.link_bounds() {
            self.is_valid = Some(false);
            self.is_processing = false;
            return Err(e)
        }
        let root_index = self.get_start_index();
        self.build_values(root_index, None, None)?;
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
        self.init_root()?;
        self.validate()?;
        Ok(())
    }

    fn link_bounds(&mut self) -> Result<(), BracketsError> {
        let mut scan = true;
        let mut stack_idx: Vec<usize> = Vec::new();
        let start_index = self.get_start_index();
        let all_bks = self.get_bounds_mut();
        let mut enm = all_bks.into_iter().peekable();
        
        while scan {
            if let Some(bk) = enm.next() {
                if bk.idx < start_index { continue; }
                if bk.warning_code == WARNING_MASK {
                    todo!()
                }
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

    fn build_values(&mut self, root_start_index: usize, p: Option<Rc<RefCell<BracketValue>>>, s: Option<usize>) -> Result<(), BracketsError> {
        if p.is_none() {
            let doc_value = match &self.root {
                BracketValue::Root(_, d) => Rc::clone(&d.borrow_mut().value),
                _ => unreachable!()
            };
            self.build_values(root_start_index, Some(doc_value), Some(root_start_index))?;
            return Ok(())
        }

        let parent = p.unwrap();
        let p_start = s.unwrap();
        let open_parent = &self.open_bks[*self.open_bks_hash.get(&p_start).unwrap()];
        let found = self.open_bks.iter().any(|o| open_parent.idx < o.idx && o.idx < open_parent.linked_idx);
        let start = open_parent.get_inside_value_index();
        
        if !found {
            let close_parent = &self.close_bks[*self.close_bks_hash.get(&open_parent.linked_idx).unwrap()];
            let end = close_parent.get_inside_value_index();
            self.build_single_value(&parent, p_start, start, end, root_start_index);
        }
        else {
            self.build_child_values(open_parent.idx, open_parent.linked_idx, start, parent, root_start_index)?;
        }
        Ok(())
    }

    fn build_child_values(&mut self, open_parent_idx: usize, close_parent_idx: usize, start: usize, parent: Rc<RefCell<BracketValue>>, root_start_index: usize) -> Result<(), BracketsError> {
        let mut children_it = self.open_bks.iter()
            .filter(|o| open_parent_idx < o.idx && o.idx < close_parent_idx)
            .peekable();
        let mut bypass_ranges: Vec<RangeInclusive<usize>> = Vec::new();
        let mut all_ids_empty = true;
        let mut child_map: Vec<(usize, Rc<RefCell<BracketValue>>)> = Vec::new();
        let trim_mode = self.config.get(CONFIG_TRIMMING).unwrap();
        while children_it.peek().is_some() {
            let child = children_it.next().unwrap();

            if bypass_ranges.iter().any(|rg| rg.contains(&child.idx)) { continue; }

            let child_close = &self.close_bks[*self.close_bks_hash.get(&child.linked_idx).unwrap()];
            bypass_ranges.push(child.get_inside_value_index()..=child_close.get_outside_value_index());
            bypass_ranges.sort_by_key(|x| *x.start());

            let key_end = child.get_outside_value_index();
            let mut key_start = start;
            if bypass_ranges.len() > 0 {
                key_start = *bypass_ranges.last().unwrap().end();
            }

            let mut id_val = BracketId::new_id(key_start, key_end);
            if id_val.get_length(&self.buffer, trim_mode) > 0 {
                all_ids_empty = false;
            }
            else if !self.is_cache_off() {
                id_val.id_value = Some(id_val.extract_string_from(&self.buffer, trim_mode).to_owned());
            }
            let val = parent.borrow_mut().init_obj(id_val);
            let obj = match val.as_ref() {
                BracketValue::Obj(_, a) => Rc::clone(a),
                _ => unreachable!()
            };
            child_map.push((child.idx, obj));
        }
        Ok(for (s, o) in child_map {
            self.build_values(root_start_index, Some(o), Some(s))?;
        })
    }

    fn build_single_value(&mut self, parent: &Rc<RefCell<BracketValue>>, p_start: usize, start: usize, end: usize, root_start_index: usize) {
        let trim_mode = self.config.get(CONFIG_TRIMMING).unwrap();
        self.define_single_value_type(Rc::clone(parent), p_start);
        let mut id_val = BracketId::new_id(start, end);
        if !self.is_cache_off() {
            id_val.id_value = Some(id_val.extract_string_from(&self.buffer, trim_mode).to_owned());
        }
        if id_val.get_length(&self.buffer, trim_mode) == 0 {
            parent.borrow_mut().set_noval(p_start == root_start_index);
        }
        else {
            parent.borrow_mut().set_str_value(&id_val);
        }
    }

    fn init_root(&mut self) -> Result<(), BracketsError> {
        let mut props: Vec<Rc<BracketValue>> = Brackets::default_props()
            .into_iter()
            .map(|v| Rc::new(v))
            .collect();
        let key = BracketId::new_value("@");
        let doc = BkDoc {
            name: self.config.get(CONFIG_NAME).unwrap().clone(), 
            value: Rc::new(RefCell::new(BracketValue::Array(Default::default())))
        };
        if self.flags.contains(&BracketFlag::HasConfig) {
            if self.open_bks[1].idx > self.close_bks[0].idx {
                // empty config => remove flag
                self.open_bks[1].is_open_first = Some(true);
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
                            Rc::new(RefCell::new(BracketId::new_value(pair.0.as_str()))),
                            Rc::new(BracketValue::Str(Rc::new(RefCell::new(BracketId::new_value(pair.1))))),
                        ))
                    })
                    .collect();

                self.config = map;
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
        self.root = BracketValue::Root(
            Rc::new(RefCell::new(BracketValue::Obj(
                Rc::new(RefCell::new(key)), 
                Rc::new(RefCell::new(BracketValue::Array(Rc::new(RefCell::new(BkArray { value: props })))))))),
            Rc::new(RefCell::new(doc))
        );
        Ok(())
    }

    fn validate(&self) -> Result<(), BracketsError> {
        if !self.is_valid.unwrap() {
            let mask_found = self
                .open_bks
                .iter()
                .find(|b| b.warning_code == WARNING_MASK);
            if mask_found.is_none()
                || self.config.get(CONFIG_CLOSURE_MODE) == Some(&CLOSURE_MODE_RAW.to_owned())
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
                // println!("warning {} broke on {}", code, index + 1);
                continue;
            }
            //println!("broke on {}", index + 1);
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
            if cbk.idx == self.open_bks[index].idx || cbk.idx == start_idx + 2 {
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

    fn get_bounds_mut<'a>(&'a mut self) -> Vec<&'a mut BracketChunk> {
        let mut list: Vec<&'a mut BracketChunk> = Vec::new();
        self.open_bks.iter_mut().for_each(|o| list.push(o));
        self.close_bks.iter_mut().for_each(|o| list.push(o));
        list.sort();
        list
    }

    fn define_single_value_type(&self, p: Rc<RefCell<BracketValue>>, idx: usize) {
        let o = &self.open_bks[*self.open_bks_hash.get(&idx).unwrap()];
        p.borrow_mut().init_single_value(&o.typ);
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
            let b = Brackets::build_from_file(f.borrow_mut()).unwrap();
            println!("{}", b);
            assert_eq!(b.get_nb_chunks(), 34);
            assert_eq!(b.get_state(), "Ok");
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
