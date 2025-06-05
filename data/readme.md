This folder contains various models, benchmarks, and datasets. The structure is following:

- `benchmarks` | Multiple sketches used for performance evaluation. Also contains expected output and metadata. Check `benchmarks/readme.md` for more details.
- `real_cases` | Sketches and datasets relevant to cases studies on biological models and real datasets. Check `real_cases/readme.md` for more details.
- `small_example` | An example sketch used to introduce the framework. Sketch is available in AEON and JSON format. We also provide results of the inference and the resulting sampled BN candidate.
- `test_data` | Sketches and datasets for internal testing purposes, mainly to test importing and inference.
- `tools_comparison` | Data and instructions for comparison of various inference tools. We compare the tools on a task of inferring a BN model of sepal development in arabidopsis. More details in `tools_comparison/readme.md`.
- `other_sketches` | A collection of additional sketches and relevant models that may be useful for testing/validation. More details in `other_sketches/readme.md`.
- `load-results-aeon-py.py` | Python script illustrating how to load a results archive exported by Sketchbook into `biodivine_aeon` library for further analysis. You need to install Python and `biodivine_aeon` library first (`https://pypi.org/project/biodivine-aeon/`). This script is just for an illustration, and its input the path is hardcoded. A more extensive example will be added.
