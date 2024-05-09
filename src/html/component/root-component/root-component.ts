import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, state, property } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './root-component.less?inline'
import '../content-pane/content-pane'
import '../nav-bar/nav-bar'
import { type TabData } from '../../util/tab-data'
import {
  aeonState,
  type LayoutNodeData,
  type LayoutNodeDataPrototype,
  type ModelData,
  type RegulationData,
  type SketchData,
  type UninterpretedFnData,
  type VariableData,
  type VariableIdUpdateData
} from '../../../aeon_events'
import { tabList } from '../../util/config'
import {
  ContentData,
  type IFunctionData,
  type ILayoutData, type IObservationSet,
  type IRegulationData,
  type IVariableData,
  type DynamicProperty,
  type StaticProperty
} from '../../util/data-interfaces'
import { dialog } from '@tauri-apps/api'
import { dummyData } from '../../util/dummy-data'
import { getNextEssentiality, getNextMonotonicity } from '../../util/utilities'

const LAYOUT = 'default'

@customElement('root-component')
export default class RootComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() data: ContentData = ContentData.create()
  @state() tabs: TabData[] = tabList

  constructor () {
    super()

    // error event listener
    aeonState.error.errorReceived.addEventListener((e) => {
      void this.#onErrorMessage(e)
    })

    // tab bar event listeners
    aeonState.tabBar.active.addEventListener(this.#onSwitched.bind(this))
    aeonState.tabBar.pinned.addEventListener(this.#onPinned.bind(this))
    aeonState.tabBar.active.refresh()
    aeonState.tabBar.pinned.refresh()

    // model editor related event listeners
    this.addEventListener('load-dummy', () => { void this.loadDummy() })
    this.addEventListener('focus-function', this.focusFunction)
    this.addEventListener('add-variable', this.addVariable)
    aeonState.sketch.model.variableCreated.addEventListener(this.#onVariableCreated.bind(this))
    this.addEventListener('add-regulation', this.addRegulation)
    aeonState.sketch.model.regulationCreated.addEventListener(this.#onRegulationCreated.bind(this))
    this.addEventListener('set-update-function-expression', this.setVariableFunction)
    aeonState.sketch.model.variableUpdateFnChanged.addEventListener(this.#onUpdateFnChanged.bind(this))
    this.addEventListener('rename-variable', this.renameVariable)
    aeonState.sketch.model.variableNameChanged.addEventListener(this.#onVariableNameChanged.bind(this))
    this.addEventListener('change-node-position', this.changeNodePosition)
    aeonState.sketch.model.nodePositionChanged.addEventListener(this.#onNodePositionChanged.bind(this))
    this.addEventListener('set-variable-id', this.setVariableId)
    aeonState.sketch.model.variableIdChanged.addEventListener(this.#onVariableIdChanged.bind(this))
    this.addEventListener('toggle-regulation-essential', this.toggleRegulationEssentiality)
    aeonState.sketch.model.regulationEssentialityChanged.addEventListener(this.#regulationEssentialityChanged.bind(this))
    this.addEventListener('toggle-regulation-monotonicity', this.toggleRegulationMonotonicity)
    aeonState.sketch.model.regulationSignChanged.addEventListener(this.#onRegulationMonotonicityChanged.bind(this))
    this.addEventListener('remove-variable', (e) => {
      void this.removeVariable(e)
    })
    aeonState.sketch.model.variableRemoved.addEventListener(this.#onVariableRemoved.bind(this))
    this.addEventListener('remove-regulation', (e) => { void this.removeRegulation(e) })
    aeonState.sketch.model.regulationRemoved.addEventListener(this.#onRegulationRemoved.bind(this))

    // refresh-event listeners
    aeonState.sketch.model.modelRefreshed.addEventListener(this.#onModelRefreshed.bind(this))
    aeonState.sketch.model.variablesRefreshed.addEventListener(this.#onVariablesRefreshed.bind(this))
    aeonState.sketch.model.layoutNodesRefreshed.addEventListener(this.#onLayoutNodesRefreshed.bind(this))
    aeonState.sketch.model.regulationsRefreshed.addEventListener(this.#onRegulationsRefreshed.bind(this))
    // when refreshing/replacing whole sketch, this component is responsible for updating the `Model` part
    aeonState.sketch.sketchRefreshed.addEventListener(this.#onSketchRefreshed.bind(this))
    aeonState.sketch.sketchReplaced.addEventListener(this.#onSketchRefreshed.bind(this))

    // event listener to capture changes from sub-modules (FunctionEditor, ObservationEditor, or PropertiesEditor)
    // with updated uninterpreted functions
    this.addEventListener('save-functions', this.saveFunctionData.bind(this))
    this.addEventListener('save-observations', this.saveObservationData.bind(this))
    this.addEventListener('save-dynamic-properties', this.saveDynamicPropertyData.bind(this))
    this.addEventListener('save-static-properties', this.saveStaticPropertyData.bind(this))

    // refreshing content from backend
    aeonState.sketch.model.refreshModel()
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
    const functions: IFunctionData[] = (event as CustomEvent).detail.functions
    this.saveFunctions(functions)
  }

  saveObservationData (event: Event): void {
    // update observations using modified data propagated from ObservationsEditor
    const datasets: IObservationSet[] = (event as CustomEvent).detail.datasets
    this.data = this.data.copy({ observations: datasets })
  }

  saveStaticPropertyData (event: Event): void {
    // update properties using modified data propagated from PropertyEditor
    const properties: StaticProperty[] = (event as CustomEvent).detail.staticProperties
    this.data = this.data.copy({ staticProperties: properties })
  }

  saveDynamicPropertyData (event: Event): void {
    // update properties using modified data propagated from PropertyEditor
    const properties: DynamicProperty[] = (event as CustomEvent).detail.dynamicProperties
    this.data = this.data.copy({ dynamicProperties: properties })
  }

  private saveFunctions (functions: IFunctionData[]): void {
    functions.sort((a, b) => (a.id > b.id ? 1 : -1))
    this.data = this.data.copy({ functions })
  }

  private saveVariables (variables: IVariableData[]): void {
    variables.sort((a, b) => (a.id > b.id ? 1 : -1))
    this.data = this.data.copy({ variables })
  }

  private saveRegulations (regulations: IRegulationData[]): void {
    regulations.sort((a, b) => (a.source + a.target > b.source + b.target ? 1 : -1))
    this.data = this.data.copy({ regulations })
  }

  private saveLayout (layout: ILayoutData): void {
    this.data = this.data.copy({ layout })
  }

  renameVariable (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.sketch.model.setVariableName(details.id, details.name)
  }

  #onVariableNameChanged (data: VariableData): void {
    const variables = [...this.data.variables]
    const variableIndex = variables.findIndex(variable => variable.id === data.id)
    variables[variableIndex] = {
      ...variables[variableIndex],
      id: data.id,
      name: data.name
    }
    this.saveVariables(variables)
  }

  private addVariable (event: Event): void {
    const details = (event as CustomEvent).detail
    const position: LayoutNodeDataPrototype = {
      layout: LAYOUT,
      px: details.position.x,
      py: details.position.y
    }
    aeonState.sketch.model.addVariable(details.id, details.name, position)
  }

  #onVariableCreated (data: VariableData): void {
    const variables = [...this.data.variables]
    variables.push({
      id: data.id,
      name: data.name,
      function: ''
    })
    this.saveVariables(variables)
  }

  private addRegulation (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.sketch.model.addRegulation(details.source, details.target, details.monotonicity, details.essential)
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
    this.saveRegulations(regulations)
  }

  private async removeVariable (event: Event): Promise<void> {
    if (!await this.confirmDialog()) return
    const variableId = (event as CustomEvent).detail.id
    aeonState.sketch.model.removeVariable(variableId)
  }

  #onVariableRemoved (data: VariableData): void {
    this.saveVariables(
      this.data.variables.filter((variable) => variable.id !== data.id)
    )
  }

  private adjustRegEditor (): void {
    const visible = this.visibleTabs()
    if (window.outerWidth <= 800 || !visible.includes(this.tabs[0])) return
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

  private changeNodePosition (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.sketch.model.changeNodePosition(LAYOUT, details.id, details.position.x, details.position.y)
  }

  #onNodePositionChanged (data: LayoutNodeData): void {
    // TODO: add support for layouts
    const layout = new Map(this.data.layout)
    layout.set(data.variable, {
      x: data.px,
      y: data.py
    })
    this.saveLayout(layout)
  }

  private setVariableId (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.sketch.model.setVariableId(details.oldId, details.newId)
  }

  #onVariableIdChanged (data: VariableIdUpdateData): void {
    // we need to refresh all the affected components - that can be any update function, any regulation,
    // and then the variable itself and its layout node
    // TODO: od this more efficiently (but still on backend)
    this.data.layout.set(data.new_id, this.data.layout.get(data.original_id) ?? { x: 0, y: 0 })
    this.data.layout.delete(data.original_id)
    aeonState.sketch.model.refreshVariables()
    aeonState.sketch.model.refreshRegulations()

    /*
    // older version that partially changes components directly follows
    const variableIndex = this.data.variables.findIndex((variable) => variable.id === data.original_id)
    if (variableIndex === -1) return
    const variables = [...this.data.variables]
    variables[variableIndex] = {
      ...variables[variableIndex],
      id: data.new_id
    }

    const regulations = [...this.data.regulations]
    regulations.forEach((reg, index) => {
      if (reg.source === data.original_id) {
        regulations[index].source = data.new_id
      }
      if (reg.target === data.original_id) {
        regulations[index].target = data.new_id
      }
    })
    this.saveVariables(variables)
     */
  }

  private toggleRegulationEssentiality (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.sketch.model.setRegulationEssentiality(details.source, details.target, getNextEssentiality(details.essential))
  }

  #regulationEssentialityChanged (data: RegulationData): void {
    const index = this.data.regulations.findIndex((reg) => reg.source === data.regulator && reg.target === data.target)
    if (index === -1) return
    const regulations = [...this.data.regulations]
    regulations[index] = {
      ...regulations[index],
      essential: data.essential
    }
    this.saveRegulations(regulations)
  }

  private toggleRegulationMonotonicity (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.sketch.model.setRegulationSign(details.source, details.target, getNextMonotonicity(details.monotonicity))
  }

  #onRegulationMonotonicityChanged (data: RegulationData): void {
    const index = this.data.regulations.findIndex((reg) => reg.source === data.regulator && reg.target === data.target)
    if (index === -1) return
    const regulations = [...this.data.regulations]
    regulations[index] = {
      ...regulations[index],
      monotonicity: data.sign
    }
    this.saveRegulations(regulations)
  }

  private setVariableFunction (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.sketch.model.setVariableUpdateFn(details.id, details.function)
  }

  #onUpdateFnChanged (data: VariableData): void {
    const variableIndex = this.data.variables.findIndex(variable => variable.id === data.id)
    if (variableIndex === -1) return
    const variables = [...this.data.variables]
    variables[variableIndex] = {
      ...variables[variableIndex],
      function: data.update_fn
    }
    this.saveVariables(variables)
  }

  private async removeRegulation (event: Event): Promise<void> {
    if (!await this.confirmDialog()) return
    const details = (event as CustomEvent).detail
    aeonState.sketch.model.removeRegulation(details.source, details.target)
  }

  #onRegulationRemoved (data: RegulationData): void {
    this.saveRegulations(
      this.data.regulations.filter((regulation) => regulation.source !== data.regulator || regulation.target !== data.target)
    )
  }

  private convertToIFunction (fnData: UninterpretedFnData): IFunctionData {
    const variables = fnData.arguments.map(
      (arg, index) => {
        return {
          id: index.toString(),
          source: 'var' + index.toString(),
          target: fnData.id,
          monotonicity: arg[0],
          essential: arg[1]
        }
      })
    return {
      id: fnData.id,
      function: fnData.expression,
      variables
    }
  }

  #onSketchRefreshed (sketch: SketchData): void {
    // when refreshing/replacing whole sketch, this component is responsible for updating the `Model` part
    this.#onModelRefreshed(sketch.model)
  }

  #onModelRefreshed (model: ModelData): void {
    const functions = model.uninterpreted_fns.map((data): IFunctionData => {
      return this.convertToIFunction(data)
    })
    this.saveFunctions(functions)

    this.#onVariablesRefreshed(model.variables)
    this.#onRegulationsRefreshed(model.regulations)
    // TODO: this will have to change if more layouts will be possible
    this.#onLayoutNodesRefreshed(model.layouts[0].nodes)
  }

  #onVariablesRefreshed (variables: VariableData[]): void {
    this.saveVariables(
      variables.map(v => { return { ...v, function: v.update_fn } })
    )
  }

  #onLayoutNodesRefreshed (layoutNodes: LayoutNodeData[]): void {
    const layout: ILayoutData = new Map()
    layoutNodes.forEach(layoutNode => {
      layout.set(layoutNode.variable, { x: layoutNode.px, y: layoutNode.py })
    })
    this.saveLayout(layout)
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
    this.saveRegulations(regs)
  }

  async loadDummy (): Promise<void> {
    // remove existing data and load dummy data

    // 1) remove update/uninterpreted fn expressions (so that we can safely remove variables, functions)
    this.data.variables.forEach((variable) => {
      aeonState.sketch.model.setVariableUpdateFn(variable.id, '')
    })
    this.data.functions.forEach((fn) => {
      aeonState.sketch.model.setUninterpretedFnExpression(fn.id, '')
    })
    await new Promise(_resolve => setTimeout(_resolve, 250))
    // 2) remove regulations
    this.data.regulations.forEach((reg) => {
      aeonState.sketch.model.removeRegulation(reg.source, reg.target)
    })
    await new Promise(_resolve => setTimeout(_resolve, 250))
    // 3) finally remove uninterpreted functions and variables
    this.data.functions.forEach((fn) => {
      aeonState.sketch.model.removeUninterpretedFn(fn.id)
    })
    this.data.variables.forEach((variable) => {
      aeonState.sketch.model.removeVariable(variable.id)
    })
    await new Promise(_resolve => setTimeout(_resolve, 250))

    // now we can saload the dummy data
    dummyData.variables.forEach((variable) => {
      aeonState.sketch.model.addVariable(variable.id, variable.name, {
        layout: LAYOUT,
        px: (dummyData.layout.get(variable.id)?.x) ?? 0,
        py: (dummyData.layout.get(variable.id)?.y) ?? 0
      })
    })
    dummyData.functions.forEach((f) => {
      aeonState.sketch.model.addUninterpretedFn(f.id, f.variables.length)
    })
    await new Promise(_resolve => setTimeout(_resolve, 250))
    dummyData.regulations.forEach((regulation) => {
      aeonState.sketch.model.addRegulation(regulation.source, regulation.target, regulation.monotonicity, regulation.essential)
    })
    dummyData.variables.forEach((variable) => {
      aeonState.sketch.model.setVariableUpdateFn(variable.id, variable.function)
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
