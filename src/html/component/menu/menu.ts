import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, query, state } from 'lit/decorators.js'
import style_less from './menu.less?inline'
import { map } from 'lit/directives/map.js'
import { open, save } from '@tauri-apps/api/dialog'
import { appWindow } from '@tauri-apps/api/window'

// TODO: close menu when clicked outside

@customElement('hamburger-menu')
export default class Menu extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @query('.menu-content') declare menuContentElement: HTMLElement
  @state() menuVisible = false
  @state() menuItems: IMenuItem[] = [
    {
      label: 'New sketch',
      action: () => { console.log('new') }
    },
    {
      label: 'Import...',
      action: () => { void this.import() }
    },
    {
      label: 'Export...',
      action: () => { void this.export() }
    },
    {
      label: 'Quit',
      action: this.quit
    }
  ]

  async import (): Promise<void> {
    let selected = await open({
      title: 'Import sketch...',
      multiple: false,
      filters: [{
        name: '*.json',
        extensions: ['json']
      }]
    })
    if (selected === null) return
    if (Array.isArray(selected)) {
      if (selected.length > 0) { selected = selected[0] }
    }

    console.log('importing', selected)
  }

  async export (): Promise<void> {
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
  }

  quit (): void {
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
