import argparse
import subprocess
import sys
from pathlib import Path

def run_fps_for_all_models(models_dir: str):
    """Run get_fps_data.py for all .aeon files in models_dir."""
    models_path = Path(models_dir)
    if not models_path.exists():
        print(f"Error: Directory {models_dir} does not exist")
        sys.exit(1)
    
    # Find all .aeon files
    aeon_files = sorted(models_path.glob("*.aeon"))
    if not aeon_files:
        print(f"No .aeon files found in {models_dir}")
        sys.exit(1)
    print(f"Found {len(aeon_files)} models\n")
    
    for idx, aeon_file in enumerate(aeon_files, 1):
        csv_out = aeon_file.with_name(aeon_file.stem + ".csv")
        print(f"[{idx}/{len(aeon_files)}] Processing {aeon_file.name}")
        print(f"  Output: {csv_out.name}")
        
        # Run get_fps_data.py, computing the model's fixed points
        cmd = [sys.executable, "get_fps_data.py", str(aeon_file), str(csv_out)]
        subprocess.run(cmd, capture_output=True, text=True, timeout=300)

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("models_dir", help="Directory containing .aeon model files")
    args = parser.parse_args()
    run_fps_for_all_models(args.models_dir)