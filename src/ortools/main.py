"""Capacited Vehicles Routing Problem (CVRP)."""

import math
from ortools.constraint_solver import routing_enums_pb2
from ortools.constraint_solver import pywrapcp
from vrp_instance import VRPInstance


def create_data_model(filename: str):
    """Stores the data for the problem."""
    data = {}
    vrp = VRPInstance(file_name=filename)

    distance_matrix = []
    for i in range(vrp.num_customers):
        row = []
        for j in range(vrp.num_customers):
            row.append(
                int(
                    math.sqrt(
                        (vrp.x_coord_of_customer[i] - vrp.x_coord_of_customer[j]) ** 2
                        + (vrp.y_coord_of_customer[i] - vrp.y_coord_of_customer[j]) ** 2
                    )
                )
            )
        distance_matrix.append(row)

    data["distance_matrix"] = distance_matrix
    data["demands"] = vrp.demand_of_customer
    data["vehicle_capacities"] = [vrp.vehicle_capacity] * vrp.num_vehicles
    data["num_vehicles"] = vrp.num_vehicles
    data["depot"] = 0
    return data


def print_solution(data, manager, routing, solution): 
    """Prints solution on console."""
    print(f"Objective: {solution.ObjectiveValue()}")
    total_distance = 0
    total_load = 0
    for vehicle_id in range(data["num_vehicles"]):
        if not routing.IsVehicleUsed(solution, vehicle_id):
            continue
        index = routing.Start(vehicle_id)
        plan_output = ""
        route_distance = 0
        route_load = 0
        while not routing.IsEnd(index):
            node_index = manager.IndexToNode(index)
            route_load += data["demands"][node_index]
            plan_output += f" {node_index}"
            previous_index = index
            index = solution.Value(routing.NextVar(index))
            route_distance += routing.GetArcCostForVehicle(
                previous_index, index, vehicle_id
            )
        plan_output += f" {manager.IndexToNode(index)}"
        # plan_output += f"Distance of the route: {route_distance}m\n"
        # plan_output += f"Load of the route: {route_load}\n"
        print(plan_output)
        total_distance += route_distance
        total_load += route_load
    print(total_distance, 0)
    print(f"Total distance of all routes: {total_distance}m")
    print(f"Total load of all routes: {total_load}")

    return total_distance


def main(filename: str):
    """Solve the CVRP problem."""
    # Instantiate the data problem.
    data = create_data_model(f"../../input/{filename}")

    # Create the routing index manager.
    manager = pywrapcp.RoutingIndexManager(
        len(data["distance_matrix"]), data["num_vehicles"], data["depot"]
    )

    # Create Routing Model.
    routing = pywrapcp.RoutingModel(manager)

    # Create and register a transit callback.
    def distance_callback(from_index, to_index):
        """Returns the distance between the two nodes."""
        # Convert from routing variable Index to distance matrix NodeIndex.
        from_node = manager.IndexToNode(from_index)
        to_node = manager.IndexToNode(to_index)
        return data["distance_matrix"][from_node][to_node]

    transit_callback_index = routing.RegisterTransitCallback(distance_callback)

    # Define cost of each arc.
    routing.SetArcCostEvaluatorOfAllVehicles(transit_callback_index)

    # Add Capacity constraint.
    def demand_callback(from_index):
        """Returns the demand of the node."""
        # Convert from routing variable Index to demands NodeIndex.
        from_node = manager.IndexToNode(from_index)
        return data["demands"][from_node]

    demand_callback_index = routing.RegisterUnaryTransitCallback(demand_callback)
    routing.AddDimensionWithVehicleCapacity(
        demand_callback_index,
        0,  # null capacity slack
        data["vehicle_capacities"],  # vehicle maximum capacities
        True,  # start cumul to zero
        "Capacity",
    )

    # Setting first solution heuristic.
    search_parameters = pywrapcp.DefaultRoutingSearchParameters()
    search_parameters.first_solution_strategy = (
        routing_enums_pb2.FirstSolutionStrategy.PATH_CHEAPEST_ARC
    )
    search_parameters.local_search_metaheuristic = (
        routing_enums_pb2.LocalSearchMetaheuristic.GUIDED_LOCAL_SEARCH
    )
    search_parameters.time_limit.FromSeconds(300)

    # Solve the problem.
    solution = routing.SolveWithParameters(search_parameters)

    # Print solution on console.
    print(solution)
    if solution:
        return print_solution(data, manager, routing, solution)


if __name__ == "__main__":
    instances = [
        # "101_11_2.vrp",
        # "101_8_1.vrp",
        # "121_7_1.vrp",
        # "135_7_1.vrp",
        # "151_15_1.vrp",
        # "16_5_1.vrp",
        # "200_16_2.vrp",
        # "21_4_1.vrp",
        # "241_22_1.vrp",
        # "262_25_1.vrp",
        # "30_4_1.vrp",
        "386_47_1.vrp",
        # "41_14_1.vrp",
        # "45_4_1.vrp",
        # "51_5_1.vrp",
        # "76_8_2.vrp",
    ]
    costs = {}
    for i in instances:
        costs[i] = main(i)
        print(costs)

    print(costs)



# 1959, 800, 1047, 1243, 3089, 329, 351, 629, 5973, 494