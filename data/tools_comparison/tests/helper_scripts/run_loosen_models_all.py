import argparse
import sys
from pathlib import Path
from biodivine_aeon import BooleanNetwork


def loosen_model(psbn_in_path: Path, psbn_out_path: Path,
                 num_fn_symbols: int=5, max_arity: int=4):
    """
    Take the fully specified BN, and make `num_fn_symbols` update functions
    unspecified (same as replacing it with uninterpreted fn symbols). Only 
    process functions with arity up to `max_arity`. Dump resulting PSBN model  
    into `psbn_out_path`.
    """
    bn = BooleanNetwork.from_file(str(psbn_in_path), repair_graph=True)

    num_fn_symbols_added = 0
    for bn_var in bn.variables():
        # Make the function unspecified
        if num_fn_symbols_added < num_fn_symbols and len(bn.predecessors(bn_var)) < max_arity:
            bn.set_update_function(bn_var, None)
            num_fn_symbols_added += 1
        # Or make sure the function expressions are simplified, e.g., (A | !B) instead of (A | (!A & !B))
        # since BoNesis seems to struggle with these
        bn.get_update_function(bn_var)

    # Rewrite the original model with new loosened BN
    updated_bn_str = bn.to_aeon()
    with open(psbn_out_path, 'w') as file:
        file.write(updated_bn_str)


def loosen_all_models(in_dir: str, out_dir: str, num_fn_symbols: int=5, max_arity: int=4):
    """Go over all dir models and apply `loosen_model` function on them."""
    in_dir_path = Path(in_dir)
    out_dir_path = Path(out_dir)
    if not in_dir_path.exists():
        print(f"Error: Directory {in_dir} does not exist")
        sys.exit(1)
    if not out_dir_path.exists():
        print(f"Error: Directory {out_dir} does not exist")
        sys.exit(1)

    # Find all .aeon files
    aeon_in_files = sorted(in_dir_path.glob("*.aeon"))
    if not aeon_in_files:
        print(f"No .aeon files found in {in_dir}")
        sys.exit(1)
    print(f"Found {len(aeon_in_files)} models\n")
    
    for idx, aeon_in_file in enumerate(aeon_in_files, 1):
        print(f"[{idx}/{len(aeon_in_files)}] Processing {aeon_in_file.name}")
        out_file = out_dir_path / Path(aeon_in_file.name)
        loosen_model(aeon_in_file, out_file, num_fn_symbols, max_arity)

        
if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("in_dir", help="input dir with BN models")
    parser.add_argument("out_dir", help="output directory for PSBN models")
    args = parser.parse_args()
    loosen_all_models(args.in_dir, args.out_dir)
