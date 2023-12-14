import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './functions-editor.less?inline'
import { map } from 'lit/directives/map.js'
import { ContentData } from '../../util/tab-data'

@customElement('functions-editor')
class FunctionsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()

  protected render (): TemplateResult {
    return html`
      <div class="uk-section">
        <div class="uk-container">
          <ul class="uk-list uk-list-divider uk-text-center">
            ${map(this.contentData?.nodes, (node) => html`
              <li>
                ${node.id} / ${node.name}
              </li>
            `)}
          </ul>
        </div>
      </div>

    `
  }
}
