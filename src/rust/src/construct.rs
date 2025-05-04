use std::sync::Arc;

use rand::seq::SliceRandom;

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

    // customers w/ largest demand first
    customer_nos.sort_by_key(|&i| Reverse(vrp_instance.demand_of_customer[i]));

    // randomly shuffled
    let seed = 42u64;
    let mut rng = StdRng::seed_from_u64(seed);
    customer_nos.shuffle(rng);

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