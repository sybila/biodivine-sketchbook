> TODO: work in progress

Here, we compare four inference tools on a task of inferring a BN model of a signalling network governing the sepal development in Arabidopsis Thaliana. The inputs for the inference, a partially specified model with 21 variables and steady-state data, come from the following paper:

> Camilo La Rota, Jérôme Chopard, Pradeep Das, Sandrine Paindavoine, Frédérique Rozier, Etienne Farcot, Christophe Godin, Jan Traas, Françoise Monéger, A Data-Driven Integrative Model of Sepal Primordium Polarity in Arabidopsis, The Plant Cell, Volume 23, Issue 12, December 2011, Pages 4318–4333, [https://doi.org/10.1105/tpc.111.092619](https://doi.org/10.1105/tpc.111.092619).

We compare Sketchbook with the following tools: [BoNesis](https://bnediction.github.io/bonesis/index.html) (also published in this [paper](doi:10.1109/ICTAI.2019.00014)), [Griffin](https://turing.iimas.unam.mx/griffin/index.html) (published [here](https://doi.org/10.3389/fgene.2018.00039)), and [BRE:IN](https://github.com/kuglerh/BREIN) (also published [here](https://doi.org/10.1007/978-3-030-31304-3_15)). Each tool is given its sub-directory with relevant files. Below, we give instruction on the setup and replication of our experiments.

### Sketchbook
Sketchbook's setup instructions are given in the README file in the main directory of the repository. In sub-directory `sketchbook`, you can find the following data:
- `arabidopsis.json` is the main sketch file, encompassing the PSBN, regulation properties, and data-based property
- the dataset is also exported in `dataset_arabidopsis.csv` for convenience
- archive `results.zip` contains precomputed results

If you are using the GUI, simply follow these instructions to run the inference yourself:
- open Sketchbook
- click `LOAD JSON PROJECT` button and open the prepared sketch file
- click the `Analysis` tab at the top navigation bar
- click `START INFERENCE SESSION`, which opens a new window
- click `RUN FULL INFERENCE` and wait for the results
- you can then explore the report and use the buttons to either sample or export results

There is also another sketch file `arabidopsis_with_additional_prop.json`. This is the same as previous, but with an additional property requiring the model to admit exactly two attractors. The pre-computed results for this sketch are at `results_with_additional_prop.zip`.

### BoNesis

To execute the experiments, you'll need to install Python (at least 3.10) and two libraries: `bonesis` and `biodivine_aeon`. Bonesis installation instructions are given [here](https://bnediction.github.io/bonesis/installation.html), `aeon` is described [here](https://github.com/sybila/biodivine-aeon-py/). In short, you can follow these steps:
- install [Python](https://www.python.org/downloads/)
- we recommend setting up a Python [virtual environment](https://docs.python.org/3/library/venv.html)
- activate the venv
- install BoNesis with `pip install bonesis`
- intall Aeon library with `pip install biodivine_aeon`

The whole experiment is given as a Jupyter notebook, as well as a Python script. Both can be found in the `bonesis` sub-directory. Either set up [Jupyter](https://jupyter.org/install) and execute the cells of the notebook `bonesis_experiment.ipynb`, or run the script with `python bonesis_experiment.py`.

### Griffin

You can download the Griffin tool from [this website](https://turing.iimas.unam.mx/griffin/downloads.html). We recommend downloading the archive with the prebuilt tool (version 0.1.5). The archive contains installation and usage instructions for unix-based system in `INSTALL.TXT` and `README` files. You'll need Java, the authors recommend Java SE Runtime Environment 8 from [Oracle](http://www.oracle.com).

Once you set everything up, use the pre-built binary `griffin` with the inputs we prepared in the `griffin` sub-directory:
```
griffin -f arabidopsis.grf -o results.out
```

Pre-computed results can be explored in `results-arabidopsis.out`.

### BRE:IN
You can download BRE:IN from the [tool's repository](https://github.com/kuglerh/BREIN/). The readme in the repository provides installation instructions. You will need both the java SDK and [NuSMV](https://nusmv.fbk.eu/downloads.html) to be on the path. The authors recommend using java version 1.8.0_144 and NuSMV-2.6.0.

Once you set everything up, use the prepared JAR program with the inputs we prepared in the `brein` sub-directory. During our experiments, all our attempts to run BRE:IN with any reasonable limits on the number of generated solutions (more than 150) always failed with an error. Very low limits on the number of generated solutions worked though. Therefore, you can run BRE:IN with a low limit of 100 generated solutions with:
```
java -jar NAE.jar 100 arabidopsis.net arabidopsis.spec time_step
```

Note that many variables had to be renamed, as BRE:IN internally performs some strange string substitutions and fails to process the model with original variable names (for example, a variable can not contain other variable's name as its prefix). This renaming does not affect any semantics though.