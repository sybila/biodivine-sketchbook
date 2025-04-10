import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, query, state } from 'lit/decorators.js'
import style_less from './menu.less?inline'
import { map } from 'lit/directives/map.js'
import { save } from '@tauri-apps/api/dialog'
import { appWindow } from '@tauri-apps/api/window'
import {
  aeonState
} from '../../../aeon_state'
import { dialog } from '@tauri-apps/api'
import { when } from 'lit/directives/when.js'
import { computePosition } from '@floating-ui/dom'

/** Component responsible for the main menu of the editor session. */
@customElement('hamburger-menu')
export default class Menu extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @query('#menu-content') declare menuContentElement: HTMLElement
  @query('#menu-button') declare menuButtonElement: HTMLElement
  @state() menuVisible = false
  @state() menuItems: IMenuItem[] = [
    {
      label: 'New empty sketch',
      action: () => { void this.newSketch() }
    },
    {
      label: 'Import JSON',
      action: () => { void this.importSketch() }
    },
    {
      label: 'Import AEON',
      action: () => { void this.importAeonModel() }
    },
    {
      label: 'Import SBML',
      action: () => { void this.importSbmlModel() }
    },
    {
      label: 'Export JSON',
      action: () => { void this.exportSketch() }
    },
    {
      label: 'Export AEON',
      action: () => { void this.exportAeon() }
    },
    {
      label: 'Export network PNG',
      action: () => { void this.exportNetworkPng() }
    },
    {
      label: 'Quit',
      action: () => { void this.quit() }
    }
  ]

  constructor () {
    super()
    document.addEventListener('click', this.closeMenu.bind(this))
  }

  async importSketch (): Promise<void> {
    const confirmation = await dialog.ask('Importing new sketch will erase the current one. Do you want to proceed?', {
      type: 'warning',
      okLabel: 'Import',
      cancelLabel: 'Cancel',
      title: 'Import new sketch'
    })
    if (!confirmation) return

    this.dispatchEvent(new CustomEvent('import-json', {
      bubbles: true,
      composed: true,
      detail: {} // maybe include some information later
    }))
  }

  async importAeonModel (): Promise<void> {
    const confirmation = await dialog.ask('Importing new model will erase the current sketch. Do you want to proceed?', {
      type: 'warning',
      okLabel: 'Import',
      cancelLabel: 'Cancel',
      title: 'Import new model'
    })
    if (!confirmation) return

    this.dispatchEvent(new CustomEvent('import-aeon', {
      bubbles: true,
      composed: true,
      detail: {} // maybe include some information later
    }))
  }

  async importSbmlModel (): Promise<void> {
    const confirmation = await dialog.ask('Importing new model will erase the current sketch. Do you want to proceed?', {
      type: 'warning',
      okLabel: 'Import',
      cancelLabel: 'Cancel',
      title: 'Import new model'
    })
    if (!confirmation) return

    this.dispatchEvent(new CustomEvent('import-sbml', {
      bubbles: true,
      composed: true,
      detail: {} // maybe include some information later
    }))
  }

  async exportSketch (): Promise<void> {
    const filePath = await save({
      title: 'Export sketch in JSON format...',
      filters: [{
        name: '*.json',
        extensions: ['json']
      }],
      defaultPath: 'project_name_here'
    })
    if (filePath === null) return

    console.log('exporting json to', filePath)
    aeonState.sketch.exportSketch(filePath)
  }

  async exportAeon (): Promise<void> {
    const filePath = await save({
      title: 'Export sketch in extended AEON format...',
      filters: [{
        name: '*.aeon',
        extensions: ['aeon']
      }],
      defaultPath: 'project_name_here'
    })
    if (filePath === null) return

    console.log('exporting aeon to', filePath)
    aeonState.sketch.exportAeon(filePath)
  }

  async exportNetworkPng (): Promise<void> {
    const filePath = await save({
      title: 'Export network into PNG...',
      filters: [{
        name: '*.png',
        extensions: ['png']
      }],
      defaultPath: 'image_name_here'
    })
    if (filePath === null) return

    // this sends an event processed in RegulationsEditor component
    this.dispatchEvent(new CustomEvent('export-png', {
      bubbles: true,
      composed: true,
      detail: { path: filePath }
    }))
  }

  async newSketch (): Promise<void> {
    const confirmation = await dialog.ask('Starting new sketch will erase the current one. Do you want to proceed?', {
      type: 'warning',
      okLabel: 'New sketch',
      cancelLabel: 'Cancel',
      title: 'Start new sketch'
    })
    if (!confirmation) return

    console.log('loading new sketch')
    aeonState.sketch.newSketch()
  }

  async quit (): Promise<void> {
    const confirmation = await dialog.ask('Quiting the application will erase the current sketch. Do you want to proceed?', {
      type: 'warning',
      okLabel: 'Quit',
      cancelLabel: 'Cancel',
      title: 'Quit'
    })
    if (!confirmation) return

    void appWindow.close()
  }

  private itemClick (action: () => void): void {
    this.menuVisible = false
    action()
  }

  openMenu (): void {
    this.menuVisible = true
    // console.log(this.menuButtonElement, this.menuContentElement)
    void computePosition(this.menuButtonElement, this.menuContentElement,
      { placement: 'bottom-start' })
      .then(({ x, y }) => {
        this.menuContentElement.style.left = x + 'px'
        this.menuContentElement.style.top = y + 'px'
      })
  }

  closeMenu (event: Event): void {
    if (!(event.composedPath()[0] as HTMLElement).matches('.menu-button')) {
      this.menuVisible = false
    }
  }

  render (): TemplateResult {
    return html`
      <div id="menu-content" class="menu-content">
      ${when(this.menuVisible,
          () => html`
            <ul class="uk-nav">
              ${map(this.menuItems, (item) => html`
                <li class="menu-item" @click="${() => {
                  this.itemClick(item.action)
                }}">
                  <a>
                    ${item.label}
                  </a>
                </li>
              `)}
            </ul>
          `)}
      </div>
      <button id="menu-button" class="uk-button uk-button-small uk-button-secondary menu-button" @click="${this.openMenu}">☰</button>      
    `
  }
}

interface IMenuItem {
  label: string
  action: () => void
}
