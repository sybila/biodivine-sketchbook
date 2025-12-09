import argparse
import bonesis.aeon
import os
import csv
import zipfile

from biodivine_aeon import BooleanNetwork, Bdd, SymbolicContext, ColorSet, BddVariableSet
from mpbn import MPBooleanNetwork
from pathlib import Path


class CanonicalBN:
    """Simple canonical representation of a Boolean network using BDDs.
    
    Args:
        update_functions: Mapping of variable name -> update fn expression
        bdd_vars: A set of BDD variables, the names must correspond to the network
            variables and must be the same for all compared BNs.

    Params:
        bdds: Mapping of variable name -> update fn BDD
    """
    def __init__(self, update_functions: dict[str, str], bdd_vars: BddVariableSet):
        self.bdds: dict[str, Bdd] = {}
        for var, func_str in sorted(update_functions.items()):
            self.bdds[var] = bdd_vars.eval_expression(func_str)
        
    def __hash__(self):
        """Hash based on canonical function representation."""
        # Convert BDDs to their string representation for hashing
        bdd_strings = tuple(sorted((var, str(bdd)) for var, bdd in self.bdds.items()))
        return hash(bdd_strings)
    
    def __eq__(self, other):
        """Semantic equality: same variables and BDD-equivalent update functions."""
        if not isinstance(other, CanonicalBN):
            return False
        if self.bdds.keys() != self.bdds.keys():
            return False
        for var in self.bdds.keys():
            if self.bdds[var] != other.bdds[var]:
                return False
        return True
    
    def __repr__(self):
        return f"CanonicalBN({list(self.bdds.keys())})"
    

def parse_observations_csv(csv_path):
    """Parse observations from a specification CSV into a dict.

    Expected CSV format:
      ID, v_1, v_2, v_3, ...
      observation_1, 1, 0, 1, ...
      observation_2, 0, 0, 0, ...
      ...

    Empty cells are treated as unspecified (variable omitted from that fixed-point dict).
    Returns: {observation: {var_name: int(value), ...}, ...}
    """
    if not os.path.exists(csv_path):
        raise FileNotFoundError(f"Specification CSV not found: {csv_path}")

    spec = {}
    with open(csv_path, newline='') as f:
        reader = csv.reader(f)
        try:
            header = next(reader)
        except:
            raise ValueError(f"Specification file is empty.")

        # header[0] expected to be the ID column
        vars = [h.strip() for h in header[1:]]
        if len(vars) == 0:
            raise ValueError(f"List of variables is empty.")
        for row in reader:
            # skip empty or whitespace-only rows (no constraints)
            if not row or all(not c.strip() for c in row):
                continue

            obs_id = row[0].strip()
            values = {}
            for var, cell in zip(vars, row[1:]):
                cell = cell.strip()
                if cell == "":
                    continue  # unspecified
                elif cell == "1":
                    values[var] = True
                elif cell == "0":
                    values[var] = False
                else:
                    # Throw error on any other than binary value
                    raise ValueError(f"Unsupported specification value `{cell}`.")
            spec[obs_id] = values
    if len(spec) == 0:
        raise ValueError(f"Specification file has no observations.")
    return spec


def main(sketchbook_zip_path: str, bonesis_psbn_path: str, bonesis_data_path: str,
         universal_fps: bool):
    # ============ Load Sketchbook results from a zip ============
    zip_file_path = Path(sketchbook_zip_path)
    bn_filename = "derived_model.aeon"
    bdd_filename = "color_bdd.bdd"

    print(f"Processing Sketchbook results...")
    with zipfile.ZipFile(zip_file_path, 'r') as archive:
        with archive.open(bn_filename) as bn_file:
            sketchbook_model = BooleanNetwork.from_aeon(bn_file.read().decode('utf-8'))
        with archive.open(bdd_filename) as bdd_file:
            bdd_content = bdd_file.read().decode('utf-8')

    context = SymbolicContext(sketchbook_model)
    bdd_variables = BddVariableSet(context.network_variable_names())
    loaded_bdd = Bdd(context.bdd_variable_set(), bdd_content)
    color_set = ColorSet(context, loaded_bdd)
    print(f"Loaded raw Sketchbook results from zip: {color_set}")

    # Extract all Sketchbook candidate networks as BooleanNetwork objects
    sketchbook_networks: list[BooleanNetwork] = []
    for color in color_set.items():
        # color.instantiate() gives the full concrete network
        network = color.instantiate(sketchbook_model)
        sketchbook_networks.append(network)
    print(f"Extracted {len(sketchbook_networks)} Sketchbook candidate networks.\n")

    # ================================================
    # ============= Load Bonesis results =============
    print(f"Computing BoNesis results...")
    # load the partially specified model, and prepare observations
    psbn = BooleanNetwork.from_file(bonesis_psbn_path)
    dom = bonesis.aeon.AEONDomain(psbn)
    data = parse_observations_csv(bonesis_data_path)

    # create the solver and add all fixed-point constraints
    bo_solver = bonesis.BoNesis(dom, data)
    for obs_id in data.keys():
        bo_solver.fixed(~bo_solver.obs(obs_id))
    if universal_fps:
        # enforce that no additional fixed points are possible
        bo_solver.all_fixpoints({bo_solver.obs(obs) for obs in data.keys()});

    # run inference and collect the networks
    bonesis_networks: list[MPBooleanNetwork] = list(bo_solver.boolean_networks(limit=1000000))
    print(f"Extracted {len(bonesis_networks)} BoNesis candidate networks.\n")

    # ===============================================
    # =============== Compare results ===============
    def get_sketchbook_expression_map(bn: BooleanNetwork):
        """Create `variable-> update_expression` mapping from a `BooleanNetwork`."""
        dictt = {variable: bn.get_update_function(variable).as_expression() for variable in bn.variable_names()}
        return dictt

    def get_bonesis_expression_map(bn: MPBooleanNetwork):
        """Create `variable -> update_expression` mapping from a `MPBooleanNetwork`."""
        dictt = {line.split(',')[0].strip(): line.split(',')[1].strip() for line in bn.source().strip().split('\n')}
        for (key, val) in dictt.items():
            if val == "1":
                dictt[key] = "true"
            if val == "0":
                dictt[key] = "false"
        return dictt

    print(f"Comparing the results...")
    sketchbook_canonic_bns = {CanonicalBN(get_sketchbook_expression_map(net), bdd_variables) for net in sketchbook_networks}
    bonesis_canonic_bns = {CanonicalBN(get_bonesis_expression_map(net), bdd_variables) for net in bonesis_networks}

    only_in_sketchbook = sketchbook_canonic_bns - bonesis_canonic_bns
    only_in_bonesis = bonesis_canonic_bns - sketchbook_canonic_bns
    in_both = sketchbook_canonic_bns & bonesis_canonic_bns

    print(f"Networks in both: {len(in_both)}")
    print(f"Networks only in Sketchbook: {len(only_in_sketchbook)}")
    print(f"Networks only in Bonesis: {len(only_in_bonesis)}\n")

    if len(only_in_sketchbook) == 0 and len(only_in_bonesis) == 0:
        print("Results match exactly!")
    else:
        print("Results differ!")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("sketchbook_zip", help="zip archive with Sketchbook results")
    parser.add_argument("psbn_aeon", help="partially specified BN for BoNesis inference")
    parser.add_argument("data_csv", help="observation data (fixed points) for BoNesis inference")
    parser.add_argument("--universal_fps", help="enforce that there cant be additional fixed points",
                        action="store_true")
    args = parser.parse_args()

    main(args.sketchbook_zip, args.psbn_aeon, args.data_csv, args.universal_fps)
