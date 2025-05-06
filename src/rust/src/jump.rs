use std::{iter::Inspect, sync::Arc};

use rand::seq::SliceRandom;

use crate::{common::{Stop, VRPSolution}, dbg_println, vrp_instance::{self, VRPInstance}};



// type Jumper = fn(&Arc<VRPInstance>, VRPSolution) -> VRPSolution;

pub fn random_drop(vrp_instance: &Arc<VRPInstance>, mut existing: VRPSolution) -> VRPSolution {
    // println!("existing solution (max cap {:?}) is {:?}", vrp_instance.vehicle_capacity, existing);

    let frac_dropped: f64 = 0.6;
    dbg_println!("JUMPING (*random drop technique* dropping {:?}%)", frac_dropped * 100f64);
    // println!("--have existing {:?} w/ cost {:?} now", existing, existing.cost());
    let rng = &mut rand::rng();

    let to_remove = (vrp_instance.num_customers as f64 * frac_dropped) as usize;
    // todo!("to remove {:?}", to_remove);

    let mut removed_cust_nos = (1..vrp_instance.num_customers as u16).collect::<Vec<_>>();
    removed_cust_nos.shuffle(rng);
    removed_cust_nos.truncate(to_remove);

    // println!("dropping {:?} here...", removed_cust_nos);

    existing.routes.iter_mut().for_each(|r|{
        r.retain_stops(|s|{
            !removed_cust_nos.contains(&s.cust_no())
        });
    });

    let mut to_add = removed_cust_nos.iter().map(|cust_no|{
        Stop::new(*cust_no, vrp_instance.demand_of_customer[*cust_no as usize])
    }).collect::<Vec<_>>();
    to_add.sort_by_key(|t| std::cmp::Reverse(t.capacity()));
    existing.routes.shuffle(rng);

    // println!("have to add back {:?}", to_add);

    for s in to_add {
        // println!("adding stop {:?} back", s);
        let mut was_added = false;
        for r in &mut existing.routes {
            if r.used_capacity() + s.capacity() <= vrp_instance.vehicle_capacity {
                // println!("can add to {:?}", r);
                let index = r.speculative_add_best(&s).1;
                r.add_stop_to_index(s, index);
                was_added = true;
                r.assert_sanity();
                break;
            }
        }
        assert!(was_added);
    }

    // println!("--have existing {:?} w/ cost {:?} now", existing, existing.cost());


    // todo!("would ret existing {:?} here", existing);
    existing
}