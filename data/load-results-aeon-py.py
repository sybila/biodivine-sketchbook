from biodivine_aeon import BooleanNetwork, Bdd, SymbolicContext, ColorSet
from pathlib import Path
import zipfile

# Path to the zip archive with results
zip_file_path = Path("results.zip")

# Filenames inside the zip archive
bn_filename = "derived_model.aeon"
bdd_filename = "color_bdd.bdd"

with zipfile.ZipFile(zip_file_path, 'r') as archive:
    with archive.open(bn_filename) as bn_file:
        bn_model = BooleanNetwork.from_aeon(bn_file.read().decode('utf-8'))
    with archive.open(bdd_filename) as bdd_file:
        bdd_content = bdd_file.read().decode('utf-8')

# generate the ColorSet instance using model's context
context = SymbolicContext(bn_model)
loaded_bdd = Bdd(context.bdd_variable_set(), bdd_content)
color_set = ColorSet(context, loaded_bdd)

print("Loaded ColorSet instance:", color_set)
