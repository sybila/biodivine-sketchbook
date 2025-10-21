import argparse
import csv
import bonesis as bo
import bonesis.aeon
import biodivine_aeon as aeon
import time


def load_data(csv_path):
    """Load observation data from a CSV file into a dictionary format for BoNesis."""
    data = {}
    with open(csv_path, newline='') as csvfile:
        reader = csv.DictReader(csvfile)
        for row in reader:
            obs_id = row["ID"]
            # Convert all other entries to integers (0/1)
            data[obs_id] = {k: int(v) for k, v in row.items() if k != "ID"}
    return data


def main():
    # Parse CLI arguments

    init_time = time.time()

    parser = argparse.ArgumentParser(
        description="Run BoNesis inference using an AEON Boolean network and a CSV dataset."
    )
    parser.add_argument("aeon_path", type=str, help="Path to the PSBN file in AEON format")
    parser.add_argument("csv_path", type=str, help="Path to the observation data in CSV format.")
    parser.add_argument("--limit", type=int, default=1_000,help="Maximum number of BNs to generate (default: 1000).",)
    args = parser.parse_args()

    # Load the model and observation data
    bn = aeon.BooleanNetwork.from_file(args.aeon_path)
    dom = bonesis.aeon.AEONDomain(bn)
    print(f"Loaded PSBN from {args.aeon_path}")
    data = load_data(args.csv_path)
    print(f"Loaded {len(data)} observations from {args.csv_path}")

    # Create and configure the solver with fixed-point observations
    solver = bo.BoNesis(dom, data)
    all_fixed_points = set()
    for obs_id in data:
        solver.fixed(solver.obs(obs_id))
        all_fixed_points.add(solver.obs(obs_id))
    solver.all_fixpoints(all_fixed_points)
    print(f"Time to process inputs: {(time.time() - init_time) * 1e3:.0f}ms")

    # Run the inference
    count_candidates = solver.boolean_networks(limit=args.limit).count()
    print(f"There are {count_candidates} candidate networks (max limit was set to {args.limit}).")
    print(f"Total time elapsed: {(time.time() - init_time) * 1e3:.0f}ms")

if __name__ == "__main__":
    main()
