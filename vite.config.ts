import { defineConfig } from "vite";
import { resolve, basename } from "path";
import { fileURLToPath } from 'url';
import posthtml from '@vituum/vite-plugin-posthtml';
import topLevelAwait from "vite-plugin-top-level-await";

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
  plugins: [
    posthtml(),
    topLevelAwait({
      // The export name of top-level await promise for each chunk module
      promiseExportName: "__tla",
      // The function to generate import names of top-level await promise in each chunk module
      promiseImportName: i => `__tla_${i}`
    })
  ],

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
        state_space_explorer: resolve(__dirname, 'src/html/state-space-explorer.html'),
        edit_node_dialog: resolve(__dirname, 'src/html/component/regulations-editor/rename-dialog/rename-dialog.html'),
        import_observation_dialog: resolve(__dirname, 'src/html/component/observations-editor/observations-import/observations-import.html'),
        edit_observation_dialog: resolve(__dirname, 'src/html/component/observations-editor/edit-observation/edit-observation.html')
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
        replacement: 'uikit/src/images',
        customResolver(updatedId, importer) {
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

    // alias: {
    //   '../../images/backgrounds': 'uikit/src/images/backgrounds',
    //   '../../images/components': 'uikit/src/images/components',
    //   '../../images/icons': 'uikit/src/images/icons'
    // }
  },

  css: {
    preprocessorOptions: {
      less: {
        math: "always",
        relativeUrls: true,
        javascriptEnabled: true
      },
    },
  }
}));
