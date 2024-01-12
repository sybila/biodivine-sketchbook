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
    this.addEventListener('focus-function-field', this.focusedFunction)
  }

  private focusedFunction (event: Event): void {
    const variableId = (event as CustomEvent).detail.variableId
    this.shadowRoot?.querySelector(`#${variableId}`)?.dispatchEvent(new Event('focus-function-field'))
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
