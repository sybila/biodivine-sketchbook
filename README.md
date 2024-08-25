# [Work in progress] AEON Sketchbook

AEON Sketchbook is a multi-platform application for designing and analysing large-scale logical models.

## Development

Make sure you have Rust and NPM installed. Then, after cloning, run `npm install` to download all JS/TS dependencies. To start the application in debug mode, run `npm run tauri dev`. To build a release version of the app, run `npm run build`. Note that the first build can take a few minutes as the application backend needs to be compiled. Subsequent builds should be faster. Furthermore, upon startup, the application window can be unresponsive for a few seconds when running using `npm run tauri dev`. This is because the whole application is running in debug mode without optimizations. This startup delay should be substantially reduced when using the release binaries produced by `npm run build`.

For format checking/fixing and for running basic tests, you can use the following commands.
Run the cargo commands inside `src-tauri` folder, and eslint in the main directory.
- `npx eslint src --config .eslintrc.yml  --ext .js,.jsx,.ts,.tsx --fix`
- `cargo fmt`
- `cargo clippy`
- `cargo test`
