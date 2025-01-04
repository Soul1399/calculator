use std::{cell::RefCell, rc::Rc};

use json::JsonValue;
use uuid::Uuid;

use super::{bk_error::BracketsError, BracketArray, BracketSection, BracketType, BracketValue, Brackets, CharSlice, CLOSE, OPEN, PIPE_CHAR};



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
            if let Some(value) = Self::get_basic_section_from_json(value, bval) {
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
                let s = Self::get_basic_section_from_json(value, bval).unwrap();
                target.array.push(Rc::new(s));
            }
        };

        if name.is_none() {
            self.root = Rc::new(RefCell::new(BracketSection::Array(Rc::new(RefCell::new(default_array)))));
        }
        
        Ok(())
    }

    fn get_basic_section_from_json(value: &JsonValue, bval: Option<BracketValue>) -> Option<BracketSection> {
        if bval.is_none() {
            return Some(BracketSection::NoVal);
        }
        else {
            match value {
                JsonValue::Array(_) | JsonValue::Object(_) => { },
                _ => {
                    let v = bval.unwrap();
                    return match v.btyp {
                        BracketType::Int => {
                            Some(BracketSection::Int(Rc::new(RefCell::new(v))))
                        },
                        BracketType::Real => {
                            Some(BracketSection::Real(Rc::new(RefCell::new(v))))
                        },
                        BracketType::Simple | BracketType::FreeText(_) => {
                            Some(BracketSection::Str(Rc::new(RefCell::new(v))))
                        },
                        _ => None
                    };
                }
            }
        }
        None
    }
    
    fn get_json_bracket_type(value: &JsonValue) -> BracketType {
        match value {
            JsonValue::Short(_) => BracketType::Int,
            JsonValue::Number(_) => BracketType::Real,
            JsonValue::String(ref s) => {
                if s.contains(OPEN) || s.contains(CLOSE) {
                    BracketType::FreeText(CharSlice { character: PIPE_CHAR, start: Default::default(), quantity: 1 })
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
            JsonValue::Boolean(_) | JsonValue::Short(_) | JsonValue::Number(_) | JsonValue::String(_) => Some(BracketValue {
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

    fn get_json_section(value: &JsonValue, name: &str) -> BracketSection {
        match value {
            JsonValue::Array(_) | JsonValue::Object(_) => {},
            _ => unreachable!()
        }

        if let JsonValue::Array(a) = value {
            let mut array = BracketArray {
                id: Uuid::new_v4(),
                name: BracketValue { value: Rc::new(RefCell::new(name.to_string())), ..Default::default() },
                ..Default::default()
            };
            //a.iter().map(|x| )
            return BracketSection::Array(Rc::new(RefCell::new(array)));
        }
        Default::default()
    }
}