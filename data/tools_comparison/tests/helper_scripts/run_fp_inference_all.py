import subprocess
import sys
from pathlib import Path

def run_inference_for_all_models(models_dir: str, rust_binary: str, output_dir: str):
    """Run fixed-point-inference Sketchbook binary for all .aeon files in models_dir."""
    models_path = Path(models_dir)
    binary_path = Path(rust_binary)
    output_path = Path(output_dir)
    
    if not models_path.exists():
        print(f"Error: Directory {models_dir} does not exist")
        sys.exit(1)
    
    if not binary_path.exists():
        print(f"Error: Binary {rust_binary} does not exist")
        sys.exit(1)
    
    output_path.mkdir(parents=True, exist_ok=True)
    
    # Find all .aeon files
    aeon_files = sorted(models_path.glob("*.aeon"))
    if not aeon_files:
        print(f"No .aeon files found in {models_dir}")
        sys.exit(1)
    print(f"Found {len(aeon_files)} models\n")
    
    for idx, aeon_file in enumerate(aeon_files, 1):
        # Find corresponding CSV file
        csv_file = aeon_file.with_suffix(".csv")
        if not csv_file.exists():
            print(f"[{idx}/{len(aeon_files)}] Skipping {aeon_file.name} (no CSV found)")
            continue
        
        # Prepare output file name
        output_zip = output_path / aeon_file.with_suffix(".zip").name
        print(f"[{idx}/{len(aeon_files)}] Processing {aeon_file.name}")
        print(f"  CSV: {csv_file.name}")
        print(f"  Output: {output_zip.name}")
        
        # Prepare command and arguments
        cmd = [
            str(binary_path),
            str(aeon_file),
            str(csv_file),
            str(output_zip)
        ]
        
        try:
            # Finally run the Sketchbook inference binary (with 10min time limit)
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
            if result.stdout:
                print(f"  stdout: {result.stdout[:200]}")
            if result.stderr:
                print(f"  stderr: {result.stderr[:200]}")
            if result.returncode == 0:
                print(f"  -> Success")
            else:
                print(f"  X Failed (exit code {result.returncode})")
        except subprocess.TimeoutExpired:
            print(f"  X Timeout (600s)")
        except Exception as e:
            print(f"  X Error: {e}")
        
        print()

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("models_dir", help="Directory containing .aeon and .csv model files")
    parser.add_argument("rust_binary", help="Path to run-fixed-point-inference binary")
    parser.add_argument("--output-dir", default="./inference_results", help="Output directory for results")
    args = parser.parse_args()

    run_inference_for_all_models(args.models_dir, args.rust_binary, args.output_dir)