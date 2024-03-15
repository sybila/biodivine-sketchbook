import { css, html, LitElement, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './functions-editor.less?inline'
import { map } from 'lit/directives/map.js'
import './editor-tile/variable-tile'
import './editor-tile/function-tile'
import { ContentData, type IFunctionData } from '../../util/data-interfaces'
import langTools from 'ace-builds/src-noconflict/ext-language_tools'
import { type Ace } from 'ace-builds'
import { getNextEssentiality, getNextMonotonicity } from '../../util/utilities'
import { dialog } from '@tauri-apps/api'
import { aeonState, type UninterpretedFnData, type UninterpretedFnIdUpdateData } from '../../../aeon_events'

@customElement('functions-editor')
export class FunctionsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  index = 0

  constructor () {
    super()
    aeonState.model.uninterpretedFnCreated.addEventListener(this.#onFunctionCreated.bind(this))
    this.addEventListener('remove-function-definition', (e) => { void this.removeFunction(e) })
    aeonState.model.uninterpretedFnRemoved.addEventListener(this.#onFunctionRemoved.bind(this))
    this.addEventListener('remove-function-definition', (e) => { void this.removeFunction(e) })
    aeonState.model.uninterpretedFnRemoved.addEventListener(this.#onFunctionRemoved.bind(this))
    this.addEventListener('rename-function-definition', this.setFunctionId)
    aeonState.model.uninterpretedFnIdChanged.addEventListener(this.#onFunctionIdChanged.bind(this))
    this.addEventListener('add-function-variable', this.addFunctionVariable)
    aeonState.model.uninterpretedFnArityIncremented.addEventListener(this.#onFunctionArityIncremented.bind(this))
    this.addEventListener('toggle-function-variable-monotonicity', this.toggleFunctionVariableMonotonicity)
    aeonState.model.uninterpretedFnMonotonicityChanged.addEventListener(this.#onFunctionMonotonicityChanged.bind(this))
    this.addEventListener('toggle-function-variable-essentiality', this.toggleFunctionVariableEssentiality)
    aeonState.model.uninterpretedFnEssentialityChanged.addEventListener(this.#onFunctionEssentialityChanged.bind(this))
    this.addEventListener('remove-function-variable', (e) => { void this.removeFunctionVariable(e) })
    aeonState.model.uninterpretedFnArityDecremented.addEventListener(this.#onFunctionArityDecremented.bind(this))
    this.addEventListener('set-uninterpreted-function-expression', this.setFunctionExpression)
    aeonState.model.uninterpretedFnExpressionChanged.addEventListener(this.#onFunctionExpressionChanged.bind(this))

    aeonState.model.uninterpretedFnsRefreshed.addEventListener(this.#onUninterpretedFnsRefreshed.bind(this))
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

  private saveFunctions (functions: IFunctionData[]): void {
    functions.sort((a, b) => (a.id > b.id ? 1 : -1))

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
    const variableId = (event as CustomEvent).detail.variableId
    const element = this.shadowRoot?.querySelector(`#${variableId}`)
    element?.dispatchEvent(new Event('focus-function-field'))
    element?.scrollIntoView()
  }

  #onUninterpretedFnsRefreshed (functions: UninterpretedFnData[]): void {
    const fns = functions.map((data): IFunctionData => {
      return this.convertToIFunction(data)
    })
    this.saveFunctions(fns)
  }

  private addFunction (): void {
    /*
    this.functions.push({
      id: 'func' + this.index,
      function: '',
      variables: []
    })
    this.index++
    this.functions = [...this.functions]
     */
    aeonState.model.addUninterpretedFn('func' + this.index, 0)
  }

  #onFunctionCreated (data: UninterpretedFnData): void {
    const newFunction = this.convertToIFunction(data)
    this.contentData.functions.push(newFunction)
    this.index++
    this.saveFunctions([...this.contentData.functions])
  }

  private async removeFunction (event: Event): Promise<void> {
    /*
    if (!await this.confirmDialog()) return
    const id = (event as CustomEvent).detail.id
    const index = this.functions.findIndex(fun => fun.id === id)
    if (index === -1) return
    const functions = [...this.functions]
    functions.splice(index, 1)
    this.functions = functions
     */
    if (!await this.confirmDialog()) return
    const id = (event as CustomEvent).detail.id
    aeonState.model.removeUninterpretedFn(id)
  }

  #onFunctionRemoved (data: UninterpretedFnData): void {
    const id = data.id
    const index = this.contentData.functions.findIndex(fun => fun.id === id)
    if (index === -1) return
    const functions = [...this.contentData.functions]
    functions.splice(index, 1)
    this.saveFunctions(functions)
  }

  private setFunctionId (event: Event): void {
    /*
    const detail = (event as CustomEvent).detail
    const index = this.functions.findIndex(fun => fun.id === detail.oldId)
    if (index === -1) return
    const functions = [...this.functions]
    functions[index] = {
      ...functions[index],
      id: detail.newId
    }
    this.functions = functions
     */
    const detail = (event as CustomEvent).detail
    aeonState.model.setUninterpretedFnId(detail.oldId, detail.newId)
  }

  #onFunctionIdChanged (data: UninterpretedFnIdUpdateData): void {
    const index = this.contentData.functions.findIndex(fun => fun.id === data.original_id)
    if (index === -1) return
    const functions = [...this.contentData.functions]
    functions[index] = {
      ...functions[index],
      id: data.new_id
    }
    this.saveFunctions(functions)

    // TODO: this refresh is a temporary solution to get potentially modified update function and uninterpreted
    // functions' expressions
    aeonState.model.refreshUninterpretedFns()
    aeonState.model.refreshVariables()
  }

  private addFunctionVariable (event: Event): void {
    /*
    const detail = (event as CustomEvent).detail
    const index = this.functions.findIndex(fun => fun.id === detail.id)
    if (index === -1) return
    const functions = [...this.functions]
    functions[index].variables.push({
      id: detail.index,
      source: detail.variable,
      target: functions[index].id,
      essential: Essentiality.UNKNOWN,
      monotonicity: Monotonicity.UNSPECIFIED
    })
    this.functions = functions
     */
    const detail = (event as CustomEvent).detail
    aeonState.model.incrementUninterpretedFnArity(detail.id)
  }

  #onFunctionArityIncremented (data: UninterpretedFnData): void {
    const index = this.contentData.functions.findIndex(fun => fun.id === data.id)
    if (index === -1) return

    // not most efficient, but probably sufficient and clear
    const modifiedFunction = this.convertToIFunction(data)
    const functions = [...this.contentData.functions]
    functions[index] = modifiedFunction
    this.saveFunctions(functions)
  }

  private toggleFunctionVariableMonotonicity (event: Event): void {
    /*
    const detail = (event as CustomEvent).detail
    const index = this.functions.findIndex(fun => fun.id === detail.id)
    if (index === -1) return
    const functions = [...this.functions]
    functions[index].variables[detail.index] = {
      ...functions[index].variables[detail.index],
      monotonicity: getNextMonotonicity(functions[index].variables[detail.index].monotonicity)
    }
    this.functions = functions
     */
    const detail = (event as CustomEvent).detail
    const newMonotonicity = getNextMonotonicity(detail.monotonicity)
    aeonState.model.setUninterpretedFnMonotonicity(detail.id, detail.index, newMonotonicity)
  }

  #onFunctionMonotonicityChanged (data: UninterpretedFnData): void {
    const index = this.contentData.functions.findIndex(fun => fun.id === data.id)
    if (index === -1) return

    // not most efficient, but probably sufficient and clear
    const modifiedFunction = this.convertToIFunction(data)
    const functions = [...this.contentData.functions]
    functions[index] = modifiedFunction
    this.saveFunctions(functions)
  }

  private toggleFunctionVariableEssentiality (event: Event): void {
    /*
    const detail = (event as CustomEvent).detail
    const index = this.functions.findIndex(fun => fun.id === detail.id)
    if (index === -1) return
    const functions = [...this.functions]
    functions[index].variables[detail.index] = {
      ...functions[index].variables[detail.index],
      essential: getNextEssentiality(functions[index].variables[detail.index].essential)
    }
    this.functions = functions
     */
    const detail = (event as CustomEvent).detail
    const newEssentiality = getNextEssentiality(detail.essentiality)
    aeonState.model.setUninterpretedFnEssentiality(detail.id, detail.index, newEssentiality)
  }

  #onFunctionEssentialityChanged (data: UninterpretedFnData): void {
    const index = this.contentData.functions.findIndex(fun => fun.id === data.id)
    if (index === -1) return

    // not most efficient, but probably sufficient and clear
    const modifiedFunction = this.convertToIFunction(data)
    const functions = [...this.contentData.functions]
    functions[index] = modifiedFunction
    this.saveFunctions(functions)
  }

  private setFunctionExpression (event: Event): void {
    const details = (event as CustomEvent).detail
    aeonState.model.setUninterpretedFnExpression(details.id, details.function)
  }

  #onFunctionExpressionChanged (data: UninterpretedFnData): void {
    const index = this.contentData.functions.findIndex(fun => fun.id === data.id)
    if (index === -1) return

    // not most efficient, but probably sufficient and clear
    const modifiedFunction = this.convertToIFunction(data)
    const functions = [...this.contentData.functions]
    functions[index] = modifiedFunction
    this.saveFunctions(functions)
  }

  private async removeFunctionVariable (event: Event): Promise<void> {
    /*
    if (!await this.confirmDialog()) return
    const detail = (event as CustomEvent).detail
    const index = this.functions.findIndex(fun => fun.id === detail.id)
    if (index === -1) return
    const functions = [...this.functions]
    functions[index].variables.splice(detail.index, 1)
    this.functions = functions
     */
    if (!await this.confirmDialog()) return
    const detail = (event as CustomEvent).detail
    aeonState.model.decrementUninterpretedFnArity(detail.id)
  }

  #onFunctionArityDecremented (data: UninterpretedFnData): void {
    const index = this.contentData.functions.findIndex(fun => fun.id === data.id)
    if (index === -1) return

    // not most efficient, but probably sufficient and clear
    const modifiedFunction = this.convertToIFunction(data)
    const functions = [...this.contentData.functions]
    functions[index] = modifiedFunction
    this.saveFunctions(functions)
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

  private async confirmDialog (): Promise<boolean> {
    return await dialog.ask('Are you sure?', {
      type: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Keep',
      title: 'Delete'
    })
  }

  protected render (): TemplateResult {
    return html`
      <div class="function-list">
        <div class="section" id="functions">
          <h2 class="heading uk-text-center">Functions</h2>
          <div class="uk-text-center uk-margin-small-bottom">
            <button @click="${this.addFunction}" class="uk-button uk-button-small">add function</button>
          </div>
          <div class="uk-list uk-list-divider uk-text-center">
            ${map(this.contentData.functions, (_node, index) => html`
              <function-tile .index="${index}"
                             .functions="${this.contentData.functions}">
              </function-tile>
            `)}
          </div>
        </div>
        <div class="section" id="variables">
          <h2 class="heading uk-text-center">Variables</h2>
          <div class="uk-list uk-list-divider uk-text-center">
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
      </div>
    `
  }
}
