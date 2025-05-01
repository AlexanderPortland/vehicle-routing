
use rand::Rng;
use crate::VRPInstance;
use crate::common::{Route, Stop};

struct VRPSolution {
    routes: Vec<Route>,
}

impl VRPSolution {
    pub fn new(vrp_instance: &VRPInstance) -> Self {
        VRPSolution {
            routes: (0..vrp_instance.num_vehicles)
                .into_iter()
                .map(|_| Route {
                    stops: Vec::with_capacity(vrp_instance.num_customers),
                    known_cost: Some(0f64),
                    capacity_left: vrp_instance.vehicle_capacity,
                })
                .collect(),
        }
    }

    pub fn get_greedy_construction(&mut self, vrp_instance: &VRPInstance) {
        let mut vehicle_index = 0;
        for i in 0..vrp_instance.num_customers {
            let demand = vrp_instance.demand_of_customer[i];
            let customer_idx = i;
            if self.routes[vehicle_index].capacity_left < demand {
                vehicle_index += 1;
            }
            assert!(vehicle_index <= self.routes.len());
            self.routes[vehicle_index].add_stop_to_end(Stop {
                customer_idx,
                demand,
            });
        }
    }

    pub fn cost(&mut self, distance_matrix: &Vec<Vec<f64>>) -> f64 {
        self.routes
            .iter_mut()
            .map(|route| {
                if let Some(known) = route.known_cost {
                    known
                } else {
                    let route_cost = route.cost(distance_matrix);
                    route.known_cost = Some(route_cost);
                    route_cost
                }
            })
            .sum()
    }

    pub fn print(self) {
        for route in self.routes {
            for customer_idx in route.stops {
                print!("{:>2} ", customer_idx);
            }
            println!();
        }
    }
}

pub struct Solver {
    vrp_instance: VRPInstance,
    vrp_solution: VRPSolution,
}

impl Solver {
    pub fn new(vrp_instance: VRPInstance) -> Self {
        Solver {
            vrp_solution: VRPSolution::new(&vrp_instance),
            vrp_instance,
        }
    }

    pub fn construct(&mut self) {
        self.vrp_solution
            .get_greedy_construction(&self.vrp_instance);
    }

    pub fn destroy(&mut self) -> Stop {
        let mut rng = rand::rng();
        let rand_vehicle_idx = rng.random_range(0..=self.vrp_instance.num_vehicles);
        let rand_route_idx =
            rng.random_range(0..=self.vrp_solution.routes[rand_vehicle_idx].stops.len() - 1);

        let removed_stop =
            self.vrp_solution.routes[rand_vehicle_idx].remove_stop_at_index(rand_route_idx);

        return removed_stop;
    }

    pub fn repair(&mut self, stop: Stop) {
        let mut rng = rand::rng();
        let rand_vehicle_idx = rng.random_range(0..=self.vrp_instance.num_vehicles);
        let rand_route_idx =
            rng.random_range(0..=self.vrp_solution.routes[rand_vehicle_idx].stops.len());

        self.vrp_solution.routes[rand_vehicle_idx].add_stop_at_index(stop, rand_route_idx);
    }

    pub fn calculate_initial_temperature(&mut self) -> f64 {
        let cost = self.vrp_solution.cost();
        let mut avg_delta_of_bad_moves = 0f64;
        let mut number_of_worse_moves = 0;
        for i in 0..1000 {
            let stop = self.destroy();
            self.repair(stop);
            let new_cost = self.vrp_solution.cost();
            if new_cost > cost {
                let delta = new_cost - cost;
                avg_delta_of_bad_moves = ((avg_delta_of_bad_moves * number_of_worse_moves as f64) + delta) / ((number_of_worse_moves + 1) as f64);
                number_of_worse_moves += 1;
            }
        }
        let percentage_of_worse_moves: f64 = number_of_worse_moves as f64 / 1000f64;
        return avg_delta_of_bad_moves / ((0.97 - 1f64 + percentage_of_worse_moves) / percentage_of_worse_moves).ln();
    }

    pub fn solve(&mut self) {
        self.construct();
        let incumbent_cost = self.vrp_solution.cost();
        let mut temperature = self.calculate_initial_temperature();
        let alpha = 0.98;

        while temperature > 0.2 {
            let stop = self.destroy();
            self.repair(stop);
            let new_cost = self.vrp_solution.cost();
            let delta = new_cost - incumbent_cost;
            if delta < 0f64 || rand::<f64>() < (-delta / temperature).exp() {
                current_solution = candidate; // accept move
            }
            temperature *= alpha;
        }
    }
}
