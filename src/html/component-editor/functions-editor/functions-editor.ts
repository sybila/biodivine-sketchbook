import { css, html, LitElement, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './functions-editor.less?inline'
import { map } from 'lit/directives/map.js'
import './editor-tile/variable-tile'
import './editor-tile/function-tile'
import { ContentData, type IFunctionData } from '../../util/data-interfaces'
import langTools from 'ace-builds/src-noconflict/ext-language_tools'
import { type Ace } from 'ace-builds'
import {
  getNextEssentiality, getNextMonotonicity,
  convertFromIFunction, convertToIFunction
} from '../../util/utilities'
import { dialog } from '@tauri-apps/api'
import { aeonState, type UninterpretedFnData } from '../../../aeon_state'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/event'

/** Component responsible for the function editor tab of the editor session. */
@customElement('functions-editor')
export class FunctionsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  dialogs: Record<string, WebviewWindow | undefined> = {}

  constructor () {
    super()

    // functions-related event listeners
    aeonState.sketch.model.uninterpretedFnCreated.addEventListener(this.#onFunctionCreated.bind(this))
    this.addEventListener('remove-function-definition', (e) => { void this.removeFunction(e) })
    // listener 'aeonState.sketch.model.uninterpretedFnRemoved' is handled by Root component (more complex update)
    this.addEventListener('edit-function-definition', (e) => { void this.editFunction(e) })
    aeonState.sketch.model.uninterpretedFnDataChanged.addEventListener(this.#onFunctionDataChanged.bind(this))
    this.addEventListener('change-function-id', this.setFunctionId)
    // listener 'aeonState.sketch.model.uninterpretedFnIdChanged' is handled by Root component (more complex update)
    this.addEventListener('change-fn-arity', this.changeFunctionArity)
    // listener 'aeonState.sketch.model.uninterpretedFnArityChanged' is handled by Root component (more complex update)
    this.addEventListener('toggle-function-variable-monotonicity', this.toggleFunctionVariableMonotonicity)
    aeonState.sketch.model.uninterpretedFnMonotonicityChanged.addEventListener(this.#onFunctionMonotonicityChanged.bind(this))
    this.addEventListener('toggle-function-variable-essentiality', this.toggleFunctionVariableEssentiality)
    aeonState.sketch.model.uninterpretedFnEssentialityChanged.addEventListener(this.#onFunctionEssentialityChanged.bind(this))
    this.addEventListener('set-uninterpreted-function-expression', this.setFunctionExpression)
    aeonState.sketch.model.uninterpretedFnExpressionChanged.addEventListener(this.#onFunctionExpressionChanged.bind(this))

    // refresh-event listeners
    aeonState.sketch.model.uninterpretedFnsRefreshed.addEventListener(this.#onUninterpretedFnsRefreshed.bind(this))

    // note that the refresh events are automatically triggered or handled (after app refresh) directly
    // from the root component (due to some dependency issues between different components)
  }

  connectedCallback (): void {
    super.connectedCallback()
    window.addEventListener('focus-function-field', this.focusedFunction.bind(this))
  }

  disconnectedCallback (): void {
    super.disconnectedCallback()
    window.removeEventListener('focus-function-field', this.focusedFunction.bind(this))
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
    langTools.setCompleters([{
      getCompletions: (_editor: Ace.Editor, _session: Ace.EditSession, _point: Ace.Point, _prefix: string, callback: Ace.CompleterCallback) => {
        callback(null, this.contentData.functions.map((func): Ace.Completion => ({
          value: func.id,
          meta: func.id,
          snippet: func.id + '()'
        })))
      }
    }])
  }

  /** Dispatch event to root component to save functions. */
  private saveFunctions (functions: IFunctionData[]): void {
    // propagate the current version of functions via event that will be captured by root component
    this.dispatchEvent(new CustomEvent('save-functions', {
      bubbles: true,
      composed: true,
      detail: {
        functions
      }
    }))
  }

  private focusedFunction (event: Event): void {
    const id = (event as CustomEvent).detail.id
    this.shadowRoot?.getElementById(id)?.scrollIntoView()
  }

  /** Process function data refreshed from backend. */
  #onUninterpretedFnsRefreshed (functions: UninterpretedFnData[]): void {
    const fns = functions.map((data): IFunctionData => {
      return convertToIFunction(data)
    })
    this.saveFunctions(fns)
  }

  /** Invoke backend to add new function. */
  private addFunction (): void {
    aeonState.sketch.model.addDefaultUninterpretedFn()
  }

  /** Process and save newly created fn data sent from backend. */
  #onFunctionCreated (data: UninterpretedFnData): void {
    const newFunction = convertToIFunction(data)
    this.contentData.functions.push(newFunction)
    this.saveFunctions([...this.contentData.functions])
  }

  /** Process and save changed fn data sent from backend. */
  #onFunctionDataChanged (data: UninterpretedFnData): void {
    const functions = [...this.contentData.functions]
    const fnIndex = functions.findIndex(f => f.id === data.id)
    if (fnIndex === -1) return

    functions[fnIndex] = {
      ...functions[fnIndex],
      id: data.id,
      name: data.name,
      annotation: data.annotation
    }
    this.saveFunctions(functions)
  }

  /** Invoke backend to remove function. */
  private async removeFunction (event: Event): Promise<void> {
    const id = (event as CustomEvent).detail.id
    const message = `Do you want to proceed removing function '${id}'?`
    if (!await this.confirmDeleteDialog(message)) return
    aeonState.sketch.model.removeUninterpretedFn(id)
  }

  /** Invoke backend to set function id. */
  private setFunctionId (event: Event): void {
    const detail = (event as CustomEvent).detail
    aeonState.sketch.model.setUninterpretedFnId(detail.oldId, detail.newId)
  }

  /** Invoke backend to change function's arity. */
  private changeFunctionArity (event: Event): void {
    const detail = (event as CustomEvent).detail
    aeonState.sketch.model.setUninterpretedFnArity(detail.id, detail.arity)
  }

  /** Invoke backend to toggle function monotonicity. */
  private toggleFunctionVariableMonotonicity (event: Event): void {
    const detail = (event as CustomEvent).detail
    const newMonotonicity = getNextMonotonicity(detail.monotonicity)
    aeonState.sketch.model.setUninterpretedFnMonotonicity(detail.id, detail.index, newMonotonicity)
  }

  /** Process and update changed fn monotonicity sent from backend. */
  #onFunctionMonotonicityChanged (data: UninterpretedFnData): void {
    const index = this.contentData.functions.findIndex(fun => fun.id === data.id)
    if (index === -1) return

    // not most efficient, but probably sufficient and clear
    const modifiedFunction = convertToIFunction(data)
    const functions = [...this.contentData.functions]
    functions[index] = modifiedFunction
    this.saveFunctions(functions)
  }

  /** Invoke backend to toggle function essentiality. */
  private toggleFunctionVariableEssentiality (event: Event): void {
    const detail = (event as CustomEvent).detail
    const newEssentiality = getNextEssentiality(detail.essentiality)
    aeonState.sketch.model.setUninterpretedFnEssentiality(detail.id, detail.index, newEssentiality)
  }

  /** Process and update changed fn essentiality sent from backend. */
  #onFunctionEssentialityChanged (data: UninterpretedFnData): void {
    const index = this.contentData.functions.findIndex(fun => fun.id === data.id)
    if (index === -1) return

    // not most efficient, but probably sufficient and clear
    const modifiedFunction = convertToIFunction(data)
    const functions = [...this.contentData.functions]
    functions[index] = modifiedFunction
    this.saveFunctions(functions)
  }

  /** Invoke backend to set function expression. */
  private setFunctionExpression (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.sketch.model.setUninterpretedFnExpression(details.id, details.function)
  }

  /** Process and update changed fn expression sent from backend. */
  #onFunctionExpressionChanged (data: UninterpretedFnData): void {
    const index = this.contentData.functions.findIndex(fun => fun.id === data.id)
    if (index === -1) return

    // not most efficient, but probably sufficient and clear
    const modifiedFunction = convertToIFunction(data)
    const functions = [...this.contentData.functions]
    functions[index] = modifiedFunction
    this.saveFunctions(functions)
  }

  /** Open dialog to edit function's name/id/annotation, and propagate changes to backend. */
  private async editFunction (event: Event): Promise<void> {
    const detail = (event as CustomEvent).detail
    const fnIndex = this.contentData.functions.findIndex(f => f.id === detail.id)
    if (fnIndex === -1) return
    const functionData = this.contentData.functions[fnIndex]

    // prepare and save dialog info
    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    if (this.dialogs[functionData.id] !== undefined) {
      await this.dialogs[functionData.id]?.setFocus()
      return
    }

    // open the dialog web view window
    const editFnDialog = new WebviewWindow(`editFunction${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component-editor/functions-editor/edit-fn-dialog/edit-fn-dialog.html',
      title: `Edit function (${functionData.id} / ${functionData.name})`,
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      height: 500,
      width: 400,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    this.dialogs[functionData.id] = editFnDialog

    // add events for communication with the dialog
    void editFnDialog.once('loaded', () => {
      void editFnDialog.emit('edit_fn_update', {
        ...functionData
      })
    })
    void editFnDialog.once('edit_fn_dialog', (event: TauriEvent<{ id: string, name: string, annotation: string }>) => {
      this.dialogs[functionData.id] = undefined
      const index = this.contentData.functions.findIndex(f => f.id === functionData.id)
      if (index === -1) return
      const newIFunctionData = {
        id: event.payload.id,
        name: event.payload.name,
        annotation: event.payload.annotation,
        function: functionData.function,
        variables: functionData.variables
      }
      this.changeFunction(functionData.id, newIFunctionData)
    })
    void editFnDialog.onCloseRequested(() => {
      this.dialogs[functionData.id] = undefined
    })
  }

  /** Propagate potential changes to function (from edit dialog) to backend. */
  private changeFunction (id: string, updatedFn: IFunctionData): void {
    const origFn = this.contentData.functions.find(f => f.id === id)
    if (origFn === undefined) return

    const fnData = convertFromIFunction(updatedFn)

    // ID might have changed
    if (origFn.id !== fnData.id) {
      aeonState.sketch.model.setUninterpretedFnId(origFn.id, fnData.id)
    }
    // name or annotation might have changed
    setTimeout(() => {
      aeonState.sketch.model.setUninterpretedFnData(fnData.id, fnData)
    }, 50)
  }

  /** Show dialog to confirm deletion of fn, with custom message. */
  private async confirmDeleteDialog (message: string): Promise<boolean> {
    return await dialog.ask(message, {
      type: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Keep',
      title: 'Delete'
    })
  }

  /** Render the function editor component. */
  protected render (): TemplateResult {
    return html`
      <div class="container">
        <div class="function-list">
          <div class="section" id="variables">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Update functions</h3>
            </div>
            ${this.contentData?.variables.length === 0 ? html`<div class="uk-text-center uk-margin-bottom"><span class="uk-label">No variables defined</span></div>` : ''}
            <div class="uk-list uk-text-center">
              ${map(this.contentData?.variables, (node, index) => html`
                <variable-tile id="${node.id}"
                               .index="${index}"
                               .variables="${this.contentData.variables}"
                               .regulations="${this.contentData.regulations.filter(edge => edge.target === node.id)}"
                               .functions="${this.contentData.functions}">
                </variable-tile>
              `)}
            </div>
          </div>
          <div class="section" id="functions">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom">Supplementary functions</h3>
              <div class="uk-text-center">
                <button id="add-fn-button" @click="${this.addFunction}" class="uk-button uk-button-small uk-button-primary"> + add </button>
              </div>
            </div>
            ${this.contentData?.functions.length === 0 ? html`<div class="uk-text-center uk-margin-bottom"><span class="uk-label">No functions defined</span></div>` : ''}
            <div class="uk-list uk-text-center">              
              ${map(this.contentData.functions, (_node, index) => html`
                <function-tile .index="${index}"
                               .functions="${this.contentData.functions}">
                </function-tile>
              `)}
            </div>
          </div>          
        </div> 
      </div>
    `
  }
}
