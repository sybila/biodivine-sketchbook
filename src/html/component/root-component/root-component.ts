import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './root-component.less?inline'
import '../content-pane/content-pane'
import '../nav-bar/nav-bar'
import { ContentData, type TabData } from '../../util/tab-data'
import { aeonState } from '../../../aeon_events'
import { tabList } from '../../util/config'
import { type IRegulationData, type IVariableData } from '../../util/data-interfaces'
import { dialog } from '@tauri-apps/api'

const SAVE_VARIABLES = 'variables'
const SAVE_REGULATIONS = 'regulations'

@customElement('root-component')
class RootComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @state() data: ContentData = ContentData.create()
  @state() tabs: TabData[] = tabList

  constructor () {
    super()
    aeonState.tab_bar.active.addEventListener(this.#onSwitched.bind(this))
    aeonState.tab_bar.pinned.addEventListener(this.#onPinned.bind(this))
    aeonState.tab_bar.active.refresh()
    aeonState.tab_bar.pinned.refresh()
    this.addEventListener('update-data', this.updateData)
    this.addEventListener('update-function', this.focusFunction)
    this.addEventListener('rename-variable', this.renameVariable)
    this.addEventListener('focus-variable', this.focusVariable)
    this.addEventListener('remove-element', (e) => { void this.removeVariable(e) })

    this.data = this.data.copy({ regulations: this.loadCachedEdges(), variables: this.loadCachedNodes() })
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

  private updateData (event: Event): void {
    this.data = (event as CustomEvent).detail
    if (this.data.variables.length > 0) {
      localStorage.setItem(SAVE_VARIABLES, JSON.stringify(this.data.variables))
    }
    if (this.data.regulations.length > 0) {
      localStorage.setItem(SAVE_REGULATIONS, JSON.stringify(this.data.regulations))
    }
    console.log('data updated', this.data)
  }

  private focusVariable (event: Event): void {
    // aeonState.tab_bar.active.emitValue(0)
    this.shadowRoot?.querySelector('#regulations')
      ?.shadowRoot?.querySelector('regulations-editor')
      ?.dispatchEvent(new CustomEvent('focus-variable', {
        detail: {
          variableId: (event as CustomEvent).detail.variableId
        }
      }))
  }

  private renameVariable (event: Event): void {
    const details = (event as CustomEvent).detail
    this.shadowRoot?.querySelector('#regulations')
      ?.shadowRoot?.querySelector('regulations-editor')
      ?.dispatchEvent(new CustomEvent('rename-variable', {
        detail: {
          variableId: details.variableId,
          nodeName: details.nodeName
        }
      }))
  }

  private async removeVariable (event: Event): Promise<void> {
    if (!await dialog.ask('Are you sure?', {
      type: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Keep',
      title: 'Delete'
    })) return
    const variableId = (event as CustomEvent).detail.id
    this.data = ContentData.create({
      variables: this.data.variables.filter((variable) => variable.id !== variableId),
      regulations: this.data.regulations.filter((regulation) => regulation.source !== variableId && regulation.target !== variableId)
    })

    // TODO: fix duplicate deletion (make separate event for functions?)
    this.data = ContentData.create({
      variables: this.data.variables,
      regulations: this.data.regulations.filter((regulation) => regulation.id !== variableId)
    })
  }

  private adjustRegEditor (): void {
    if (window.outerWidth <= 800) return
    // a bit of messy solution but should eventually go through backend
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
    this.shadowRoot?.querySelector('#functions')
      ?.shadowRoot?.querySelector('functions-editor')
      ?.dispatchEvent(new CustomEvent('focus-function', {
        detail: {
          variableId: (event as CustomEvent).detail.variableId
        }
      }))
  }

  private visibleTabs (): TabData[] {
    return this.tabs.sort((a, b) => a.id - b.id).filter((tab) => tab.pinned || tab.active)
  }

  loadCachedNodes (): IVariableData[] {
    try {
      const parsed = (JSON.parse(localStorage.getItem(SAVE_VARIABLES) ?? '[]') as IVariableData[])
      console.log('nodes loaded')
      return parsed
    } catch (e) {
      return []
    }
  }

  loadCachedEdges (): IRegulationData[] {
    try {
      const parsed = (JSON.parse(localStorage.getItem(SAVE_REGULATIONS) ?? '[]') as IRegulationData[])
      console.log('edges loaded')
      return parsed
    } catch (e) {
      return []
    }
  }

  render (): TemplateResult {
    const visibleTabs = this.visibleTabs()
    return html`
      <div class="root-component">
        <nav-bar .tabs=${this.tabs}></nav-bar>
        <div class="content uk-flex uk-flex-row uk-flex-stretch uk-flex-wrap-stretch">
          ${map(this.tabs, (tab) => html`
            <content-pane id="${tab.name.toLowerCase()}" ?hidden="${!(tab.pinned || tab.active)}"
                          class="uk-width-1-${visibleTabs.length} ${tab.active ? 'active' : 'inactive'}" .tab=${tab}
                          .data=${this.data}></content-pane>
          `)}
        </div>
      </div>
    `
  }
}
