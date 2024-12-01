A set of benchmarks based on partially specified variants of real models, with synthetic attractor data.
The data was sampled from the attractors of the original model.

Each folder contains information for one of the benchmarks:
- a sketch file used as inference input
- a CSV dataset with sampled attractor data (included in the sketch)
- metadata with link to original repo/publication
- concrete BN that was used for the sampling of synthetic data

To run the benchmarks, execute `python3 run_performance_eval.py` from the main directory.
The expected output is in `output_eval.txt`. 
Below, we briefly summarize the models and results (numbers of candidates after evaluating static/dynamic properties):
| Model    | Variables | After static | After dynamic |
| -------- |------- | ------- | ------- |
| cell_div_b  | 9 | 64000 | 14088 |
| eprotein | 35 | 1179648 | 128 |
| nsp4 | 60 | 1179648 | 128 |
| etc | 84 | 242337096 | 3167262 |
| interferon | 121 | 53378248050663950000000 | 682290 |
| nsp9 | 252 | 3143273693184 | 55145152512 |
| macrophage | 321 | 9973140848784  | 787353224904 |

You can also run the individual benchmarks following these steps (but we recommend using the prepared Python script):
- go to the `src-tauri` folder
- compile the Rust code with `cargo build --release --bin run-inference`
- execute binary `./target/release/run-inference PATH_TO_SKETCH` (where PATH_TO_SKETCH points to the sketch input file)
