# AEON Sketchbook Architecture

This documents gives a general rationale behind the chosen architecture and technologies.

## Technologies

- Overall, we rely primarily on [Rust](https://www.rust-lang.org/), [TypeScript](https://www.typescriptlang.org/) and HTML/CSS. We use [Less](https://lesscss.org/usage/) as a CSS pre-processor. 
- Dependencies for Rust are managed through `cargo` (obviously), while the JS part is managed by `npm`.
- The connection between the Rust back-end and the TypeScript front-end is facilitated using [tauri](https://tauri.app/).
- To build TypeScript and Less, we use [Vite](https://vitejs.dev/). It is not a particularly mainstream choice, but it is officially recommended by Tauri, so hopefully it will give us the best compatibility.
- To help with the UI, we use [UIkit](https://getuikit.com/). It seems to be reasonably popular, stable, and above all simple (i.e. just sensible widgets, no special framework logic, templates, etc.).

Overall, you should be able to work with the source code comfortably from Visual Studio Code, although IntelliJ IDEA may be preferable for the Rust portion.

## Project structure

