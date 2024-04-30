import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, query, state } from 'lit/decorators.js'
import style_less from './menu.less?inline'
import { map } from 'lit/directives/map.js'
import { open, save } from '@tauri-apps/api/dialog'
import { appWindow } from '@tauri-apps/api/window'
import {
  aeonState
} from '../../../aeon_events'
import { dialog } from '@tauri-apps/api'
import { when } from 'lit/directives/when.js'
import { computePosition } from '@floating-ui/dom'

@customElement('hamburger-menu')
export default class Menu extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @query('#menu-content') declare menuContentElement: HTMLElement
  @query('#menu-button') declare menuButtonElement: HTMLElement
  @state() menuVisible = false
  @state() menuItems: IMenuItem[] = [
    {
      label: 'New sketch',
      action: () => { void this.newSketch() }
    },
    {
      label: 'Import...',
      action: () => { void this.importSketch() }
    },
    {
      label: 'Export...',
      action: () => { void this.exportSketch() }
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
    const confirmation = await dialog.ask('Are you sure? This operation is irreversible.', {
      type: 'warning',
      okLabel: 'Import',
      cancelLabel: 'Cancel',
      title: 'Import new sketch'
    })
    if (!confirmation) return

    const selected = await open({
      title: 'Import sketch...',
      multiple: false,
      filters: [{
        name: '*.json',
        extensions: ['json']
      }]
    })
    if (selected === null) return
    let importFile = ''
    if (Array.isArray(selected)) {
      if (selected.length === 0) return
      importFile = selected[0]
    } else {
      importFile = selected
    }

    console.log('importing', importFile)
    aeonState.sketch.importSketch(importFile)
  }

  async exportSketch (): Promise<void> {
    const filePath = await save({
      title: 'Export sketch...',
      filters: [{
        name: '*.json',
        extensions: ['json']
      }],
      defaultPath: 'project_name_here'
    })
    if (filePath === null) return

    console.log('exporting to', filePath)
    aeonState.sketch.exportSketch(filePath)
  }

  async newSketch (): Promise<void> {
    const confirmation = await dialog.ask('Are you sure? This operation is irreversible.', {
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
    const confirmation = await dialog.ask('Are you sure? This operation is irreversible.', {
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
    console.log(this.menuButtonElement, this.menuContentElement)
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
      <button id="menu-button" class="menu-button" @click="${this.openMenu}">☰</button>      
    `
  }
}

interface IMenuItem {
  label: string
  action: () => void
}
