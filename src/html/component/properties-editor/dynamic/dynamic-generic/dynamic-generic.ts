import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './dynamic-generic.less?inline'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import PropertyTile from '../../property-tile/property-tile'
import { type IGenericDynamicProperty } from '../../../../util/data-interfaces'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../../../util/config'

@customElement('dynamic-generic')
export default class DynamicGeneric extends PropertyTile {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IGenericDynamicProperty

  valueUpdated = debounce((value: string) => {
    this.updateProperty({
      ...this.property,
      value
    })
  }, functionDebounceTimer)

  render (): TemplateResult {
    return html`
      <div class="property-body uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="uk-input uk-text-center" value="${this.property.name}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          <button class="uk-button uk-button-small uk-button-secondary" @click="${this.removeProperty}">
            ${icon(faTrash).node}
          </button>
        </div>
        <input id="value-editor" class="uk-input" value="${this.property.value}"
               @input="${(e: Event) => { this.valueUpdated((e.target as HTMLInputElement).value) }}">
      </div>
      <hr>
    `
  }
}
