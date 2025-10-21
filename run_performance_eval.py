import subprocess
import sys
from pathlib import Path

# Directories and models
SOURCE_DIR = Path("src-tauri")
BENCH_DIR = Path("data/benchmarks/")
MODELS = ["celldivb", "eprotein", "nsp4", "etc", "interferon", "nsp9", "macrophage"]
# MODELS = ["nsp9", "macrophage"]

# Step 1: Compile Rust binaries
print(">>>>>>>>>> COMPILE RUST BINARIES", flush=True)
try:
    subprocess.run(["cargo", "build", "--release", "--bin", "run-inference"], cwd=SOURCE_DIR, check=True)
    print("Compilation completed successfully.\n", flush=True)
except subprocess.CalledProcessError as e:
    print(f"Error during Rust compilation: {e}", file=sys.stderr, flush=True)
    sys.exit(1)

# Step 2: Run benchmarks
print(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>", flush=True)
print(">>>>>>>>>> START BENCHMARKS RUN", flush=True)
print(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>\n", flush=True)

for model in MODELS:
    print("==========================", flush=True)
    print(f"Model {model}", flush=True)
    print("==========================\n", flush=True)
    
    model_dir = BENCH_DIR / model
    aeon_file = model_dir / f"{model}_sketch.aeon"
    # aeon_file = model_dir / f"{model}_sketch_v2.aeon"

    if not aeon_file.exists():
        print(f"File not found: {aeon_file}", file=sys.stderr, flush=True)
        continue
    
    try:
        subprocess.run([str(SOURCE_DIR / "target/release/run-inference"), str(aeon_file), "--input-format", "aeon"], check=True)
    except FileNotFoundError:
        print(f"Executable not found: {SOURCE_DIR / 'target/release/run-inference'}", file=sys.stderr, flush=True)
    except subprocess.CalledProcessError as e:
        print(f"Error running inference for {model}: {e}", file=sys.stderr, flush=True)
