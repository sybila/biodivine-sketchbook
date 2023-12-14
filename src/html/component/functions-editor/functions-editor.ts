import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './functions-editor.less?inline'
import { map } from 'lit/directives/map.js'
import { ContentData } from '../../util/tab-data'

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
          <ul class="function-list uk-list uk-list-divider uk-text-center">
            ${map(this.contentData?.nodes, (node) => html`
              <li class="${node.id === this.focusedFunctionId ? 'uk-background-primary' : ''} uk-margin-remove-top uk-padding-small">
                ${node.id} / ${node.name}
              </li>
            `)}
          </ul>
    `
  }
}
