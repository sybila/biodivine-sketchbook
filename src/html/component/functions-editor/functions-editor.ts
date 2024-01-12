import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './functions-editor.less?inline'
import { map } from 'lit/directives/map.js'
import { ContentData } from '../../util/tab-data'
import './function-tile/function-tile'

@customElement('functions-editor')
class FunctionsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()

  constructor () {
    super()
    this.addEventListener('focus-function', this.focusedFunction)
    this.addEventListener('update-variable', this.updateVariable)
    this.addEventListener('update-regulation', this.updateRegulation)
  }

  private updateRegulation (event: Event): void {
    const regulationId = (event as CustomEvent).detail.id
    const regulation = this.contentData.regulations.find((n) => n.id === regulationId)
    if (regulation === undefined) return
    regulation.observable = (event as CustomEvent).detail.observable
    regulation.monotonicity = (event as CustomEvent).detail.monotonicity
    this.updateData()
  }

  private focusedFunction (event: Event): void {
    const variableId = (event as CustomEvent).detail.variableId
    this.shadowRoot?.querySelector(`#${variableId}`)?.dispatchEvent(new Event('focus-function'))
  }

  private updateVariable (event: Event): void {
    const variableId = (event as CustomEvent).detail.variableId
    const variable = this.contentData.variables.find((n) => n.id === variableId)
    if (variable === undefined) return
    variable.function = (event as CustomEvent).detail.function
    variable.name = (event as CustomEvent).detail.variableName
    this.updateData()
  }

  private updateData (): void {
    this.shadowRoot?.dispatchEvent(new CustomEvent('update-data', {
      detail: {
        variables: this.contentData.variables,
        regulations: this.contentData.regulations
      },
      composed: true,
      bubbles: true
    }))
  }

  protected render (): TemplateResult {
    return html`
          <div class="function-list uk-list uk-list-divider uk-text-center">
            ${map(this.contentData?.variables, (node) => html`
              <function-tile id="${node.id}" 
                             .variable="${node}" 
                             .regulations="${this.contentData.regulations.filter(edge => edge.target === node.id)}">
              </function-tile>
            `)}
          </div>
    `
  }
}
