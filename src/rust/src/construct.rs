use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::{rng, Rng, SeedableRng};

use crate::common::Route;
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
    let shuffle_seed = rng().random_range(0..customer_nos.len());
    customer_nos.rotate_left(shuffle_seed);


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

    crate::explode::single_swap::naive_greedy(sol, vrp_instance).0
    // sol
}

fn calculate_polar_angle(vrp_instance: &Arc<VRPInstance>, cust_no: usize) -> f64 {
    let depot_x = vrp_instance.x_coord_of_customer[0];
    let depot_y = vrp_instance.y_coord_of_customer[0];
    let cust_x = vrp_instance.x_coord_of_customer[cust_no];
    let cust_y = vrp_instance.y_coord_of_customer[cust_no];

    let delta_x = depot_x - cust_x;
    let delta_y = depot_y - cust_y;

    delta_y.atan2(delta_x)
}

pub fn clarke_wright(vrp: &Arc<VRPInstance>) -> VRPSolution {
    let n = vrp.num_customers;

    let mut routes: Vec<Route> = (1..n)
        .map(|cust_no| Route::new(vrp.clone(), cust_no))
        .collect();

    let mut savings: Vec<(usize, usize, f64)> = Vec::with_capacity(((n - 1) * (n - 2) / 2) as usize);
    for i in 1..n {
        for j in i+1..n {
            let s = vrp.distance_matrix.dist(i, 0)
                  + vrp.distance_matrix.dist(0, j)
                  - vrp.distance_matrix.dist(i, j);
            savings.push((i, j, s));
        }
    }
    savings.sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    for (i, j, s) in savings {
        if routes.len() <= vrp.num_vehicles {
            break;
        }

        let ri = match routes.iter().position(|r| r.contains_stop(i.try_into().unwrap())) {
            Some(x) => x,
            None => continue,
        };
        let rj = match routes.iter().position(|r| r.contains_stop(j.try_into().unwrap())) {
            Some(x) => x,
            None => continue,
        };
        if ri == rj {
            continue;
        }

        // check for the "tail-to-head" merge: route_i.last == i, route_j.first == j
        let (last_i, first_j) = (routes[ri].last(), routes[rj].first());
        if last_i == i && first_j == j {
            println!("merging");
            let cap_i = routes[ri].used_capacity();
            let cap_j = routes[rj].used_capacity();
            if cap_i + cap_j <= vrp.vehicle_capacity {
                // take route_j out, append its stops onto route_i
                let mut tail = routes.remove(rj);
                let mut head = routes.remove(ri);
                // merge head <- tail
                for stop in tail.stops().iter().cloned() {
                    head.add_stop_to_index(stop, head.stops().len());
                }
                routes.insert(ri, head);
            }
        }
        else {
            
            let (last_j, first_i) = (routes[rj].last(), routes[ri].first());
            if last_j == j && first_i == i {
                println!("merging");
                let cap_i = routes[ri].used_capacity();
                let cap_j = routes[rj].used_capacity();
                if cap_i + cap_j <= vrp.vehicle_capacity {
                    let mut tail = routes.remove(ri);
                    let mut head = routes.remove(rj);
                    // merge head <- tail
                    for stop in tail.stops().iter().cloned() {
                        head.add_stop_to_index(stop, head.stops().len());
                    }
                    routes.insert(rj, head);
                }
            }
        }
    }
    for r in routes.iter() {
        println!("{:?}", r);
    }

    let mut sol = VRPSolution::new(vrp.clone());
    for (i, route) in routes.iter().enumerate() {
        sol.routes[i] = route.clone();
    }
    sol
}
