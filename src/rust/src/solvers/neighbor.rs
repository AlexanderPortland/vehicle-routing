use std::sync::Arc;

use rand::{rng, Rng};

use crate::{common::Stop, construct, old_solver::VRPSolution, solver::{IterativeSolver, LNSSolver}, vrp_instance::VRPInstance};


/// An LNS solver which greedily **removes the highest cost stop** from the solution,
/// **inserting it at the lowest cost location**.
pub struct MoveLNSSolver {
    tabu: Vec<Stop>,
    current: VRPSolution,
}

const MAX_TABU: usize = 5;

impl LNSSolver for MoveLNSSolver {

    /// corresponding to the (stop, route #) that was removed
    type DestroyResult = (Stop, usize);

    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self {
        MoveLNSSolver { tabu: Vec::new(), current: initial_solution }
    }

    fn destroy(&mut self) -> Self::DestroyResult {
        Self::remove_worst_stop(&mut self.current, &self.tabu)
    }

    fn repair(&mut self, res: Self::DestroyResult) -> VRPSolution {
        
    }

    fn update_search_location(&mut self, new_best: Option<(&VRPSolution, f64)>) {
        todo!()
    }

    fn update_tabu(&mut self, res: &Self::DestroyResult) {
        self.tabu.push(res.0);
        if self.tabu.len() > MAX_TABU { self.tabu.pop(); }
    }
}

impl MoveLNSSolver {
    fn remove_worst_stop(sol: &mut VRPSolution, tabu: &Vec<Stop>) -> (Stop, usize) {
        let (mut worst_spot_r, mut worst_spot_i, mut worst_spot_cost) = (100000, 100000, f64::MIN);

    let mut feas_vals = Vec::new();
    for (r, route) in sol.routes.iter().enumerate() {
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
        (worst_spot_r, worst_spot_i) = *feas_vals.get(rng().random_range(0..feas_vals.len())).unwrap();
    }

    // println!("best was to remove {:?} from {:?} @ {:?}", self.vrp_solution.routes[worst_spot_r].stops()[worst_spot_i], self.vrp_solution.routes[worst_spot_r], worst_spot_i);

    let res = sol.routes[worst_spot_r].remove_stop(worst_spot_i);
    return (res, worst_spot_r);
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