use crate::{old_solver::VRPSolution, vrp_instance::VRPInstance};

// trait for a large neighborhood search (LNS) solver
pub trait LNSSolver<'a> {
    type DestroyResult;

    fn new(instance: &'a VRPInstance) -> Self;

    /// Construct an initial feasible solution.
    fn construct(&mut self) -> VRPSolution<'a>;

    /// Partially destroy the solution.
    fn destroy(&mut self) -> Self::DestroyResult;

    /// Repair the solution and return the result.
    fn repair(&mut self, res: Self::DestroyResult) -> VRPSolution<'a>;

    /// Update the solution to search from next.
    fn update_search_location(&mut self, new_best: Option<(&VRPSolution<'a>, f64)>);
}

pub struct SolveParams {
    pub max_iters: usize,
}

pub trait IterativeSolver<'a> {
    fn new(instance: &'a VRPInstance) -> Self;

    fn initial_solution(&mut self) -> VRPSolution<'a>;
    fn find_new_solution(&mut self) -> VRPSolution<'a>;

    /// Update the solution to search from next.
    fn update_search_location(&mut self, new_best: Option<(&VRPSolution<'a>, f64)>);
}

/// Completely solve a VRP instance and return the best solution found.
fn solve<'a, S: IterativeSolver<'a>>(instance: &'a VRPInstance, params: SolveParams) -> VRPSolution<'a> {
    let mut solver = S::new(instance);

    let mut best = solver.initial_solution();
    let mut best_cost = best.cost();
    // let mut small_float_diff = 0;

    for _iter in 0..params.max_iters {
        let new_solution = solver.find_new_solution();
        debug_assert!(new_solution.is_valid_solution(&instance));

        if new_solution.cost() < best_cost {
            (best_cost, best) = (new_solution.cost(), new_solution);
            solver.update_search_location(Some((&best, best_cost)));
        } else {
            solver.update_search_location(None);
        }
    }

    best
}

impl<'a, T> IterativeSolver<'a> for T
    where T: LNSSolver<'a> {
    fn new(instance: &'a VRPInstance) -> Self {
        Self::new(&instance)
    }

    fn initial_solution(&mut self) -> VRPSolution<'a> {
        self.construct()
    }

    fn find_new_solution(&mut self) -> VRPSolution<'a> {
        let destroy_res = self.destroy();
        self.repair(destroy_res)
    }

    fn update_search_location(&mut self, new_best: Option<(&VRPSolution<'a>, f64)>) {
        self.update_search_location(new_best);
    }
}


mod tabu_solver {
    use crate::{common::Stop, old_solver::VRPSolution};

    const MAX_TABU: usize = 5;
    pub struct TabuLocalSolver<'a> {
        tabu: Vec<Stop>,
        current: VRPSolution<'a>,
    }
}
