import subprocess
import sys
from pathlib import Path

# All model paths to run inference on
MODEL_PATHS = [
    "test_data/test_sketch_1.aeon",
    "test_data/test_sketch_1.json",
    "test_data/test_sketch_2.aeon",
    "test_data/test_sketch_2.json",
    "real_cases/arabidopsis/arabidopsis_sketch.aeon",
    "real_cases/arabidopsis/arabidopsis_sketch_hctl.json",
    "real_cases/arabidopsis/arabidopsis_sketch.json",
    "real_cases/arabidopsis/arabidopsis_with_additional_prop.json",
    "real_cases/tlgl/tlgl_sketch.aeon",
    "real_cases/tlgl/tlgl_sketch.json",
    "real_cases/tlgl/tlgl_sketch_hctl.json",
    "small_example/small_example_sketch.json",
    "small_example/small_example_sketch.aeon",
    "small_example/previous_version/small_example.aeon",
    "small_example/previous_version/small_example.json",
    "benchmarks/celldivb/celldivb_sketch.aeon", 
    "benchmarks/eprotein/eprotein_sketch.aeon", 
    "benchmarks/nsp4/nsp4_sketch.aeon", 
    "benchmarks/etc/etc_sketch.aeon", 
    "benchmarks/interferon/interferon_sketch.aeon", 
    "benchmarks/nsp9/nsp9_sketch.aeon", 
    "benchmarks/macrophage/macrophage_sketch.aeon",
    "other_sketches/fgf_signalling/fgf_sketch.json",
    "other_sketches/myeloid/myeloid-sketch-no-updates.json",
    "other_sketches/oscillatory_models/bovine_estrous/bovine-estrous_model_final_cutdown.json",
    "other_sketches/oscillatory_models/gyn_cycle/FSH_submodel/FSH_submodel.json",
    "other_sketches/oscillatory_models/gyn_cycle/GnRH_submodel/GnRH_submodel.json",
    "other_sketches/oscillatory_models/gyn_cycle/LH_submodel/LH_submodel.json",
    "other_sketches/oscillatory_models/gyn_cycle/reduced_submodel/gyn-cycle_model_red_ts_properties_31k.json",
    "other_sketches/oscillatory_models/predator_prey/predator-prey_model_final.json",
]

# Step 1: Compile Rust binaries
SOURCE_DIR = Path("../src-tauri")
print(">>>>>>>>>> COMPILE RUST BINARIES", flush=True)
try:
    subprocess.run(["cargo", "build", "--release", "--bin", "run-inference"], cwd=SOURCE_DIR, check=True)
    print("Compilation completed successfully.\n", flush=True)
except subprocess.CalledProcessError as e:
    print(f"Error during Rust compilation: {e}", file=sys.stderr, flush=True)
    sys.exit(1)

# Step 2: Run inference for each model and collect failures
print(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>", flush=True)
print(">>>>>>>>>> START INFERENCE RUN", flush=True)
print(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n", flush=True)

failures = []
for model_path in MODEL_PATHS:
    print("==========================", flush=True)
    print(f"Model {model_path}", flush=True)
    print("==========================", flush=True)
    
    model_path = Path(model_path)
    model_format = model_path.suffix[1:]  # Get the file extension without the dot
    
    if not model_path.exists():
        print(f"File not found: {model_path}", file=sys.stderr, flush=True)
        continue
    
    try:
        subprocess.run([str(SOURCE_DIR / "target/release/run-inference.exe"), str(model_path), "--input-format", model_format], check=True)
    except FileNotFoundError:
        print(f"Executable not found: {SOURCE_DIR / 'target/release/run-inference.exe'}", file=sys.stderr, flush=True)
    except subprocess.CalledProcessError as e:
        failures.append(model_path)
        print(f"Error running inference for {model_path}: {e}", file=sys.stderr, flush=True)

# Step 3: Print summary of failures or success
if failures:
    print("\n\n==========================", flush=True)
    print("INFERENCE FAILED FOR THE FOLLOWING MODELS:", flush=True)
    for failure in failures:
        print(failure, flush=True)
    print("==========================\n", flush=True)
else:
    print("\n\n==========================", flush=True)
    print("ALL COMPUTATIONS COMPLETED SUCCESSFULLY", flush=True)
    print("==========================\n", flush=True)