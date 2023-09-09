
use std::error::Error;

use calculator::{data, fiscalyear::FiscalYear, indic::{SLC, FY}, Descriptive};

fn main() -> Result<(), Box<dyn Error>> {
    println!("\nCalcultor is running");
    println!();
    let mut context = data::load_context(1);
    let config = crate::data::get_config();
    let mut list = data::get_all_inputs();
    let compute_keys = FiscalYear::get_keys(&context);

    for k in compute_keys {
        println!("\nComputing fiscal year {}", k.date.to_string());
        data::inputs::compute_by_key(&mut list, &mut context, &k)?;
    }

    println!();
    for i in list {
        let info = i.get_computer(&config);
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
                info.indicator().unwrap().default_name(),
                value);
        }
    }

    Ok(())
}

