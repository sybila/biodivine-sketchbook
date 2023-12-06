import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './root-component.less?inline'
import '../content-pane/content-pane'
import '../nav-bar/nav-bar'
import { type TabData } from '../../util/tab-data'
import { tabList } from '../../util/config'
import { aeon_state } from '../../../aeon_events'

const SAVE_KEY = 'tab-data'

@customElement('root-component')
class RootComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @state() tabs: TabData[] = tabList
  constructor () {
    super()    
    aeon_state.tab_bar.active.addEventListener(this.#onSwitched.bind(this))
    aeon_state.tab_bar.pinned.addEventListener(this.#onPinned.bind(this))    
  }

  #onPinned (pinned: number[]): void {    
    this.tabs = this.tabs.map((tab) => 
      tab.copy({
        pinned: pinned.includes(tab.id)
      })
    )
  }

  #onSwitched (tabId: number) {          
    this.tabs = this.tabs.map((tab) =>
      tab.copy({
        active: tab.id === tabId
      })
    )
    this.adjustRegEditor()
  }

  private adjustRegEditor (): void {
    console.log(window.outerWidth)
    if (window.outerWidth <= 800) return
    this.shadowRoot?.querySelector('content-pane')
      ?.shadowRoot?.querySelector('regulations-editor')
      ?.dispatchEvent(new CustomEvent('adjust-graph', {
        detail: {
          tabCount: this.visibleTabs().length
        }
      }))
  }

  private visibleTabs (): TabData[] {
    return this.tabs.sort((a, b) => a.id - b.id).filter((tab) => tab.pinned || tab.active)
  }

  render (): TemplateResult {
    const visibleTabs = this.visibleTabs()
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
