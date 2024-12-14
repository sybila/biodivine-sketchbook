This folder contains various models, benchmarks, and datasets. The structure is following:

- `benchmarks` | Multiple sketches used for performance evaluation. Also contains expected output and metadata. Check `benchmarks/readme.md` for more details.
- `real_cases` | Sketches and datasets relevant to cases studies on biological models and real datasets. Check `real_cases/readme.md` for more details.
- `small_example` | An example sketch used to introduce the framework. Sketch is available in AEON and JSON format. We also provide results of the inference and the resulting sampled BN candidate.
- `test_data` | Sketches and datasets for testing purposes. Used to test importing and inference.
- `load-results-aeon-py.py` | Python script illustrating how to load a results archive exported by Sketchbook and load it into `aeon-py` library. You need to install python and biodivine_aeon library first (`https://pypi.org/project/biodivine-aeon/`). Script is just for illustration, so the path is fixed.
