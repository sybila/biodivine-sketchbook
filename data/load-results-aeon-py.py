from biodivine_aeon import BooleanNetwork, Bdd, SymbolicContext, ColorSet
from pathlib import Path
import zipfile

# Path to the zip archive with results
zip_file_path = Path("real_cases/tlgl/results.zip")

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

# check the color set (prints also the number of sat interpretations)
print("Loaded ColorSet instance:", color_set, "\n")

# print summary of admissible variants for each update function
for var in bn_model.variables():
    var_name = bn_model.get_variable_name(var)
    update_fn = bn_model.get_update_function(var)
    params = update_fn.support_parameters()
    color_iter = color_set.items(list(params))

    print(f"==== {var_name} ====")
    print(f"original PSBN update: {update_fn}")
    print(f"n. of valid candidate updates: {len(list(color_iter))}")

    for color in color_set.items(list(params)):
        fn_instance = color.instantiate(bn_model.get_update_function(var))
        print(f"> {fn_instance}")
