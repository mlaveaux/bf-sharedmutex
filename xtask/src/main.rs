use std::{env, error::Error, path::{Path, PathBuf}, io::Write, fs::{self, File}, collections::HashMap};

use regex::Regex;
use serde::Deserialize;
use serde_json;
use duct::cmd;

mod sanitizer;

#[derive(Deserialize)]
struct MeasurementJSON {
    estimate: f64,
}

#[derive(Deserialize)]
struct CriterionJSON {
    reason: String,
    id: String,
    mean: MeasurementJSON
}

fn benchmark() -> Result<(), Box<dyn Error>> {
    // Write the JSON file for preservations
    let tmp = Path::new("tmp/");

    let mut output_path = PathBuf::new();
    output_path.push(tmp);
    output_path.push("output.log");

    // Either read the previous result or do the benchmarks and generate the output log.
    let output = if !tmp.is_dir() {
        fs::create_dir(tmp)?;

        // Run the benchmarks only once
        let output = match cmd!("cargo", "criterion", "--message-format=json").read() {
            Ok(result) => result,
            result @ Err(_) => {
                println!("Cannot execute \"cargo criterion\", make sure that this cargo feature is installed");
                result?
            }
        };
    
        fs::write(output_path, &output)?;

        output
    } else {
        fs::read_to_string(output_path)?
    };

    // The regex to extract the benchmark ID
    let name_re = Regex::new(r"(.*) ([0-9]*) ([0-9\.]*) ([0-9\.]*)")?;

    let mut benchmarks = HashMap::<String, HashMap::<u64, HashMap::<u64, f64>>>::new();

    // Construct a Tikz image based on the data
    for line in output.lines() {
        if let Ok(result) = serde_json::from_str::<CriterionJSON>(line) {

            if result.reason == "benchmark-complete" {
                // Figure out the benchmark ID
                if name_re.is_match(&result.id) {
                    let mut cap = name_re.captures(&result.id).expect("Benchmark name should match this regex");

                    // format!(
                    //     "{} {} {} {}",
                    //     name, num_threads, num_iterations, read_percentage
                    // )

                    let name = cap.get(0).unwrap().as_str();
                    let num_threads = cap.get(0).unwrap().as_str().parse::<u64>().unwrap();
                    let _ = cap.get(0).unwrap().as_str();
                    let read_ratio = cap.get(0).unwrap().as_str().parse::<u64>().unwrap();

                    //benchmarks[name][&read_ratio][&num_threads] = result.mean.estimate;
                }

            }
        }
    }
    
    let mut latex_output = PathBuf::new();
    latex_output.push(tmp);
    latex_output.push("output.tex");
    let mut file = File::create(latex_output).unwrap();

    write!(&mut file, r"
        \include{{tikz}}

        \begin{{tikzpicture}}
        \begin{{axis}}[
          xtick={{0,4,8,12,16,20,24,28,32}},
          xmin = .8,
          xmax = 32.2,
          ymin = 0,
          width=7cm
          ]")?;

    write!(&mut file, r"
        \addplot[mark options={{fill=gray}}, mark=triangle*] coordinates 
    ")?;

    let bf_sharedmutex = benchmarks.get("bf-sharedmutex::BfSharedMutex").unwrap();



    
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