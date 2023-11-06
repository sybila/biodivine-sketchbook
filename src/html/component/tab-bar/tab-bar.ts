import { html, css, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import style_less from './tab-bar.less'

const SAVE_KEY = 'tabs'

@customElement('tab-bar')
class TabBar extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  @state() declare tabs: HTMLButtonElement[]
  tabIndex = 0

  constructor () {
    super()
    this.tabs = []
    this.loadTabs()
    if (this.tabs.length === 0) {
      this.reset()
    }
  }

  private addTab (): void {
    this.tabIndex++
    this.createTab(this.tabIndex, `Tab ${this.tabIndex}`, ['tab', 'uk-button', 'uk-button-secondary'])
    console.log(`tab ${this.tabIndex} added`)
  }

  private saveTabs (): void {
    const tabData = this.tabs.map((tab) => ({
      classList: Array.from(tab.classList),
      textContent: tab.textContent,
      index: tab.dataset.index
    }))
    localStorage.setItem(SAVE_KEY, JSON.stringify(tabData))
  }

  private loadTabs (): void {
    const tabData = JSON.parse(localStorage.getItem(SAVE_KEY) ?? '[]')
    tabData.forEach((data: { classList: string[], textContent: string, index: string }) => {
      this.createTab(+data.index, data.textContent, data.classList)
      this.tabIndex = Math.max(this.tabIndex, +data.index)
    })
    console.log('tabs loaded')
    console.log('index: ', this.tabIndex)
  }

  private createTab (index: number, title: string, classList: string[]): void {
    const newTabButton = document.createElement('button')
    newTabButton.classList.add(...classList)
    newTabButton.onclick = this.switchTab(newTabButton)
    newTabButton.dataset.index = index.toString()
    newTabButton.textContent = title
    this.tabs = this.tabs.concat(newTabButton)

    this.saveTabs()
  }

  switchTab (newTabButton: HTMLButtonElement) {
    return () => {
      this.tabs.forEach((tab) => {
        tab.classList.remove('uk-button-primary')
        tab.classList.add('uk-button-secondary')
      })
      newTabButton.classList.remove('uk-button-secondary')
      newTabButton.classList.add('uk-button-primary')
      const tabIndex = +(newTabButton.dataset.index ?? 0)
      this.dispatchEvent(new CustomEvent('switch-tab', {
        detail: {
          tabId: tabIndex
        },
        bubbles: true,
        composed: true
      }))
      console.log(`clicked tab ${tabIndex}`)
    }
  }

  private reset (): void {
    this.tabIndex = 0
    this.tabs.forEach(tab => { tab.remove() })
    this.tabs = []
    this.addTab()
    localStorage.removeItem(SAVE_KEY)
  }

  render (): TemplateResult {
    return html`
      <div class="tabs">
        ${this.tabs}
        <button class="uk-button uk-button-default uk-button-small new-tab-button" @click=${this.addTab}><span class="plus-symbol">+</span></button>
        <button class="uk-button-small uk-button-secondary uk-margin-left" @click=${this.reset}>\u21bb</button>
      </div>
    `
  }
}
