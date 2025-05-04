use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::{SeedableRng, rng};

use crate::{common::Stop, common::VRPSolution, vrp_instance::VRPInstance};
use std::cmp::Reverse;
use rand::rngs::StdRng;

pub fn greedy(vrp_instance: &Arc<VRPInstance>) -> VRPSolution {
    let mut customer_nos: Vec<usize> = (1..vrp_instance.num_customers).collect();
    customer_nos.sort_by_key(|&i| Reverse(vrp_instance.demand_of_customer[i]));

    let mut sol = VRPSolution::new(vrp_instance.clone());

    for cust_no in customer_nos {
        let demand = vrp_instance.demand_of_customer[cust_no];
        let mut found = false;
        for vehicle_idx in 0..vrp_instance.num_vehicles {
            if vrp_instance.vehicle_capacity - sol.routes[vehicle_idx].used_capacity() >= demand
            {
                let len = sol.routes[vehicle_idx].stops().len();
                sol.routes[vehicle_idx].add_stop_to_index(
                    Stop::new(cust_no.try_into().unwrap(), demand),
                    len,
                );

                found = true;
                break;
            }
        }
        if !found { panic!("greedy strategy doesn't work here!!"); }
    }
    sol
}

pub fn cheapest_insertion(vrp_instance: &Arc<VRPInstance>) -> VRPSolution {
    let mut customer_nos: Vec<usize> = (1..vrp_instance.num_customers).collect();

    // randomly shuffled
    // let seed = 42u64;
    // let mut rng = StdRng::seed_from_u64(seed);
    let mut rng = rng();
    customer_nos.shuffle(&mut rng);

    let mut sol = VRPSolution::new(vrp_instance.clone());
    for cust_no in customer_nos {
        let demand = vrp_instance.demand_of_customer[cust_no];
        let mut best_stop_idx: Option<usize> = None;
        let mut best_vehicle_idx: Option<usize> = None;
        let mut best_cost_delta = f64::MAX;

        for vehicle_idx in 0..vrp_instance.num_vehicles {
            let route = &sol.routes[vehicle_idx];
            let ((cost, feasible), stop_idx) = route.speculative_add_best(&Stop::new(cust_no.try_into().unwrap(), demand));
            if feasible && cost - route.cost() < best_cost_delta {
                best_cost_delta = cost - route.cost();
                best_stop_idx = Some(stop_idx);
                best_vehicle_idx = Some(vehicle_idx);
            }
        }
        if best_cost_delta == f64::MAX {
            panic!("Could not insert cust no: {}", cust_no);
        }

        sol.routes[best_vehicle_idx.unwrap()].add_stop_to_index(
            Stop::new(cust_no.try_into().unwrap(), demand),
            best_stop_idx.unwrap(),
        );
    }
    sol
}

pub fn sweep(vrp_instance: &Arc<VRPInstance>) -> VRPSolution {
    let mut sol = VRPSolution::new(vrp_instance.clone());

    let mut customer_nos: Vec<usize> = (1..vrp_instance.num_customers).collect();
    customer_nos.sort_by(|&a, &b| {
        let angle_a = calculate_polar_angle(vrp_instance, a);
        let angle_b = calculate_polar_angle(vrp_instance, b);
        angle_a.total_cmp(&angle_b)
    });


    for cust_no in customer_nos {
        let demand = vrp_instance.demand_of_customer[cust_no];
        let mut found = false;
        for vehicle_idx in 0..vrp_instance.num_vehicles {
            if vrp_instance.vehicle_capacity - sol.routes[vehicle_idx].used_capacity() >= demand
            {
                let len = sol.routes[vehicle_idx].stops().len();
                sol.routes[vehicle_idx].add_stop_to_index(
                    Stop::new(cust_no.try_into().unwrap(), demand),
                    len,
                );

                found = true;
                break;
            }
        }
        if !found { panic!("sweep strategy failed to assigned customer"); }
    }
    sol
}

pub fn calculate_polar_angle(vrp_instance: &Arc<VRPInstance>, cust_no: usize) -> f64 {
    let depot_x = vrp_instance.x_coord_of_customer[0];
    let depot_y = vrp_instance.y_coord_of_customer[0];
    let cust_x = vrp_instance.x_coord_of_customer[cust_no];
    let cust_y = vrp_instance.y_coord_of_customer[cust_no];

    let delta_x = depot_x - cust_x;
    let delta_y = depot_y - cust_y;

    delta_y.atan2(delta_x)
}