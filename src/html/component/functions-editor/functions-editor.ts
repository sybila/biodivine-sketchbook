import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './functions-editor.less?inline'
import { map } from 'lit/directives/map.js'
import './editor-tile/variable-tile'
import './editor-tile/function-tile'
import { ContentData, type IVariableData } from '../../util/data-interfaces'

@customElement('functions-editor')
class FunctionsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  @state() functions: IVariableData[] = []
  index = 0

  connectedCallback (): void {
    super.connectedCallback()
    window.addEventListener('focus-function-field', this.focusedFunction.bind(this))
    this.addEventListener('remove-function-definition', this.removeFunction)
    this.addEventListener('rename-function-definition', this.renameFunction)
  }

  disconnectedCallback (): void {
    super.disconnectedCallback()
    window.removeEventListener('focus-function-field', this.focusedFunction.bind(this))
    this.removeEventListener('remove-function-definition', this.removeFunction)
    this.removeEventListener('rename-function-definition', this.renameFunction)
  }

  private focusedFunction (event: Event): void {
    const variableId = (event as CustomEvent).detail.variableId
    const element = this.shadowRoot?.querySelector(`#${variableId}`)
    element?.dispatchEvent(new Event('focus-function-field'))
    element?.scrollIntoView()
  }

  private addFunction (): void {
    this.functions.push({
      id: 'func' + this.index,
      name: 'func' + this.index,
      function: ''
    })
    this.index++
    this.functions = [...this.functions]
  }

  private removeFunction (event: Event): void {
    const id = (event as CustomEvent).detail.id
    const index = this.functions.findIndex(fun => fun.id === id)
    if (index === -1) return
    const functions = [...this.functions]
    functions.splice(index, 1)
    this.functions = functions
  }

  private renameFunction (event: Event): void {
    const detail = (event as CustomEvent).detail
    const index = this.functions.findIndex(fun => fun.id === detail.id)
    if (index === -1) return
    const functions = [...this.functions]
    functions[index] = {
      ...functions[index],
      id: detail.name,
      name: detail.name
    }
    this.functions = functions
  }

  protected render (): TemplateResult {
    return html`
      <div class="function-list">
        <h2 class="divider uk-heading-line uk-text-center">Functions</h2>
        <div class="uk-text-center uk-margin-small-bottom">
          <button @click="${this.addFunction}" class="uk-button uk-button-small">add function</button>
        </div>
        <div class="uk-list uk-list-divider uk-text-center">
          ${map(this.functions, (_node, index) => html`
            <function-tile .variableIndex="${index}"
                           .variables="${this.functions}">
            </function-tile>
          `)}
        </div>
        <h2 class="divider uk-heading-line uk-text-center">Variables</h2>
        <div class="uk-list uk-list-divider uk-text-center">
          ${map(this.contentData?.variables, (node, index) => html`
            <variable-tile id="${node.id}"
                           .variableIndex="${index}"
                           .variables="${this.contentData.variables}"
                           .regulations="${this.contentData.regulations.filter(edge => edge.target === node.id)}">
            </variable-tile>
          `)}
        </div>
      </div>
    `
  }
}
