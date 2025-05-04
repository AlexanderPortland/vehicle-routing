use std::time::{Duration, Instant};
use std::cmp::Reverse;

use crate::VRPInstance;
use crate::common::{Route, Stop};
use rand::{rng, Rng};
use rand::rngs::ThreadRng;

#[derive(Clone)]
pub struct VRPSolution<'a> {
    routes: Vec<Route<'a>>,
}

impl<'a> std::fmt::Debug for VRPSolution<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in self.routes.iter() {
            f.write_fmt(format_args!("{}\n", r.to_string())).unwrap();
        }
        Ok(())
    }
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

    pub fn get_greedy_construction(&mut self, vrp_instance: &VRPInstance) {
        let mut customer_nos: Vec<usize> = (1..vrp_instance.num_customers).collect();
        customer_nos.sort_by_key(|&i| Reverse(vrp_instance.demand_of_customer[i]));

        for cust_no in customer_nos {
            let demand = vrp_instance.demand_of_customer[cust_no];
            let mut assigned = false;
            for vehicle_idx in 0..vrp_instance.num_vehicles {
                if (vrp_instance.vehicle_capacity - self.routes[vehicle_idx].used_capacity())
                    >= demand
                {
                    let len = self.routes[vehicle_idx].stops().len();
                    self.routes[vehicle_idx].add_stop_to_index(
                        Stop::new(cust_no.try_into().unwrap(), demand),
                        len,
                    );
                    assigned = true;
                    break;
                }
            }
            if !assigned {
                println!("{:?}", self);
                panic!("Could not assign customer: {} with demand {}", cust_no, demand);
            }
        }
    }

    pub fn cost(&self) -> f64 {
        self.routes.iter().map(|route| route.cost()).sum()
    }

    pub fn to_stdout_string(&self) -> String {
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

    pub fn to_file_string(&self) -> String {
        let mut res = String::from(format!("{} 0\n", self.cost()));
        let route_strings: Vec<String> = self.routes.iter().map(|route| {
            let mut result = String::from("0");
            
            for stop in route.stops() {
                result.push_str(&format!(" {}", stop.cust_no()));
            }
            
            result.push_str(" 0\n");
            result
        }).collect();
        res.push_str(&route_strings.join(""));
        res
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

    // * Simulated Annealing
    // pub fn calculate_initial_temperature(&mut self) -> f64 {
    //     let cost = self.vrp_solution.cost();
    //     let mut avg_delta_of_bad_moves = 0f64;
    //     let mut number_of_worse_moves = 0;
    //     for i in 0..1000 {
    //         let old = self.vrp_solution.clone();
    //         let stop = self.random_destroy();
    //         if !self.random_repair(stop) {
    //             self.vrp_solution = old;
    //             continue;
    //         }
    //         let new_cost = self.vrp_solution.cost();
    //         if new_cost > cost {
    //             let delta = new_cost - cost;
    //             avg_delta_of_bad_moves = ((avg_delta_of_bad_moves * number_of_worse_moves as f64)
    //                 + delta)
    //                 / ((number_of_worse_moves + 1) as f64);
    //             number_of_worse_moves += 1;
    //         }
    //     }
    //     let percentage_of_worse_moves: f64 = number_of_worse_moves as f64 / 1000f64;
    //     return avg_delta_of_bad_moves
    //         / ((0.97 - 1f64 + percentage_of_worse_moves) / percentage_of_worse_moves).ln();
    // }


    pub fn solve(&mut self) -> VRPSolution<'a> {
        self.construct();
        
        let alpha = 0.98;
        let mut incumbent_cost = self.vrp_solution.cost();
        let mut best = self.vrp_solution.clone();
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(15) {
            let old_solution = self.vrp_solution.clone();
            let stop = self.random_destroy();
            if !self.random_repair(stop) {
                self.vrp_solution = old_solution;
                continue;
            }

            let new_cost = self.vrp_solution.cost();
            if new_cost < best.cost() {
                println!("NEW BEST COST: {:?}", new_cost);
                best = self.vrp_solution.clone();
            }

            let delta = new_cost - incumbent_cost;
            if delta < 0f64 || rand::random::<f64>() < 0.02_f64 { // accept move
                // println!("accepting move to {:?}", self.vrp_solution);
                incumbent_cost = self.vrp_solution.cost();
            } else { // reject move
                
                // println!("reverting back to {:?}", self.vrp_solution);
                self.vrp_solution = old_solution;´
        self.construct();
        self.assert_sanity_solution(&self.vrp_solution);
        let mut incumbent_cost = self.vrp_solution.cost();
        let mut best = self.vrp_solution.clone();
        let mut tabu = Vec::new();

        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(15) {
            // look at best thing to remove, and best place to put it
            let (rem, rem_r) = self.remove_worst_stop(&tabu);
            tabu.push(rem.clone());
            if tabu.len() > 5 { tabu.pop(); }
            self.reinsert_in_best_spot(rem);
            // self.reinsert_replace_stop(rem, rem_r);

            // no concept of accepting worse moves here either...
            // is that because strictly better moves are always made?
            // this strategy also might benefit from randomized restarts

            if self.vrp_solution.cost() < best.cost() {
                // println!("FOUND NEW BEST IM THE GOAT {:?}", self.vrp_solution.cost());
                best = self.vrp_solution.clone();
            }
            self.assert_sanity_solution(&self.vrp_solution);
        }

        self.assert_sanity_solution(&best);
        println!("solver is {:?} w/ cost {:?}", best, best.cost());
        return best;
    }

    fn assert_sanity_solution(&self, sol: &VRPSolution) {
        let mut total_cost = 0f64;
        for route in sol.routes.iter() {
            route.assert_sanity();
            total_cost += route.cost();
            if route.used_capacity() > self.vrp_instance.vehicle_capacity {
                panic!("Route ({}) failed", route.to_string());
            }
        }
        
        for i in 1..self.vrp_instance.num_customers {
            let mut found = false;
            for route in sol.routes.iter() {
                if route.contains_stop(i.try_into().unwrap()) {
                    found = true;
                    break;
                }
            }
            if !found {
                panic!("Customer {} not visited", i);
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
            // TODO: sometimes this will fail b/c no feasible values (try on 5_4_10.vrp)
            // println!("go for a walk...");
            (worst_spot_r, worst_spot_i) = *feas_vals.get(rng().random_range(0..feas_vals.len())).unwrap();
        }

        // println!("best was to remove {:?} from {:?} @ {:?}", self.vrp_solution.routes[worst_spot_r].stops()[worst_spot_i], self.vrp_solution.routes[worst_spot_r], worst_spot_i);

        let res = self.vrp_solution.routes[worst_spot_r].remove_stop(worst_spot_i);
        return (res, worst_spot_r);
    }

    fn reinsert_replace_stop(&mut self, stop: Stop, old_r: usize) {
        let (mut best_spot_r, mut best_spot_i, mut best_spot_cost) = (100000, 100000, f64::MAX);

        for (r, route) in self.vrp_solution.routes.iter().enumerate() {
            for i in 0..(route.stops().len()) {
                let (new_cost, feas) = route.speculative_replace_stop(&stop, i);
                let cost = route.cost() - new_cost;
                // println!("absolute cost {:?}, relative {:?} (existing {:?})", new_cost, cost);
                // println!("res for replacing {:?} w/ {:?} in {:?} (@{:?}) is {:?}", route.stops()[i], stop, route, i, (cost, feas));
                // if feas { valid.push((r, i)); }
                if feas && cost < best_spot_cost {
                    (best_spot_r, best_spot_i) = (r, i);
                    best_spot_cost = cost;
                }
            }
        }

        // println!("best was to replace {:?} w/ {:?} in {:?} @ {:?}", self.vrp_solution.routes[best_spot_r].stops()[best_spot_i], stop, self.vrp_solution.routes[best_spot_r], best_spot_i);
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
