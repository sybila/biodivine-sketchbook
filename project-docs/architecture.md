# Sketchbook Architecture

This document gives a general rationale behind the chosen architecture and technologies.
It is divided into sections describing:
- structure of the project repository
  - details on structure of Rust modules
  - details on structure of data folder
- technologies used
- early architecture notes

## Project structure

We first list the most important components of the project repository, mainly various source folders and configuration files. After that, we add more details on how the Rust back-end code and also `data` folder are structured.

- `manual.pdf` | PDF manual describing tool's functionality, sessions, and various syntax/format details.
- `tutorial.mp4` | Brief video tutorial introducing the tool and its basic functionality.
- `vite.config.ts` | Configuration for the Vite build system (HTML/JS/CSS processing). Most stuff is generated by Tauri, there's just some extra code related to properly building UIKit and to declaring multiple entry points for different windows.
- `tsconfig.json` | Configuration for the TypeScript compiler. Generated by Tauri for now. Later we might want to make it a bit more restrictive.
- `package.json` and `package-lock.json` | List of `npm` dependencies (and its frozen variant). Mostly just Tauri, UIkit, Less, PostHTML, some packages with types declarations, and testing frameworks.
- `.eslintrc.yml` | Configuration for static code analysis for TS with ESLint.
- `src` | The HTML/JS/CSS frontend:
  - `uikit-theme.less` | A Less file where you can place all custom overrides and hooks for the UIkit widgets. If this grows too much, we should create a `theme` folder with Less files and split the overrides into per-widget files.
  - `styles.less` | Other relevant styling unrelated to UIkit. Again, if this grows too much, feel free to refactor it into something like `styles` folder.
  - `main.ts` | The entry point for all JavaScript code. Later we'll probably want different entry points for different windows, but for now it's just a single file.
  - `aeon_events.ts` and `aeon_state.ts`| The front-end part of the event-based communication. The first one defines the event processing mechanisms in general, the second defines particular structure with event wrapper API.
  - `html/window.html` | The default HTML "wrapper" that is extended by individual windows. Later, we'll probably need to add other wrappers for things like dialog windows.
  - `html/component-editor` | A directory with HTML files of self-contained components for sketch editor workflows. 
  - `html/component-analysis` | A directory with HTML files of self-contained components for analysis workflows. 
  - `html/util` | A directory with various (TypeScript) utilities, data interfaces, and so on. 
  - `html` | Other HTML content goes here. For now, this includes various windows.
  - `assets` | Any images/icons/whatever.
  - `tests` | TypeScript unit-tests that will be run with `vitest` framework. For now just very simple example.
- `src-tauri` | The rust backend:
  - `tauri.conf.json` | Mostly default Tauri config. Might need tweaking once we start working more with windows.
  - `icons` | Icon logo files for the app.  
  - `resources` | Any kind of files that should be bundled together with the code and easily accessible. For now, we use it to store example models (that must not get lost).  
  - `Cargo.toml`, `build.rs` and `src` | Mostly standard Rust project (with dependencies in Cargo file and backend code in src). The hierarchy of the source directory is described in the following section `Backend modules hierarchy`.
- `test` | Folder for end-to-end tests with `selenium` and `mocha` frameworks. For now just very simple example.
- `project-docs` | A directory to collect design/documentation regarding the project's architecture, technologies, design, and more. This is not meant as tool documentation for users, but for developers.
- `data` | A directory with all the models, benchmarks, and datasets. Basic information is given in section `Data folder`

### Backend modules hierarchy

Below, we give introduction into the hierarchical structure of the Rust modules (inside of the `src-tauri/src` folder).
We do not cover the files on the lowest levels of the hierarchy, just the important parts to get to know the project.
More details can be found in the Rust documentation that you can generate with `cargo doc --no-deps --document-private-items`. 

Note that unit tests are mostly defined within the same file as the tested functionality, as is usual in Rust. In the list below, we mention few modules solely targeting more extensive integration tests.

- `main.rs` | Main entry point for the application, starts Tauri and sets up event listeners.
- `logging.rs` | Custom logging macros.
- `lib.rs` | Library file encompassing all top-level Rust modules.
- `algorithms` | Module with all the main internal algorithms and low-level implementation details.
  - `eval_dynamic` | Algorithms and wrappers for evaluation of dynamic properties.
  - `eval_static` | Algorithms and wrappers for evaluation of static properties.
  - `fo_logic` | Parsing and evaluation for the FOL formulas.
- `inference` | Module to handle the state and high-level computation for the inference session.
  - `_test_inference` | Tests for the whole inference computation pipeline.
- `app` | Module defining the core structures and traits behind the application's architecture (like sessions, events, undo-redo stack).
  - `state` | Structures for managing application state and event handling.
    - `inference` | Skeleton of the top-level inference session structure.
    - `editor` | Skeleton of the top-level editor session structure.
- `bin` | Sources for additional binaries that can be run from CLI.
  - `run_inference.rs` | Program for running the inference on the given sketch from CLI.
- `sketchbook` | Module to handle the state of the sketch and events of the editor session.
  - `data_structs` | Simplified structures that are used for communication with frontend and for serialization of more complex internal structs.
  - `layout` | State management for the layouts of the network editor.
  - `model` | State management for the main model - variables, regulations, update functions, supplementary functions, ...
  - `observations` | State management for the observations and datasets.
  - `properties` | State management for the static and dynamic properties.
    - `dynamic_props` | Structures for various dynamic properties.
    - `static_props` | Structures for various static properties.
  - `_tests_events` | Tests for processing of the most significant events.
  - `_sketch` | Implementation of the API for the `Sketch` structure, import/export of the sketch, and so on.

### Data folder

Directory `data` contains various models, benchmarks, and datasets. The structure is following:

- `benchmarks` | Multiple sketches used for performance evaluation. Also contains expected output and metadata. Check `benchmarks/readme.md` for more details.
- `real_cases` | Sketches and datasets relevant to cases studies on biological models and real datasets. Check `real_cases/readme.md` for more details.
- `small_example` | An example sketch used to introduce the framework. Sketch is available in AEON and JSON format. We also provide results of the inference and the resulting sampled BN candidate.
- `test_data` | Sketches and datasets for testing purposes. Used to test importing and inference.
- `load-results-aeon-py.py` | Python script illustrating how to load a results archive exported by Sketchbook and load it into `aeon-py` library. You need to install python and biodivine_aeon library first (`https://pypi.org/project/biodivine-aeon/`). Script is just for illustration, so the path is fixed.

## Technologies

- Overall, we rely primarily on [Rust](https://www.rust-lang.org/), [TypeScript](https://www.typescriptlang.org/) and HTML/CSS. We use [less](https://lesscss.org/usage/) as a CSS pre-processor and [posthtml](https://posthtml.org/#/) as a HTML pre-processor. 
- Dependencies for Rust are managed through `cargo` (obviously), while the JS part is managed by `npm`, because it's the default choice almost everywhere.
- The connection between the Rust back-end and the TypeScript front-end is facilitated using [tauri](https://tauri.app/).
- To build TypeScript and less, we use [Vite](https://vitejs.dev/). It is not a particularly mainstream choice, but it is officially recommended by Tauri, so hopefully it will give us the best compatibility.
- To help with the UI, we use [UIkit](https://getuikit.com/). It seems to be reasonably popular, stable, and above all simple (i.e. just sensible widgets, no special framework logic, templates, etc.).

Overall, you should be able to work with the source code comfortably from Visual Studio Code, although IntelliJ IDEA may be preferable for the Rust portion.

We use standard `cargo` utilities for Rust testing and linting. For TypeScript static analysis, we use [ESLint](https://eslint.org/). For some further TypeScript unit-testing, we use [vitest](https://vitest.dev/) that is recommended by Tauri. For some limited end-to-end testing, we utilize [WebDriver](https://www.w3.org/TR/webdriver/) based approach recommended by Tauri. We go with `selenium-webdriver` framework and `mocha` test runner. More details are in the main README, or you can see this [Tauri testing tutorial](https://jonaskruckenberg.github.io/tauri-docs-wip/development/testing.html)

## Architecture notes (early stage)

Since this is very early in the project, these are mostly just ideas regarding how things should work:

- Each window has a data structure which maintains it's whole state in Rust. The reason why this is in Rust is that this data structure might include (a) detached windows or (b) dialogs, in which case it might actually span multiple physical windows.
- We should be able to completely serialise this window state in order to "save" the state of the program.
- The JS/TS front-end sends "input events" which are reflected in the Rust state, which then sends "display events" to the frontend to actually update the contents of inputs. This is similar to what we did in Pythia. It is a bit worse in terms of performance, but should be manageable.
- Each event that changes the internal state should be also saved into an undo/redo stack based on which events can be replayed or reversed.
- The UI consists of self-contained "components". Each component should  only span a specific sub-tree of HTML elements. However, a component can have sub-components. Ideally, sub-components can be detached into separate windows, with a placeholder in the original location.  