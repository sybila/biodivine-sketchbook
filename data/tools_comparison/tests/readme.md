Scripts and data for testing that Sketchbook and BoNesis compute the same results.

The benchmarks are selected so that:
- Small enough so we can enumerate all results with BoNesis and compare exhaustively.
- Dynamic properties fall into the subset that BoNesis can process (we chose fixed points and universal fixed points)
- The input PSBNs have monotone functions (BoNesis cant currently work with non-monotone uninterpreted functions)

You'll need to install `bonesis` and `biodivine_aeon` Python libraries. Simply follow the BoNesis installation instructions in the main `instructions-vm.md` of the parent directory.

The script `compare_sketchbook_bonesis.py` expects precomputed Sketchbook results (exported as zip), and it compares it with on-the-fly computed BoNesis results. For BoNesis, it expects the partially specified BN and observation data (same as used for Sketchbook) on input. You can specify if you want the observations be the only universal fixed points. Note that the BoNesis computation can be quite long, as we need to fully enumerate all solutions.
You can run it as:
```
python .\compare_sketchbook_bonesis.py "arabidopsis/results_universal_fps.zip" "arabidopsis/sketch.aeon" "arabidopsis/dataset.csv" --universal_fps
```

Or use `python .\compare_sketchbook_bonesis.py -h` for usage help.

Be careful since BoNesis struggles with processing more complex update function expressions. It uses a simple syntax check for monotonicity - positive regulators must not be negated in update functions, and the other way around. This means it sometimes cannot directly process standard valid logical expressions used in .aeon files, but they need to be simplified first (into equivalent but simpler form).
