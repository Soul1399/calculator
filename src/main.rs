use std::{error::Error, rc::Rc, cell::RefCell};
use calculator::{data::{self, inputs::InputContext, monitoring::InputMonitoring}, indic::{SLC, FY}, Descriptive, build_entities};


fn main() -> Result<(), Box<dyn Error>> {
    build_entities!(json "path");
    let x = 1;
    build_entities!(valueof x);

    let v = build_entities!(listof SLC, FY);
    println!("{}", v.join(","));

    //start_compute()
    Ok(())
}


fn start_compute() -> Result<(), Box<dyn Error>> {
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
        let ltm: String = match *i.ltm.borrow() {
            Some(f) => f.to_string(),
            None => if i.key.span == None { String::from("None") } else { String::from("N/A") }
        };
        if i.key.span == Some(&FY) || i.key.span == Some(&SLC) || i.key.span == None && ltm != "" {
            println!(
                "\nInput: {} {} {} {} (ltm {})", 
                i.key.date.to_string(),
                i.key.span.unwrap_or("None"),
                i.get_indicator().default_name(),
                value,
                ltm);
        }
    }

    Ok(())
}