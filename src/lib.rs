
use std::{error::Error, fmt::{Display, Formatter}, rc::Rc};
use date::DateKey;

pub struct ComputeKey {
    pub date: DateKey,
    pub span: Option<&'static str>
}

#[derive(Debug)]
pub struct ComputeError {
    details: String
}

impl Error for ComputeError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl Display for ComputeError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f,"{}",self.details)
    }
}

pub trait Computable {
    fn compute<T>(&self, values: &Vec<Rc<T>>) -> Result<T, ComputeError> where T: Default {
        let length = values.len();
        if length == 0 {
            return Err(ComputeError { details: String::new() });
        }
        Ok(Default::default())
    }
}

pub trait Descriptive {
    fn default_name(&self) -> String;
    fn name(&self) -> String;
}

pub mod indic;
pub mod data;
pub mod date;
pub mod tools;
pub mod fiscalyear;
pub mod webserver;