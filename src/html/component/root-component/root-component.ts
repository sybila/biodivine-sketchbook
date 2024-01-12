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
import { dummyData } from '../../util/dummy-data'

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
    this.addEventListener('load-dummy', () => { this.saveData(dummyData.variables, dummyData.regulations) })
    this.addEventListener('focus-function', this.focusFunction)
    this.addEventListener('focus-variable', this.focusVariable)
    this.addEventListener('add-variable', this.addVariable)
    this.addEventListener('add-regulation', this.addRegulation)
    this.addEventListener('update-variable', this.updateVariable)
    this.addEventListener('update-regulation', this.updateRegulation)
    this.addEventListener('remove-element', (e) => { void this.removeVariable(e) })

    this.data = this.data.copy({ variables: this.loadCachedNodes(), regulations: this.loadCachedEdges() })
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

  private updateRegulation (event: Event): void {
    const regulationId = (event as CustomEvent).detail.id
    const index = this.data.regulations.findIndex((reg) => reg.id === regulationId)
    if (index === -1) return
    const regulations = [...this.data.regulations]
    regulations[index] = {
      ...regulations[index],
      observable: (event as CustomEvent).detail.observable,
      monotonicity: (event as CustomEvent).detail.monotonicity
    }
    this.saveData(this.data.variables, regulations)
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

  private updateVariable (event: Event): void {
    const details = (event as CustomEvent).detail
    const variables = [...this.data.variables]
    const regulations = [...this.data.regulations]
    const variableIndex = variables.findIndex(variable => variable.id === details.oldId)
    variables[variableIndex] = {
      ...variables[variableIndex],
      id: details.id,
      name: details.name,
      function: details.function,
      position: details.position
    }
    if (details.oldId !== details.id) {
      this.data.regulations.forEach((_, i) => {
        if (this.data.regulations[i].source === details.oldId) {
          regulations[i].source = details.id
        }
        if (this.data.regulations[i].target === details.oldId) {
          regulations[i].target = details.id
        }
      })
    }
    this.saveData(variables, regulations)
  }

  private addVariable (event: Event): void {
    const details = (event as CustomEvent).detail
    const variables = [...this.data.variables]
    variables.push({
      id: details.id,
      name: details.name,
      position: details.position,
      function: details.function ?? ''
    })
    this.saveData(variables, this.data.regulations)
  }

  private addRegulation (event: Event): void {
    const details = (event as CustomEvent).detail
    const regulations = [...this.data.regulations]
    regulations.push({
      id: details.id,
      source: details.source,
      target: details.target,
      observable: details.observable,
      monotonicity: details.monotonicity
    })
    this.saveData(this.data.variables, regulations)
  }

  private async removeVariable (event: Event): Promise<void> {
    if (!await dialog.ask('Are you sure?', {
      type: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Keep',
      title: 'Delete'
    })) return
    const variableId = (event as CustomEvent).detail.id
    this.saveData(
      this.data.variables.filter((variable) => variable.id !== variableId),
      this.data.regulations.filter((regulation) => regulation.source !== variableId && regulation.target !== variableId && regulation.id !== variableId)
    )
  }

  private adjustRegEditor (): void {
    if (window.outerWidth <= 800) return
    // TODO: a bit of messy solution but should eventually go through backend
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
    // TODO: a bit of messy solution but should eventually go through backend
    this.shadowRoot?.querySelector('#functions')
      ?.shadowRoot?.querySelector('functions-editor')
      ?.dispatchEvent(new CustomEvent('focus-function-field', {
        detail: {
          variableId: (event as CustomEvent).detail.variableId
        }
      }))
  }

  private visibleTabs (): TabData[] {
    return this.tabs.sort((a, b) => a.id - b.id).filter((tab) => tab.pinned || tab.active)
  }

  private saveData (variables: IVariableData[], regulations: IRegulationData[]): void {
    // sort nodes to keep alphabetical order in lists
    variables.sort((a, b) => (a.id > b.id ? 1 : -1))
    regulations.sort((a, b) => (a.source + a.target > b.source + b.target ? 1 : -1))

    localStorage.setItem(SAVE_VARIABLES, JSON.stringify(variables))
    localStorage.setItem(SAVE_REGULATIONS, JSON.stringify(regulations))
    this.data = ContentData.create({
      variables,
      regulations
    })
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
