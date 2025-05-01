use crate::VRPInstance;
use crate::common::{Route, Stop};
use rand::rngs::ThreadRng;
use rand::Rng;

#[derive(Debug, Clone)]
struct VRPSolution<'a> {
    routes: Vec<Route<'a>>,
    // routes_before_destroy: Vec<Route<'a>>,
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
        let mut vehicle_index = 0;
        for i in 1..vrp_instance.num_customers {
            let demand = vrp_instance.demand_of_customer[i];
            let customer_idx = i;
            if (vrp_instance.vehicle_capacity - self.routes[vehicle_index].route_used_cap()) < demand {
                vehicle_index += 1;
            }
            assert!(vehicle_index < self.routes.len());
            let len = self.routes[vehicle_index].stops().len();
            self.routes[vehicle_index].add_stop_to_index(Stop::new(customer_idx.try_into().unwrap(), demand), len);
        }
    }

    pub fn cost(&self) -> f64 {
        self.routes.iter().map(|route| route.route_cost()).sum()
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

        println!("trying to remove customer at index {:?} from vehicle {:?} for {:?}", rand_route_idx, rand_vehicle_idx, self.vrp_solution);
        let removed_stop =
            self.vrp_solution.routes[rand_vehicle_idx].remove_stop(rand_route_idx);

        return removed_stop;
    }

    pub fn random_empty_vehicle_idx(&self, rng: &mut ThreadRng) -> usize {
        // let mut rng = rand::rng();
        let start = rng.random_range(0..=self.vrp_instance.num_vehicles);

        let rand_vehicle_idx = (start..self.vrp_instance.num_vehicles).chain(0..start).filter_map(|i|
            if self.vrp_solution.routes[i].stops().len() > 0 {
                Some(i)
            } else { None }
        ).next().unwrap();
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

        let mut best = self.vrp_solution.clone();
        // let mut current_solution = self.vrp_solution;
        for _ in 0..1000 {
            // look at best thing to remove, and best place to put it
            let rem = self.remove_worst_stop();
            self.reinsert_in_best_spot(rem);

            if self.vrp_solution.cost() < best.cost() {
                println!("FOUND NEW BEST IM THE GOAT {:?}", self.vrp_solution.cost());
                best = self.vrp_solution.clone();
            } else {
                println!("didn't find a new best im not really that good ... :( {:?}", self.vrp_solution.cost());
            }
        }

        println!("solver is {:?}", best);
    }

    fn remove_worst_stop(&mut self) -> Stop {
        println!("removing worst stop from {:?}", self.vrp_solution);

        let (mut worst_spot_r, mut worst_spot_i, worst_spot_cost) = (100000, 100000, f64::MAX);

        for (r, route) in self.vrp_solution.routes.iter().enumerate() {
            for i in 0..(route.stops().len() + 1) {
                let (cost, feas) = route.speculative_remove_stop(i);
                if feas && cost < worst_spot_cost {
                    (worst_spot_r, worst_spot_i) = (r, i);
                }
            }
        }

        let res = self.vrp_solution.routes[worst_spot_r].remove_stop(worst_spot_i);

        println!("best was to remove from {:?} @ {:?}", self.vrp_solution.routes[worst_spot_r], worst_spot_i);
        return res;
    }

    fn reinsert_in_best_spot(&mut self, stop: Stop) {
        let (mut best_spot_r, mut best_spot_i, best_spot_cost) = (100000, 100000, f64::MAX);

        for (r, route) in self.vrp_solution.routes.iter().enumerate() {
            for i in 0..(route.stops().len() + 1) {
                let (cost, feas) = route.speculative_add_stop(&stop, i);
                if feas && cost < best_spot_cost {
                    (best_spot_r, best_spot_i) = (r, i);
                }
            }
        }

        println!("best was to add {:?} to {:?} @ {:?}", stop, self.vrp_solution.routes[best_spot_r], best_spot_i);
        self.vrp_solution.routes[best_spot_r].add_stop_to_index(stop, best_spot_i);
    }
}
