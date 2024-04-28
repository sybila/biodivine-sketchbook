import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement } from 'lit/decorators.js'
import style_less from './dynamic-attractor-count.less?inline'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import PropertyTile from '../../property-tile/property-tile'

@customElement('dynamic-attractor-count')
export default class DynamicAttractorCount extends PropertyTile {
  static styles = css`${unsafeCSS(style_less)}`

  render (): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="uk-input uk-text-center" value="${this.prop.name}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          <button class="uk-button uk-button-small">
            ${icon(faTrash).node}
          </button>
        </div>
        <span class="uk-align-left uk-text-left uk-margin-remove">Value:</span>
        <input id="value-editor" class="uk-input" value="${this.prop.value}"
               @input="${(e: InputEvent) => this.valueUpdated((e.target as HTMLInputElement).value)}">
      </div>
      <hr>
    `
  }
}
