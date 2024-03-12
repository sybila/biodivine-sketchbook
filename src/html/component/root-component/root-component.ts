import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, state, property } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './root-component.less?inline'
import '../content-pane/content-pane'
import '../nav-bar/nav-bar'
import { type TabData } from '../../util/tab-data'
import {
  aeonState,
  type LayoutNodeData, type LayoutNodeDataPrototype,
  type RegulationData,
  type UpdateFnData,
  type VariableData,
  type VariableIdUpdateData
} from '../../../aeon_events'
import { tabList } from '../../util/config'
import {
  ContentData,
  type ILayoutData,
  type IRegulationData,
  type IVariableData
} from '../../util/data-interfaces'
import { dialog } from '@tauri-apps/api'
import { dummyData } from '../../util/dummy-data'
import { getNextEssentiality, getNextMonotonicity } from '../../util/utilities'

const LAYOUT = 'default'

@customElement('root-component')
class RootComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() data: ContentData = ContentData.create()
  @state() tabs: TabData[] = tabList

  constructor () {
    super()
    aeonState.error.errorReceived.addEventListener((e) => {
      void this.#onErrorMessage(e)
    })
    aeonState.tabBar.active.addEventListener(this.#onSwitched.bind(this))
    aeonState.tabBar.pinned.addEventListener(this.#onPinned.bind(this))
    aeonState.tabBar.active.refresh()
    aeonState.tabBar.pinned.refresh()
    this.addEventListener('load-dummy', () => { void this.loadDummy() })
    this.addEventListener('focus-function', this.focusFunction)
    this.addEventListener('add-variable', this.addVariable)
    aeonState.model.variableCreated.addEventListener(this.#onVariableCreated.bind(this))
    this.addEventListener('add-regulation', this.addRegulation)
    aeonState.model.regulationCreated.addEventListener(this.#onRegulationCreated.bind(this))
    this.addEventListener('set-update-function-expression', this.setVariableFunction)
    aeonState.model.updateFnExpressionChanged.addEventListener(this.#onUpdateFnChanged.bind(this))
    this.addEventListener('rename-variable', this.renameVariable)
    aeonState.model.variableNameChanged.addEventListener(this.#onVariableNameChanged.bind(this))
    this.addEventListener('change-node-position', this.changeNodePosition)
    aeonState.model.nodePositionChanged.addEventListener(this.#onNodePositionChanged.bind(this))
    this.addEventListener('set-variable-id', this.setVariableId)
    aeonState.model.variableIdChanged.addEventListener(this.#onVariableIdChanged.bind(this))
    this.addEventListener('toggle-regulation-essential', this.toggleRegulationEssentiality)
    aeonState.model.regulationEssentialityChanged.addEventListener(this.#regulationEssentialityChanged.bind(this))
    this.addEventListener('toggle-regulation-monotonicity', this.toggleRegulationMonotonicity)
    aeonState.model.regulationSignChanged.addEventListener(this.#onRegulationMonotonicityChanged.bind(this))
    this.addEventListener('remove-variable', (e) => {
      void this.removeVariable(e)
    })
    aeonState.model.variableRemoved.addEventListener(this.#onVariableRemoved.bind(this))
    this.addEventListener('remove-regulation', (e) => { void this.removeRegulation(e) })
    aeonState.model.regulationRemoved.addEventListener(this.#onRegulationRemoved.bind(this))

    aeonState.model.variablesRefreshed.addEventListener(this.#onVariablesRefreshed.bind(this))
    aeonState.model.layoutNodesRefreshed.addEventListener(this.#onLayoutNodesRefreshed.bind(this))
    aeonState.model.regulationsRefreshed.addEventListener(this.#onRegulationsRefreshed.bind(this))

    aeonState.model.refreshVariables()
    aeonState.model.refreshLayoutNodes(LAYOUT)
    aeonState.model.refreshRegulations()
    aeonState.model.refreshUninterpretedFns()

    // event from FunctionEditor with new functions
    this.addEventListener('save-functions', this.saveFunctionData.bind(this))
  }

  async #onErrorMessage (errorMessage: string): Promise<void> {
    await dialog.message(errorMessage, {
      type: 'error',
      title: 'Error'
    })
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

  saveFunctionData (event: Event): void {
    // update functions using modified data propagated from FunctionsEditor
    const details = (event as CustomEvent).detail
    this.data = ContentData.create({
      variables: this.data.variables,
      regulations: this.data.regulations,
      layout: this.data.layout,
      functions: details.functions
    })
  }

  renameVariable (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.model.setVariableName(details.id, details.name)
  }

  #onVariableNameChanged (data: VariableData): void {
    const variables = [...this.data.variables]
    const variableIndex = variables.findIndex(variable => variable.id === data.id)
    variables[variableIndex] = {
      ...variables[variableIndex],
      id: data.id,
      name: data.name
    }
    this.saveData(variables, this.data.regulations, this.data.layout)
  }

  private addVariable (event: Event): void {
    const details = (event as CustomEvent).detail
    const position: LayoutNodeDataPrototype = {
      layout: LAYOUT,
      px: details.position.x,
      py: details.position.y
    }
    aeonState.model.addVariable(details.id, details.name, position)
  }

  #onVariableCreated (data: VariableData): void {
    const variables = [...this.data.variables]
    variables.push({
      id: data.id,
      name: data.name,
      function: ''
    })
    this.saveData(variables, this.data.regulations, this.data.layout)
  }

  private addRegulation (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.model.addRegulation(details.source, details.target, details.monotonicity, details.essential)
  }

  #onRegulationCreated (data: RegulationData): void {
    const regulations = [...this.data.regulations]
    regulations.push({
      id: data.regulator + data.target,
      source: data.regulator,
      target: data.target,
      essential: data.essential,
      monotonicity: data.sign
    })
    this.saveData(this.data.variables, regulations, this.data.layout)
  }

  private async removeVariable (event: Event): Promise<void> {
    if (!await this.confirmDialog()) return
    const variableId = (event as CustomEvent).detail.id
    aeonState.model.removeVariable(variableId)
  }

  #onVariableRemoved (data: VariableData): void {
    this.saveData(
      this.data.variables.filter((variable) => variable.id !== data.id),
      this.data.regulations,
      this.data.layout
    )
  }

  private adjustRegEditor (): void {
    const visible = this.visibleTabs()
    if (window.outerWidth <= 800 || visible.includes(this.tabs[0])) return
    window.dispatchEvent(new CustomEvent('adjust-graph', {
      detail: {
        tabCount: visible.length
      }
    }))
  }

  private focusFunction (event: Event): void {
    aeonState.tabBar.active.emitValue(1)
    window.dispatchEvent(new CustomEvent('focus-function-field', {
      detail: {
        variableId: (event as CustomEvent).detail.variableId
      }
    }))
  }

  private visibleTabs (): TabData[] {
    return this.tabs.sort((a, b) => a.id - b.id).filter((tab) => tab.pinned || tab.active)
  }

  private saveData (variables: IVariableData[], regulations: IRegulationData[], layout: ILayoutData): void {
    // save variable/regulation/layout data, leave functions as is

    // sort nodes to keep alphabetical order in lists
    variables.sort((a, b) => (a.id > b.id ? 1 : -1))
    regulations.sort((a, b) => (a.source + a.target > b.source + b.target ? 1 : -1))

    this.data = ContentData.create({
      variables,
      regulations,
      layout,
      functions: this.data.functions
    })
  }

  private changeNodePosition (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.model.changeNodePosition(LAYOUT, details.id, details.position.x, details.position.y)
  }

  #onNodePositionChanged (data: LayoutNodeData): void {
    // TODO: add support for layouts
    this.data.layout.set(data.variable, {
      x: data.px,
      y: data.py
    })
    this.saveData(this.data.variables, this.data.regulations, this.data.layout)
  }

  private setVariableId (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.model.setVariableId(details.oldId, details.newId)
  }

  #onVariableIdChanged (data: VariableIdUpdateData): void {
    const variableIndex = this.data.variables.findIndex((variable) => variable.id === data.original_id)
    if (variableIndex === -1) return
    const variables = [...this.data.variables]
    variables[variableIndex] = {
      ...variables[variableIndex],
      id: data.new_id
    }
    // TODO: should this be calculated on FE?
    this.data.layout.set(data.new_id, this.data.layout.get(data.original_id) ?? { x: 0, y: 0 })
    this.data.layout.delete(data.original_id)
    const regulations = [...this.data.regulations]
    regulations.forEach((reg, index) => {
      if (reg.source === data.original_id) {
        regulations[index].source = data.new_id
      }
      if (reg.target === data.original_id) {
        regulations[index].target = data.new_id
      }
    })
    this.saveData(variables, this.data.regulations, this.data.layout)
  }

  private toggleRegulationEssentiality (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.model.setRegulationEssentiality(details.source, details.target, getNextEssentiality(details.essential))
  }

  #regulationEssentialityChanged (data: RegulationData): void {
    const index = this.data.regulations.findIndex((reg) => reg.source === data.regulator && reg.target === data.target)
    if (index === -1) return
    const regulations = [...this.data.regulations]
    regulations[index] = {
      ...regulations[index],
      essential: data.essential
    }
    this.saveData(this.data.variables, regulations, this.data.layout)
  }

  private toggleRegulationMonotonicity (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.model.setRegulationSign(details.source, details.target, getNextMonotonicity(details.monotonicity))
  }

  #onRegulationMonotonicityChanged (data: RegulationData): void {
    const index = this.data.regulations.findIndex((reg) => reg.source === data.regulator && reg.target === data.target)
    if (index === -1) return
    const regulations = [...this.data.regulations]
    regulations[index] = {
      ...regulations[index],
      monotonicity: data.sign
    }
    this.saveData(this.data.variables, regulations, this.data.layout)
  }

  private setVariableFunction (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.model.setUpdateFnExpression(details.id, details.function)
  }

  #onUpdateFnChanged (data: UpdateFnData): void {
    const variableIndex = this.data.variables.findIndex(variable => variable.id === data.var_id)
    if (variableIndex === -1) return
    const variables = [...this.data.variables]
    variables[variableIndex] = {
      ...variables[variableIndex],
      function: data.expression
    }
    this.saveData(variables, this.data.regulations, this.data.layout)
  }

  private async removeRegulation (event: Event): Promise<void> {
    if (!await this.confirmDialog()) return
    const details = (event as CustomEvent).detail
    aeonState.model.removeRegulation(details.source, details.target)
  }

  #onRegulationRemoved (data: RegulationData): void {
    this.saveData(
      this.data.variables,
      this.data.regulations.filter((regulation) => regulation.source !== data.regulator || regulation.target !== data.target),
      this.data.layout
    )
  }

  #onVariablesRefreshed (variables: VariableData[]): void {
    this.saveData(variables.map(v => { return { ...v, function: '' } }), this.data.regulations, this.data.layout)
  }

  #onLayoutNodesRefreshed (layoutNodes: LayoutNodeData[]): void {
    const layout: ILayoutData = new Map()
    layoutNodes.forEach(layoutNode => {
      layout.set(layoutNode.variable, { x: layoutNode.px, y: layoutNode.py })
    })
    this.saveData(this.data.variables, this.data.regulations, layout)
  }

  #onRegulationsRefreshed (regulations: RegulationData[]): void {
    const regs = regulations.map((data): IRegulationData => {
      return {
        id: data.regulator + data.target,
        source: data.regulator,
        target: data.target,
        essential: data.essential,
        monotonicity: data.sign
      }
    })
    this.saveData(this.data.variables, regs, this.data.layout)
  }

  async loadDummy (): Promise<void> {
    // remove existing data and load dummy data

    // 1) remove update/uninterpreted fn expressions (so that we can safely remove variables, functions)
    this.data.variables.forEach((variable) => {
      aeonState.model.setUpdateFnExpression(variable.id, '')
    })
    this.data.functions.forEach((fn) => {
      aeonState.model.setUninterpretedFnExpression(fn.id, '')
    })
    await new Promise(_resolve => setTimeout(_resolve, 250))
    // 2) remove regulations
    this.data.regulations.forEach((reg) => {
      aeonState.model.removeRegulation(reg.source, reg.target)
    })
    await new Promise(_resolve => setTimeout(_resolve, 250))
    // 3) finally remove uninterpreted functions and variables
    this.data.functions.forEach((fn) => {
      aeonState.model.removeUninterpretedFn(fn.id)
    })
    this.data.variables.forEach((variable) => {
      aeonState.model.removeVariable(variable.id)
    })
    await new Promise(_resolve => setTimeout(_resolve, 250))

    // now we can saload the dummy data
    dummyData.variables.forEach((variable) => {
      aeonState.model.addVariable(variable.id, variable.name, {
        layout: LAYOUT,
        px: (dummyData.layout.get(variable.id)?.x) ?? 0,
        py: (dummyData.layout.get(variable.id)?.y) ?? 0
      })
    })
    dummyData.functions.forEach((f) => {
      aeonState.model.addUninterpretedFn(f.id, f.variables.length)
    })
    await new Promise(_resolve => setTimeout(_resolve, 250))
    dummyData.regulations.forEach((regulation) => {
      aeonState.model.addRegulation(regulation.source, regulation.target, regulation.monotonicity, regulation.essential)
    })
    dummyData.variables.forEach((variable) => {
      this.dispatchEvent(new CustomEvent('set-update-function-expression', {
        detail: {
          id: variable.id,
          function: variable.function
        }
      }))
    })
    await new Promise(_resolve => setTimeout(_resolve, 250))
  }

  private async confirmDialog (): Promise<boolean> {
    return await dialog.ask('Are you sure?', {
      type: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Keep',
      title: 'Delete'
    })
  }

  render (): TemplateResult {
    const visibleTabs = this.visibleTabs()
    return html`
      <div class="root-component">
        <div class="header">
          <nav-bar .tabs=${this.tabs}></nav-bar>
        </div>
        <div class="content">
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
