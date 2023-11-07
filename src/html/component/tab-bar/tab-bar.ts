import { html, css, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import style_less from './tab-bar.less?inline'

const SAVE_KEY = 'tabs'

@customElement('tab-bar')
class TabBar extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  @state() private tabs: HTMLButtonElement[] = []
  private currentIndex = 0

  constructor () {
    super()
    this.loadTabs()
    if (this.tabs.length === 0) {
      this.reset()
    }
  }

  private addTab (): void {
    this.currentIndex++
    this.createTab(this.currentIndex, `Tab ${this.currentIndex}`, ['tab', 'uk-button', 'uk-button-secondary'])
    console.log(`tab ${this.currentIndex} added`)
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
      this.currentIndex = Math.max(this.currentIndex, +data.index)
    })
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
      const currentIndex = +(newTabButton.dataset.index ?? 0)
      this.dispatchEvent(new CustomEvent('switch-tab', {
        detail: {
          tabId: currentIndex
        },
        bubbles: true,
        composed: true
      }))
    }
  }

  private reset (): void {
    this.currentIndex = 0
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
