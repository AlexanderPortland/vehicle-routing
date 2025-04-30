use std::collections::HashSet;

use crate::VRPInstance;

pub struct Stop {
    pub index: usize,
    pub capacity: usize
}

pub struct Route {
    pub stops: Vec<u16>,
    pub known_cost: Option<f64>,
    pub capacity: usize
}

impl Route {
    pub fn calc_route_cost(&self, distance_matrix: &Vec<Vec<f64>>) -> f64 {
        let mut cost = 0f64;

        for i in 1..self.stops.len() {
            cost += distance_matrix[self.stops[i - 1] as usize][self.stops[i] as usize];
        }

        if self.stops.len() > 0 {
            cost += distance_matrix[0][self.stops[0] as usize];
            cost += distance_matrix[self.stops[self.stops.len() - 1] as usize][0];
        }

        cost
    }

    pub fn contains_stop(&self, stop: u16) -> bool {
        self.stops.iter().any(|a|{ *a == stop })
    }

    pub fn assert_sanity(&self) {
        let mut existing = HashSet::new();

        for el in &self.stops {
            assert!(!existing.contains(el));
            existing.insert(el);
        }

        assert!(existing.len() == self.stops.len());
    }

    pub fn add_stop_to_end(&mut self, stop: Stop) {
        todo!()
    }

    pub fn add_stop_to_index(&mut self, stop: Stop, index: usize) {
        todo!()
    }
}

struct VRPSolution {
    routes: Vec<Route>
}

impl VRPSolution {
    pub fn new(vrp_instance: VRPInstance) -> Self {
        VRPSolution { 
            routes: (0..vrp_instance.num_vehicles).into_iter().map(
                |i| Route { stops: Vec::with_capacity(vrp_instance.num_customers), known_cost: Some(0f64) }
            ).collect()
        }
    }

    pub fn calculate_cost(&mut self, distance_matrix: &Vec<Vec<f64>>) -> f64 {
        self.routes.iter_mut().map(|route|{
            if let Some(known) = route.known_cost {
                known
            } else {
                let route_cost = route.calc_route_cost(distance_matrix);
                route.known_cost = Some(route_cost);
                route_cost
            }
        }).sum()
    }
}

struct Solver {
    vrp_instance: VRPInstance
}


impl Solver {
    pub fn new(vrp_instance: VRPInstance) -> Self {
        Solver {
            vrp_instance
        }
    }

    pub fn construct() {
        
    }

    pub fn perturb() {

    }

    pub fn solve() {

    }
}
