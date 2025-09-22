class VRPInstance:
    def __init__(self, file_name):
        try:
            with open(file_name, 'r') as file:
                lines = file.readlines()
                
                # Parse the first line for number of customers, vehicles, and capacity
                first_line = lines[0].strip().split()
                self.num_customers = int(first_line[0])
                self.num_vehicles = int(first_line[1])
                self.vehicle_capacity = int(first_line[2])
                
                # Initialize arrays for customer data
                self.demand_of_customer = [0] * self.num_customers
                self.x_coord_of_customer = [0.0] * self.num_customers
                self.y_coord_of_customer = [0.0] * self.num_customers
                
                # Parse customer data
                for i in range(self.num_customers):
                    if i + 1 < len(lines):
                        customer_data = lines[i + 1].strip().split()
                        self.demand_of_customer[i] = int(customer_data[0])
                        self.x_coord_of_customer[i] = float(customer_data[1])
                        self.y_coord_of_customer[i] = float(customer_data[2])
                
                # Print customer data
                for i in range(self.num_customers):
                    print(f"{self.demand_of_customer[i]} {self.x_coord_of_customer[i]} {self.y_coord_of_customer[i]}")
                    
        except FileNotFoundError:
            print(f"Error: in VRPInstance() {file_name}\nFile not found")
            exit(-1)
        except Exception as e:
            print(f"Error: in VRPInstance() {file_name}\n{str(e)}")
            exit(-1)

    def to_string(self):
        print(f"Number of customers: {self.num_customers}")
        print(f"Number of vehicles: {self.num_vehicles}")
        print(f"Vehicle capacity: {self.vehicle_capacity}")
