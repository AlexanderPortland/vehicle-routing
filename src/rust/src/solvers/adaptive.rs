use std::{
    cmp::Reverse, collections::{BinaryHeap, HashMap, VecDeque}, sync::Arc
};

use rand::{rng, rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::{common::{Route, Stop, VRPSolution}, dbg_println, vrp_instance};
use crate::construct;
use crate::solver::stats::SolveStats;
use crate::solver::{IterativeSolver, LNSSolver};
use crate::vrp_instance::VRPInstance;
use ordered_float::OrderedFloat;

use rand::prelude::*;

#[derive(Debug, Clone)]
struct Operator {
    id: usize,
    score: usize,
    weight: f64,
    usage_count: usize,
}

impl Operator {
    fn new(id: usize) -> Self {
        Self {
            id,
            score: 0,
            weight: 1.0,
            usage_count: 0,
        }
    }

    fn update_score(&mut self, delta: usize) {
        self.score += delta;
        self.usage_count += 1;
    }

    fn update_weight(&mut self, learning_rate: f64) {
        self.weight = (1.0 - learning_rate) * self.weight + learning_rate * (self.score as f64);
        self.score = 0;
    }
}


/// An LNS solver which greedily **removes the highest cost stop** from the solution,
/// **inserting it at the lowest cost location**.
pub struct ALNSSolver {
    instance: Arc<VRPInstance>,
    stop_tabu: VecDeque<usize>,
    stop_not_tabu: Vec<usize>,
    current: VRPSolution,
    stats: SolveStats,
    rng: ThreadRng,
    repair_ops: Vec<Operator>,
    destroy_ops: Vec<Operator>,
    last_used_repair_op: usize,
    last_used_destroy_op: usize,
}

impl LNSSolver for ALNSSolver {

    /// corresponding to the (cust_no, route #) that was removed
    type DestroyResult = Vec<(Stop, usize)>;

    fn new(instance: Arc<VRPInstance>, initial_solution: VRPSolution) -> Self {
        ALNSSolver {
            stop_tabu: VecDeque::new(),
            current: initial_solution,
            stop_not_tabu: (1..instance.num_customers).collect(),
            instance,
            stats: SolveStats::new(),
            rng: rand::rng(),
            repair_ops: vec![Operator::new(0), Operator::new(1)],
            destroy_ops: vec![Operator::new(0), Operator::new(1)],
            last_used_destroy_op: 0,
            last_used_repair_op: 0
        }
    }

    fn current(&self) -> &VRPSolution {
        return &self.current;
    }

    fn destroy(&mut self) -> Self::DestroyResult {
        // TODO: tune the number of stops to remove / have it be variable??
        let removed_stops = if rng().random_bool(self.destroy_ops[0].weight / (self.destroy_ops[0].weight + self.destroy_ops[1].weight)) {
            self.last_used_destroy_op = 0;
            self.remove_n_random_stops(5)
        } else {
            self.last_used_destroy_op = 1;
            self.remove_n_shaw(5)
        };

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

    fn repair(&mut self, res: Self::DestroyResult) -> Result<(), String> {
        let route_idxs = if rng().random_bool(self.repair_ops[0].weight / (self.repair_ops[0].weight + self.destroy_ops[1].weight)) {
            self.last_used_repair_op = 0;
            self.reinsert_n_stops_in_best_spots(res)?
        } else {
            self.last_used_repair_op = 1;
            self.reinsert_two_regret(res)?
        };

        for route_idx in route_idxs {
            *self.stats.route_add_freq.entry(route_idx).or_insert(0) += 1;
        }
        // println!("Current solution: {:?}", self.current);
        // write_out_sol.clone_from(&self.current);
        Ok(())
    }

    fn jump_to_solution(&mut self, sol: &VRPSolution) {
        self.current.clone_from(sol); // clone directly into exising allocations
        // self.current = sol;
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

    fn update_scores(&mut self, delta: usize) {
        self.repair_ops[self.last_used_repair_op].update_score(delta);
        self.destroy_ops[self.last_used_destroy_op].update_score(delta);
    }

    fn update_weights(&mut self) {
        for op_idx in 0..self.repair_ops.len() {
            self.repair_ops[op_idx].update_weight(0.01);
        }

        for op_idx in 0..self.destroy_ops.len() {
            self.destroy_ops[op_idx].update_weight(0.01);
        }

        // for op_idx in 0..self.repair_ops.len() {
        //     println!("op_idx ({}): weight: {}, # times used: {}", op_idx, self.repair_ops[op_idx].weight, self.repair_ops[op_idx].usage_count);
        // }

        // for op_idx in 0..self.destroy_ops.len() {
        //     println!("op_idx ({}): {}, # times used: {}", op_idx, self.destroy_ops[op_idx].weight, self.destroy_ops[op_idx].usage_count);
        // }
    }
}

impl ALNSSolver {
    fn remove_n_shaw(&mut self, n: usize) -> Vec<(Stop, usize)> {
        let seed_cust_no = rng().random_range(1..self.instance.num_customers);
        let alpha = 0.5;
        let beta = 0.5;
        
        let tabu = &self.stop_tabu;
        let sol = &mut self.current;

        let mut similarity_scores: Vec<(usize, f64)> = (1..self.instance.num_customers).map(|cust_no| {
            let dist = self.instance.distance_matrix.dist(seed_cust_no, cust_no);
            let demand_diff = (self.instance.demand_of_customer[seed_cust_no] as f64 - self.instance.demand_of_customer[cust_no] as f64).abs();
            let score = alpha * dist + beta * demand_diff;
            (cust_no, score)
        }).collect();
        similarity_scores.sort_by_key(|(cust_no, score)| OrderedFloat(*score)); 

        let mut customer_nos = Vec::new();
        for i in 0..n {
            customer_nos.push(similarity_scores[i].0);
        }

        let mut res = Vec::new();
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

        let mut valid = Vec::with_capacity(self.instance.num_customers);

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

    fn reinsert_two_regret(&mut self, removed_stops: Vec<(Stop, usize)>) -> Result<Vec<usize>, String> {
        let mut res = Vec::new();
        let mut removed_stops = removed_stops.clone();

        removed_stops.sort_by_key(|(stop, _)| {
            Reverse(OrderedFloat(self.regret_k(stop, 2)))
        });

        for (stop, _) in removed_stops {
            res.push(self.reinsert_in_best_spot(stop)?);
        }
        Ok(res)
    }

    fn regret_k(&self, stop: &Stop, k: usize) -> f64 {
        let mut costs = BinaryHeap::new();
        for route in &self.current.routes {
            for stop_idx in 0..(route.stops().len() + 1) {
                let (new_cost, feasible) = route.speculative_add_stop(stop, stop_idx);
                
                let cost_increase = new_cost - route.cost();

                // BinaryHeap is a max heap. Since we want to keep a min-heap of the costs, we wrap in reverse
                // OrderedFloat is to make life easy w.r.t to putting floats in a heap
                if feasible {
                    costs.push(Reverse(OrderedFloat(cost_increase)));
                }

                if costs.len() > k {
                    costs.pop();
                }
            }
        }

        if costs.len() < k {
            return costs.peek().unwrap().0.0;
        }

        let best = costs.pop().unwrap().0.0;
        let kth_best = costs.peek().unwrap().0.0;

        kth_best - best
    }
}
