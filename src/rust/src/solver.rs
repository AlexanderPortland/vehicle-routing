

use crate::{common::{DistanceMatrix, Route}, VRPInstance};

struct VRPSolution<'a> {
    routes: Vec<Route<'a>>
}

impl<'a> VRPSolution<'a> {
    pub fn new(vrp_instance: &'a VRPInstance) -> Self {
        let mut routes = Vec::new();

        for _ in 0..vrp_instance.num_vehicles {
            routes.push(Route::new(vrp_instance.num_customers, &vrp_instance.distance_matrix));
        }
        
        VRPSolution { 
            routes
        }
    }

    pub fn total_cost(&self, distance_matrix: &DistanceMatrix) -> f64 {
        self.routes.iter().map(|route|{
            route.route_cost(distance_matrix)
        }).sum()
    }

    pub fn is_feasible_w_cap(&self, cap: usize)
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
