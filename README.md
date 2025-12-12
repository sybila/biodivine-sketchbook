# Biodivine Sketchbook

Sketchbook is a multi-platform tool for synthesis of Boolean models using the framework of Boolean network sketches.
It provides a user-friendly interface for designing a BN sketches and inferring all admissible candidate BNs.
You can find the newest version of the Sketchbook at [our Github repository](https://github.com/sybila/biodivine-sketchbook).

Boolean network sketches, introduced in [this paper](https://doi.org/10.1093/bioinformatics/btad158), serve as a method for integrating various kinds of prior knowledge and experimental data. The sketch consists of an influence graph, partially specified update functions, update function properties, and dynamic properties (including experimental data). 
Sketchbook presents a way to design all these components and more.

Once you finish designing your sketch, you can run the state-of-the-art synthesis algorithms and symbolically compute all BNs consistent with your requirements. You can then sample individual BNs, or export the results for further analysis with libraries like [AEON.py](https://pypi.org/project/biodivine-aeon/).

#### Manual and tutorial

In the provided `manual.pdf`, we discuss the details regarding
- installation,
- designing model specification in Sketchbook,
- running BN inference in Sketchbook,
- supported formats and syntax.

The manual also provides a tutorial section showcasing Sketchbook on a simplified FGFR signalling model. Data for this tutorial can be found in `data/tutorial` subfolder.

Furthermore, we give a short video introduction that you can find as `intro-video.mp4`.
This introduction covers basics on how to edit an sketch and run the inference.
For more details and explanations, see the manual.

The installation is also summarized below, with an additional development guide.

#### Citation

If you used Sketchbook for some academic work, we'd be very happy if you could cite it using the following publication:

```
Beneš, N., Brim, L., Huvar, O., Pastva, S., & Šafránek, D. (2023). 
Boolean network sketches: a unifying framework for logical model inference.
Bioinformatics, 39(4), https://doi.org/10.1093/bioinformatics/btad158.
```

## Installation

We provide pre-built binaries and installation files for the application in the [release section](https://github.com/sybila/biodivine-sketchbook/releases) (includes versions for Windows, Linux and macOS). 
To start using Sketchbook, choose the latest release, download binary for your operating system - you choose between `.app` and `.dmg` for macOS, `.AppImage`, `.deb` and `.rpm` for Linux, or `.exe` and `.msi` for Windows.
If you need a different pre-built binary for a specific platform, let us know!

> Note that on newer Linux distributions (Ubuntu 24 or Debian 13), some Tauri dependencies may not be supported by default. This does not affect the setup through `.AppImage` (as it bundles all dependencies inside). However, it may affect installation with the `.deb` package, where you need to first install `libwebkit2gtk-4.0`. See a solution for that at [this GitHub issue](https://github.com/tauri-apps/tauri/issues/9662#issuecomment-2604798408). This will be resolved once we switch to Tauri 2.

> The binaries are not signed with official developer certificates, so macOS and Windows will most likely require you to grant special permissions to run the app. **On newer versions of macOS, the message is that the app is "corrupted". This is still the same issue regarding app certificates. You should run `xattr -c /path/to/biodivine-sketchbook.app` to ["enable" the app](https://discussions.apple.com/thread/253714860?sortBy=rank).**

Alternatively, if you want to build the tool locally, the instructions are provided in the Development guide below. Note that the local build requires additional dependencies to be installed.

## Development

The following instructions describe the local installation of the application and relevant frameworks. We recommend using the pre-built binaries described in the previous section.

For a summary of all technologies and detailed project structure, see `project-docs/architecture.md`. The directory `project-docs` also contains other documents relevant to the design/development.

We also provide web-based Rust documentation (including internal modules), currently hosted on [these GitHub pages](https://ondrej33.github.io/biodivine_sketchbook/). Instructions to generate Rust documentation locally are given below.


### Installation of dependencies

First, make sure you have Rust and NPM installed. For Rust, we recommend following the instructions on [rustlang.org](https://www.rust-lang.org/learn/get-started). For instructions on NPM and Node installation, feel free to check [their website](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm). On Windows, we recommend [this tutorial](https://learn.microsoft.com/en-us/windows/dev-environment/javascript/nodejs-on-windows).

On some Linux distributions, additional packages might be needed for developing with Tauri. We recommend checking the [setup instructions by Tauri](https://v1.tauri.app/v1/guides/getting-started/prerequisites/).

> As discussed in the previous section, some Tauri dependencies may no longer be supported by default in newer Linux distributions (Ubuntu 24 or Debian 13). You need to first install `libwebkit2gtk-4.0`, see a solution for that at [this GitHub issue](https://github.com/tauri-apps/tauri/issues/9662#issuecomment-2604798408) (also commented out below). This will be resolved once we switch to Tauri 2.

[//]: <> (open file /etc/apt/sources.list)
[//]: <> (insert new line deb http://archive.ubuntu.com/ubuntu jammy main universe)
[//]: <> (sudo apt update)
[//]: <> (sudo apt upgrade)
[//]: <> (sudo apt install libwebkit2gtk-4.0-dev)

The latest version of Sketchbook is developed and tested using the following versions:
- npm 11.3.0 
- node 22.11.0
- rust 1.86.0

Then, after cloning the repository, run `npm install` to download all JS/TS dependencies. Rust dependencies are downloaded automatically during the build (next step).

### Building the app

To build a release version of the app, run `npm run tauri build`. It will create an installation bundle at `src-tauri/target/release/bundle/` (the exact path will be displayed at the end of the command's standard output). Note that the first build can take a few minutes as the application backend needs to be compiled. Subsequent builds should be a lot faster. 

To start the application in debug mode, run `npm run tauri dev`. Note that upon startup, the application window can be unresponsive for a few seconds when using development mode. This is because the whole application is running in debug mode without any optimizations. This startup delay should be substantially reduced when using the release binaries produced by `npm run tauri build`.

Also note that the debug mode is substantially slower in general. This may sometimes cause issues in complex event processing, where some actions with multiple events can exceptionally "overload" the UI (it won't show all the changes caused by the action immediately). Refreshing the UI usually helps to reload latest state from the backend. This should not happen for the release mode, as event processing is substantially faster, and processing delays are optimized for release.

### Static analysis, tests, documentation

This section describes the setup and instructions for static analysis tools, testing frameworks, and documentation generation. You don't need any of these to run the application, but they are useful for development. For all the following, run the cargo commands inside `src-tauri` folder, and npm/npx commands in the main project directory.

For format checking/fixing for the TypeScript part of the project, you can run `npx eslint "src/**/*.{js,jsx,ts,tsx}" --config .eslintrc.yml --fix`. You can use `cargo fmt` and `cargo clippy` for the Rust side.

To run the full Rust test suite, use `cargo test`. The test suite contains both simple unit tests, as well as complex integration and inference tests. For instance, some tests regarding the inference computation pipeline are present in the module `src-tauri/src/inference/_test_inference`. Another important part of the tests for event processing are present in the module `src-tauri/src/sketchbook/_test_events`. Unit tests are present in all relevant modules, as is standard in Rust.

To run the TypeScript tests, run `npx vitest --run` or `npm test`.

To automatically generate Rust documentation, execute `cargo doc --no-deps --document-private-items`.

All previous commands listed together for simplicity:
```
npx eslint "src/**/*.{js,jsx,ts,tsx}" --config .eslintrc.yml --fix
npx vitest --run
cd src-tauri
cargo fmt
cargo clippy
cargo test
cargo doc --no-deps --document-private-items
```

#### End-to-end tests
We also utilize an end-to-end Selenium-based testing framework. Note that these tests require additional dependencies, and they are limited for Linux and Windows (due to MacOS issues with WebDriver). You should also update the test configs based on your system.

You can follow this [detailed tutorial](https://jonaskruckenberg.github.io/tauri-docs-wip/development/testing.html) for setup. In short, you should install `tauri-driver` (with `cargo install tauri-driver`), and then you will need either `WebKitWebDriver` on Linux or `Microsoft Edge Driver` on Windows (make sure that you have updated Microsoft Edge too, and that you have matching versions). The mocha test runner can be installed with `npm install mocha chai selenium-webdriver`. You might need to update the configuration at the top of the testing script `test/test.js`, mainly the path to your Sketchbook binary and to your webdriver.

To run the tests, first build the app with `npm run tauri build` and then use `npx mocha` (you might need a longer timeout, like `npx mocha --timeout 20000`).
The framework was tested on Windows with `Microsoft Edge WebDriver` version `134.0.3124.83`.
However, note that we found the testing framework a bit unstable when the testing machine is overloaded with other tasks. Sometimes, the tests do not go through due to internal WebDriver issues, and we are investigating this.

## Benchmarks and data

The performance benchmark models, results, and more details are in `data/benchmarks`. 
There is also a README with further instructions, you can follow it.
Tldr, to run the performance benchmarks, you can use python and execute them all with `python3 run_performance_eval.py`.

Sketches and datasets relevant to cases studies on biological models and real datasets are in `data/real_cases`. There is also a README with further details.

An example sketch used to introduce the framework is in `data/small_example`. Sketch is available in AEON and JSON format. We also provide results of the inference and the resulting sampled BN candidate.

## Using Sketchbook from the command line

Although the main interface of the tool is its GUI, we provide the most important inference functionality through a library and CLI as well. If you want to use the CLI variant, you must prepare the sketch first. Once you have the sketch file, you can run the inference from the command line as described below. The program then computes all BN instances satisfying the sketch, and based on selected mode either prints a summary, exports symbolic results, or samples candidate BNs.

First, compile the code in `src-tauri` with: `cargo build --release`.
Then, you should find the binary `run-inference` (with an extension according to your system) in `src-tauri/target/release`. The binary has multiple options regarding results export, sampling or logging. See all the details and instructions by running `run-inference --help`. 
The example of use on Linux would be:

```
./src-tauri/target/release/run-inference data/real_cases/tlgl/tlgl.json --results-path "results-tlgl.zip"
```

We also prepared a binary `run-fixed-point-inference` specifically for running inference with just a set of fixed-point properties. It takes a PSBN in `aeon` format and fixed-point data in the standard `csv` format. It computes all instances of the PSBN that exhibit the required fixed-point states. It exports the symbolic inference results to the given path. Use it as:
```
./src-tauri/target/release/run-fixed-point-inference <PSBN_PATH> <CSV_PATH> <RESULTS_PATH>
```
