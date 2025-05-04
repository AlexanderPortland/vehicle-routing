use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::VRPInstance;
use crate::common::{Route, Stop};
use rand::{rng, Rng};
use rand::rngs::ThreadRng;

#[derive(Debug, Clone)]
pub struct VRPSolution<'a> {
    pub routes: Vec<Route<'a>>,
}

impl<'a> VRPSolution<'a> {
    pub fn new(vrp_instance: &'a VRPInstance) -> Self {
        VRPSolution {
            routes: (0..vrp_instance.num_vehicles)
                .into_iter()
                .map(|i| Route::new(&vrp_instance, i))
                .collect(),
        }
    }

    pub fn is_valid_solution(&self, vrp_instance: &'a VRPInstance) -> bool {
        todo!()
    }

    // pub fn get_greedy_construction(&mut self, vrp_instance: &VRPInstance) {
    //     for customer_idx in 1..vrp_instance.num_customers {
    //         let demand = vrp_instance.demand_of_customer[customer_idx];
    //         println!("considering customer {:?}", customer_idx);
    //         let mut found = false;
    //         for vehicle_idx in 0..vrp_instance.num_vehicles {
    //             if (vrp_instance.vehicle_capacity - self.routes[vehicle_idx].used_capacity())
    //                 >= demand
    //             {
    //                 println!("adding customer {:?}", customer_idx);
    //                 let len = self.routes[vehicle_idx].stops().len();
    //                 self.routes[vehicle_idx].add_stop_to_index(
    //                     Stop::new(customer_idx.try_into().unwrap(), demand),
    //                     len,
    //                 );
    //                 found = true;
    //                 break;
    //             }
    //         }
    //         assert!(found);
    //     }
    // }

    pub fn cost(&self) -> f64 {
        self.routes.iter().map(|route| route.cost()).sum()
    }

    pub fn to_string(self) -> String {
        let route_strings: Vec<String> = self.routes.iter().map(|route| {
            let mut result = String::from("0");
            
            for stop in route.stops() {
                result.push_str(&format!(" {}", stop.cust_no()));
            }
            
            result.push_str(" 0");
            result
        }).collect();
        
        let mut combined = String::from("0 ");
        combined.push_str(&route_strings.join(" "));
        combined
    }
}

#[derive(Clone)]
pub struct Solver<'a> {
    vrp_instance: &'a VRPInstance,
    vrp_solution: VRPSolution<'a>,
}

impl<'a> Solver<'a> {
    pub fn new(vrp_instance: &'a VRPInstance) -> Self {
        Solver {
            vrp_solution: VRPSolution::new(&vrp_instance),
            vrp_instance,
        }
    }

    pub fn construct(&mut self) {
        self.vrp_solution
            .get_greedy_construction(&self.vrp_instance);
    }

    pub fn random_destroy(&mut self) -> Stop {
        let mut rng = rand::rng();
        // println!("num vehicles {:?} range {:?}", self.vrp_instance.num_vehicles, 0..=self.vrp_instance.num_vehicles);

        // let start = rand_vehicle_idx;

        let rand_vehicle_idx = self.random_nonempty_vehicle_idx(&mut rng);

        let rand_route_idx =
            rng.random_range(0..=self.vrp_solution.routes[rand_vehicle_idx].stops().len() - 1);

        // println!(
        //     "trying to remove customer at index {:?} from vehicle {:?} for {:?}",
        //     rand_route_idx, rand_vehicle_idx, self.vrp_solution
        // );

        let removed_stop = self.vrp_solution.routes[rand_vehicle_idx].remove_stop(rand_route_idx);

        return removed_stop;
    }

    pub fn random_nonempty_vehicle_idx(&self, rng: &mut ThreadRng) -> usize {
        let start = rng.random_range(0..=self.vrp_instance.num_vehicles);
        let rand_vehicle_idx = (start..self.vrp_instance.num_vehicles)
            .chain(0..start)
            .filter_map(|i| {
                if self.vrp_solution.routes[i].stops().len() > 0 {
                    Some(i)
                } else {
                    None
                }
            })
            .next()
            .unwrap();
        // println!("got random idx {:?}", rand_vehicle_idx);
        return rand_vehicle_idx;
    }

    pub fn random_repair(&mut self, stop: Stop) -> bool {
        let mut rng = rand::rng();
        let rand_vehicle_idx = self.random_nonempty_vehicle_idx(&mut rng);
        let rand_route_idx =
            rng.random_range(0..=self.vrp_solution.routes[rand_vehicle_idx].stops().len());

        let route = &self.vrp_solution.routes[rand_vehicle_idx];
        if !route.speculative_add_stop(&stop, rand_route_idx).1 {
            return false;
        }

        self.vrp_solution.routes[rand_vehicle_idx].add_stop_to_index(stop, rand_route_idx);
        return true;
    }

    pub fn solve(mut self) -> VRPSolution<'a> {
        println!("\n\n------- INIT ------");
        self.construct();
        let mut incumbent_cost = self.vrp_solution.cost();

        println!("solver is {:?}", self.vrp_solution);
        // let mut temperature = self.calculate_initial_temperature();

        let mut best = self.vrp_solution.clone();
        // let mut current_solution = self.vrp_solution;
        let mut tabu = Vec::new();
        let mut small_float_diff = 0;
        let start = Instant::now();
        for i in 0..9000 {
            // println!("\n\n------ ITER {} ------", i);
            // look at best thing to remove, and best place to put it
            let (rem, rem_r) = self.remove_worst_stop(&tabu);
            tabu.push(rem.clone());
            if tabu.len() > 5 { tabu.pop(); }
            self.reinsert_in_best_spot(rem);
            // self.reinsert_replace_stop(rem, rem_r);

            if self.vrp_solution.cost() < best.cost() {
                if (self.vrp_solution.cost() - best.cost()).abs() < 0.01 {
                    small_float_diff += 1;
                    println!("FOUND NEW (small) BEST on iter {i} IM THE GOAT {:?}", self.vrp_solution.cost());
                    if small_float_diff >= 15 {
                        println!("just small fry...");
                        break;
                    }
                } else {
                    small_float_diff = 0;
                    println!("FOUND NEW BEST on iter {i} IM THE GOAT {:?}", self.vrp_solution.cost());
                }
                // println!("FOUND NEW BEST on iter {i} IM THE GOAT {:?}", self.vrp_solution.cost());
                best = self.vrp_solution.clone();
            } else {
                // println!("didn't find a new best im not really that good ... :( {:?}", self.vrp_solution.cost());
            }
            // println!("finish iter {i}");
        }

        self.assert_sanity_solution(&best);
        println!("solver is {:?} w/ cost {:?} \nin {:?}", best, best.cost(), start.elapsed());
        return best;
    }

    fn assert_sanity_solution(&mut self, sol: &VRPSolution) {
        let mut total_cost = 0f64;

        for route in &sol.routes {
            route.assert_sanity();
            total_cost += route.cost();
            if route.used_capacity() > self.vrp_instance.vehicle_capacity {
                panic!("Route ({}) failed", route.to_string());
            }
        }
    }

    fn remove_worst_stop(&mut self, tabu: &Vec<Stop>) -> (Stop, usize) {
        // println!("removing worst stop from {:?} w/ tabu {:?}", self.vrp_solution, tabu);

        let (mut worst_spot_r, mut worst_spot_i, mut worst_spot_cost) = (100000, 100000, f64::MIN);

        let mut feas_vals = Vec::new();
        for (r, route) in self.vrp_solution.routes.iter().enumerate() {
            for i in 0..(route.stops().len()) {
                if tabu.contains(&route.stops()[i]) { continue; }
                let (new_cost, feas) = route.speculative_remove_stop(i);
                // we want the new cost to be much less than previous, so maximize cost
                let cost = route.cost() - new_cost;
                // println!("removing i{:?} from {:?} has cost {:?} & feas {:?} (cur existing {:?})", i, route, cost, feas, worst_spot_cost);
                if feas {
                    feas_vals.push((r, i));
                    if cost > worst_spot_cost {
                        // println!"
                        (worst_spot_r, worst_spot_i) = (r, i);
                        worst_spot_cost = cost;
                    }
                }
            }
        }

        // if rng().random_bool(0.05_f64) {
        if true {
            // go for a fucking walk
            // println!("go for a fucking walk...");

            (worst_spot_r, worst_spot_i) = *feas_vals.get(rng().random_range(0..feas_vals.len())).unwrap();
        }

        // println!("best was to remove {:?} from {:?} @ {:?}", self.vrp_solution.routes[worst_spot_r].stops()[worst_spot_i], self.vrp_solution.routes[worst_spot_r], worst_spot_i);

        let res = self.vrp_solution.routes[worst_spot_r].remove_stop(worst_spot_i);
        return (res, worst_spot_r);
    }

    fn reinsert_replace_stop(&mut self, stop: Stop, old_r_i: usize) {
        let (mut best_spot_r, mut best_spot_i, mut best_spot_cost) = (100000, 100000, f64::MAX);
        let old_r_cap = self.vrp_solution.routes[old_r_i].used_capacity();
        let old_r = &self.vrp_solution.routes[old_r_i];

        for (r, route) in self.vrp_solution.routes.iter().enumerate() {
            let route_cap = route.used_capacity();
            if (r == old_r_i) { continue; }

            for i in 0..(route.stops().len()) {
                let stop_ref = &route.stops()[i];
                let old_new_cap = (old_r_cap + stop_ref.capacity());
                let new_new_cap = (route_cap + stop.capacity() - stop_ref.capacity());
                println!("old cap {:?} new cap {:?}, new_cap no removed {:?}", old_r_cap, route_cap, route_cap - stop_ref.capacity());
                // let can_new_go_to_old = (old_r_cap - stop_ref.capacity()) >= 0;
                // let can_old_go_to_new = (route_cap + stop_ref.capacity() - stop.capacity()) >= 0;
                println!("trying to swap {:?} into {:?} and {:?} into {:?} to get {:?} and {:?} (of {:?})", stop_ref, old_r, stop, route, old_new_cap, new_new_cap, self.vrp_instance.vehicle_capacity);
                println!("that is ({:?}, {:?})", old_new_cap, new_new_cap);

                if !(old_new_cap < self.vrp_instance.vehicle_capacity 
                    && new_new_cap < self.vrp_instance.vehicle_capacity) {
                    continue;
                }

                println!("HORAYYYY");

                // speculatively replace
                let ((new_cost_1, feas), i) = route.speculative_add_best(&stop);
                let ((new_cost_2, feas), i) = old_r.speculative_add_best(&stop_ref);

                println!("people used to be ({:?}, {:?})", old_r.cost(), route.cost());
                println!("people now are to be ({:?}, {:?})", new_cost_2, new_cost_1);
                let cost_diff = (route.cost() - new_cost_1) + (old_r.cost() - new_cost_2);
                println!("net diff is {:?}", cost_diff);

                // try to add back to the old one

                // we want the new cost to be much less than the old cost, so we maximize this difference
                let cost = (route.cost()) - new_cost_1;

                
                // println!("absolute cost {:?}, relative {:?} (existing {:?})", new_cost, cost);
                println!("res for replacing {:?} w/ {:?} in {:?} (@{:?}) is {:?}", route.stops()[i], stop, route, i, (cost, feas));
                // if feas { valid.push((r, i)); }
                if feas && cost < best_spot_cost {
                    (best_spot_r, best_spot_i) = (r, i);
                    best_spot_cost = cost;
                }
            }
        }

        println!("best was to replace {:?} w/ {:?} in {:?} @ {:?}", self.vrp_solution.routes[best_spot_r].stops()[best_spot_i], stop, self.vrp_solution.routes[best_spot_r], best_spot_i);
        todo!();
        self.vrp_solution.routes[best_spot_r].add_stop_to_index(stop, best_spot_i);
    }

    fn reinsert_in_best_spot(&mut self, stop: Stop) {
        let (mut best_spot_r, mut best_spot_i, mut best_spot_cost_increase) = (100000, 100000, f64::MAX);

        let mut valid = Vec::new();

        for (r, route) in self.vrp_solution.routes.iter().enumerate() {
            for i in 0..(route.stops().len() + 1) {
                let (new_cost, feas) = route.speculative_add_stop(&stop, i);

                // we want the one that will increase the new cost by the least, so minimize
                let cost_increase = new_cost - route.cost();
                // println!("res for adding {:?} to {:?} (@{:?}) is {:?}", stop, route, i, (cost_increase, feas));
                // println!("existing is {:?}", best_spot_cost_increase);
                if feas { valid.push((r, i)); }
                if feas && cost_increase < best_spot_cost_increase {
                    (best_spot_r, best_spot_i) = (r, i);
                    best_spot_cost_increase = cost_increase;
                }
            }
        }

        if rng().random_bool(0.05_f64) {
            let i = rng().random_range(0..valid.len());
            // println!("RANDOM DROP at i {i}...");
            (best_spot_r, best_spot_i) = *valid.get(i).unwrap();
        }

        // println!("best was to add {:?} to {:?} @ {:?}", stop, self.vrp_solution.routes[best_spot_r], best_spot_i);
        self.vrp_solution.routes[best_spot_r].add_stop_to_index(stop, best_spot_i);
    }
}
