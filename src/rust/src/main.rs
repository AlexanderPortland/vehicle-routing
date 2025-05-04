mod old_solver;
mod common;
mod vrp_instance;
mod solver;
mod construct;

use std::{env, sync::Arc, time::Instant};
use solver::SolveParams;
use vrp_instance::VRPInstance;
use old_solver::Solver;

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

// fn test_route_cost(vrp_instance: &VRPInstance) {
//     let route = Route {
//         stops: vec![1],
//         known_cost: None,
//         capacity_left: 0,
//     };
//     let expected_cost = 20.0;
//     let actual_cost = route.cost(&vrp_instance.distance_matrix);

//     println!("Expected cost: {}", expected_cost);
//     println!("Actual cost: {}", actual_cost);

//     let route = Route {
//         stops: vec![],
//         known_cost: None,
//         capacity_left: 0,
//     };
//     let expected_cost = 0.0;
//     let actual_cost = route.cost(&vrp_instance.distance_matrix);

//     println!("Expected cost: {}", expected_cost);
//     println!("Actual cost: {}", actual_cost);

//     assert!(expected_cost == actual_cost);

//     let route = solver::Route {
//         stops: vec![1, 2, 3],
//         known_cost: None,
//         capacity_left: 0,
//     };
//     let expected_cost = 10_f64 + 10_f64 + 500_f64.sqrt() + 10_f64;
//     let actual_cost = route.cost(&vrp_instance.distance_matrix);

//     println!("Expected cost: {}", expected_cost);
//     println!("Actual cost: {}", actual_cost);

//     assert!(expected_cost == actual_cost);
// }

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
    let vrp_instance = VRPInstance::new(file_name);
    // let mut solver = Solver::new(vrp_instance);
    // let sol = solver.solve();
    let sol = solver::solve::<solver::TodoSolver>(Arc::new(vrp_instance), SolveParams{max_iters: 1000});
    let duration = start.elapsed();


    let output = json!({
        "Instance": file_name,
        "Time": (duration.as_secs_f64() * 100.0).round() / 100.0,
        "Result": sol.cost(),
        "Solution": sol.to_stdout_string(),
    });
    
    println!("{}", serde_json::to_string(&output).unwrap());

    let sol_path = &format!("./{}.sol", file_name);
    let path = Path::new(sol_path);
    let mut file = File::create(&path).unwrap();
    
    // Write the string to the file
    file.write_all(sol.to_file_string().as_bytes()).unwrap();
}
