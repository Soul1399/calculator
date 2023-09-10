
use std::error::Error;

use calculator::{data::{self, inputs::InputContext, monitoring::InputMonitoring}, indic::{SLC, FY}, Descriptive};


fn main() -> Result<(), Box<dyn Error>> {
    let monitor = InputMonitoring::build(
        InputContext::build(1), 
        data::load_context(1));
    
    let mut inputs = data::get_all_inputs();

    let inputs = monitor.compute(&mut inputs)?;

    println!();
    for i in inputs {
        let value: String;
        match i.input.borrow().inputed {
            Some(f) => value = f.to_string(),
            None => match i.input.borrow().computed {
                Some(f) => value = f.to_string(),
                None => value = String::from("None")
            }
        }
        if i.key.span == Some(&FY) || i.key.span == Some(&SLC) {
            println!(
                "\nInput: {} {} {} {}", 
                i.key.date.to_string(),
                i.key.span.unwrap(),
                i.get_indicator().default_name(),
                value);
        }
    }

    Ok(())
}
