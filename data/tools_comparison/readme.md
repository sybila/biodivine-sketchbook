Here, we experiment with four inference tools on a task of inferring a BN model of a signalling network governing the sepal development in Arabidopsis Thaliana. The inputs for the inference, a partially specified model with 21 variables and steady-state data, come from the following paper:

> Camilo La Rota, Jérôme Chopard, Pradeep Das, Sandrine Paindavoine, Frédérique Rozier, Etienne Farcot, Christophe Godin, Jan Traas, Françoise Monéger, A Data-Driven Integrative Model of Sepal Primordium Polarity in Arabidopsis, The Plant Cell, Volume 23, Issue 12, December 2011, Pages 4318–4333, [https://doi.org/10.1105/tpc.111.092619](https://doi.org/10.1105/tpc.111.092619).

We compare Sketchbook with the following three tools: [BoNesis](https://bnediction.github.io/bonesis/index.html) (also published in this [paper](doi:10.1109/ICTAI.2019.00014)), [Griffin](https://turing.iimas.unam.mx/griffin/index.html) (published [here](https://doi.org/10.3389/fgene.2018.00039)), and [BRE:IN](https://github.com/kuglerh/BREIN) (also published [here](https://doi.org/10.1007/978-3-030-31304-3_15)). Each tool is given its sub-directory with relevant files. Below, we give instruction on the setup and replication of our experiments.

> This readme describes the general setup of the experiments, including setup and installation from scratch. If you want to use the VM provided as part of our paper artefact (with all tools already prepared), check `instructions-vm.md`.

### Sketchbook

Sketchbook's setup instructions are given in the README file in the main directory of the tool's [GitHub repository](https://github.com/sybila/biodivine-sketchbook). TLDR, you can download pre-built binaries for your system and use them directly.

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

There is also another sketch file `arabidopsis_with_additional_prop.json`. This is the same as previous, but with an additional property requiring the model to admit exactly two attractors. The pre-computed results for this sketch are at `results_with_additional_prop.zip`.

### BoNesis

To set up for the BoNesis experiment, you'll first need to install Python (at least 3.10) and two libraries: `bonesis` and `biodivine_aeon`. BoNesis installation instructions are given [here](https://bnediction.github.io/bonesis/installation.html), `aeon` is described [here](https://github.com/sybila/biodivine-aeon-py/). In short, you can follow these steps:
- Install [Python](https://www.python.org/downloads/).
- We recommend setting up a Python [virtual environment](https://docs.python.org/3/library/venv.html).
- Activate the virtual environment (if you set it up in previous step).
- Install BoNesis with `pip install bonesis`.
- Intall aeon library with `pip install biodivine_aeon`.

The whole experiment is given as a Python script, as well as a Jupyter notebook for convenience. Both can be found in the `bonesis` sub-directory. You can either run the script with `python bonesis_experiment.py`, or set up [Jupyter](https://jupyter.org/install) and execute the cells of the notebook `bonesis_experiment.ipynb`.
The tool will generate all 439296 solutions in a matter of seconds.

### Griffin

You can download the Griffin tool from [this website](https://turing.iimas.unam.mx/griffin/downloads.html). We recommend downloading the archive with the prebuilt tool (version 0.1.5). The archive contains installation and usage instructions for unix-based system in `INSTALL.TXT` and `README` files. You'll need Java, the authors recommend Java SE Runtime Environment 8 from [Oracle](http://www.oracle.com).

Once you set everything up, use the pre-built binary `griffin` with the inputs we prepared in the `griffin` sub-directory:
```
griffin -f arabidopsis.grf -o results.out
```

The tool will generate all 439296 solutions one by one, which make take a few hours.
The pre-computed results are not part of the repository due to the size of the file.

### BRE:IN

You can download BRE:IN from the [tool's repository](https://github.com/kuglerh/BREIN/). The readme in the repository provides installation instructions. You will need both the Java SDK and [NuSMV](https://nusmv.fbk.eu/downloads.html) to be on the path. The authors recommend using Java version 1.8.0_144 and NuSMV-2.6.0.

Once you set everything up, use the corresponding JAR program with the inputs we prepared in the `brein` sub-directory. 
During our experiments, all of our attempts to run BRE:IN with any reasonable limit on the number of generated solutions (more than 150) always failed with an error. Very low limits on the number of generated solutions worked though. Therefore, you can run BRE:IN with a low limit of 120 generated solutions with:

```
java -jar NAE.jar 120 arabidopsis.net arabidopsis.spec time_step
```

The computation will enumerate these 120 solutions within a few seconds.
The solutions will be printed to the standard output.

Note that many model variables had to be renamed, as BRE:IN internally performs string substitutions and fails to process the model with original variable names (for example, a variable can not contain other variable's name as its prefix). This renaming does not affect any semantics though.