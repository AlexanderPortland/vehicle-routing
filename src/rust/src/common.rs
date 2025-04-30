use std::collections::HashSet;

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
}

pub struct Route<'a> {
    distance_matrix: &'a DistanceMatrix,
    stops: Vec<Stop>,
    cost: f64,
    used_cap: usize
}

impl<'a> Route<'a> {
    pub fn new(num_customers: usize, distance_matrix: &'a DistanceMatrix) -> Self {
        Route { distance_matrix, stops: Vec::with_capacity(num_customers), cost: 0f64, used_cap: 0 }
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
        
        // capacity just increases by the stop's capacity
        self.used_cap += stop.capacity;

        // cost will decrease by the two current ones being split up
        let cost_dec = self.cost_at_index(index);

        // and will increase by the two new paths we have to take...
        let c = if let Some(t) = self.stops.first() {

        } else { 0f64 };
        let cost_inc = self.distance_matrix.dist(self.val, b)
            + self.distance_matrix.dist(a, b);

        self.assert_sanity();
        todo!()
    }

    // 0 -> stop[0] -> stop[1] -...-> stop[len - 1] -> 0
    /// The cost of going from the previous index to `index`. (if `index` == `len`, cost of going home after...)
    pub fn cost_at_index(&self, index: usize) -> f64 {
        assert!(index <= self.stops.len());

        let start = self.stops.get(index - 1).map(|s|s.cust_no).or(Some(0)).unwrap();
        let end = self.stops.get(index).map(|s|s.cust_no).or(Some(0)).unwrap();

        self.distance_matrix.dist(start, end)
    }

    /// If you give an index in the full, real route (NOT SELF.STOPS)
    pub fn cust_no_at_index(&self, index: usize) -> f64 {
        assert!(index <= self.stops.len() + 1);
        
    }

    pub fn remove_at_index(&mut self, index: usize) -> Stop {
        todo!()
    }

    // *********** SANITY CHECKING SHIT ***********

    pub fn assert_sanity(&self) {
        self.check_route_cost();
        self.check_capacity();
        self.check_no_duplicate_stops();
    }
    
    fn check_route_cost(&self) {
        let mut cost = 0f64;

        for i in 1..self.stops.len() {
            cost += self.distance_matrix.dist(self.stops[i - 1].cust_no, self.stops[i].cust_no);
        }

        if self.stops.len() > 0 {
            cost += self.distance_matrix.dist(0, self.stops[0].cust_no);
            cost += self.distance_matrix.dist(self.stops[self.stops.len() - 1].cust_no, 0);
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