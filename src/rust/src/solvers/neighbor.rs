use std::{collections::{HashMap, VecDeque}, sync::Arc};

use rand::{rng, Rng};

use crate::{common::Stop, construct, common::VRPSolution, solver::{IterativeSolver, LNSSolver}, vrp_instance::VRPInstance};


/// An LNS solver which greedily **removes the highest cost stop** from the solution,
/// **inserting it at the lowest cost location**.
pub struct SimpleLNSSolver {
    instance: Arc<VRPInstance>,
    tabu: VecDeque<Stop>,
    current: VRPSolution,
    moves: HashMap<Stop, usize>,
}

const MAX_TABU: usize = 5;

impl LNSSolver for SimpleLNSSolver {
    /// corresponding to the (stop, route #) that was removed
    type DestroyResult = (Stop, usize);

    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self {
        SimpleLNSSolver { tabu: VecDeque::new(), current: initial_solution, moves: HashMap::new(), instance }
    }

    fn destroy(&mut self) -> Self::DestroyResult {
        self.remove_random_stop()
    }

    fn repair(&mut self, res: Self::DestroyResult) -> VRPSolution {
        Self::reinsert_in_best_spot(&mut self.current, res.0);
        self.current.clone()
    }

    fn update_tabu(&mut self, res: &Self::DestroyResult) {
        self.tabu.push_back(res.0);
        if self.tabu.len() > MAX_TABU { self.tabu.pop_front(); }
        *self.moves.entry(res.0).or_insert(0) += 1;
        let mut a = self.moves.iter().collect::<Vec<(&Stop, &usize)>>();
        a.sort_by(|a, b| a.1.cmp(b.1));
        println!("moves are {:?}", a);
    }
}

impl SimpleLNSSolver {
    fn remove_random_stop(&mut self) -> (Stop, usize) {
        let tabu = &self.tabu;
        // let (mut worst_spot_r, mut worst_spot_i, mut worst_spot_cost) = (100000, 100000, f64::MIN);
        let sol = &mut self.current;

        let mut feas_vals = Vec::new();
        for (r, route) in sol.routes.iter().enumerate() {
            for i in 0..(route.stops().len()) {
                if tabu.contains(&route.stops()[i]) { continue; }
                let (new_cost, feas) = route.speculative_remove_stop(i);
                assert!(feas);
                // we want the new cost to be much less than previous, so maximize cost
                // let cost = route.cost() - new_cost;
                // println!("removing i{:?} from {:?} has cost {:?} & feas {:?}", i, route, cost, feas);

                if feas {
                    feas_vals.push((r, i));
                }
            }
        }

        println!("feas_vals.len {:?}", feas_vals);
        // assert!(feas_vals.len() == (self.instance.num_customers - 1));
        let (chosen_spot_r, chosen_spot_i) = *feas_vals.get(rng().random_range(0..feas_vals.len())).unwrap();

        println!("chose to remove {:?} from {:?} @ {:?}", sol.routes[chosen_spot_r].stops()[chosen_spot_i], sol.routes[chosen_spot_r], chosen_spot_i);

        let res = sol.routes[chosen_spot_r].remove_stop(chosen_spot_i);
        return (res, chosen_spot_r);
    }

    fn reinsert_in_best_spot(sol: &mut VRPSolution, stop: Stop) {
        let (mut best_spot_r, mut best_spot_i, mut best_spot_cost_increase) = (100000, 100000, f64::MAX);

        let mut valid = Vec::new();

        for (r, route) in sol.routes.iter().enumerate() {
            for i in 0..(route.stops().len() + 1) {
                let (new_cost, feas) = route.speculative_add_stop(&stop, i);

                // we want the one that will increase the new cost by the least, so minimize
                let cost_increase = new_cost - route.cost();
                println!("res for adding {:?} to {:?} (@{:?}) is {:?}", stop, route, i, (cost_increase, feas));
                // println!("existing is {:?}", best_spot_cost_increase);
                if feas { valid.push((r, i)); }
                if feas && cost_increase < best_spot_cost_increase {
                    (best_spot_r, best_spot_i) = (r, i);
                    best_spot_cost_increase = cost_increase;
                }
            }
        }

        if rng().random_bool(0.02_f64) {
            let i = rng().random_range(0..valid.len());
            // println!("RANDOM DROP at i {i}...");
            (best_spot_r, best_spot_i) = *valid.get(i).unwrap();
        }

        println!("best was to add {:?} to {:?} @ {:?}", stop, sol.routes[best_spot_r], best_spot_i);
        sol.routes[best_spot_r].add_stop_to_index(stop, best_spot_i);
    }
}


