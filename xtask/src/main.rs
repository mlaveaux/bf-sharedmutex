use std::{env, error::Error};

mod sanitizer;
mod benchmark;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();

    // Ignore the first argument (which should be xtask)
    args.next();

    // The name of the task
    let task = args.next();

    match task.as_deref() {
        Some("benchmark") => {
            benchmark::benchmark()?
        },
        Some("sanitizer") => {
            sanitizer::address_sanitizer(args.collect())?
        }
        Some(x) => {
            println!("Unknown task {}", x);
        }
        None => {    
            println!("Not enough arguments provided, expect at least one");        
        }
    }

    Ok(())
}