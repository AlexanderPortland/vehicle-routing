use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process;

pub struct VRPInstance {
    pub num_customers: usize,
    pub num_vehicles: usize,
    pub vehicle_capacity: usize,
    pub demand_of_customer: Vec<usize>,
    pub x_coord_of_customer: Vec<f64>,
    pub y_coord_of_customer: Vec<f64>,
    pub distance_matrix: Vec<Vec<f64>>,
}

impl VRPInstance {
    pub fn new<P: AsRef<Path>>(file_name: P) -> Self {
        let file = match File::open(&file_name) {
            Ok(file) => file,
            Err(_) => {
                eprintln!("Error: in VRPInstance() {:?}\nFile not found", file_name.as_ref());
                process::exit(-1);
            }
        };

        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines()
            .map(|line| line.unwrap_or_else(|e| {
                eprintln!("Error reading line: {}", e);
                process::exit(-1);
            }))
            .collect();

        if lines.is_empty() {
            eprintln!("Error: in VRPInstance() {:?}\nFile is empty", file_name.as_ref());
            process::exit(-1);
        }

        // Parse the first line for number of customers, vehicles, and capacity
        let first_line: Vec<&str> = lines[0].trim().split_whitespace().collect();
        if first_line.len() < 3 {
            eprintln!("Error: in VRPInstance() {:?}\nInvalid first line format", file_name.as_ref());
            process::exit(-1);
        }

        let num_customers = first_line[0].parse::<usize>().unwrap_or_else(|e| {
            eprintln!("Error parsing number of customers: {}", e);
            process::exit(-1);
        });
        
        let num_vehicles = first_line[1].parse::<usize>().unwrap_or_else(|e| {
            eprintln!("Error parsing number of vehicles: {}", e);
            process::exit(-1);
        });
        
        let vehicle_capacity = first_line[2].parse::<usize>().unwrap_or_else(|e| {
            eprintln!("Error parsing vehicle capacity: {}", e);
            process::exit(-1);
        });

        // Initialize arrays for customer data
        let mut demand_of_customer = vec![0; num_customers];
        let mut x_coord_of_customer = vec![0.0; num_customers];
        let mut y_coord_of_customer = vec![0.0; num_customers];

        // Parse customer data
        for i in 0..num_customers {
            if i + 1 < lines.len() {
                let customer_data: Vec<&str> = lines[i + 1].trim().split_whitespace().collect();
                if customer_data.len() < 3 {
                    eprintln!("Error: in VRPInstance() {:?}\nInvalid customer data format at line {}", file_name.as_ref(), i + 2);
                    process::exit(-1);
                }

                demand_of_customer[i] = customer_data[0].parse::<usize>().unwrap_or_else(|e| {
                    eprintln!("Error parsing customer demand: {}", e);
                    process::exit(-1);
                });
                
                x_coord_of_customer[i] = customer_data[1].parse::<f64>().unwrap_or_else(|e| {
                    eprintln!("Error parsing x coordinate: {}", e);
                    process::exit(-1);
                });
                
                y_coord_of_customer[i] = customer_data[2].parse::<f64>().unwrap_or_else(|e| {
                    eprintln!("Error parsing y coordinate: {}", e);
                    process::exit(-1);
                });
            }
        }

        // Print customer data
        for i in 0..num_customers {
            println!("{} {} {}", demand_of_customer[i], x_coord_of_customer[i], y_coord_of_customer[i]);
        }

        // calculate distance matrix
        let distance_matrix: Vec<Vec<f64>> = (0..num_customers).into_iter().map(
            |i| (0..num_customers).into_iter().map(
               |j| ((x_coord_of_customer[i] - x_coord_of_customer[j]).powi(2) +  (y_coord_of_customer[i] - y_coord_of_customer[j]).powi(2)).sqrt()
            ).collect()
        ).collect();

        VRPInstance {
            num_customers,
            num_vehicles,
            vehicle_capacity,
            demand_of_customer,
            x_coord_of_customer,
            y_coord_of_customer,
            distance_matrix
        }
    }

    pub fn to_string(&self) {
        println!("Number of customers: {}", self.num_customers);
        println!("Number of vehicles: {}", self.num_vehicles);
        println!("Vehicle capacity: {}", self.vehicle_capacity);
    }
}
