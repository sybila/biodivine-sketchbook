import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './root-component.less?inline'
import '../content-pane/content-pane'
import '../nav-bar/nav-bar'
import { ContentData, type TabData } from '../../util/tab-data'
import { aeonState } from '../../../aeon_events'
import { dummyData } from '../../util/dummy-data'
import { tabList } from '../../util/config'

@customElement('root-component')
class RootComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @state() data: ContentData = ContentData.create()
  @state() tabs: TabData[] = tabList

  constructor () {
    super()
    aeonState.tab_bar.active.addEventListener(this.#onSwitched.bind(this))
    aeonState.tab_bar.pinned.addEventListener(this.#onPinned.bind(this))
    this.addEventListener('update-data', this.updateData)
    this.addEventListener('update-function', this.focusFunction)
    this.data = dummyData
  }

  #onPinned (pinned: number[]): void {
    this.tabs = this.tabs.map((tab) =>
      tab.copy({
        pinned: pinned.includes(tab.id)
      })
    )
    this.adjustRegEditor()
  }

  #onSwitched (tabId: number): void {
    this.tabs = this.tabs.map((tab) =>
      tab.copy({
        active: tab.id === tabId
      })
    )
    this.adjustRegEditor()
  }

  updateData (event: Event): void {
    this.data = (event as CustomEvent).detail
  }

  private adjustRegEditor (): void {
    if (window.outerWidth <= 800) return
    this.shadowRoot?.querySelector('#regulations')
      ?.shadowRoot?.querySelector('regulations-editor')
      ?.dispatchEvent(new CustomEvent('adjust-graph', {
        detail: {
          tabCount: this.visibleTabs().length
        }
      }))
  }

  private focusFunction (event: Event): void {
    aeonState.tab_bar.active.emitValue(1)
    console.log(this.shadowRoot?.querySelector('#functions')?.shadowRoot?.querySelector('functions-editor'))
    this.shadowRoot?.querySelector('#functions')
      ?.shadowRoot?.querySelector('functions-editor')
      ?.dispatchEvent(new CustomEvent('focus-function', {
        detail: {
          nodeId: (event as CustomEvent).detail.nodeId
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
          ${map(this.tabs, (tab) => html`
            <content-pane id="${tab.name.toLowerCase()}" ?hidden="${!(tab.pinned || tab.active)}" class="uk-width-1-${visibleTabs.length} ${tab.active ? 'active' : 'inactive'}" .tab=${tab}
                          .data=${this.data}></content-pane>
          `)}
        </div>
      </div>
    `
  }
}
