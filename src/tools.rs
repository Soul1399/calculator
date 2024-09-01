use std::rc::Rc;
use std::ops::Deref;

#[macro_use]
pub mod macros;

pub fn is_between<'a, T>(value: &'a T, min: Option<&'a T>, max: Option<&'a T>) -> Result<Option<bool>, &'static str> where T: PartialOrd + Default {
    if min == None && max == None {
        return Err("Bounds are undefined");
    }
    let df = T::default();
    let a = min.unwrap_or(&df);
    let b = max.unwrap_or(&df);
    Ok(Some(a < value && value < b))
}

pub fn is_between_copy<T>(value: Rc<T>, min: T, max: T) -> bool where T: PartialOrd {
    min < *value && *value < max
}

/// Trait to allow trimming ascii whitespace from a &[u8].
pub trait TrimAsciiWhitespace {
    /// Trim ascii whitespace (based on `is_ascii_whitespace()`) from the
    /// start and end of a slice.
    fn trim_ascii_whitespace(&self) -> &[u8];
}

impl<T: Deref<Target=[u8]>> TrimAsciiWhitespace for T {
    fn trim_ascii_whitespace(&self) -> &[u8] {
        let from = match self.iter().position(|x| !x.is_ascii_whitespace()) {
            Some(i) => i,
            None => return &self[0..0],
        };
        let to = self.iter().rposition(|x| !x.is_ascii_whitespace()).unwrap();
        &self[from..=to]
    }
}

pub mod bracket;
