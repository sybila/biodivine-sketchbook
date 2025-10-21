## Instructions to replicate experiments in the provided VM

This readme describes how to replicate all the experiments on the provided VM. We have already installed all the tools and requirements on the VM. If you want to see general instructions to set up the tools elsewhere, check the main readme.

We start with instructions regarding the VM and its structure. In the rest of the README, we first comment on replicating the Arabidopsis case study in Sketchbook. We then give instruction s on how to execute the inference on the other three relevant tools. Lastly, we give instructions on replicating the performance evaluation.


#### Virtual machine overview

We prepared everything on a Ubuntu 22.04 VM. 
We recommend using [VirtualBox](https://www.virtualbox.org/). You simply import the provided OVA file to the VirtualBox, and once loaded, start the VM.
Once the Ubuntu VM boots, use the following credentials to log in: user name `user` and password `password`.
After logging in, you should see the `Biodivine Sketchbook` app directly on the desktop. 
All the data and tools can be found in the `Experiments/` directory directly in the Home folder (`/home/user/`). We'll consider this directory in the rest of the instruction.

### Case study in Sketchbook

In sub-directory `sketchbook`, you can find the following data:
- `arabidopsis.json` is the exported arabidopsis sketch file prepared for the inference
- the binarized dataset is also exported in `dataset_arabidopsis.csv` for convenience
- archive `results.zip` contains precomputed results

If you are using the tool's GUI, simply follow these instructions to run the inference yourself:
- Open Sketchbook.
- Click `LOAD JSON PROJECT` button on the initial screen.
- A file dialog will open. Navigate to and open the prepared sketch file `arabidopsis.json` mentioned above. 
- At the top of the editor, there is a navigation bar with multiple tabs. Click the `Analysis` tab.
- Click the `START INFERENCE SESSION` button on the left half of the screen.
- New window with the inference session opens. Click `RUN FULL INFERENCE` and wait for the results.
- Once the compution is done, you can then explore the report or use the buttons to either sample or export results.

There is also another sketch file `arabidopsis_with_additional_prop.json`. This is the same as previous, but with an additional property requiring the model to admit exactly two attractors.
The pre-computed results for this sketch are at `results_with_additional_prop.zip`.

### Running experiments with other tools

All the experiments here are run from the terminal.

#### BoNesis

Go to the `bonesis` sub-folder. Then activate the prepared Python virtual environment and execute the Python script with:

```
source venv/bin/activate
python3 bonesis_experiment.py
```

The tool will generate all 439296 solutions in a matter of seconds.
A short output will be shown on the standard output. 


#### Griffin

Go to the `griffin` sub-folder and run the tool with prepared inputs using:

```
griffin -f arabidopsis.grf -o results.out
```

It will generate all 439296 solutions one by one, which make take a few hours. 
The results will be generated into `results.out`.

Also, you can compare the results with our pre-computed results in `results-arabidopsis.out`.

#### BRE:IN

Go to the `brein` sub-folder and run the tool with prepared inputs using:

```
java -jar BREIN/NAE.jar 120 arabidopsis.net arabidopsis.spec time_step
```

This runs the inference with a limit of 120 solutions. When we tried larger limits, the tool terminated with an error.

The solutions will be printed to the standard output.


### Sketchbook performance evaluation

We've prepared a script to execute all the performance benchmarks with Sketchbook.
In sub-directory `sketchbook`, go to the `biodivine-sketchbook` (a clone of the tool's repository). 
To run the experiments, just execute the prepared Python script by running

```
python3 run_performance_eval.py
```

The results will be printed to the standard output (including the number of candidates and computation times).
If you want to examine the models and pre-computed results, see `data/benchmarks` with all the details.

#### Comparison with enumeration approach 

To run Sketchbook performance benchmarking on the two models with large solution spaces, you can modify the `run_performance_eval.py` script. Simply uncomment lines 9 and 32, and you can run the modified Python script the same way as described above.

To run this experiment BoNesis, first activate the Python virtual environment in the `bonesis` sub-folder as described above. Then go to `bonesis/larger_benchmarks` subfolder and use the `bonesis_experiment_large.py` script as follows (choose a solution limit). 

```
python3 bonesis_experiment_large.py ./nsp9/nsp9_sketch_v2.aeon ./nsp9/nsp9_dataset.csv --limit 10000
python3 bonesis_experiment_large.py ./macrophage/macrophage_sketch_v2.aeon ./macrophage/macrophage_dataset.csv --limit 10000
```

You can examine the input files, and you can check precomputed results in `sketchbook-output.txt` and `bonesis-out.txt`.