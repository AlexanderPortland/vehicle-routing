mod common;
mod construct;
mod jump;
mod solver;
pub mod solvers;
mod swap;
mod vrp_instance;
mod check_sol;

use check_sol::check;
use common::VRPSolution;
use solver::{SolveParams, TermCond};
use core::num;
use std::cmp::Reverse;
use std::thread;
use std::time::Duration;
use std::{env, sync::Arc, time::Instant};
use vrp_instance::VRPInstance;

use serde_json::{json, to_string_pretty};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use ordered_float::OrderedFloat;

fn get_filename_from_path(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|filename| filename.to_str())
        .unwrap_or("")
}

fn main() {
    // run check() w/ filename if you want to verify correctness of a x.logs file

    // Check if a file name was provided as a command-line argument
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        // If no arguments provided, run the test
        return;
    }

    let file_path = &args[1];
    let file_name = get_filename_from_path(file_path);

    let start = Instant::now();
    let frac_dropped = 0.00;
    let patience = 10;
    let mut join_handles = Vec::new();
    let num_cpus: usize = num_cpus::get();
    println!("Running {} threads...", num_cpus);
    for i in 0..num_cpus {
        let vrp_instance = Arc::new(VRPInstance::new(file_path));
        let constructor = if i % 3 == 0 {
            construct::sweep_then_clarke_wright
        } else {
            construct::clarke_wright_and_then_sweep
        };
        join_handles.push(thread::spawn( move || {
            solver::solve::<solvers::ALNSSolver>(
                vrp_instance,
                SolveParams {
                    // terminate: TermCond::MaxIters(0),
                    terminate: TermCond::TimeElapsed(Duration::from_secs(299)),
                    frac_dropped: frac_dropped,
                    patience: patience,
                    constructor: constructor,
                    jumper: jump::random_jump,
                    initial_solution: None,
                },
            )
        }));
    }

    let sols = join_handles.into_iter().map(|x| x.join().unwrap()).collect::<Vec<VRPSolution>>();
    let best_sol = sols.into_iter().max_by_key(|x| Reverse(OrderedFloat(x.cost()))).unwrap();
    best_sol.check();

    let duration = start.elapsed();
    let output = json!({
        "Instance": file_name,
        "Time": (duration.as_secs_f64() * 100.0).round() / 100.0,
        "Result": (best_sol.cost() * 100.0).round() / 100.0,
        "Solution": best_sol.to_string(),
    });
    println!("{}", serde_json::to_string(&output).unwrap());

    // Write the string to the file
    // let sol_path = &format!("./{}.sol", file_name);
    // let path = Path::new(sol_path);
    // let mut file = File::create(&path).unwrap();
    // file.write_all( best_sol.to_file_string().as_bytes()).unwrap();
}
