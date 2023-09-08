use std::{env, error::Error};

mod sanitizer;
mod benchmark;
mod coverage;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();

    // Ignore the first argument (which should be xtask)
    args.next();

    // The name of the task
    let task = args.next();

    match task.as_deref() {
        Some("convert-benchmark") => {
            let json_path = args.next().expect("Expected .json argument for convert-benchmark task");
            benchmark::benchmark_to_latex(json_path)?
        },
        Some("coverage") => {
            coverage::coverage()?
        },
        Some("address-sanitizer") => {
            sanitizer::address_sanitizer(args.collect())?
        },
        Some("thread-sanitizer") => {
            sanitizer::thread_sanitizer(args.collect())?
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