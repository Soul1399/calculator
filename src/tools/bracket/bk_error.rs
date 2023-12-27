use std::{error::Error, fmt::{Display, Formatter}};

pub const UNKNOWN_ERROR: &str = "Unknown brackets error";
pub const INVALID_CONFIG: &str = "Invalid configuration";
pub const EMPTY_STRING: &str = "Empty string";
pub const BASE_FORMAT_ERROR: &str = "Start and/or end of the text is malformed";
pub const FORMAT_ERROR: &str = "Input string is malformed";

pub const WARNING_EMPTY_FREE_TEXT: usize = 1;
pub const WARNING_ESCAPED: usize = 2;
pub const WARNING_FREE_TEXT: usize = 3;
pub const WARNING_MASK: usize = 4;

#[derive(Debug)]
pub struct BracketsError {
    message: String,
    warning: String
}

impl BracketsError {
    pub fn new(message: &str) -> BracketsError {
        Self::error(String::from(message))
    }

    pub fn error(message: String) -> BracketsError {
        BracketsError { message, warning: Default::default() }
    }

    pub fn warning(message: String) -> BracketsError {
        BracketsError { message: Default::default(), warning: message }
    }

    pub(crate) fn message(&self) -> &String {
        if &self.message.len() > &0 { &self.message }
        else { &self.warning }
    }
}

impl Error for BracketsError {
    fn description(&self) -> &str {
        &self.message()
    }
}

impl Default for BracketsError {
    fn default() -> Self {
        Self::error(UNKNOWN_ERROR.to_owned())
    }
}

impl Display for BracketsError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f,"{}", self.message())
    }
}