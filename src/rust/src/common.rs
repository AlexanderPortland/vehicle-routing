use std::{cmp::{max, min}, collections::{HashMap, HashSet}, sync::Arc};

use crate::vrp_instance::{self, VRPInstance};

pub struct DistanceMatrix(Vec<Vec<f64>>);

impl DistanceMatrix {
    pub fn new(vec: Vec<Vec<f64>>) -> Self { DistanceMatrix(vec) }

    pub fn dist<T: Into<usize>>(&self, a: T, b: T) -> f64 {
        self.0[a.into()][b.into()]
    }
}

#[derive(Clone, Copy, Hash)]
pub struct Stop {
    cust_no: u16,
    capacity: usize
}

impl PartialEq for Stop {
    fn eq(&self, other: &Self) -> bool {
        self.cust_no == other.cust_no
    }
}

impl Eq for Stop { }

impl std::fmt::Debug for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}({:?})", self.cust_no, self.capacity))
    }
}

impl Stop {
    pub fn new(cust_no: u16, capacity: usize) -> Self {
        Stop { cust_no, capacity }
    }

    pub fn cust_no(&self) -> u16 { self.cust_no }
    pub fn capacity(&self) -> usize { self.capacity }
}


#[derive(Clone)]
pub struct VRPSolution {
    pub routes: Vec<Route>,
}


impl std::fmt::Debug for VRPSolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in self.routes.iter() {
            f.write_fmt(format_args!("{:?}, capacity: {}\n", r, r.used_capacity())).unwrap();
        }
        Ok(())
    }
}

impl VRPSolution {
    pub fn new(vrp_instance: Arc<VRPInstance>) -> Self {
        VRPSolution {
            routes: (0..vrp_instance.num_vehicles)
                .into_iter()
                .map(|i| Route::new(vrp_instance.clone(), i))
                .collect(),
        }
    }

    pub fn is_valid_solution(&self, vrp_instance: &Arc<VRPInstance>) -> bool {
        // all routes should be under capacity
        self.routes.iter().for_each(|r| if r.used_capacity() > vrp_instance.vehicle_capacity {
            panic!("route {:?} is over cap {:?}", r, vrp_instance.vehicle_capacity);
        });

        // all customers should be visited
        for c in 1..vrp_instance.num_customers {
            let is_visited = self.routes.iter().any(|r| r.contains_stop(c.try_into().unwrap()));
            if !is_visited {
                panic!("customer {} isn't visited in solution {:?}", c, self);
                return false;
            }
        }
        
        return true;
    }

    pub fn cost(&self) -> f64 {
        self.routes.iter().map(|route| route.cost()).sum()
    }

    pub fn to_string(&self) -> String {
        let route_strings: Vec<String> = self.routes.iter().map(|route| {
            let mut result = String::from("0");
            
            for stop in route.stops() {
                result.push_str(&format!(" {}", stop.cust_no()));
            }
            
            result.push_str(" 0");
            result
        }).collect();
        
        let mut combined = String::from("0 ");
        combined.push_str(&route_strings.join(" "));
        combined
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

impl VRPSolution {
    pub fn distance(a: &Self, b: &Self, instance: &Arc<VRPInstance>) -> f64 {
        let mut dist = 0;
        let map_a = a.make_vector(instance);
        let map_b = b.make_vector(instance);
        assert!(map_a.len() == map_b.len());
        assert!(map_b.len() == (instance.num_customers - 1) * (instance.num_customers - 2) / 2);

        for i in 1..instance.num_customers {
            for j in (i + 1)..instance.num_customers {
                let key = &(i.try_into().unwrap(), j.try_into().unwrap());
                dist += map_a.get(key).unwrap() ^ map_b.get(key).unwrap();
            }
        }
        (dist as f64).sqrt()
    }

    fn make_vector(&self, instance: &Arc<VRPInstance>) -> HashMap<(u16, u16), usize> {
        let mut map = HashMap::new();

        for r in &self.routes {
            for el in &r.stops {
                for other_cust in (el.cust_no + 1)..instance.num_customers.try_into().unwrap() {
                    let entry = (el.cust_no, other_cust);
                    assert_eq!(entry, (min(el.cust_no, other_cust), max(el.cust_no, other_cust)));
                    let val = if r.contains_stop(other_cust) {
                        1
                    } else { 0 };
                    let insert_res = map.insert(entry, val);
                    assert!(insert_res.is_none());
                }
            }
        }

        map
    }
}



#[derive(Clone)]
pub struct Route {
    pub instance: std::sync::Arc<VRPInstance>,
    id: usize,
    stops: Vec<Stop>,
    cost: f64,
    used_cap: usize
}

impl std::fmt::Debug for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.to_string()))
    }
}

impl Route {
    pub fn to_string(&self) -> String {
        let middle = self.stops.iter().map(|i| format!("{:?}", i)).collect::<Vec<String>>();
        let middle = middle.join(" -> ");
        format!("r{}[{middle}]", self.id)
    }
    pub fn new(instance: Arc<VRPInstance>, id: usize) -> Self {
        Route { stops: Vec::with_capacity(instance.num_customers), instance, cost: 0f64, used_cap: 0, id }
    }

    pub fn stops(&self) -> &Vec<Stop> { &self.stops }

    pub fn first(&self) -> usize {
        return self.stops.first().unwrap().cust_no().try_into().unwrap();
    }

    pub fn last(&self) -> usize {
        return self.stops.last().unwrap().cust_no().try_into().unwrap();
    }

    pub fn cost(&self) -> f64 {
        self.assert_sanity(); // TODO: remove for debug
        self.cost
    }

    pub fn used_capacity(&self) -> usize {
        self.assert_sanity(); // TODO: remove for debug
        self.used_cap
    }

    pub fn contains_stop(&self, cust_no: u16) -> bool {
        self.stops.iter().any(|a|{ a.cust_no == cust_no })
    }

    pub fn index_of_stop(&self, cust_no: u16) -> Option<usize> {
        self.stops.iter().position(|a| {a.cust_no == cust_no})
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

    pub fn remove_stop_at_index(&mut self, index: usize) -> Stop {
        self.assert_sanity();
        assert!(index <= self.stops.len()); // should be less than stops.len()

        let (new_cost, _) = self.speculative_remove_stop(index);
        let stop = self.stops.remove(index);
        self.used_cap -= stop.capacity;
        self.cost = new_cost;

        self.assert_sanity();
        
        return stop;
    }

    pub fn speculative_replace_stop(&self, stop: &Stop, index: usize) -> (f64, bool) {
        self.assert_sanity();
        assert!(index <= self.stops.len());

        let mut new_cost = self.cost;
        let before = if index != 0 {
            self.stops[index - 1].cust_no
        } else { 0 };

        let after = if index != self.stops.len() {
            self.stops[index].cust_no
        } else { 0 };

        new_cost -= self.instance.distance_matrix.dist(before, self.stops[index].cust_no);
        new_cost -= self.instance.distance_matrix.dist(self.stops[index].cust_no, after);
        new_cost += self.instance.distance_matrix.dist(before, stop.cust_no);
        new_cost += self.instance.distance_matrix.dist(stop.cust_no, after);

        return (new_cost, self.used_cap - self.stops[index].capacity + stop.capacity <= self.instance.vehicle_capacity);
    }

    pub fn speculative_add_best(&self, stop: &Stop) -> ((f64, bool), usize) {
        self.assert_sanity();

        // println!("stops are {:?}", self.stops);
        let best_index = if self.stops.is_empty() { 
            0 
        } else {
            (0..self.stops.len()).into_iter().max_by_key(|i|{
                -1 * (self.speculative_add_stop(stop, *i).0 as isize)
            }).unwrap()
        };

        let best_val = self.speculative_add_stop(stop, best_index);

        (best_val, best_index)
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

        let res = (new_cost, self.used_cap + stop.capacity <= self.instance.vehicle_capacity);

        return res;
    }

    pub fn speculative_remove_stop(&self, index: usize) -> (f64, bool) {
        self.assert_sanity();
        assert!(index < self.stops.len());
        // assert!(self.stops[index])
        let stop = &self.stops[index];

        let mut new_cost = self.cost; // TODO: could change to be relative
        // new_cost += self.cost_at_index(index);

        let before = if index != 0 {
            self.stops[index - 1].cust_no
        } else { 0 };

        let after = if index != (self.stops.len() - 1) {
            self.stops[index + 1].cust_no
        } else { 0 };

        // println!("before {:?}, after {:?}", before, after);

        let to_me = self.instance.distance_matrix.dist(before, stop.cust_no);
        let from_me = self.instance.distance_matrix.dist(stop.cust_no, after);
        let bypass = self.instance.distance_matrix.dist(before, after);

        // println!("tome {:?}, from me {:?}, bypass {:?}", to_me, from_me, bypass);
        new_cost -= self.instance.distance_matrix.dist(before, stop.cust_no);
        new_cost -= self.instance.distance_matrix.dist(stop.cust_no, after);
        new_cost += self.instance.distance_matrix.dist(before, after);

        // println!("spec remove for index {:?} of {:?} is {:?}", index, self, new_cost);

        (new_cost, self.used_cap - self.stops[index].capacity <= self.instance.vehicle_capacity)
    }

    pub fn cost_if_cust_no_was(&self, new_stop: &Stop, index: usize) -> f64 {
        self.assert_sanity();
        assert!(index < self.stops.len());
        // assert!(self.stops[index])
        let old_stop = &self.stops[index];

        let mut new_cost = self.cost; // TODO: could change to be relative

        let before = if index != 0 {
            self.stops[index - 1].cust_no
        } else { 0 };

        let after = if index != (self.stops.len() - 1) {
            self.stops[index + 1].cust_no
        } else { 0 };

        new_cost -= self.instance.distance_matrix.dist(before, old_stop.cust_no);
        new_cost -= self.instance.distance_matrix.dist(old_stop.cust_no, after);
        new_cost += self.instance.distance_matrix.dist(before, new_stop.cust_no);
        new_cost += self.instance.distance_matrix.dist(new_stop.cust_no, after);

        new_cost
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
        // println!("** trying to assert sanity on {:?}", self);

        self.check_route_cost();
        self.check_capacity();
        self.check_no_duplicate_stops();

        // println!("** asserted sanity on {:?}", self);
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

        // println!("got cost {:?} for {:?} (vs {:?})", cost, self, self.cost);

        assert!((cost - self.cost).abs() < 0.5f64);
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