
use std::{error::Error, fmt::{Display, Formatter}};
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

pub trait Descriptive {
    fn default_name(&self) -> String;
    fn name(&self) -> String;
}

pub mod indic;
pub mod data;
pub mod date;
pub mod tools;
pub mod fiscalyear;