use std::sync::Arc;

use stats::SolveStats;

use crate::{old_solver::VRPSolution, vrp_instance::VRPInstance};

// trait for a large neighborhood search (LNS) solver
pub trait LNSSolver {
    type DestroyResult;

    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self;

    /// Partially destroy the solution.
    fn destroy(&mut self) -> Self::DestroyResult;

    /// Repair the solution and return the result.
    fn repair(&mut self, res: Self::DestroyResult) -> VRPSolution;

    /// Update the solution to search from next.
    fn update_search_location(&mut self, new_best: Option<(&VRPSolution, f64)>);

    // Optionally update the tabu for the solver.
    fn update_tabu(&mut self, res: &Self::DestroyResult) {}
}

pub struct SolveParams {
    pub max_iters: usize,
    pub constructor: fn(&Arc<VRPInstance>) -> VRPSolution,
}

pub trait IterativeSolver {
    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self;

    fn find_new_solution(&mut self) -> VRPSolution;

    /// Update the solution to search from next.
    fn update_search_location(&mut self, new_best: Option<(&VRPSolution, f64)>);
}

#[cfg(debug_assertions)]
mod stats {
    use crate::old_solver::VRPSolution;

    pub struct SolveStats {
        improvements: Vec<(usize, f64)>,
        small_improvements: usize,
    }
    
    impl SolveStats {
        pub fn new() -> Self {
            SolveStats { 
                improvements: Vec::new(),
                small_improvements: 0,
            }
        }

        pub fn update_stats(&mut self, iter: usize, new_sol: &VRPSolution, improvement: f64) {
            println!("iter {} improved by {:?}", iter, improvement);
            if improvement > 0f64 {
                self.improvements.push((iter, new_sol.cost()));
                if improvement < 0.01 {
                    self.small_improvements += 1;
                    println!("SMALL IMPROVEMENT");
                }
            }
        }
    }
}

#[cfg(not(debug_assertions))]
mod stats {
    pub struct SolveStats();
    impl SolveStats {
        pub fn new() -> Self {
            SolveStats()
        }

        pub fn update_stats(&mut self, iter: usize, new_sol: &VRPSolution, new_best: bool);
    }
}


type SolveResult = (VRPSolution, SolveStats);
/// Completely solve a VRP instance and return the best solution found.
pub fn solve<'a, S: IterativeSolver>(instance: Arc<VRPInstance>, params: SolveParams) -> VRPSolution {
    let initial_solution = (params.constructor)(&instance);
    let mut solver = S::new(instance.clone(), initial_solution.clone());
    let mut solve_stats = SolveStats::new();

    let mut best = initial_solution; // TODO: stop cloning these
    let mut best_cost = best.cost();
    // let mut small_float_diff = 0;

    for iter in 0..params.max_iters {
        let new_solution = solver.find_new_solution();
        debug_assert!(new_solution.is_valid_solution(&instance));

        solve_stats.update_stats(iter, &new_solution, best_cost - new_solution.cost());
        
        if new_solution.cost() < best_cost {
            (best_cost, best) = (new_solution.cost(), new_solution);
            solver.update_search_location(Some((&best, best_cost)));
        } else {
            solver.update_search_location(None);
        }
    }

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

    fn update_search_location(&mut self, new_best: Option<(&VRPSolution, f64)>) {
        self.update_search_location(new_best);
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

pub struct TodoSolver;

impl IterativeSolver for TodoSolver {
    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self {
        todo!()
    }

    fn find_new_solution(&mut self) -> VRPSolution {
        todo!()
    }

    fn update_search_location(&mut self, new_best: Option<(&VRPSolution, f64)>) {
        todo!()
    }
}