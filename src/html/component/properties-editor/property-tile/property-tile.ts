import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './property-tile.less?inline'
import { type IProperty } from '../../../util/data-interfaces'
import { debounce } from 'lodash'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import { functionDebounceTimer } from '../../../util/config'

@customElement('property-tile')
export default class PropertyTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare prop: IProperty

  nameUpdated = debounce((name: string) => {
    console.log(name)
  }, functionDebounceTimer)

  valueUpdated = debounce((name: string) => {
    console.log(name)
  }, functionDebounceTimer)

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
