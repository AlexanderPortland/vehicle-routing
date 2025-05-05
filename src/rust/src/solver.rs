use std::sync::Arc;

use stats::SolveStats;

use crate::{common::VRPSolution, vrp_instance::VRPInstance};

pub struct SolveParams {
    pub max_iters: usize,
    /// jump after this many stagnant iterations
    pub patience: usize,
    pub constructor: fn(&Arc<VRPInstance>) -> VRPSolution,
}

// trait for a large neighborhood search (LNS) solver
pub trait LNSSolver {
    type DestroyResult;

    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self;

    /// Partially destroy the solution.
    fn destroy(&mut self) -> Self::DestroyResult;

    /// Repair the solution and return the result.
    fn repair(&mut self, res: Self::DestroyResult) -> VRPSolution;

    fn get_stats_mut(&mut self) -> &mut SolveStats;

    // / Update the solution to search from next.
    // fn update_search_location(&mut self, new_best: Option<(&VRPSolution, f64)>);
    fn jump_to_solution(&mut self, sol: VRPSolution);

    // Optionally update the tabu for the solver.
    fn update_tabu(&mut self, res: &Self::DestroyResult) {}
}

pub trait IterativeSolver {
    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self;

    fn find_new_solution(&mut self) -> VRPSolution;

    fn jump_to_solution(&mut self, sol: VRPSolution);

    fn get_stats_mut(&mut self) -> &mut SolveStats;
}

#[cfg(debug_assertions)]
pub mod stats {
    use std::collections::HashMap;

    use crate::common::VRPSolution;

    #[derive(Debug)]
    pub struct SolveStats {
        iterations: usize,
        improvements: Vec<(usize, f64)>,
        restarts: Vec<usize>,
        cust_change_freq: HashMap<usize, usize>,
        route_change_freq: HashMap<usize, usize>,
    }
    
    impl SolveStats {
        pub fn new() -> Self {
            SolveStats { 
                iterations: 0,
                improvements: Vec::new(),
                restarts: Vec::new(),
                cust_change_freq: HashMap::new(),
                route_change_freq: HashMap::new(),
            }
        }

        pub fn update_on_iter(&mut self, iter: usize, new_sol: &VRPSolution, improvement_on_best: f64) {
            if improvement_on_best > 0.01 {
                self.improvements.push((iter, new_sol.cost()));
            }
            self.iterations += 1;
        }

        pub fn on_restart(&mut self, iter: usize) {
            self.restarts.push(iter);
        }
    }
}

#[cfg(not(debug_assertions))]
mod stats {
    use crate::common::VRPSolution;

    #[derive(Debug)]
    pub struct SolveStats();

    impl SolveStats {
        pub fn new() -> Self {
            SolveStats()
        }

        pub fn update_stats(&mut self, iter: usize, new_sol: &VRPSolution, improvement: f64) {}

        pub fn on_restart(&mut self, iter: usize) {}
    }
}


type SolveResult = (VRPSolution, SolveStats);

/// Completely solve a VRP instance and return the best solution found.
pub fn solve<S: IterativeSolver>(instance: Arc<VRPInstance>, params: SolveParams) -> VRPSolution {
    let initial_solution = (params.constructor)(&instance);
    let mut solver = S::new(instance.clone(), initial_solution.clone());

    let mut best = initial_solution; // TODO: stop cloning these
    let mut best_cost = best.cost();
    let mut stagnant_iterations = 0;
    let mut last_cost = best.cost();

    for iter in 0..params.max_iters {
        let new_solution = solver.find_new_solution();
        debug_assert!(new_solution.is_valid_solution(&instance));

        let new_cost = new_solution.cost();
        solver.get_stats_mut().update_on_iter(iter, &new_solution, best_cost - new_cost);
        
        if new_cost < best_cost {
            (best_cost, best) = (new_solution.cost(), new_solution);
        } else {
            if iter % 20 == 0 {println!("iter {:?} has cost {:?}", iter, new_cost);}
        }

        if last_cost < new_cost || (new_cost - last_cost).abs() < 0.01 {
            // no improvement
            stagnant_iterations += 1;
        } else {
            stagnant_iterations = 0;
        }

        last_cost = new_cost;

        if stagnant_iterations > params.patience {
            stagnant_iterations = 0;
            solver.jump_to_solution((params.constructor)(&instance));
            solver.get_stats_mut().on_restart(iter);
        }
    }

    println!("Stats: {:?}", solver.get_stats_mut());
    best
}

impl<T> IterativeSolver for T where T: LNSSolver {
    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self {
        Self::new(instance, initial_solution)
    }

    fn get_stats_mut(&mut self) -> &mut SolveStats {
        self.get_stats_mut()
    }

    fn find_new_solution(&mut self) -> VRPSolution {
        let destroy_res = self.destroy();
        self.update_tabu(&destroy_res);
        self.repair(destroy_res)
    }
    
    fn jump_to_solution(&mut self, sol: VRPSolution) {
        self.jump_to_solution(sol);
    }
}
