use std::{cell::RefCell, rc::Rc};

use date_format_parser::parse_date;
use json::JsonValue;

use super::{bk_error::BracketsError, bk_regex::RGX_STD_INT, BkArray, BracketArray, BracketSection, BracketType, BracketValue, Brackets, CharSlice, CLOSE, COLON_CHAR, OPEN, PIPE_CHAR};

use lexical_parse_integer::FromLexical;

impl Brackets {
    pub fn build_from_json(json_string: &str) -> Result<Brackets, BracketsError> {
        let json_result = json::parse(json_string);
        if let Err(e) = json_result {
            return Err(BracketsError::new(&format!("Unable to read json ({})", e)));
        }
        let json = json_result.unwrap();
        Self::build_from_json_value(json)
    }

    pub fn build_from_json_value(value: JsonValue) -> Result<Brackets, BracketsError> {
        let mut new = Brackets::default();
        new.start_process_json(&value)?;
        Ok(new)
    }

    fn start_process_json(&mut self, value: &JsonValue) -> Result<(), BracketsError> {
        self.preset();
        self.is_processing = true;
        if let Err(e) = self.process_json(&value, None, &mut Default::default()) {
            self.is_valid = Some(false);
            return Err(e);
        }
        self.is_processing = false;
        Ok(())
    }
    
    fn process_json(&mut self, value: &JsonValue, name: Option<&str>, target: &mut BracketArray) -> Result<(), BracketsError> {
        let mut default_array = BracketArray {
            name: BracketValue { value: Rc::new(RefCell::new(name.unwrap_or_default().to_string())), btyp: BracketType::Name, ..Default::default() },
            ..Default::default()
        };
        if name.is_none() {
            let bval = Self::get_json_bracket_value(value, name);
            if let Some(value) = Self::get_basic_section_from_json(value, bval, name) {
                self.root = Rc::new(RefCell::new(value));
                return Ok(());
            }
        }
        
        match value {
            JsonValue::Array(_) | JsonValue::Object(_) => {
                let mut new_array = default_array.clone();
                let container: &mut BracketArray = match name {
                    None => &mut default_array,
                    Some(_) => &mut new_array
                };
                match value {
                    JsonValue::Array(a) => {
                        for item in a.iter() {
                            self.process_json(item, Some(""), container)?;
                        }
                    },
                    JsonValue::Object(o) => {
                        for (n,v) in o.iter() {
                            self.process_json(v, Some(n), container)?;
                        }
                    },
                    _ => unreachable!()
                };
                if name.is_some() {
                    target.array.push(Rc::new(BracketSection::Array(Rc::new(RefCell::new(new_array)))));
                }
            },
            _ => {
                let bval = Self::get_json_bracket_value(value, name);
                let s = Self::get_basic_section_from_json(value, bval, name).unwrap();
                target.array.push(Rc::new(s));
            }
        };

        if name.is_none() {
            self.root = Rc::new(RefCell::new(BracketSection::Array(Rc::new(RefCell::new(default_array)))));
        }
        
        Ok(())
    }

    fn get_basic_section_from_json(value: &JsonValue, bval: Option<BracketValue>, name: Option<&str>) -> Option<BracketSection> {
        let array = BkArray::new(RefCell::new(Default::default()));
        array.borrow_mut().name = BracketValue { btyp: BracketType::Name, value: Rc::new(RefCell::new(name.unwrap_or_default().to_string())), ..Default::default() };
        if bval.is_none() {
            if name.is_some() {
                array.borrow_mut().array.push(Rc::new(BracketSection::NoVal));
            }
            else {
                return Some(BracketSection::NoVal);
            }
        }
        else {
            match value {
                JsonValue::Array(_) | JsonValue::Object(_) => { },
                _ => {
                    let v = bval.unwrap();
                    let sub_section: BracketSection;
                    match v.btyp {
                        BracketType::Int => {
                            sub_section = BracketSection::Int(Rc::new(RefCell::new(v)));
                        },
                        BracketType::Real => {
                            sub_section = BracketSection::Real(Rc::new(RefCell::new(v)));
                        },
                        BracketType::Simple | BracketType::FreeText(_) | BracketType::Date => {
                            sub_section = BracketSection::Str(Rc::new(RefCell::new(v)));
                        },
                        _ => {
                            return None;
                        }
                    };
                    if name.is_some() {
                        array.borrow_mut().array.push(Rc::new(sub_section));
                    }
                    else {
                        return Some(sub_section);
                    }
                }
            }
        }
        if array.borrow().array.len() > 0 {
            let section = BracketSection::Array(array);
            return Some(section)
        }
        None
    }
    
    fn get_json_bracket_type(value: &JsonValue) -> BracketType {
        if Self::json_number_is_integer(value) {
            return BracketType::Int;
        }
        if Self::json_string_is_date(value) {
            return BracketType::Date;
        }
        match value {
            JsonValue::Number(_) => BracketType::Real,
            JsonValue::Short(_) | JsonValue::String(_) => {
                let s: &str = match value {
                    JsonValue::Short(short) => short.as_str(),
                    JsonValue::String(s) => &s,
                    _ => unreachable!()
                };
                if s.contains(OPEN) || s.contains(CLOSE) {
                    BracketType::FreeText(CharSlice {
                        character: if s.starts_with(PIPE_CHAR) { COLON_CHAR } else { PIPE_CHAR }, 
                        start: Default::default(), 
                        quantity: 1
                    })
                }
                else {
                    BracketType::Simple
                }
            },
            _ => BracketType::Simple
        }
    }

    fn get_json_bracket_value(value: &JsonValue, name: Option<&str>) -> Option<BracketValue> {
        let btype = Brackets::get_json_bracket_type(value);
        match value {
            JsonValue::Null => None,
            JsonValue::Short(_) | JsonValue::String(_) => Some(BracketValue {
                value: Rc::new(RefCell::new(value.to_string())), 
                btyp: btype,
                ..Default::default()
            }),
            JsonValue::Boolean(_) | JsonValue::Number(_) => Some(BracketValue {
                value: Rc::new(RefCell::new(value.dump())), 
                btyp: btype,
                ..Default::default()
            }),
            JsonValue::Array(_) => Some(Default::default()),
            JsonValue::Object(_) => Some(BracketValue {
                value: Rc::new(RefCell::new(name.unwrap_or_default().to_string())),
                btyp: btype,
                ..Default::default()
            })
        }
    }

    fn json_number_is_integer(value: &JsonValue) -> bool {
        match value {
            JsonValue::Number(_) => {
                let s = value.dump();
                if RGX_STD_INT.is_match(&s) {
                    return true;
                }
                let bytes = s.as_bytes();
                let parse_int = isize::from_lexical(bytes);
                if parse_int.is_ok() {
                    return true;
                }
                let parse_int = usize::from_lexical(bytes);
                if parse_int.is_ok() {
                    return true;
                }
                let parse_int = i64::from_lexical(bytes);
                if parse_int.is_ok() {
                    return true;
                }
                let parse_int = u64::from_lexical(bytes);
                if parse_int.is_ok() {
                    return true;
                }
                false
            },
            _ => false
        }
    }

    fn json_string_is_date(value: &JsonValue) -> bool {
        match value {
            JsonValue::Short(_) | JsonValue::String(_) => {
                let s: &str = match value {
                    JsonValue::Short(short) => short.as_str(),
                    JsonValue::String(s) => &s,
                    _ => unreachable!()
                };
                if let Ok(_) = parse_date(s) {
                    return true;
                }
                false
            },
            _ => false
        }
    }
}

#[cfg(test)]
mod tests_brackets_json {
    use std::{fs::File, io::Read, path::Path};
    use crate::tools::bracket::{BracketSection, BracketType, Brackets};

    const PATH_FILES: &str = "/home/soul/dev/rust/calculator/src/data";

    #[test]
    fn test_empty_json() {
        let filename = Path::new(PATH_FILES).join("empty.json");
        let file = File::open(filename.to_str().unwrap());
        if let Ok(mut f) = file {
            let mut buf: String = Default::default();
            _ = f.read_to_string(&mut buf);
            let result = Brackets::build_from_json(&buf);
            assert!(result.is_ok());
            let bk = result.unwrap();
            let root = bk.root.borrow_mut();
            if let BracketSection::Array(ref a) = *root {
                assert_eq!(a.borrow().array.len(), 0);
            }
            else {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_json_values() {
        let json = "toto";
        let result = Brackets::build_from_json(&format!("\"{}\"", json));
        assert!(result.is_ok());
        let bk = result.unwrap();
        let root = bk.root.borrow_mut();
        if let BracketSection::Str(ref a) = *root {
            assert_eq!(a.borrow().get_length("", bk.get_trim_mode()), json.len());
            assert_eq!(a.borrow().value.borrow().as_str(), json);
            assert!(match a.borrow().btyp { BracketType::Simple => true, _ => false });
        }
        else {
            assert!(false);
        }

        let json = "object[45]";
        let result = Brackets::build_from_json(&format!("\"{}\"", json));
        assert!(result.is_ok());
        let bk = result.unwrap();
        let root = bk.root.borrow_mut();
        if let BracketSection::Str(ref a) = *root {
            assert_eq!(a.borrow().get_length("", bk.get_trim_mode()), json.len());
            assert_eq!(a.borrow().value.borrow().as_str(), json);
            assert!(match a.borrow().btyp { BracketType::FreeText(_) => true, _ => false });
        }
        else {
            assert!(false);
        }

        let json = 40;
        let result = Brackets::build_from_json(&json.to_string());
        assert!(result.is_ok());
        let bk = result.unwrap();
        let root = bk.root.borrow_mut();
        if let BracketSection::Int(ref a) = *root {
            assert_eq!(a.borrow().value.borrow().as_str(), &json.to_string());
            assert!(match a.borrow().btyp { BracketType::Int => true, _ => false });
        }
        else {
            assert!(false);
        }
        
        let json = 140.6652;
        let result = Brackets::build_from_json(&json.to_string());
        assert!(result.is_ok());
        let bk = result.unwrap();
        let root = bk.root.borrow_mut();
        if let BracketSection::Real(ref a) = *root {
            assert_eq!(a.borrow().value.borrow().as_str(), &json.to_string());
            assert!(match a.borrow().btyp { BracketType::Real => true, _ => false });
        }
        else {
            assert!(false);
        }
    }

    #[test]
    fn test_single_line_file() {
        let filename = Path::new(PATH_FILES).join("products.json");
        let file = File::open(filename.to_str().unwrap());
        if let Ok(mut f) = file {
            let mut buf: String = Default::default();
            _ = f.read_to_string(&mut buf);
            let result = Brackets::build_from_json(&buf);
            assert!(result.is_ok());
            let bk = result.unwrap();
            let root = bk.root.borrow_mut();
            if let BracketSection::Array(ref a) = *root {
                assert_eq!(a.borrow().array.len(), 4);
                let item = a.borrow().array.iter().next().unwrap().clone();
                if let BracketSection::Array(ref i) = item.as_ref() {
                    assert_eq!(i.borrow().name.value.borrow().as_str(), "products");
                    assert_eq!(i.borrow().array.len(), 30);
                    for item in i.borrow().array.iter() {
                        match &*item.clone() {
                            BracketSection::Array(a) => {
                                assert_eq!(a.borrow().name.value.borrow().len(), 0)
                            }
                            _ => {}
                        }
                    }
                }
                assert!(true);
            }
            else {
                assert!(false);
            }
        }
    }
}
