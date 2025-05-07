use std::{cmp::{max, min}, collections::{HashMap, HashSet}, sync::Arc};

use crate::vrp_instance::{self, VRPInstance};

// const DEBUG_PRINTS: bool = true;

#[macro_export]
macro_rules! dbg_println {
    ($($arg:tt)*) => (if false { println!($($arg)*); });
}

// macro_rules! dbg_println {
//     ($($inner: tt,)*) => {
//         if DEBUG_PRINTS {
//             println!($($inner: block,)*);
//         }
//     };
// }

pub struct DistanceMatrix(&'static mut [&'static mut [f64]]);

impl DistanceMatrix {
    pub fn new(vec: Vec<Vec<f64>>) -> Self { 
        let v = vec.into_iter().map(|v|{
            v.leak()
        }).collect::<Vec<_>>().leak();

        DistanceMatrix(v)
    }

    pub fn dist<T: Into<usize>>(&self, a: T, b: T) -> f64 {
        let (a, b): (usize, usize) = (a.into(), b.into());

        debug_assert!(a < self.0.len());
        debug_assert!(b < self.0[a].len());

        // SAFETY: we gotta trust ourselves here that we did the bounds checking 
        //         properly outside this function. if we believe, and use the power of friendship,
        //         i think nothings impossible.
        let a = unsafe { self.0.get_unchecked(a).get_unchecked(b) };
        
        *a
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


// #[derive(Clone)]
pub struct VRPSolution {
    pub routes: Vec<Route>,
}

impl Clone for VRPSolution {
    fn clone(&self) -> Self {
        // println!("cloning normally");
        VRPSolution { routes: self.routes.clone() }
    }

    fn clone_from(&mut self, source: &Self) {
        // println!("cloning from ...");
        assert!(self.routes.len() == source.routes.len());

        for (my_route, source_route) in self.routes.iter_mut().zip(source.routes.iter()) {
            let Route { instance, id, stops, cost, used_cap } = my_route;
            // println!("stops is {:?} w/ cap {:?}, source is {:?} w/ cap {:?}", stops, stops.capacity(), source_route.stops, source_route.stops.capacity());
            assert!(stops.capacity() == source_route.stops.capacity());

            *instance = source_route.instance.clone();
            *id = source_route.id;
            *cost = source_route.cost;
            *used_cap = source_route.used_cap;

            // println!("old stops is {:?}", stops);

            // copy over stops to use exisiting allocation
            // SAFETY: both vectors have the same capacity, which much be less than the source vec's length.
            //         that means we can safely copy that many elements into the destination.
            unsafe { 
                std::ptr::copy(source_route.stops.as_ptr(), stops.as_mut_ptr(), source_route.stops.len()); 
                stops.set_len(source_route.stops.len());
            }

            // println!("new stops is {:?}", stops);

            // todo!();
        }
    }
}


impl std::fmt::Debug for VRPSolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in self.routes.iter() {
            f.write_fmt(format_args!("{:?}\n", r)).unwrap();
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



// #[derive(Clone)]
#[repr(C)]
pub struct Route {
    used_cap: usize,
    pub instance: std::sync::Arc<VRPInstance>,
    id: usize,
    stops: Vec<Stop>,
    cost: f64
}

impl Clone for Route {
    fn clone(&self) -> Self {
        let mut new_stops = Vec::with_capacity(self.stops.capacity());
        new_stops.extend(self.stops.clone());
        Self { instance: self.instance.clone(), id: self.id, stops: new_stops, cost: self.cost, used_cap: self.used_cap }
    }
}

impl std::fmt::Debug for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}, cap: {}", self.to_string(), self.used_capacity()))
    }
}

impl Route {
    pub fn retain_stops(&mut self, f: impl Fn(&Stop) -> bool) {
        self.assert_sanity();

        self.stops.retain(f);

        self.cost = self.recalculate_cost();
        self.used_cap = self.recalculate_capacity();

        self.assert_sanity();
    }

    pub fn to_string(&self) -> String {
        let middle = self.stops.iter().map(|i| format!("{:?}", i)).collect::<Vec<String>>();
        let middle = middle.join(" -> ");
        format!("r{}[{middle}--c{}]", self.id, self.used_cap)
    }
    pub fn new(instance: Arc<VRPInstance>, id: usize) -> Self {
        Route { stops: Vec::with_capacity(instance.max_route_len), instance, cost: 0f64, used_cap: 0, id }
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
        debug_assert!(index <= self.stops.len());

        
        let c = self.instance.vehicle_capacity;
        let e = stop.capacity;

        let f = self.used_cap; // TODO: why the hell is this so slow...

        let mut new_cost = self.cost; // TODO: could change to be relative

        let before = if index != 0 {
            // SAFETY: exactly same as for in self.cost_at_index 
            //         (should probably reuse code eventually...)
            unsafe { self.stops.get_unchecked(index - 1).cust_no }
        } else { 0 };

        let after = if index != self.stops.len() {
            // SAFETY: see above ^^
            unsafe { self.stops.get_unchecked(index).cust_no }
        } else { 0 };

        new_cost -= self.instance.distance_matrix.dist(before, after);
        new_cost += self.instance.distance_matrix.dist(before, stop.cust_no);
        new_cost += self.instance.distance_matrix.dist(stop.cust_no, after);


        let a = new_cost;
        // let jj = std::hint::black_box((self.cost, self.used_cap));
        
        let b = e + f <= c;
        let res = (a, b);

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
        debug_assert!(index <= self.stops.len());

        let start = if index != 0 {
            // SAFETY: since index is a usize and it cannot be 0, index - 1 cannot be OOB below.
            //         we also have to trust that we aren't passing in an index > (self.stops.len() + 1),
            //         which we would have noticed by now if we did!!
            unsafe { self.stops.get_unchecked(index - 1).cust_no }
        } else { 0 };

        let end = if index != self.stops.len() {
            // SAFETY: we have to trust here that index isn't > self.stops.len(), but we have debug
            //         asserts for that so i'm confident the logic elsewhere accounts for that...
            unsafe { self.stops.get_unchecked(index).cust_no }
        } else { 0 };

        self.instance.distance_matrix.dist(start, end)
    }

    // *********** SANITY CHECKING ***********

    #[cfg(debug_assertions)]
    pub fn assert_sanity(&self) {
        // println!("** trying to assert sanity on {:?}", self);

        self.check_route_cost();
        self.check_capacity();
        self.check_no_duplicate_stops();

        // println!("** asserted sanity on {:?}", self);
    }

    #[cfg(not(debug_assertions))]
    pub fn assert_sanity(&self) { 
        // don't do any sanity checking in release mode
    }
    
    fn check_route_cost(&self) {
        assert!((self.recalculate_cost() - self.cost).abs() < 0.5f64);
    }

    fn recalculate_cost(&self) -> f64 {
        let mut cost = 0f64;

        for i in 1..self.stops.len() {
            cost += self.instance.distance_matrix.dist(self.stops[i - 1].cust_no, self.stops[i].cust_no);
        }

        if self.stops.len() > 0 {
            cost += self.instance.distance_matrix.dist(0, self.stops[0].cust_no);
            cost += self.instance.distance_matrix.dist(self.stops[self.stops.len() - 1].cust_no, 0);
        }
        cost
    }

    fn check_capacity(&self) {
        assert!(self.recalculate_capacity() == self.used_cap);
    }

    fn recalculate_capacity(&self) -> usize {
        self.stops.iter().map(|s|{ s.capacity }).sum()
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