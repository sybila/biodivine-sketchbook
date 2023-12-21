import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './functions-editor.less?inline'
import { map } from 'lit/directives/map.js'
import { ContentData } from '../../util/tab-data'
import './function-tile/function-tile'

@customElement('functions-editor')
class FunctionsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  @state() focusedFunctionId = ''

  constructor () {
    super()
    this.addEventListener('focus-function', this.focusedFunction)
  }

  private focusedFunction (event: Event): void {
    this.focusedFunctionId = (event as CustomEvent).detail.nodeId
  }

  protected render (): TemplateResult {
    return html`
          <div class="function-list uk-list uk-list-divider uk-text-center">
            ${map(this.contentData?.nodes, (node) => html`
              <function-tile .variable="${node}" .regulations="${this.contentData.edges.filter(edge => edge.source === node.id)}"></function-tile>
            `)}
          </div>
    `
  }
}
