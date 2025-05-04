use crate::{common::Stop, old_solver::VRPSolution, vrp_instance::VRPInstance};



pub fn greedy<'a>(vrp_instance: &'a VRPInstance) -> Option<VRPSolution<'a>> {
    let mut sol = VRPSolution::new(vrp_instance);
    for customer_idx in 1..vrp_instance.num_customers {
        let demand = vrp_instance.demand_of_customer[customer_idx];
        println!("considering customer {:?}", customer_idx);
        let mut found = false;
        for vehicle_idx in 0..vrp_instance.num_vehicles {
            if (vrp_instance.vehicle_capacity - sol.routes[vehicle_idx].used_capacity())
                >= demand
            {
                println!("adding customer {:?}", customer_idx);
                let len = sol.routes[vehicle_idx].stops().len();
                sol.routes[vehicle_idx].add_stop_to_index(
                    Stop::new(customer_idx.try_into().unwrap(), demand),
                    len,
                );
                found = true;
                break;
            }
        }
        if !found { return None; }
    }
    Some(sol)
}