import subprocess
import sys
from pathlib import Path


def run_comparisons_for_all_models(models_dir: str, universal_fps: bool = False):
    """Run compare_sketchbook_bonesis.py for all matching zip-aeon-csv triplets."""
    models_path = Path(models_dir)
    if not models_path.exists():
        print(f"Error: Directory {models_dir} does not exist")
        sys.exit(1)
    
    # Find all .zip files (Sketchbook results)
    zip_files = sorted(models_path.glob("*.zip"))
    if not zip_files:
        print(f"No .zip files found in {models_dir}")
        sys.exit(1)
    print(f"Found {len(zip_files)} Sketchbook result files\n")
    
    # Now run the comparison script for all models in the dir
    results = []
    for idx, zip_file in enumerate(zip_files, 1):
        # Derive corresponding .aeon and .csv filenames
        base_name = zip_file.stem
        aeon_file = zip_file.parent / f"{base_name}.aeon"
        csv_file = zip_file.parent / f"{base_name}.csv"

        if not aeon_file.exists():
            print(f"[{idx}/{len(zip_files)}] Skipping {zip_file.name} (no matching .aeon)")
            continue
        if not csv_file.exists():
            print(f"[{idx}/{len(zip_files)}] Skipping {zip_file.name} (no matching .csv)")
            continue
        
        print(f"[{idx}/{len(zip_files)}] Comparing {base_name}")
        print(f"  Sketchbook: {zip_file.name}")
        print(f"  Model: {aeon_file.name}")
        print(f"  Data: {csv_file.name}")
        
        # Prepare and execute the comparison script
        cmd = [
            sys.executable,
            "compare_sketchbook_bonesis.py",
            str(zip_file),
            str(aeon_file),
            str(csv_file)
        ]
        if universal_fps:
            cmd.append("--universal_fps")
        
        try:
            # Timeout is set to 600 since we only use this for simple models
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
            
            if result.returncode == 0:
                print(f"  -> Success")
                # Extract summary from output
                if "Results match exactly!" in result.stdout:
                    results.append((base_name, "MATCH"))
                elif "Results differ!" in result.stdout:
                    results.append((base_name, "DIFFER"))
                else:
                    results.append((base_name, "UNKNOWN"))
                # Print the output for logging or debugging
                lines = result.stdout.split('\n')
                for line in lines:
                    if line.strip():
                        print(f"    {line}")
            else:
                print(f"  X Failed (exit code {result.returncode})")
                results.append((base_name, "ERROR"))
                if result.stderr:
                    print(f"    stderr: {result.stderr}")
        except subprocess.TimeoutExpired:
            print(f"  X Timeout (600s)")
            results.append((base_name, "TIMEOUT"))
        except Exception as e:
            print(f"  X Error: {e}")
            results.append((base_name, "EXCEPTION"))
        
        print()
    
    # Print the summary of all runs
    print("\n" + "="*80)
    print("SUMMARY")
    print("="*80)
    
    status_counts = {}
    for model_name, status in results:
        status_counts[status] = status_counts.get(status, 0) + 1
        print(f"{model_name:60} {status}")
    
    print("\n" + "-"*80)
    for status, count in sorted(status_counts.items()):
        print(f"{status:15} {count}")


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("models_dir", help="Directory containing .zip, .aeon, and .csv files")
    parser.add_argument("--universal_fps", help="Enforce that there can't be additional fixed points",
                        action="store_true")
    args = parser.parse_args()

    run_comparisons_for_all_models(args.models_dir, args.universal_fps)