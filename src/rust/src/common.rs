use std::collections::HashSet;

use crate::vrp_instance::VRPInstance;

pub struct DistanceMatrix(Vec<Vec<f64>>);

impl DistanceMatrix {
    pub fn new(vec: Vec<Vec<f64>>) -> Self { DistanceMatrix(vec) }

    pub fn dist<T: Into<usize>>(&self, a: T, b: T) -> f64 {
        self.0[a.into()][b.into()]
    }
}

pub struct Stop {
    cust_no: u16,
    capacity: usize
}

impl Stop {
    pub fn new(cust_no: u16, capacity: usize) -> Self {
        Stop { cust_no, capacity }
    }

    pub fn cust_no(&self) -> u16 { self.cust_no }
    pub fn capacity(&self) -> usize { self.capacity }
}

pub struct Route<'a> {
    instance: &'a VRPInstance,
    stops: Vec<Stop>,
    cost: f64,
    used_cap: usize
}

impl<'a> Route<'a> {
    pub fn new(instance: &'a VRPInstance) -> Self {
        Route { instance, stops: Vec::with_capacity(instance.num_customers), cost: 0f64, used_cap: 0 }
    }

    pub fn stops(&self) -> &Vec<Stop> { &self.stops }

    pub fn route_cost(&self) -> f64 {
        self.assert_sanity(); // TODO: remove for debug
        self.cost
    }

    pub fn route_used_cap(&self) -> usize {
        self.assert_sanity(); // TODO: remove for debug
        self.used_cap
    }

    pub fn contains_stop(&self, cust_no: u16) -> bool {
        self.stops.iter().any(|a|{ a.cust_no == cust_no })
    }

    pub fn add_stop_to_index(&mut self, stop: Stop, index: usize) {
        self.assert_sanity();
        assert!(index <= self.stops.len()); // should be less than stops.len()
        
        let cap = stop.capacity;
        let (new_cost, _) = self.speculative_add_stop(&stop, index);
        self.stops.insert(index, stop);
        self.used_cap += cap;
        self.cost = new_cost;

        self.assert_sanity();
    }

    pub fn remove_stop(&mut self, index: usize) {
        self.assert_sanity();
        assert!(index <= self.stops.len()); // should be less than stops.len()

        let (new_cost, _) = self.speculative_remove_stop(index);
        let stop = self.stops.remove(index);
        self.used_cap -= stop.capacity;
        self.cost = new_cost;

        self.assert_sanity();
    }

    // the change in cost for how much adding 
    pub fn speculative_add_stop(&self, stop: &Stop, index: usize) -> (f64, bool) {
        self.assert_sanity();
        assert!(index <= self.stops.len());

        let mut new_cost = self.cost; // TODO: could change to be relative
        new_cost -= self.cost_at_index(index);

        let before = if index != 0 {
            self.stops[index - 1].cust_no
        } else { 0 };

        let after = if index != self.stops.len() {
            self.stops[index].cust_no
        } else { 0 };

        new_cost += self.instance.distance_matrix.dist(before, stop.cust_no);
        new_cost += self.instance.distance_matrix.dist(stop.cust_no, after);

        return (new_cost, self.used_cap + stop.capacity >= self.instance.vehicle_capacity);
    }

    pub fn speculative_remove_stop(&self, index: usize) -> (f64, bool) {
        self.assert_sanity();
        assert!(index <= self.stops.len());
        // assert!(self.stops[index])
        let stop = &self.stops[index];

        let mut new_cost = self.cost; // TODO: could change to be relative
        new_cost += self.cost_at_index(index);

        let before = if index != 0 {
            self.stops[index - 1].cust_no
        } else { 0 };

        let after = if index != self.stops.len() {
            self.stops[index].cust_no
        } else { 0 };

        new_cost -= self.instance.distance_matrix.dist(before, stop.cust_no);
        new_cost -= self.instance.distance_matrix.dist(stop.cust_no, after);

        (new_cost, self.used_cap - self.stops[index].capacity >= self.instance.vehicle_capacity)
    }

    // -1      0         1
    // 0 -> stop[0] -> stop[1] -...-> stop[len - 1] -> 0
    /// The cost of going from the previous index to `index`. (if `index` == `len`, cost of going home after...)
    pub fn cost_at_index(&self, index: usize) -> f64 {
        assert!(index <= self.stops.len());

        let start = if index != 0 {
            self.stops[index - 1].cust_no
        } else { 0 };

        let end = if index != self.stops.len() {
            self.stops[index].cust_no
        } else { 0 };

        self.instance.distance_matrix.dist(start, end)
    }

    // *********** SANITY CHECKING ***********

    pub fn assert_sanity(&self) {
        self.check_route_cost();
        self.check_capacity();
        self.check_no_duplicate_stops();
    }
    
    fn check_route_cost(&self) {
        let mut cost = 0f64;

        for i in 1..self.stops.len() {
            cost += self.instance.distance_matrix.dist(self.stops[i - 1].cust_no, self.stops[i].cust_no);
        }

        if self.stops.len() > 0 {
            cost += self.instance.distance_matrix.dist(0, self.stops[0].cust_no);
            cost += self.instance.distance_matrix.dist(self.stops[self.stops.len() - 1].cust_no, 0);
        }

        assert!(cost == self.cost);
    }

    fn check_capacity(&self) {
        let real_cap: usize = self.stops.iter().map(|s|{
            s.capacity
        }).sum();
        assert!(real_cap == self.used_cap);
    }

    fn check_no_duplicate_stops(&self) {
        let mut existing = HashSet::new();

        for el in &self.stops {
            assert!(!existing.contains(&el.cust_no));
            existing.insert(el.cust_no);
        }

        assert!(existing.len() == self.stops.len());
    }
}