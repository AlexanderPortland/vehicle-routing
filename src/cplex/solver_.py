import numpy as np
import math
from docplex.mp.model import Model
from vrp_instance import VRPInstance

class VRPSolver:
    def __init__(self, prob_inst: VRPInstance) -> None:
        self.p = prob_inst

        self.model = Model()
        self.model.context.cplex_parameters.threads = 1

        # Vehicles | Node I | Node J
        self.edge_var = np.zeros((self.p.num_vehicles, self.p.num_customers, self.p.num_customers), dtype=object)
        for v in range(self.p.num_vehicles):
            for i in range(self.p.num_customers):
                for j in range(self.p.num_customers):
                    if i != j:
                        self.edge_var[v][i][j] = self.model.integer_var(0, 1, f"edge: V: {v}, I: {i}, J: {j}")
                    else:
                        self.edge_var[v][i][j] = self.model.integer_var(0, 0, f"edge: NO-OP {v} {i} {j}")

        self.order_var = np.zeros((self.p.num_vehicles, self.p.num_customers), dtype=object)
        for v in range(self.p.num_vehicles):
            for i in range(self.p.num_customers):
                if i == 0:
                    self.order_var[v][i] = self.model.integer_var(0, 0, f"order: NO-OP: {v}, I: {i}")    
                else:
                    self.order_var[v][i] = self.model.integer_var(1, self.p.num_customers - 1, f"order: V: {v}, I: {i}")

        # flow
        # continuity — if a vehicle visits a node, it must also leave that node
        # demand — each node (besides the depot) must have a flow of 1
        for c in range(self.p.num_customers):
            incoming_flow = self.model.sum(self.edge_var[:, :, c].flatten().tolist())
            outgoing_flow = self.model.sum(self.edge_var[:, c, :].flatten().tolist())

            if c != 0: # non-depot
                self.model.add_constraint(incoming_flow == 1)
                self.model.add_constraint(outgoing_flow == 1)
            else: # depot
                self.model.add_constraint(incoming_flow == outgoing_flow)

            # per vehicle flow
            for v in range(self.p.num_vehicles):
                per_vehicle_incoming_flow = self.model.sum(self.edge_var[v, :, c].flatten().tolist())
                per_vehicle_outgoing_flow = self.model.sum(self.edge_var[v, c, :].flatten().tolist())
                self.model.add_constraint(per_vehicle_incoming_flow == per_vehicle_outgoing_flow)

        # each vehicle can only leave / return to the depot at most once
        for v in range(self.p.num_vehicles):
            self.model.add_constraint(self.model.sum(self.edge_var[v, 0, :].tolist()) <= 1) # type: ignore
            self.model.add_constraint(self.model.sum(self.edge_var[v, :, 0].tolist()) <= 1) # type: ignore
            self.model.add_constraint(self.model.sum(self.edge_var[v, 0, :].tolist()) == self.model.sum(self.edge_var[v, :, 0].tolist()))

        # capacity
        for v in range(self.p.num_vehicles):
            total_demand_served = []
            for c in range(self.p.num_customers):
                total_demand_served.append(self.model.scal_prod(
                    terms=self.edge_var[v, c, :],
                    coefs=self.p.demand_of_customer
                ))
            self.model.add_constraint(self.model.sum(total_demand_served) <= self.p.vehicle_capacity) # type: ignore

        # order — mtz
        for v in range(0, self.p.num_vehicles):
            for i in range(1, self.p.num_customers):
                for j in range(1, self.p.num_customers):
                    self.model.add_constraint(
                        self.order_var[v][i] >= self.order_var[v][j] + 1 - self.p.num_customers * (1 - self.edge_var[v][i][j])
                    )
        
        distance_traveled = self.model.sum(
            self.edge_var[v][i][j] * self.distance(i, j)
            for v in range(self.p.num_vehicles)
            for i in range(self.p.num_customers)
            for j in range(self.p.num_customers)
        )
        
        self.model.minimize(distance_traveled)

    def solve(self):
        sol = self.model.solve()
        if sol:
            vehicle_routes = []
            for v in range(self.p.num_vehicles):
                print(f"======= Vehicle {v+1} =======")
                pretty_print_matrix([[self.edge_var[v, i, j].solution_value for j in range(self.p.num_customers)] for i in range(self.p.num_customers)])
                vehicle_i_route = [str(0)]
                current_node = 0
                first_row_of_edge_matrix = [x.solution_value for x in self.edge_var[v, current_node, :]]
                if 1 not in first_row_of_edge_matrix:
                    vehicle_routes.append(["0", "0"])
                    continue
                
                while True:
                    next_node = [x.solution_value for x in self.edge_var[v, current_node, :]].index(1)        
                    vehicle_i_route.append(str(next_node))
                    if next_node == 0:
                        break
                    current_node = next_node
                    
                vehicle_routes.append(vehicle_i_route)
                
            sol_string = f"{round(self.model.objective_value, 1)} {1}\n"
            for vehicle_route in vehicle_routes:
                sol_string += f"{' '.join(vehicle_route)}\n"
            return sol_string
        else:    
            raise Exception("no solution found")

        
    def distance(self, i: int, j: int) -> float:
        x1, y1 = self.p.x_coord_of_customer[i], self.p.y_coord_of_customer[i]
        x2, y2 = self.p.x_coord_of_customer[j], self.p.y_coord_of_customer[j]
        return math.sqrt((x1 - x2)**2 + (y1 - y2)**2)

def pretty_print_matrix(matrix, sep="  "):
    # Compute maximum width of each column
    num_cols = len(matrix[0])
    col_widths = [
        max(len(str(row[col])) for row in matrix)
        for col in range(num_cols)
    ]
    # Print each row, right-justified per column width
    for row in matrix:
        print(sep.join(str(val).rjust(col_widths[i])
                        for i, val in enumerate(row)))