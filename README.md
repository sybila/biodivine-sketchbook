# Biodivine SketchBook

SketchBook is a multi-platform application for synthesis of Boolean network models.
It provides a user-friendly way to integrate various kinds of prior knowledge and experimental data into a Boolean network sketch, and then infer admissible BNs.

### Boolean network sketches

Boolean network sketches were introduced in [this paper](https://doi.org/10.1093/bioinformatics/btad158).

### Citation

If you used SketchBook for some academic work, we'd be very happy if you could cite it using 
the following publication:

```
Beneš, N., Brim, L., Huvar, O., Pastva, S., & Šafránek, D. (2023). 
Boolean network sketches: a unifying framework for logical model inference.
Bioinformatics, 39(4), https://doi.org/10.1093/bioinformatics/btad158.
```

## Development

### Installation of dependencies

First, make sure you have Rust and NPM installed. For Rust, we recommend following the instructions on [rustlang.org](https://www.rust-lang.org/learn/get-started). For instructions on NPM and Node installation, feel free to check [their website](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm).

We have tested the app using following versions:
- npm 10.9.0 
- node 22.11.0
- rust 1.77.1

Then, after cloning the repository, run `npm install` to download all JS/TS dependencies. Rust dependencies are downloaded automatically during build (next step).

### Building the app

To build a release version of the app, run `npm run build`. Note that the first build can take a few minutes as the application backend needs to be compiled. Subsequent builds should be faster. 
To properly build the full release bundle for the app, you can also use `cargo tauri build`.

To start the application in debug mode, run `npm run tauri dev`. Note that upon startup, the application window can be unresponsive for a few seconds when using development mode. This is because the whole application is running in debug mode without optimizations. This startup delay should be substantially reduced when using the release binaries produced by `npm run build`.

### Static analysis, tests, documentation
For all the following, run the cargo commands inside `src-tauri` folder, and eslint in the main directory.

For format checking/fixing for TypeScript part of the project, you can run `npx eslint "src/**/*.{js,jsx,ts,tsx}" --config .eslintrc.yml --fix`. You can use `cargo fmt` and `cargo clippy` for the Rust side.

To run the full Rust test suite, use `cargo test`.

To automatically generate Rust documentation, execute `cargo doc --no-deps --document-private-items`.

All previous commands together for simplicity:
```
npx eslint "src/**/*.{js,jsx,ts,tsx}" --config .eslintrc.yml --fix
cd src-tauri
cargo fmt
cargo clippy
cargo test
cargo doc --no-deps --document-private-items
```
