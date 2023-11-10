import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './root-component.less?inline'
import '../content-pane/content-pane'
import '../nav-bar/nav-bar'
import { type TabData } from '../../util/tab-data'
import { tabList } from '../../util/config'

const SAVE_KEY = 'tab-data'

@customElement('root-component')
class RootComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @state() tabs: TabData[] = tabList
  constructor () {
    super()
    this.loadTabs()
    this.addEventListener('switch-tab', this.switchTab)
    this.addEventListener('pin-tab', this.pinTab)
    this.addEventListener('unpin-tab', this.pinTab)
  }

  private saveTabs (): void {
    const tabData = this.tabs.map((tab) => ({
      id: tab.id,
      active: tab.active,
      pinned: tab.pinned
    }))
    localStorage.setItem(SAVE_KEY, JSON.stringify(tabData))
  }

  private loadTabs (): void {
    const tabData = JSON.parse(localStorage.getItem(SAVE_KEY) ?? '[]')
    tabData.forEach((data: { id: number, active: boolean, pinned: boolean }) => {
      this.tabs[data.id] = this.tabs[data.id].copy({
        active: data.active,
        pinned: data.pinned
      })
    })
    console.log(this.tabs)
  }

  private pinTab (e: Event): void {
    const tabId = (e as CustomEvent).detail.tabId
    if (tabId === undefined) return
    const tabIndex = this.tabs.findIndex((tab) => tab.id === tabId)
    if (tabIndex === -1) return
    const updatedTabs = this.tabs.slice()
    updatedTabs[tabIndex] = updatedTabs[tabIndex].copy({ pinned: e.type === 'pin-tab' })
    this.tabs = updatedTabs
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

  render (): TemplateResult {
    const visibleTabs = this.tabs.sort((a, b) => a.id - b.id).filter((tab) => tab.pinned || tab.active)
    return html`
      <div class="root-component">
        <nav-bar .tabs=${this.tabs}></nav-bar>
          <div class="content uk-flex uk-flex-row uk-flex-stretch uk-flex-wrap-stretch">
              ${map(visibleTabs, (tab) => html`
                  <content-pane class="uk-width-1-${visibleTabs.length} ${tab.active ? 'active' : 'inactive'}" .tab=${tab}></content-pane>
              `)}
          </div>
      </div>
    `
  }
}
