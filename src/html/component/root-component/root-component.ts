import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './root-component.less?inline'
import '../content-pane/content-pane'
import '../nav-bar/nav-bar'
import { TabData } from '../../util/tab-data'

const SAVE_KEY = 'tab-data'

@customElement('root-component')
class RootComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @state() tabs: TabData[] = []
  currentIndex = 0
  constructor () {
    super()
    this.loadTabs()
    this.addEventListener('switch-tab', this.switchTab)
    this.addEventListener('pin-tab', this.pinTab)
    this.addEventListener('unpin-tab', this.pinTab)
    this.addEventListener('add-tab', this.addTab)
    this.addEventListener('reset', this.reset)
  }

  private addTab (): void {
    this.currentIndex++
    this.tabs = this.tabs.concat(TabData.create({
      id: this.currentIndex,
      name: `Tab ${this.currentIndex}`,
      data: `Contents of tab ${this.currentIndex}`
    }))
    this.saveTabs()
  }

  private saveTabs (): void {
    const tabData = this.tabs.map((tab) => ({
      data: tab.data,
      name: tab.name,
      id: tab.id,
      active: tab.active
    }))
    localStorage.setItem(SAVE_KEY, JSON.stringify(tabData))
  }

  private loadTabs (): void {
    const tabData = JSON.parse(localStorage.getItem(SAVE_KEY) ?? '[]')
    this.tabs = tabData.map((data: { id: number, name: string, data: string, active: boolean }) => {
      this.currentIndex = Math.max(this.currentIndex, +data.id)
      return TabData.create({
        id: data.id,
        name: data.name,
        data: data.data,
        active: data.active
      })
    })
    console.log(this.tabs)
  }

  private pinTab (e: Event): void {
    const tabId = (e as CustomEvent).detail.tabId
    if (tabId === undefined) return
    const tabIndex = this.tabs.findIndex((tab) => tab.id === tabId)
    if (tabIndex === -1) return
    this.tabs[tabIndex] = this.tabs[tabIndex].copy({ pinned: e.type === 'pin-tab' })
    console.log('tab pin:', this.tabs[tabIndex], e.type)
    this.requestUpdate()
    this.saveTabs()
  }

  private switchTab (e: Event): void {
    const tabId = (e as CustomEvent).detail.tabId
    if (tabId === undefined) return
    this.tabs = this.tabs.map((tab) =>
      tab.copy({
        active: tab.id === tabId
      })
    )
    this.requestUpdate()
    this.saveTabs()
  }

  private reset (): void {
    this.currentIndex = 0
    this.tabs = []
    this.addTab()
    localStorage.removeItem(SAVE_KEY)
  }

  render (): TemplateResult {
    return html`
      <div class="uk-container-expand">
        <nav-bar .tabs=${this.tabs}></nav-bar>
          <div class="uk-flex uk-flex-row">
              ${map(this.tabs.sort((a, b) => a.id - b.id).filter((tab) => tab.pinned || tab.active), (tab) => html`
                  <content-pane .tab=${tab}></content-pane>
              `)}
          </div>
      </div>
    `
  }
}
