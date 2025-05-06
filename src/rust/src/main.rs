mod common;
mod vrp_instance;
mod solver;
mod construct;
mod swap;
mod jump;
pub mod solvers;

use std::time::Duration;
use std::{env, sync::Arc, time::Instant};
use solver::{SolveParams, TermCond};
use vrp_instance::VRPInstance;

use serde_json::{json, to_string_pretty};
use std::path::Path;
use std::fs::File;
use std::io::Write;

fn get_filename_from_path(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|filename| filename.to_str())
        .unwrap_or("")
}

fn main() {
    // Check if a file name was provided as a command-line argument
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // If no arguments provided, run the test
        return;
    }

    let file_path = &args[1];
    let file_name = get_filename_from_path(file_path);

    let start = Instant::now();
    let vrp_instance = VRPInstance::new(file_path);
    let frac_patience = 0.05;
    let patience = (vrp_instance.num_customers as f64 * frac_patience) as usize;

    let sol = solver::solve::<solvers::MultiLNSSolver>(
        Arc::new(vrp_instance), 
        SolveParams {
            terminate: TermCond::MaxIters(40000),
            // terminate: TermCond::TimeElapsed(Duration::from_secs(60 * 1)),
            patience,
            constructor: construct::clarke_wright_and_then_sweep,
            jumper: jump::random_drop,
        }
    );
    let duration = start.elapsed();

    let output = json!({
        "Instance": file_name,
        "Time": (duration.as_secs_f64() * 100.0).round() / 100.0,
        "Result": sol.cost(),
        "Solution": sol.to_string(),
    });
    
    println!("{}", serde_json::to_string(&output).unwrap());

    let sol_path = &format!("./{}.sol", file_name);
    let path = Path::new(sol_path);
    let mut file = File::create(&path).unwrap();
    
    // Write the string to the file
    file.write_all(sol.to_file_string().as_bytes()).unwrap();
}

