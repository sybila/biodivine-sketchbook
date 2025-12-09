import argparse
import csv
import random

from biodivine_aeon import AsynchronousGraph, BooleanNetwork, FixedPoints, VertexModel


def compute_fixed_point_data(psbn_path: str, csv_out_path: str):
    """Compute fixed points of the given BN, output (max 10) fixed-point states
    to the output path as CSV. If the BN has inputs, add them self-loops and rewrite 
    the model in-place.
    """
    bn = BooleanNetwork.from_file(psbn_path, repair_graph=True)

    # Make sure all inputs are using self-loop to get a single BN instance
    for bn_var in bn.variables():
        if not bn.predecessors(bn_var):
            var_name = bn.get_variable_name(bn_var)
            bn.add_regulation(f"{str(var_name)} -> {str(var_name)}")
            bn.set_update_function(bn_var, str(var_name))

    # Rewrite the original model with bn with fixed inputs
    updated_bn_str = bn.to_aeon()
    with open(psbn_path, 'w') as file:
        file.write(updated_bn_str)

    graph = AsynchronousGraph(bn)
    fixed_points = FixedPoints.symbolic(graph).vertices()
    if fixed_points.is_empty():
        print("NO FIXED POINTS")
        return

    collected_fixed_points = []
    for fp in fixed_points:
        fp_renamed = {bn.get_variable_name(variable): update for variable, update in fp.items()}
        collected_fixed_points.append(fp_renamed)
    
    # If there are more than 10, randomly select 10 fixed points
    if len(collected_fixed_points) > 10:
        collected_fixed_points = random.sample(collected_fixed_points, 10)
    
    export_fps(csv_out_path, collected_fixed_points)


def export_fps(csv_out_path: str, fixed_points: list[VertexModel]):
    """Export fixed-point states into a CSV."""
    with open(csv_out_path, 'w', newline='') as f:
        # Get variable names from first fixed point, sorted alphabetically
        fieldnames = ['ID'] + sorted(fixed_points[0].keys())
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        
        writer.writeheader()
        for i, fp in enumerate(fixed_points, 1):
            row = {'ID': f'Observation{i}'}
            # Convert bool strings to 0/1
            row.update({k: 1 if v else 0 for k, v in fp.items()})
            writer.writerow(row)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("psbn_aeon", help="partially specified BN for BoNesis inference")
    parser.add_argument("data_csv", help="output path for observation data csv")
    args = parser.parse_args()
    compute_fixed_point_data(args.psbn_aeon, args.data_csv)
