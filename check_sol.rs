use std::collections::HashSet;
use std::{env, path::Path};
use crate::vrp_instance::VRPInstance;
use crate::common::{Stop, Route, VRPSolution};
use std::fs::File;
use std::io::{self, BufRead};
use serde_json::Deserializer;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
struct Solution {
    Instance: String,
    Result: f64,
    Solution: String,
    Time: f64,
}

pub fn check() {
    // Check if a file name was provided as a command-line argument
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // If no arguments provided, run the test
        return;
    }

    let file_path = &args[1];

    let file = File::open(file_path).unwrap();
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let sol: Solution = serde_json::from_str(&line.unwrap()).expect("deserialization failed");
        println!("Processing: {}", sol.Instance);
        let vrp_instance = VRPInstance::new(format!("../../input/{}", sol.Instance));
        let mut routes = Vec::new();
        let mut route = Vec::new();
        let mut customer_set= HashSet::new();
        for (i, cust_no_str) in sol.Solution.split_whitespace().enumerate() {
            if i == 0 {
                continue
            }
            let cust_no: usize = cust_no_str.parse().unwrap();
            if cust_no == 0 {
                if !route.is_empty() {
                    routes.push(route);
                }
                route = Vec::new();
            } else {
                customer_set.insert(cust_no);
                route.push(cust_no);
            }
        }
        for i in 1..vrp_instance.num_customers {
            if !customer_set.contains(&i) {
                panic!("uh oh; a customer isn't in the final route");
            }
        }

        for route in routes.iter() {
            let demands: Vec<usize> = route.iter().map(|x| vrp_instance.demand_of_customer[*x]).collect();
            let demand_served: usize = demands.iter().sum();
            if demand_served > vrp_instance.vehicle_capacity {
                panic!("uh oh; over capacity");
            }

            let mut distance = 0f64;
            for i in 0..(route.len() - 1) {
                let cust_a = route[i];
                let cust_b = route[i + 1];
                distance += ((vrp_instance.x_coord_of_customer[cust_a] - vrp_instance.x_coord_of_customer[cust_b]).powi(2) +  (vrp_instance.y_coord_of_customer[cust_a] - vrp_instance.y_coord_of_customer[cust_b]).powi(2)).sqrt();
            }
            distance += ((vrp_instance.x_coord_of_customer[route[0]] - vrp_instance.x_coord_of_customer[0]).powi(2) +  (vrp_instance.y_coord_of_customer[route[0]] - vrp_instance.y_coord_of_customer[0]).powi(2)).sqrt();
            distance += ((vrp_instance.x_coord_of_customer[route[route.len() - 1]] - vrp_instance.x_coord_of_customer[0]).powi(2) +  (vrp_instance.y_coord_of_customer[route[route.len() - 1]] - vrp_instance.y_coord_of_customer[0]).powi(2)).sqrt();

            if (distance - sol.Result) > 0.1 {
                panic!("uh oh; result might be wrong");
            }
        }

        println!("{} passed checks", sol.Instance);
    }
}
