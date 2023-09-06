use std::{error::Error, path::{Path, PathBuf}, io::Write, fs::{self, File}, collections::HashMap};

use regex::Regex;
use serde::Deserialize;
use duct::cmd;
use indoc::indoc;

use iter_tools::Itertools;

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

/// Sanitize a string such that it can be rendered by pdflatex
fn sanitize_str(str: String) -> String {
    str.replace('_', r"\_")
}

pub fn benchmark() -> Result<(), Box<dyn Error>> {

    // Create a tmp directory
    let tmp = Path::new("tmp/");
    if !tmp.is_dir() {
        fs::create_dir(tmp)?;
    }

    // Either read the previous result or do the benchmarks and generate the output log.
    let mut output_path = PathBuf::new();
    output_path.push(tmp);
    output_path.push("benchmark.json");

    let output = if !output_path.is_file() {
        // Run the benchmarks and capture the output
        let output = cmd!("cargo", "criterion", "--message-format=json").stdout_capture().read()?;
    
        // Write the JSON file for preservation
        fs::write(output_path, &output)?;

        output
    } else {
        fs::read_to_string(output_path)?
    };

    // The regex to extract the benchmark ID
    let name_re = Regex::new(r"(.*) ([0-9]*) ([0-9]*) ([0-9]*)")?;

    let mut benchmarks = HashMap::<String, HashMap::<u64, Vec::<(u64, f64)>>>::new();

    // Take the benchmark data into memory
    for line in output.lines() {
        if let Ok(result) = serde_json::from_str::<CriterionJSON>(line) {

            if result.reason == "benchmark-complete" {
                // Figure out the benchmark ID
                if name_re.is_match(&result.id) {
                    let cap = name_re.captures(&result.id).expect("Benchmark name should match this regex");

                    let name = cap.get(1).unwrap().as_str();
                    let num_threads = cap.get(2).unwrap().as_str().parse::<u64>().unwrap();
                    let num_iterations = cap.get(3).unwrap().as_str();
                    let read_ratio = cap.get(4).unwrap().as_str().parse::<u64>().unwrap();

                    // This is in nanoseconds, convert to milliseconds
                    let estimate = result.mean.estimate / 1_000_000.0;

                    println!(
                        "{} {} {} {} took {}",
                        name, num_threads, num_iterations, read_ratio, estimate
                    );

                    // Insert the entry into the correct hashtable.
                    let read_ratio_map = benchmarks.entry(name.to_string()).or_default();
                    let num_threads_map = read_ratio_map.entry(read_ratio).or_default();

                    num_threads_map.push((num_threads, estimate));
                }
            }
        }
    }

    // Construct a table from the data.
    let mut latex_output = PathBuf::new();
    latex_output.push(tmp);
    latex_output.push("result.tex");
    let mut file = File::create(latex_output).unwrap();

    writeln!(&mut file, indoc!(r"
        \documentclass{{article}}

        \usepackage{{tikz}}
        \usepackage{{pgfplots}}
        
        \pgfplotsset{{compat=1.18}}

        \begin{{document}}"))?;

    // Figure out the tables from the read_ratio.
    let mut read_ratios = Vec::new();
    for result in benchmarks.values() {
        for read_ratio in result.keys() {
            if !read_ratios.contains(&read_ratio) {
                read_ratios.push(read_ratio);
            }
        }
    }
    
    let mut threads = Vec::new();
    for result in benchmarks.values() {
        for timing in result.values() {
            for (num_threads, _) in timing {
                if !threads.contains(&num_threads) {
                    threads.push(num_threads);
                }
            }
        }
    }

    // Sort the tables
    read_ratios.sort_unstable();
    threads.sort_unstable();

    println!("Read ratios: [{}]", read_ratios.iter().join(", "));
    println!("Threads: [{}]", threads.iter().join(", "));

    // Construct one table per ratio
    for ratio in read_ratios {        
        writeln!(&mut file, indoc!(r"
            \begin{{table}}[h]        
            \begin{{tabular}}{{r|r|r|r|r|r|r|r}}
            name & 1 & 2 & 4 & 8 & 16 & 20 \\ \hline"))?;

        for (name, result) in &benchmarks {
            write!(&mut file, "{}", sanitize_str(name.clone()))?;

            for (read_ratio, timing) in result {
                if read_ratio != ratio {
                    continue;
                }

                for (_, time) in timing {
                    // threads should have at least this amount of threads so unwrap is safe.
                    write!(&mut file, " & {:.2}", time)?;
                }
            }

            writeln!(&mut file, r" \\")?;
        }

        // Derive the read percentage.
        let read_percentage = 1.0 - 1.0 / *ratio as f64;

        writeln!(&mut file, indoc!(r"
            \end{{tabular}}
            \caption{{Timing benchmarks for number of threads in ms, with read percentage {}.}}
            \end{{table}}"), read_percentage)?;
        writeln!(&mut file)?;
    }

    writeln!(&mut file, r"\end{{document}}")?;
    
    // Construct Tikz images based on the data

    // for (name, result) in benchmarks {

    //     // Put the read ratio from high to low.
    //     let mut results: Vec::<(u64, Vec::<(u64, f64)>)> = result.into_iter().collect();
    //     results.sort_by(|x, y| {
    //         if x.0 < y.0 {
    //             Ordering::Less
    //         } else {
    //             Ordering::Equal
    //         }
    //     });

    //     for (read_ratio, result) in results.iter_mut() {
            
    //         writeln!(&mut file, indoc!(r"
    //         \begin{{figure}}
    //         \begin{{tikzpicture}}
    //         \begin{{axis}}[
    //             xtick={{1,2,4,8,16,20}},
    //             xmin = 1,
    //             xmax = 20,
    //             xlabel = {{threads}},
    //             ymin = 0.9,
    //             ylabel = {{read}},
    //             ]"))?;

    //         writeln!(&mut file, r"\addplot coordinates {{")?;

    //         // Sort the results by number of threads.
    //         result.sort_by(|x, y| {
    //             if x.0 < y.0 {
    //                 Ordering::Less
    //             } else {
    //                 Ordering::Equal
    //             }
    //         });
            
    //         // Derive the read percentage.
    //         let read_percentage = 1.0 - 1.0 / *read_ratio as f64;

    //         for (num_threads, time) in result {
    //             writeln!(&mut file, "({}, {})", num_threads, time)?;
    //         }

    //         writeln!(&mut file, indoc!(r"}};
            
    //         \end{{axis}}
    //         \end{{tikzpicture}}
    //         \caption{{Benchmark {} with read percentage {}}}
    //         \end{{figure}}"), sanitize_str(name.clone()), read_percentage)?;
    //     }
        
    //     writeln!(&mut file)?;
    // }
    
    Ok(())
}