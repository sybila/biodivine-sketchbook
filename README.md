# Biodivine Sketchbook

SketchBook is a multi-platform application for the synthesis of Boolean network models.
It provides a user-friendly interface for designing a Boolean network sketch and inferring admissible BNs.

Boolean network sketches, introduced in [this paper](https://doi.org/10.1093/bioinformatics/btad158), are a framework for integrating various kinds of prior knowledge and experimental data. The sketch consists of an influence graph, partially specified update functions, update function properties, and dynamic properties (including experimental data). 
Sketchbook presents a way to design all these components and more.

Once you finish designing your sketch, you can run the state-of-the-art synthesis algorithms and symbolically compute all BNs consistent with your requirements. You can then sample individual BNs, or export the results and process it with libraries like [AEON.py](https://pypi.org/project/biodivine-aeon/).

### Citation

If you used Sketchbook for some academic work, we'd be very happy if you could cite it using the following publication:

```
Beneš, N., Brim, L., Huvar, O., Pastva, S., & Šafránek, D. (2023). 
Boolean network sketches: a unifying framework for logical model inference.
Bioinformatics, 39(4), https://doi.org/10.1093/bioinformatics/btad158.
```

## Installation

We provide pre-built binaries and installation files for the application in the [release section](https://github.com/sybila/biodivine-sketchbook/releases) (includes versions for Windows, Linux and macOS). 
To start using Sketchbook, download binary for your operating system - you choose between `.app` and `.dmg` for macOS, `.AppImage`, `.deb` and `.rpm` for Linux, or `.exe` and `.msi` for Windows.
If you need a different pre-built binary for a specific platform, let us know!

> Note that the binaries are not signed, so macOS and Windows will likely ask if you trust the application or otherwise require you to allow it to run explicitly. We do not include instructions for these steps.

Alternatively, if you want to build the tool locally, the instructions are provided in the Development guide below. Note that the local build requires additional dependencies to be installed.

## Development

The following instructions describe the local installation of the application and relevant frameworks. We recommend using the pre-built binaries described in the previous section.

For a summary of all technologies and detailed project structure, see `ARCHITECTURE.md`.

### Installation of dependencies

First, make sure you have Rust and NPM installed. For Rust, we recommend following the instructions on [rustlang.org](https://www.rust-lang.org/learn/get-started). For instructions on NPM and Node installation, feel free to check [their website](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm).

We have tested the app using the following versions:
- npm 10.9.0 
- node 22.11.0
- rust 1.82.0

Then, after cloning the repository, run `npm install` to download all JS/TS dependencies. Rust dependencies are downloaded automatically during the build (next step).

### Building the app

To build a release version of the app, run `npm run build`. Note that the first build can take a few minutes as the application backend needs to be compiled. Subsequent builds should be a lot faster. 
To properly build the full installation file for the app, you can also use `cargo tauri build`. It will create an installation bundle at `src-tauri/target/release/bundle`.

To start the application in debug mode, run `npm run tauri dev`. Note that upon startup, the application window can be unresponsive for a few seconds when using development mode. This is because the whole application is running in debug mode without any optimizations. This startup delay should be substantially reduced when using the release binaries produced by `npm run build`.

### Static analysis, tests, documentation

This section describes the setup and instructions for static analysis tools, testing frameworks, and documentation generation. You don't need any of these to run the application, but they are useful for development. For all the following, run the cargo commands inside `src-tauri` folder, and npm/npx commands in the main project directory.

For format checking/fixing for the TypeScript part of the project, you can run `npx eslint "src/**/*.{js,jsx,ts,tsx}" --config .eslintrc.yml --fix`. You can use `cargo fmt` and `cargo clippy` for the Rust side.

To run the full Rust test suite, use `cargo test`. To run the TypeScript tests, run `npx vitest --run` or `npm test`.

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
We also utilize an end-to-end Selenium-based testing framework. Note that these tests require additional dependencies, and they are limited for Linux and Windows (due to MacOS issues with WebDriver). 

You can follow this [detailed tutorial](https://jonaskruckenberg.github.io/tauri-docs-wip/development/testing.html) for setup. In short, you should install `tauri-driver` (with `cargo install tauri-driver`), and then you will need either `WebKitWebDriver` on Linux or `Microsoft Edge Driver` on Windows (make sure that you have updated Microsoft Edge too). The mocha test runner can be installed with `npm install mocha chai selenium-webdriver`. 
To run the tests, first build the app with `cargo tauri build` and then use `npx mocha`. 
The framework was tested on Windows with `Microsoft Edge WebDriver` version `130.0.2849.89`.
