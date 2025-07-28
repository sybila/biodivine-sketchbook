This folder contains several sketches and datasets for internal testing purposes, mainly to test importing and inference. All datasets and sketches are derived from the same simple model of four variables.

- `test_model.aeon`, `test_model.json`, `test_model.sbml` - same base PSBN model to test import in all formats (no datasets or user-defined properties, just a standard PSBN)
- `data_fp.csv`, `data_mts.csv`, `data_time_series.csv` - three datasets to define fixed points, minimal trap spaces, and time series properties
- `test_model_attractors.png` - screenshot snippet with model attractors from AEON (helpful to define some properties)

Then we have the following variants of the sketch used for the main inference testing. Various properties are manually added during the tests to them:
- `test_sketch_1.aeon`, `test_sketch_1.json` - base model with all three datasets loaded
- `test_sketch_2.aeon`, `test_sketch_2.json` - semantically same as variant 1, but uses additional function symbols, uninterpreted function expressions, and properties of uninterpreted functions
