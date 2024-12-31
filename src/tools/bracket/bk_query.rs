use super::{bk_error::BracketsError, BracketSection, Brackets};

pub const BRACKETS_NO_READY: &str = "Brackets is currently processing data";
pub const BRACKETS_INVALID: &str = "Brackets data are invalid";
pub const INVALID_SEARCH_TERM: &str = "Invalid search term";

impl Brackets {
    pub fn can_search(&self) -> Result<(), BracketsError> {
        if self.is_processing | self.is_valid.is_none() {
            return Err(BracketsError::new(BRACKETS_NO_READY));
        }
        if let Some(false) = self.is_valid {
            return Err(BracketsError::new(BRACKETS_INVALID));
        }
        Ok(())
    }
    
    pub fn find(&self, term: &str) -> Result<Vec<&BracketSection>, BracketsError> {
        Brackets::find_term_check(term)?;
        self.can_search()?;
        
        Ok(vec![])
    }

    fn find_term_check(term: &str) -> Result<(), BracketsError> {
        if term.trim_end().len() == 0 {
            return Err(BracketsError::new(INVALID_SEARCH_TERM));
        }

        Ok(())
    }
}