#!/usr/bin/env python3

import sys
import os
import json
from solver_ import VRPSolver
from timer_ import Timer
from vrp_instance import VRPInstance

def main():
    """
    Main entry point for the VRP solver.
    """
    if len(sys.argv) < 2:
        print("Usage: python main.py <file>")
        return
    
    input_file = sys.argv[1]
    filename = os.path.basename(input_file)
    print(f"Instance: {input_file}")
    
    watch = Timer()
    watch.start()
    instance = VRPInstance(input_file)
    solver = VRPSolver(prob_inst=instance)
    sol_string = solver.solve()
    print(sol_string)
    watch.stop()

    with open("./vrp.sol", "w+") as f:
        f.write(sol_string)
    
    result = {
        "Instance": filename,
        "Time": f"{watch.get_time():.2f}",
        "Result": "--",
        "Solution": "--"
    }
    
    print(json.dumps(result))

if __name__ == "__main__":
    main()
