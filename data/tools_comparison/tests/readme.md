Scripts and data for testing that Sketchbook and BoNesis compute exactly the same results.

### Test cases

#### Arabidopsis

We first compare the results on the biological model from the Arabidopsis case study.
All data is present in `arabidopsis` sub-folder.
There is the plain PSBN model, fixed-point dataset, and precomputed Sketchbook results (once without enforcing universal fixed-point property, and the other with this property).

#### BBM testing benchmarks

We have further selected a set of more diverse testing benchmarks. We collected 41 smaller models from the [BBM database](https://bbm.sybila.fi.muni.cz/models) (monotone models with up to 15 variables, so that we can fully enumerate all results with BoNesis and compare exhaustively). We only selected models with monotone regulations since BoNesis currently can not work with non-monotone uninterpreted functions.
To specify dynamics, we chose fixed-point properties (which fall into the subset of properties that BoNesis can process).

All test cases (benchmarks) are provided in `test_bbm_benchmarks`.
For each test case, there is the PSBN (`.aeon` file), fixed-point data (`.csv` file), and precomputed Sketchbook results (`.zip` file). The Sketchbook results can be computed through GUI, or from command line using the `run-fixed-point-inference` binary.

### Installation

We've prepared Python testing scripts to automate the comparison. For successful testing, install the Python libraries below with `pip install` commands (we recommend using a virtual environment, and we used Python 12). For `aeon` and `bonesis`, you can also check the BoNesis installation instructions in the main `instructions-vm.md` of the parent directory.
```
pip install biodivine_aeon
pip install bonesis
pip install --force-reinstall git+https://github.com/hklarner/pyboolnet
pip install pyeda
pip install sympy
```

### Test scripts

The script `compare_sketchbook_bonesis.py` compares that the set of models inferred with Sketchbook exactly matches the set inferred by Bonesis.
It expects precomputed Sketchbook inference results (exported as zip), and it compares it with on-the-fly computed BoNesis inference results. For BoNesis, it expects the partially specified BN (in aeon format) and observation data (the same as used for Sketchbook) on input. You can specify whether you want the observations be the only fixed points of the model (enforcing so-called universal fixed-point property). 

Note that the BoNesis computation can be quite long when there are many solutions, as we need to fully enumerate all of them for the exhaustive comparison.

#### Single test case

You can run the validation on a single test case with the prepared script `compare_sketchbook_bonesis.py` directly. 

For the `arabidopsis` model, execute one of the following (one compares the results with enforced universal fixed-point property, and the other without this property). The second can take longer, enumerating almost 500k models.
```
python .\compare_sketchbook_bonesis.py "arabidopsis/results_universal_fps.zip" "arabidopsis/sketch.aeon" "arabidopsis/dataset.csv" --universal_fps
python .\compare_sketchbook_bonesis.py "arabidopsis/results.zip" "arabidopsis/sketch.aeon" "arabidopsis/dataset.csv"

```

A short summary is printed on standard output. Expected outputs are provided in `comparison_arabidopsis_output.txt`.

Use `python .\compare_sketchbook_bonesis.py -h` for usage help.

#### Automated test benchmark run

You can use `run_compare_all.py` to run the comparison on all models in a directory. It runs the comparisons one-by-one, printing intermediate output, and then summarizes the test run at the end.
The directory must contain all three previously mentioned files (`.aeon`, `.zip`, and `.csv`) for each test case. For the prepared BBM test set, execute:

```
python ./run_compare_all.py test_bbm_benchmarks/
```

Expected output is provided in `comparison_bbm_output.txt`.

### Extending test cases, helper scripts

When adding more benchmarks, be careful since BoNesis struggles with processing more complex update function expressions. BoNesis uses a simple syntax check for monotonicity - positive regulators must not be negated in update function expressions, and the other way around. This means it sometimes fails to directly process standard valid logical expressions used in `.aeon` files and these expressions need to be simplified first. Transformation into some strict DNF usually works.
To do that, you can use `run_simplify_all.py` script in the `helper_scripts` dir.

To add further benchmarks, you can follow the prepared pipeline outlined in the `pipeline.sh`. All the scripts can be found in `helper_scripts` dir. Essentially, the pipeline is as follows:
- Start with a directory of fully specified BN models (with monotone regulations, and preferably small)
- Compute fixed point data for the models (up to 10 states is used)
- Parametrize the models into PSBNs by replacing some updates with function symbols (up to 5 functions is used)
- Simplify the update expressions and transform into DNF (as discussed above)
- Run fixed-point inference with Sketchbook (using the PSBNs and fixed-point data)
- Compare the Sketchbook results with Bonesis (using the same PSBNs and fixed-point data)

