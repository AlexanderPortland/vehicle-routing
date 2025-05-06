use std::{
    cmp::Reverse, collections::{HashMap, VecDeque}, sync::Arc
};

use rand::{rng, rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::{common::{Route, Stop, VRPSolution}, dbg_println, vrp_instance};
use crate::construct;
use crate::solver::stats::SolveStats;
use crate::solver::{IterativeSolver, LNSSolver};
use crate::vrp_instance::VRPInstance;

/// An LNS solver which greedily **removes the highest cost stop** from the solution,
/// **inserting it at the lowest cost location**.
pub struct ALNSSolver {
    instance: Arc<VRPInstance>,
    stop_tabu: VecDeque<usize>,
    stop_not_tabu: Vec<usize>,
    current: VRPSolution,
    stats: SolveStats,
    rng: ThreadRng,
}

impl LNSSolver for ALNSSolver {
    /// corresponding to the (cust_no, route #) that was removed
    type DestroyResult = Vec<(Stop, usize)>;

    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self {
        // println!("num customers is {:?}", instance.num_customers);
        ALNSSolver {
            stop_tabu: VecDeque::new(),
            current: initial_solution,
            stop_not_tabu: (1..instance.num_customers).collect(),
            instance,
            stats: SolveStats::new(),
            rng: rand::rng()
        }
    }

    fn current(&self) -> VRPSolution {
        return self.current.clone();
    }

    fn destroy(&mut self) -> Self::DestroyResult {
        // TODO: tune the number of stops to remove / have it be variable??
        let removed_stops = self.remove_n_random_stops(5);

        for (stop, route_idx) in removed_stops.iter() {
            *self
                .stats
                .cust_change_freq
                .entry(stop.cust_no().try_into().unwrap())
                .or_insert(0) += 1;
            *self.stats.route_remove_freq.entry(*route_idx).or_insert(0) += 1;
        }
        // dbg_println!("Removing: {:?}", removed_stops);
        return removed_stops;
    }

    fn get_stats_mut(&mut self) -> &mut SolveStats {
        &mut self.stats
    }

    fn repair(&mut self, res: Self::DestroyResult) -> Result<VRPSolution, String> {
        let route_idxs = self.reinsert_n_stops_in_best_spots(res)?;

        for route_idx in route_idxs {
            *self.stats.route_add_freq.entry(route_idx).or_insert(0) += 1;
        }
        // println!("Current solution: {:?}", self.current);
        Ok(self.current.clone())
    }

    fn jump_to_solution(&mut self, sol: VRPSolution) {
        self.current = sol;
        // for s in self.
        self.stop_not_tabu = (1..self.instance.num_customers).collect();
        self.stop_tabu.clear();
    }

    fn update_tabu(&mut self, res: &Self::DestroyResult) {
        for (stop, _) in res {
            let cust_no = stop.cust_no().try_into().unwrap();
            self.stop_tabu.push_back(cust_no);
            // self.valid_remove_cust_nos.swap_remove(self.valid_remove_cust_nos.index);
        }
        
        while self.stop_tabu.len() > (self.instance.num_customers / 10) {
            let allowed = self.stop_tabu.pop_front();
            if let Some(allowed) = allowed {
                self.stop_not_tabu.push(allowed);
            }
        }
    }
}

impl ALNSSolver {
    fn remove_n_shaw(&mut self, n: usize) -> Vec<(Stop, usize)> {
        let seed_cust_no = rng().random_range(1..self.instance.num_customers);
        let mut customer_nos = vec![seed_cust_no];
        let mut similarity_scores: Vec<(usize, f64)> = Vec::new();

        let alpha = 1.0;
        let beta = 0.1;

        for cust_no in 1..self.instance.num_customers {
            if cust_no != seed_cust_no {
                let dist = self.instance.distance_matrix.dist(seed_cust_no, cust_no);
                let demand_diff = (self.instance.demand_of_customer[seed_cust_no] as f64 - self.instance.demand_of_customer[cust_no] as f64).abs();
                let score = alpha * dist + beta * demand_diff;
                similarity_scores.push((cust_no, score));
            }
        }



        Vec::new()
    }

    fn remove_n_random_stops(&mut self, n: usize) -> Vec<(Stop, usize)> {
        assert!(n > 0);
        self.assert_tabu_sanity();

        let tabu = &self.stop_tabu;
        let sol = &mut self.current;

        // TODO: keep Vec metadata of valid customer ids to remove (so we dont have to filter)
        //              * update on tabu change
        //       remove from random index of it w/ .swap_remove() for quick removal
        let mut customer_nos = Vec::new();
        for _ in 0..n {
            let rem_index = self.rng.random_range(0..self.stop_not_tabu.len());
            customer_nos.push(self.stop_not_tabu.swap_remove(rem_index));
        }
        assert!(customer_nos.len() == n);
        // let mut customer_nos: Vec<usize> = (1..self.instance.num_customers).filter(|x| !tabu.contains(x)).collect();
        // customer_nos.shuffle(&mut self.rng);
        // customer_nos.truncate(n);

        let mut res = Vec::new();

        // TODO: Keep a hashmap (cust_no --> route_idx) for quick removals
        for cust_no in customer_nos {
            for (route_idx, route) in sol.routes.iter_mut().enumerate() {
                if let Some(index) = route.index_of_stop(cust_no.try_into().unwrap()) {
                    let removed_stop = route.remove_stop_at_index(index);
                    res.push((removed_stop, route_idx));
                    break;
                }
            }
        }
        res
    }

    #[cfg(debug_assertions)]
    fn assert_tabu_sanity(&self) {
        let full_tabu = self.stop_tabu.iter().chain(self.stop_not_tabu.iter()).collect::<Vec<_>>();

        // println!("stop tabu is {:?}, non tabu is {:?}", self.stop_tabu, self.stop_not_tabu);
        // println!("len is {:?}, num cust is {:?}", full_tabu.len(), self.instance.num_customers - 1);
        assert!(full_tabu.len() == (self.instance.num_customers - 1));
        for cust_no in 1..self.instance.num_customers {
            assert!(full_tabu.contains(&&cust_no));
        }
    }
    #[cfg(not(debug_assertions))]
    fn assert_tabu_sanity(&self) { }

    fn reinsert_n_stops_in_best_spots(&mut self, removed_stops: Vec<(Stop, usize)>) -> Result<Vec<usize>, String> {
        let mut res = Vec::new();
        let mut removed_stops = removed_stops.clone();
        removed_stops.sort_by_key(|x| Reverse(x.0.capacity()));
        for (stop, _) in removed_stops {
            res.push(self.reinsert_in_best_spot(stop)?);
        }
        Ok(res)
    }

    fn reinsert_in_best_spot(&mut self, stop: Stop) -> Result<usize, String> {
        let (mut best_spot_r, mut best_spot_i, mut best_spot_cost_increase) =
            (usize::MAX, usize::MAX, f64::MAX);

        let mut valid = Vec::new();

        for (r, route) in self.current.routes.iter().enumerate() {
            for i in 0..(route.stops().len() + 1) {
                let (new_cost, feas) = route.speculative_add_stop(&stop, i);

                // we want the one that will increase the new cost by the least, so minimize
                let cost_increase = new_cost - route.cost();
                if feas {
                    valid.push((r, i));
                }
                if feas && cost_increase < best_spot_cost_increase {
                    (best_spot_r, best_spot_i) = (r, i);
                    best_spot_cost_increase = cost_increase;
                }
            }
        }
        if best_spot_r == usize::MAX {
            return Err("no place to put customer".to_string());
        }

        if rng().random_bool(0.02_f64) {
            let i = rng().random_range(0..valid.len());
            (best_spot_r, best_spot_i) = *valid.get(i).unwrap();
        }
        // dbg_println!("Reinserting: {:?} at {}", stop, best_spot_r);
        self.current.routes[best_spot_r].add_stop_to_index(stop, best_spot_i);

        // println!("Solution after inserting {:?}: {:?}", stop, self.current);
        return Ok(best_spot_r);
    }
}
