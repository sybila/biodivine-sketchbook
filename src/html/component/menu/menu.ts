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

// TODO: close menu when clicked outside

@customElement('hamburger-menu')
export default class Menu extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @query('.menu-content') declare menuContentElement: HTMLElement
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

  private toggleMenu (): void {
    this.menuVisible = !this.menuVisible
  }

  private itemClick (action: () => void): void {
    this.toggleMenu()
    action()
  }

  render (): TemplateResult {
    return html`
      <button class="uk-button uk-button-small hamburger-menu uk-margin-small-left"
      @click="${this.toggleMenu}">☰</button>
      <div class="menu-content ${this.menuVisible ? 'show' : ''}">
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
      </div>
    `
  }
}

interface IMenuItem {
  label: string
  action: () => void
}
