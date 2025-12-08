# Comprehensive illustration of the pipeline to start with a collection 
# of BBM models, prepare Python virtual environment, compute fixed-point
# data, loosen models into PSBNs by adding function symbols, convert
# the logical functions to DNF, and then compare Sketchbook vs Bonesis
# inference results.

# This script is not really meant to be run as a stand-alone from cli,
# it just illustrates the whole process and individual scripts to use.

# First, set the in/out paths to the model sets
models_dir="./raw_bbm_models"
out_dir="./final_benchmarks"

# Copy to models to working dir (will contain the output)
mkdir ${out_dir}
cp ${models_dir}/* ${out_dir}/ -r

# Prepare and start the venv, installing all the libraries
# Note that some libraries are only needed for the preprocessing,
# not the actual comparison at the end
python3 -m venv venv-ubuntu
source venv-ubuntu/bin/activate
pip install pip --upgrade
pip install biodivine_aeon
pip install bonesis
pip install --force-reinstall git+https://github.com/hklarner/pyboolnet
pip install pyeda
pip install sympy

# Compute fixed points of the fully specified models to get target
# dynamic specification
python ./helper_scripts/run_compute_fps_all.py "${out_dir}"

# Loosen the models by adding function symbols, making PSBNs for inference
python ./helper_scripts/run_loosen_models_all.py "${out_dir}" "${out_dir}"

# Simplify the update function expressions by converting them to DNF
# This is needed since Bonesis api cannot deal with general logical functions
python ./helper_scripts/run_simplify_all.py "${out_dir}"

# Run Sketchbook fixed-point inference (using pre-compiled binary)
# See the main repo readme on how to compile Rust binaries
python ./helper_scripts/run_fp_inference_all.py "./${out_dir}" "../../../src-tauri/target/release/run-fixed-point-inference.exe" --output-dir "./${out_dir}/"

# Run the comparison with BoNesis 
python ./run_compare_all.py "./${out_dir}"