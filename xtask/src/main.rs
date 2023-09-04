use std::{env, error::Error};


use duct::cmd;

fn benchmark() -> Result<(), Box<dyn Error>> {
    let output = cmd!("cargo", "bench", "bench --message-format=json").read();

    // Write the JSON file for preservations

    // Construct a Tikz image based on the data


    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();

    // Ignore the first argument (which should be xtask)
    args.next();

    // The name of the task
    let task = args.next();

    match task.as_deref() {
        Some("benchmark") => {
            benchmark()?
        },
        Some(x) => {
            println!("Unknown task {}", x);
        }
        None => {    
            println!("Not enough arguments provided, expect at least one");        
        }
    }

    Ok(())
}