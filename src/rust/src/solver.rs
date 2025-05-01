use crate::VRPInstance;
use crate::common::{Route, Stop};
use rand::Rng;
use rand::rngs::ThreadRng;

#[derive(Debug, Clone)]
struct VRPSolution<'a> {
    routes: Vec<Route<'a>>,
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
        for customer_idx in 1..vrp_instance.num_customers {
            let demand = vrp_instance.demand_of_customer[customer_idx];
            for vehicle_idx in 0..vrp_instance.num_vehicles {
                if (vrp_instance.vehicle_capacity - self.routes[vehicle_idx].used_capacity())
                    >= demand
                {
                    let len = self.routes[vehicle_idx].stops().len();
                    self.routes[vehicle_idx].add_stop_to_index(
                        Stop::new(customer_idx.try_into().unwrap(), demand),
                        len,
                    );
                    break;
                }
            }
        }
    }

    pub fn cost(&self) -> f64 {
        self.routes.iter().map(|route| route.cost()).sum()
    }

    pub fn print(self) {
        for route in self.routes {
            for stop in route.stops() {
                print!("{:>2} ", stop.cust_no());
            }
            println!();
        }
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

        let rand_vehicle_idx = self.random_empty_vehicle_idx(&mut rng);

        let rand_route_idx =
            rng.random_range(0..=self.vrp_solution.routes[rand_vehicle_idx].stops().len() - 1);

        println!(
            "trying to remove customer at index {:?} from vehicle {:?} for {:?}",
            rand_route_idx, rand_vehicle_idx, self.vrp_solution
        );
        let removed_stop = self.vrp_solution.routes[rand_vehicle_idx].remove_stop(rand_route_idx);

        return removed_stop;
    }

    pub fn random_empty_vehicle_idx(&self, rng: &mut ThreadRng) -> usize {
        // let mut rng = rand::rng();
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
        let rand_vehicle_idx = self.random_empty_vehicle_idx(&mut rng);
        let rand_route_idx =
            rng.random_range(0..=self.vrp_solution.routes[rand_vehicle_idx].stops().len());

        let route = &self.vrp_solution.routes[rand_vehicle_idx];
        if !route.speculative_add_stop(&stop, rand_route_idx).1 {
            return false;
        }

        self.vrp_solution.routes[rand_vehicle_idx].add_stop_to_index(stop, rand_route_idx);
        return true;
    }

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

    pub fn solve(&mut self) {
        self.construct();
        let mut incumbent_cost = self.vrp_solution.cost();

        println!("solver is {:?}", self.vrp_solution);
        // let mut temperature = self.calculate_initial_temperature();
        let alpha = 0.98;

        let mut best = self.vrp_solution.clone();
        // let mut current_solution = self.vrp_solution;
        for _ in 0..1000000 {
            let old_solution = self.vrp_solution.clone();
            let stop = self.random_destroy();
            if !self.random_repair(stop) {
                // println!("")x
                self.vrp_solution = old_solution;
                continue;
            }

            let new_cost = self.vrp_solution.cost();

            if new_cost < best.cost() {
                println!("new best w/ cost {:?}", new_cost);
                best = self.vrp_solution.clone();
                // incumbent_cost = best.cost();
            } else {
                println!("NOT NEW best w/ shitty cost {:?}", new_cost);
            }

            let delta = new_cost - incumbent_cost;
            if delta < 0f64 || rand::random::<f64>() < 0.02_f64 {
                // current_solution = candidate; // accept move
                // accept move
                println!("accepting move to {:?}", self.vrp_solution);
                incumbent_cost = self.vrp_solution.cost();
            } else {
                println!("reverting back to {:?}", self.vrp_solution);
                self.vrp_solution = old_solution;
            }
            // temperature *= alpha;
        }

        println!("solver is {:?}", best);
        self.assert_sanity_solution(best);
    }

    fn assert_sanity_solution(&mut self, sol: VRPSolution) {
        let mut total_cost = 0f64;
        for route in sol.routes {
            route.assert_sanity();
            total_cost += route.cost();
            if route.used_capacity() > self.vrp_instance.vehicle_capacity {
                panic!("Route ({}) failed", route.to_string());
            }
        }

    }
}
