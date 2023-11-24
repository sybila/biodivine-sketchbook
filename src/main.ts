import { emit, listen } from '@tauri-apps/api/event'
import UIkit from 'uikit';
import Icons from 'uikit/dist/js/uikit-icons';



// loads the Icon plugin
UIkit.use(Icons)

window.addEventListener("DOMContentLoaded", async () => {
  const counter = document.querySelector("#counter") as HTMLElement;
  const counter_up = document.querySelector("#counter-up") as HTMLElement;
  const counter_down = document.querySelector("#counter-down") as HTMLElement;
  const text = document.querySelector("#text") as HTMLInputElement;
  await listen("aeon-state-change", (event) => {
    const aeon_path = event.payload.full_path;
    const aeon_payload = event.payload.payload;
    if(aeon_path.length == 2 && aeon_path[0] == "editor" && aeon_path[1] == "counter") {
      counter.innerText = aeon_payload
    }
    if(aeon_path.length == 2 && aeon_path[0] == "editor" && aeon_path[1] == "text") {
      text.value = aeon_payload
    }
  });
  counter_up.addEventListener("click", () => {
    emit("aeon-user-action", {
      full_path: ["editor", "counter"],
      payload: (parseInt(counter.innerText) + 1).toString()
    });
  });
  counter_down.addEventListener("click", () => {
    emit("aeon-user-action", {
      full_path: ["editor", "counter"],
      payload: (parseInt(counter.innerText) - 1).toString()
    });
  });
  text.addEventListener("input", () => {
    emit("aeon-user-action", {
      full_path: ["editor", "text"],
      payload: text.value
    });
  });

  const undo = document.querySelector("#undo") as HTMLElement;
  undo.addEventListener("click", () => {
    emit("undo");
  });
  const redo = document.querySelector("#redo") as HTMLElement;
  redo.addEventListener("click", () => {
    emit("redo");
  });

});

// components can be called from the imported UIkit reference
// UIkit.notification('Hello world.');

// let greetInputEl: HTMLInputElement | null
// let greetMsgEl: HTMLElement | null

// async function greet () {
//   if (greetMsgEl && greetInputEl) {
//     // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
//     greetMsgEl.textContent = await invoke('greet', {
//       name: greetInputEl.value
//     })
//   }
// }
//
// window.addEventListener('DOMContentLoaded', () => {
//   greetInputEl = document.querySelector('#greet-input')
//   greetMsgEl = document.querySelector('#greet-msg')
//   document.querySelector('#greet-form')?.addEventListener('submit', (e) => {
//     e.preventDefault()
//     await greet()
//   })
// })
