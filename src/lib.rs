


pub trait Descriptive {
    fn default_name(&self) -> String;
    fn name(&self) -> String;
}

pub mod compute;
pub mod indic;
pub mod data;
pub mod date;
pub mod tools;
pub mod fiscalyear;
pub mod webserver;