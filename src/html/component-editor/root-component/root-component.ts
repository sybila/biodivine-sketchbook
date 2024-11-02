import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, state, property } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './root-component.less?inline'
import '../content-pane/content-pane'
import '../nav-bar/nav-bar'
import '../initial-screen/initial-screen'
import { type TabData } from '../../util/tab-data'
import {
  aeonState, type LayoutNodeData, type LayoutNodeDataPrototype,
  type ModelData, type RegulationData, type SketchData, type VariableData
} from '../../../aeon_state'
import { tabList } from '../../util/config'
import {
  ContentData, type IFunctionData, type ILayoutData,
  type IObservationSet, type IRegulationData, type IVariableData,
  type DynamicProperty, type StaticProperty
} from '../../util/data-interfaces'
import { open } from '@tauri-apps/api/dialog'
import { dialog } from '@tauri-apps/api'
import {
  getNextEssentiality, getNextMonotonicity,
  convertToIFunction, convertToILayout, convertToIVariable,
  convertToIObservationSet, convertToIRegulation
} from '../../util/utilities'
import { resolveResource } from '@tauri-apps/api/path'

const exampleModelPath = await resolveResource('resources/tlgl.json')

const LAYOUT = 'default'

@customElement('root-component')
export default class RootComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() data: ContentData = ContentData.create()
  @state() tabs: TabData[] = tabList
  @state() editorStarted: boolean = false

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

    // window focus event listeners
    window.addEventListener('focus-function-field', this.focusFunction.bind(this))
    window.addEventListener('focus-variable', this.focusVariable.bind(this))

    // listeners to events from initial screen
    window.addEventListener('start-new-sketch', this.startNewSketch.bind(this))
    window.addEventListener('start-import-json', (e) => { void this.startImportJson(e) })
    window.addEventListener('start-import-aeon', (e) => { void this.startImportAeon(e) })
    window.addEventListener('start-import-sbml', (e) => { void this.startImportSbml(e) })
    window.addEventListener('start-import-example', this.startImportExample.bind(this))

    // listeners to import events from editor menu
    window.addEventListener('import-json', (e) => { void this.importJson(e) })
    window.addEventListener('import-aeon', (e) => { void this.importAeon(e) })
    window.addEventListener('import-sbml', (e) => { void this.importSbml(e) })

    // variable-related events
    this.addEventListener('add-variable', this.addNewVariable)
    aeonState.sketch.model.variableCreated.addEventListener(this.#onVariableCreated.bind(this))
    this.addEventListener('add-regulation', this.addRegulation)
    aeonState.sketch.model.regulationCreated.addEventListener(this.#onRegulationCreated.bind(this))
    this.addEventListener('set-update-function-expression', this.setVariableFunction)
    aeonState.sketch.model.variableUpdateFnChanged.addEventListener(this.#onUpdateFnChanged.bind(this))
    this.addEventListener('set-variable-data', this.setVariableData)
    aeonState.sketch.model.variableDataChanged.addEventListener(this.#onVariableDataChanged.bind(this))
    this.addEventListener('change-node-position', this.changeNodePosition)
    aeonState.sketch.model.nodePositionChanged.addEventListener(this.#onNodePositionChanged.bind(this))
    this.addEventListener('set-variable-id', this.setVariableId)
    // Since variable ID change can affect many parts of the model (update fns, regulations, layout, ...),
    // the event fetches the whole updated model data.
    aeonState.sketch.model.variableIdChanged.addEventListener(this.#onModelRefreshed.bind(this))
    this.addEventListener('remove-variable', (e) => { void this.removeVariable(e) })
    aeonState.sketch.model.variableRemoved.addEventListener(this.#onVariableRemoved.bind(this))

    // regulation-related events
    this.addEventListener('toggle-regulation-essential', this.toggleRegulationEssentiality)
    aeonState.sketch.model.regulationEssentialityChanged.addEventListener(this.#regulationEssentialityChanged.bind(this))
    this.addEventListener('toggle-regulation-monotonicity', this.toggleRegulationMonotonicity)
    aeonState.sketch.model.regulationSignChanged.addEventListener(this.#onRegulationMonotonicityChanged.bind(this))
    this.addEventListener('remove-regulation', (e) => { void this.removeRegulation(e) })
    aeonState.sketch.model.regulationRemoved.addEventListener(this.#onRegulationRemoved.bind(this))

    // listeners for refresh events from backend
    aeonState.sketch.model.modelRefreshed.addEventListener(this.#onModelRefreshed.bind(this))
    aeonState.sketch.model.variablesRefreshed.addEventListener(this.#onVariablesRefreshed.bind(this))
    aeonState.sketch.model.layoutNodesRefreshed.addEventListener(this.#onLayoutNodesRefreshed.bind(this))
    aeonState.sketch.model.regulationsRefreshed.addEventListener(this.#onRegulationsRefreshed.bind(this))
    // when refreshing/replacing whole sketch, this component is responsible for updating the whole `Sketch`, that means
    // updating model components, and distributing the rest (observations, properties) to particular sub-modules
    aeonState.sketch.sketchRefreshed.addEventListener(this.#onSketchRefreshed.bind(this))
    aeonState.sketch.sketchReplaced.addEventListener(this.#onSketchRefreshed.bind(this))

    // event listener to capture changes from sub-modules (FunctionEditor, ObservationEditor, or PropertiesEditor)
    // with updated uninterpreted functions
    this.addEventListener('save-functions', this.saveFunctionData.bind(this))
    this.addEventListener('save-observations', this.saveObservationData.bind(this))
    this.addEventListener('save-dynamic-properties', this.saveDynamicPropertyData.bind(this))
    this.addEventListener('save-static-properties', this.saveStaticPropertyData.bind(this))
    this.addEventListener('save-annotation', this.saveAnnotationData.bind(this))

    // Since function ID change can affect many parts of the model (update fns, other uninterpreted fns, ...),
    // the event fetches the whole updated model data, and we make the change here in the root component.
    aeonState.sketch.model.uninterpretedFnIdChanged.addEventListener(this.#onModelRefreshed.bind(this))

    // load variable editorStarted from session storage (so it survives refresh)
    const storedEditorStarted = sessionStorage.getItem('editorStarted')
    this.editorStarted = storedEditorStarted === 'true'
    if (this.editorStarted) {
      // at the beginning, refresh content of the whole sketch from backend
      aeonState.sketch.refreshSketch()
    }
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

  // utility to set the flag for editor rendering (and save to session storage)
  private startEditor (): void {
    this.editorStarted = true
    sessionStorage.setItem('editorStarted', 'true')
  }

  startNewSketch (_event: Event): void {
    this.startEditor()
  }

  async startImportJson (_event: Event): Promise<void> {
    const success = await this.importJsonInternal()
    if (success) {
      this.startEditor()
    }
  }

  async startImportAeon (_event: Event): Promise<void> {
    const success = await this.importAeonInternal()
    if (success) {
      this.startEditor()
    }
  }

  async startImportSbml (_event: Event): Promise<void> {
    const success = await this.importSbmlInternal()
    if (success) {
      this.startEditor()
    }
  }

  startImportExample (_event: Event): void {
    console.log('importing example model')
    aeonState.sketch.importSketch(exampleModelPath)
    this.startEditor()
  }

  async importJson (_event: Event): Promise<void> {
    await this.importJsonInternal()
  }

  async importAeon (_event: Event): Promise<void> {
    await this.importAeonInternal()
  }

  async importSbml (_event: Event): Promise<void> {
    await this.importSbmlInternal()
  }

  private async importJsonInternal (): Promise<boolean> {
    const selected = await open({
      title: 'Import sketch...',
      multiple: false,
      filters: [{
        name: '*.json',
        extensions: ['json']
      }]
    })
    if (selected === null) return false

    let importFile = ''
    if (Array.isArray(selected)) {
      if (selected.length === 0) return false
      importFile = selected[0]
    } else {
      importFile = selected
    }

    console.log('importing', importFile)
    aeonState.sketch.importSketch(importFile)
    return true
  }

  private async importAeonInternal (): Promise<boolean> {
    const selected = await open({
      title: 'Import aeon model...',
      multiple: false,
      filters: [{
        name: '*.aeon',
        extensions: ['aeon']
      }]
    })
    if (selected === null) return false

    let importFile = ''
    if (Array.isArray(selected)) {
      if (selected.length === 0) return false
      importFile = selected[0]
    } else {
      importFile = selected
    }

    console.log('importing', importFile)
    aeonState.sketch.importAeon(importFile)
    return true
  }

  private async importSbmlInternal (): Promise<boolean> {
    const selected = await open({
      title: 'Import sbml model...',
      multiple: false,
      filters: [{
        name: '*.sbml',
        extensions: ['sbml']
      }]
    })
    if (selected === null) return false

    let importFile = ''
    if (Array.isArray(selected)) {
      if (selected.length === 0) return false
      importFile = selected[0]
    } else {
      importFile = selected
    }

    console.log('importing', importFile)
    aeonState.sketch.importSbml(importFile)
    return true
  }

  saveFunctionData (event: Event): void {
    // update functions using modified data propagated from FunctionsEditor
    const functions: IFunctionData[] = (event as CustomEvent).detail.functions
    this.saveFunctions(functions)
  }

  saveObservationData (event: Event): void {
    // update observations using modified data propagated from ObservationsEditor
    const datasets: IObservationSet[] = (event as CustomEvent).detail.datasets
    this.saveObservations(datasets)
  }

  saveStaticPropertyData (event: Event): void {
    // update properties using modified data propagated from PropertyEditor
    const properties: StaticProperty[] = (event as CustomEvent).detail.staticProperties
    this.saveStaticProperties(properties)
  }

  saveDynamicPropertyData (event: Event): void {
    // update properties using modified data propagated from PropertyEditor
    const properties: DynamicProperty[] = (event as CustomEvent).detail.dynamicProperties
    this.saveDynamicProperties(properties)
  }

  saveAnnotationData (event: Event): void {
    // update annotation propagated from AnnotationTab
    const annotation: string = (event as CustomEvent).detail.annotation
    this.saveAnnotation(annotation)
  }

  private saveDynamicProperties (dynamicProperties: DynamicProperty[]): void {
    dynamicProperties.sort((a, b) => (a.id > b.id ? 1 : -1))
    this.data = this.data.copy({ dynamicProperties })
  }

  private saveStaticProperties (staticProperties: StaticProperty[]): void {
    staticProperties.sort((a, b) => (a.id > b.id ? 1 : -1))
    this.data = this.data.copy({ staticProperties })
  }

  private saveObservations (observations: IObservationSet[]): void {
    observations.sort((a, b) => (a.id > b.id ? 1 : -1))
    this.data = this.data.copy({ observations })
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

  private saveAnnotation (annotation: string): void {
    this.data = this.data.copy({ annotation })
  }

  // Wrapper to save all components of the model at the same time.
  // Saving everything at the same time can help dealing with inconsistencies.
  private saveWholeModel (
    functions: IFunctionData[],
    variables: IVariableData[],
    regulations: IRegulationData[],
    layout: ILayoutData
  ): void {
    functions.sort((a, b) => (a.id > b.id ? 1 : -1))
    variables.sort((a, b) => (a.id > b.id ? 1 : -1))
    regulations.sort((a, b) => (a.source + a.target > b.source + b.target ? 1 : -1))
    this.data = this.data.copy({ functions, variables, regulations, layout })
  }

  // Wrapper to save all components of the model at the same time.
  // Saving everything at the same time can help dealing with inconsistencies.
  private saveWholeSketch (
    functions: IFunctionData[],
    variables: IVariableData[],
    regulations: IRegulationData[],
    layout: ILayoutData,
    observations: IObservationSet[],
    staticProperties: StaticProperty[],
    dynamicProperties: DynamicProperty[],
    annotation: string
  ): void {
    functions.sort((a, b) => (a.id > b.id ? 1 : -1))
    variables.sort((a, b) => (a.id > b.id ? 1 : -1))
    regulations.sort((a, b) => (a.source + a.target > b.source + b.target ? 1 : -1))
    staticProperties.sort((a, b) => (a.id > b.id ? 1 : -1))
    dynamicProperties.sort((a, b) => (a.id > b.id ? 1 : -1))
    observations.sort((a, b) => (a.id > b.id ? 1 : -1))
    this.data = this.data.copy({
      functions,
      variables,
      regulations,
      layout,
      staticProperties,
      dynamicProperties,
      observations,
      annotation
    })
  }

  // Set variable data (currently, sets a name and annotation).
  // The event should have fields 'id' with the variables (unchanged) ID and then (modified)
  // 'name' and 'annotation'.
  setVariableData (event: Event): void {
    const details = (event as CustomEvent).detail
    const variableIndex = this.data.variables.findIndex(v => v.id === details.id)
    if (variableIndex === -1) return

    const varData = {
      id: details.id,
      name: details.name,
      annotation: details.annotation,
      update_fn: this.data.variables[variableIndex].function
    }
    aeonState.sketch.model.setVariableData(details.id, varData)
  }

  #onVariableDataChanged (data: VariableData): void {
    const variables = [...this.data.variables]
    const variableIndex = variables.findIndex(variable => variable.id === data.id)
    if (variableIndex === -1) return

    variables[variableIndex] = {
      ...variables[variableIndex],
      id: data.id,
      name: data.name,
      annotation: data.annotation
    }
    this.saveVariables(variables)
  }

  private addNewVariable (event: Event): void {
    const details = (event as CustomEvent).detail
    const position: LayoutNodeDataPrototype = {
      layout: LAYOUT,
      px: details.position.x,
      py: details.position.y
    }
    aeonState.sketch.model.addDefaultVariable(position)
  }

  #onVariableCreated (data: VariableData): void {
    const variables = [...this.data.variables]
    variables.push({
      id: data.id,
      name: data.name,
      annotation: data.annotation,
      function: data.update_fn
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
    const variableId = (event as CustomEvent).detail.id
    const message = `Removing variable ${variableId} will also erase all its regulations. Do you want to proceed?`
    if (!await this.confirmDeleteDialog(message)) return
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

  private focusFunction (): void {
    aeonState.tabBar.active.emitValue(1)
  }

  private focusVariable (): void {
    aeonState.tabBar.active.emitValue(0)
  }

  private visibleTabs (): TabData[] {
    return this.tabs.sort((a, b) => a.id - b.id).filter((tab) => tab.pinned || tab.active)
  }

  private changeNodePosition (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.sketch.model.changeNodePosition(LAYOUT, details.id, details.position.x, details.position.y)
  }

  #onNodePositionChanged (data: LayoutNodeData): void {
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
    const details = (event as CustomEvent).detail
    aeonState.sketch.model.removeRegulation(details.source, details.target)
  }

  #onRegulationRemoved (data: RegulationData): void {
    this.saveRegulations(
      this.data.regulations.filter((regulation) => regulation.source !== data.regulator || regulation.target !== data.target)
    )

    // todo: this is a hack for now, to avoid issues in static prop removing after the regulation is removed
    setTimeout(() => {
      aeonState.sketch.properties.refreshStaticProps()
    }, 50)
  }

  #onSketchRefreshed (sketch: SketchData): void {
    const datasets = sketch.datasets.map(d => convertToIObservationSet(d))
    const functions = sketch.model.uninterpreted_fns.map(f => convertToIFunction(f))
    const variables = sketch.model.variables.map(v => convertToIVariable(v))
    const regulations = sketch.model.regulations.map(r => convertToIRegulation(r))
    const layout = convertToILayout(sketch.model.layouts[0].nodes)

    this.saveWholeSketch(
      functions,
      variables,
      regulations,
      layout,
      datasets,
      sketch.stat_properties,
      sketch.dyn_properties,
      sketch.annotation
    )
  }

  // refresh all components of the model, and save them at the same time
  #onModelRefreshed (model: ModelData): void {
    const functions = model.uninterpreted_fns.map(f => convertToIFunction(f))
    const variables = model.variables.map(v => convertToIVariable(v))
    const regulations = model.regulations.map(r => convertToIRegulation(r))
    const layout = convertToILayout(model.layouts[0].nodes)
    // save everything at once
    this.saveWholeModel(functions, variables, regulations, layout)
  }

  #onVariablesRefreshed (variables: VariableData[]): void {
    this.saveVariables(variables.map(v => convertToIVariable(v)))
  }

  #onLayoutNodesRefreshed (layoutNodes: LayoutNodeData[]): void {
    this.saveLayout(convertToILayout(layoutNodes))
  }

  #onRegulationsRefreshed (regulations: RegulationData[]): void {
    this.saveRegulations(regulations.map(r => convertToIRegulation(r)))
  }

  private async confirmDeleteDialog (message: string): Promise<boolean> {
    return await dialog.ask(message, {
      type: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Keep',
      title: 'Delete'
    })
  }

  render (): TemplateResult {
    const visibleTabs = this.visibleTabs()
    return html`
      ${this.editorStarted
? html`
        <div class="root-component">
          <div class="header uk-margin-small-top uk-margin-small-bottom">
            <nav-bar .tabs=${this.tabs}></nav-bar>
          </div>
          <div class="content">
            ${map(this.tabs, (tab) => html`
              <content-pane id="${tab.name.toLowerCase()}" ?hidden="${!(tab.pinned || tab.active)}"
                            class="uk-width-1-${visibleTabs.length} ${tab.active ? 'active' : 'inactive'} ${(tab.active || tab.pinned) ? 'visible' : ''}" .tab=${tab}
                            .data=${this.data}></content-pane>
            `)}
          </div>
        </div>
      `
: html`<initial-screen></initial-screen>`
      }`
  }
}
