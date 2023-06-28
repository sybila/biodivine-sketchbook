import { defineConfig } from "vite";
import { resolve, basename } from "path";
import { fileURLToPath } from 'url';
import posthtml from '@vituum/vite-plugin-posthtml';

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  
  // prevent vite from obscuring rust errors
  clearScreen: false,
  
  // tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },
  
  // to make use of `TAURI_DEBUG` and other env variables
  // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
  envPrefix: ["VITE_", "TAURI_"],
  
  // PostHTML plugin is used to implement modular HTML components 
  // across windows.
  plugins: [posthtml()],

  build: {
    // Tauri supports es2021
    target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      input: {
        signpost: resolve(__dirname, 'src/html/signpost.html'),
        sketch_editor: resolve(__dirname, 'src/html/sketch-editor.html'),
        state_space_explorer: resolve(__dirname, 'src/html/state-space-explorer.html')
      }
    }
  },

  /* 
    This is a workaround for correctly showing images imported from UIKit.
    See https://stackoverflow.com/questions/71519410/how-can-vite-be-configured-to-load-relative-images-in-stylesheets-imported-from
  */
  resolve: {
    alias: [
      {
        find: '../../images',
        replacement: '',
        customResolver(updatedId, importer, _resolveOptions) {
          // don't replace if importer is not our my-uikit.less
          if (importer === undefined || basename(importer) !== 'uikit-theme.less') {
            return '../../images';
          }

          return fileURLToPath(
            new URL(
              './node_modules/uikit/src/images' + updatedId,
              import.meta.url
            )
          );
        },
      },
    ],
  },
}));
