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

  connectedCallback (): void {
    super.connectedCallback()
    window.addEventListener('focus-function-field', this.focusedFunction.bind(this))
  }

  disconnectedCallback (): void {
    super.disconnectedCallback()
    window.removeEventListener('focus-function-field', this.focusedFunction.bind(this))
  }

  private focusedFunction (event: Event): void {
    const variableId = (event as CustomEvent).detail.variableId
    const element = this.shadowRoot?.querySelector(`#${variableId}`)
    element?.dispatchEvent(new Event('focus-function-field'))
    element?.scrollIntoView()
  }

  protected render (): TemplateResult {
    return html`
          <div class="function-list uk-list uk-list-divider uk-text-center">
            ${map(this.contentData?.variables, (node, index) => html`
              <function-tile id="${node.id}" 
                             .variableIndex="${index}"
                             .variables="${this.contentData.variables}"
                             .regulations="${this.contentData.regulations.filter(edge => edge.target === node.id)}">
              </function-tile>
            `)}
          </div>
    `
  }
}
