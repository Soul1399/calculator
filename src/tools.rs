use std::rc::Rc;


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

pub mod macros;
