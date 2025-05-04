use std::sync::Arc;

use stats::SolveStats;

use crate::{common::VRPSolution, vrp_instance::VRPInstance};

// trait for a large neighborhood search (LNS) solver
pub trait LNSSolver {
    type DestroyResult;

    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self;

    /// Partially destroy the solution.
    fn destroy(&mut self) -> Self::DestroyResult;

    /// Repair the solution and return the result.
    fn repair(&mut self, res: Self::DestroyResult) -> VRPSolution;

    // / Update the solution to search from next.
    // fn update_search_location(&mut self, new_best: Option<(&VRPSolution, f64)>);
    fn jump_to_solution(&mut self, sol: VRPSolution);

    // Optionally update the tabu for the solver.
    fn update_tabu(&mut self, res: &Self::DestroyResult) {}
}

pub struct SolveParams {
    pub max_iters: usize,
    /// jump after this many stagnant iterations
    pub patience: usize,
    pub constructor: fn(&Arc<VRPInstance>) -> VRPSolution,
}

pub trait IterativeSolver {
    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self;

    fn find_new_solution(&mut self) -> VRPSolution;

    fn jump_to_solution(&mut self, sol: VRPSolution);
}

#[cfg(debug_assertions)]
mod stats {
    use crate::common::VRPSolution;

    #[derive(Debug)]
    pub struct SolveStats {
        improvements: Vec<(usize, f64)>,
        restarts: Vec<usize>,
        small_improvements: usize,
    }
    
    impl SolveStats {
        pub fn new() -> Self {
            SolveStats { 
                improvements: Vec::new(),
                restarts: Vec::new(),
                small_improvements: 0,
            }
        }

        pub fn restart_count(&self) -> usize {
            self.restarts.len()
        }

        pub fn update_stats(&mut self, iter: usize, new_sol: &VRPSolution, improvement_on_best: f64) {
            // println!("iter {} improved by {:?}", iter, improvement);
            if improvement_on_best > 0f64 {
                println!("update stats says new best!!");
                self.improvements.push((iter, new_sol.cost()));
                if improvement_on_best < 0.01 {
                    self.small_improvements += 1;
                    // println!("SMALL IMPROVEMENT");
                }
            }
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
    println!("in solve...");
    let initial_solution = (params.constructor)(&instance);
    let mut solver = S::new(instance.clone(), initial_solution.clone());
    let mut solve_stats = SolveStats::new();

    let mut best = initial_solution; // TODO: stop cloning these
    let mut best_cost = best.cost();
    let mut stagnant_iterations = 0;
    let mut last_cost = best.cost();

    for iter in 0..params.max_iters {
        let new_solution = solver.find_new_solution();
        debug_assert!(new_solution.is_valid_solution(&instance));

        let new_cost = new_solution.cost();
        solve_stats.update_stats(iter, &new_solution, best_cost - new_cost);
        

        // TODO: Update
        // println!("iter {:?} has cost {:?}", iter, new_solution.cost());
        if new_cost < best_cost {
            println!("NEW BEST iter {:?} has cost {:?}", iter, new_cost);
            (best_cost, best) = (new_solution.cost(), new_solution);
        } else {
            if iter % 20 == 0 {println!("iter {:?} has cost {:?}", iter, new_cost);}
        }

        if last_cost < new_cost || (new_cost - last_cost).abs() < 0.01 {
            // no improvement
            println!("stagnating at iter {:?} (new cost {:?}, last {:?})", iter, new_cost, last_cost);
            stagnant_iterations += 1;
        } else {
            println!("NOT stagnating at iter {:?} (new cost {:?}, last {:?})", iter, new_cost, last_cost);
            stagnant_iterations = 0;
        }

        last_cost = new_cost;

        if stagnant_iterations > params.patience {
            println!("best is {:?}", best_cost);
            println!("restarting bc of {} iters of stagnation", params.patience);
            // todo!();
            stagnant_iterations = 0;
            solver.jump_to_solution((params.constructor)(&instance));
            solve_stats.on_restart(iter);
        }

        
    }

    println!("solve stats {:?}", solve_stats);
    // println!("{:?} restarts", solve_stats.restart_count());

    best
}

impl<T> IterativeSolver for T
    where T: LNSSolver {
    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self {
        Self::new(instance, initial_solution)
    }

    fn find_new_solution(&mut self) -> VRPSolution {
        let destroy_res = self.destroy();
        self.update_tabu(&destroy_res);
        self.repair(destroy_res)
    }
    
    fn jump_to_solution(&mut self, sol: VRPSolution) {
        self.jump_to_solution(sol);
    }

    // fn update_search_location(&mut self, new_best: Option<(&VRPSolution, f64)>) {
    //     self.update_search_location(new_best);
    // }
}
