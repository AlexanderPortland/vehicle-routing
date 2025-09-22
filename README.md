# Vehicle Routing Problem (VRP) Solver

A high-performance Capacitated Vehicle Routing Problem solver implemented in Rust, featuring adaptive [Large Neighborhood Search](https://en.wikipedia.org/wiki/Very_large-scale_neighborhood_search) algorithms and multiple construction heuristics. This was a very fun project, and I'm especially proud of the 155x speedup [my performance engineering](#4-optimizations) produced, so feel free to check that out!

## Table of Contents

- [Overview](#overview)
- [Approach & Implementation](#approach--implementation)
- [Usage](#usage)
- [Development](#development)
- [License](#license)
- [Acknowledgments](#acknowledgments)

## Overview

This project implements a solver for [Capacitated Vehicle Routing Problems](https://en.wikipedia.org/wiki/Vehicle_routing_problem#:~:text=Capacitated%20Vehicle%20Routing%20Problem%3A%20CVRP,goods%20that%must%20be%20delivered.) (CVRP). 
In these kinds of problems, you have a set of customers that each have geographical positions on a cartesian plane. 
Each customer has a certain amount of demand (essentially the number of packages they need to receive), and you have a certain number of trucks that can each carry up to a certain number of packages.
Your job (as the optimizer) is to find routes for all your trucks that will minimize the total distance they have to travel and, by proxy, how much you have to spend on gas.

This problem is NP-hard, meaning there doesn't (yet!) exist an algorithm that can provably solve it optimally in polynomial time.
So our solver uses a combination of clever heuristics and local search strategies to try to get a *very good solution* very quickly, even if it won't be *the best solution*.

See below for an overview of the project's approach and optimizations. If you just want to run the solver, skip to the [usage section](#usage). You can also check out [the presentation](/presentation.pdf) we gave to the class, but there was a 7 minute limit so the slides are fairly terse!

## Approach & Implementation

Our approach is a version of **Large Neighborhood Search (LNS)** that combines different techniques to have more resilience on different kinds of inputs.
We first build up a starting feasible solution, then try to repeatedly destroy and repair it in ways that make sure the new solution is also feasible while also likely being better than the ones we've had before.
A key portion of our design philosophy was to make that iterative process **as fast as possible** (see [our optimizations](#4-performance-tricks)). 
We observed that if our search loop was incredibly smooth, our solver could churn through states and cover lots of the search space in the alloted time, producing better solutions.

### 1. Construction Heuristics

First, we need to build an initial solution that's feasible (i.e., doesn't violate capacity constraints). 
We use a fallback strategy here, starting with greedier algorithms that should produce better results but are likely to fail:

1. **Clarke-Wright Savings**: This is the main algorithm we try first. It's a classic approach that looks at pairs of customers and tries to merge their routes if it saves distance. We add some randomness with normal distribution noise to keep things interesting.
2. **Sweep Algorithm**: If Clarke-Wright fails, we fall back to this. It sorts customers by their angle from the depot and assigns them to trucks in order.
3. **Greedy**: Last resort - just assign customers to the first truck that has capacity. Simple but it works.

### 2. Large Neighborhood Search

Now that we have a starting feasible solution, so we'll start doing a combo of *search* (what's the best local move we can make from here) and *exploration* (how can we keep exploring the space and not get stuck anywhere).

For search, we first destroy part of the solution, removing 5 customers from the route at random. 
We also use a [Tabu list](https://en.wikipedia.org/wiki/Tabu_search) to ensure we don't remove the 10% of customers we've most recently removed or we might keep making the same few moves. 

Then, we take those removed customers and try to re-insert them to the routes in the best possible spots. 
We insert high-demand customers first because they're the hardest to fit into a route without breaking capacity. 
And 2% of the time, we'll put them back in a random (but still valid) spot to ensure we're searching new pieces of the space.

This forms the backbone of our search, allowing us to generate new solutions and see if they're any better than those we already know.

### 3. Exploration Strategy

If the new solution we found through search is better, we'll always take it, but there's a 10% chance we accept a worse solution too. 
This strategy is similar to [Simulated Annealing](https://en.wikipedia.org/wiki/Simulated_annealing) in that it improves our coverage of the search space and keeps us from getting stuck in spots that are only locally optimal.

#### Restarts
We have high expectations on our solver, and don't give it much leeway if it isn't making progress.
After just 50 iterations without an improvement in cost, we assume we're not going anywhere and restart.
80% of the time, we'll restart from the globally best solution, and 20% of the time, we'll restart from the recent best, to ensure we're giving the recent search space a chance and not always taking similar paths from the start.

Instead of just restarting from there, we'll instead take a pretty big 'jump' by removing some customers and putting them back optimally. This ensures that we're restarting *around* an area of the search space we know to be good, but not necessarily in *the exact same spot* where we'll just repeat the same exploration.

### 4. Optimizations

Since this code needs to be fast, we did heavy optimizations, including:

#### Helping the Compiler
- **Avoid bounds checks on distance matrix lookups** when we know they're safe *(~20% faster)*
- **Manually inline** hot distance calculating functions

#### Exploiting Data Structures
- **Remove redundant computations** in our sanity checks *(~60x faster)*
- **Don't filter all possible moves** by whether they're in Tabu, just keep a list of those in and out of Tabu, moving between them *(38% faster)*

#### Avoid Allocations
- **Initialize vectors `with_capacity()`** to avoid resizing allocations *(7% faster)*
- **Reuse existing memory allocations** when copying solutions *(16% faster)*

We also had a **highly-parallel multithreading scheme** which essentially had independent worker thread communicate with an orchestrator to ensure they were covering disparate pieces of the search space.

All together, this 50k of our solver's search iterations go from taking 60s to just 385ms, a 155x speedup! You can check out the solver's binary at different stages of optimization in the [`binaries`](/binaries/) directory, or see more about our profiling methodology [below](#profiling).

### 5. Stopping Criteria

We can run the solver for a certain amount of time or # of iterations, returning the best solution it encountered once time is up. When evaluating for the class, we ran it for 4m 59s as the time limit was 5m.

## Usage

### Single Instance
```bash
cargo run --release -- <path_to_vrp_file>
```

### Using Shell Scripts
The `run.sh` shell script is a remnant of the course's grading infrastructure, but `runAll.sh` is useful if you'd like to run all the instances in a folder with a given time limit.
```bash
# Run single instance
./run.sh input/16_5_1.vrp

# Run all instances in a folder with time limit
./runAll.sh input/ 15 results.log
```

### Input Format

VRP instance files (`*.vrp`) should follow this format. 
The examples we were evaluated on in the class can be found in the [`input`](input/) directory.
```
<num_customers> <num_vehicles> <vehicle_capacity>
<demand_0> <x_coord_0> <y_coord_0>
<demand_1> <x_coord_1> <y_coord_1>
...
<demand_n> <x_coord_n> <y_coord_n>
```

Example (`16_5_1.vrp`):
```
16 5 55
0 30.0 40.0
7 37.0 52.0
30 49.0 49.0
16 52.0 64.0
...
```

### Output Format

The solver outputs results in JSON format:
```json
{
  "Instance": "16_5_1.vrp",
  "Time": 1.23,
  "Result": 245.67,
  "Solution": "0 1 3 5 0 0 2 4 6 0 ..."
}
```

## Development

### Profiling
Building with the `profiling` compilation profile includes debug symbols in the binary for performance analysis:

```bash
# Build for profiling
cargo build --profile profiling
```

We recommend [samply](https://github.com/mstange/samply) as a profiling tool, which provides interactive flamegraph output using Firefox's UI and excellent macOS support compared to [`flamegraph-rs`](https://github.com/flamegraph-rs/). To use samply once installed:

```bash
# Profile the solver
samply record target/profiling/vrp <path_to_vrp_file>
```

### Adding New Algorithms
1. Implement the `LNSSolver` or `IterativeSolver` trait
2. Add construction heuristics to `construct.rs`
3. Add local search operators to appropriate modules
4. Update solver selection in `main.rs`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.