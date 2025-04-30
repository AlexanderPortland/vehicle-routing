mod vrp_instance;
mod solver;
mod common;

use std::env;
use vrp_instance::VRPInstance;

// fn test_route_cost(vrp_instance: VRPInstance) {
//     let route = solver::Route {
//         stops: vec![1],
//         known_cost: None,
//         capacity: 0,
//     };

//     for row in &vrp_instance.distance_matrix {
//         for val in row {
//             print!("{:>5.2} ", val); // Right-align each value in a 5-char wide field
//         }
//         println!();
//     }

//     let expected_cost = 20.0;
//     let actual_cost = route.calc_route_cost(&vrp_instance.distance_matrix);
    
//     println!("Expected cost: {}", expected_cost);
//     println!("Actual cost: {}", actual_cost);

//     let route = solver::Route {
//         stops: vec![],
//         known_cost: None,
//         capacity: 0,
//     };
//     let expected_cost = 0.0;
//     let actual_cost = route.calc_route_cost(&vrp_instance.distance_matrix);
    
//     println!("Expected cost: {}", expected_cost);
//     println!("Actual cost: {}", actual_cost);
    
//     assert!(expected_cost == actual_cost);

//     let route = solver::Route {
//         stops: vec![1, 2, 3],
//         known_cost: None,
//         capacity: 0,
//     };
//     let expected_cost = 10_f64 + 10_f64 + 500_f64.sqrt() + 10_f64;
//     let actual_cost = route.calc_route_cost(&vrp_instance.distance_matrix);
    
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
    
    let file_name = &args[1];
    
    // Create a new VRPInstance from the provided file
    let vrp_instance = VRPInstance::new(file_name);
    
    // Print instance information
    vrp_instance.to_string();

    // test_route_cost(vrp_instance);
}
