import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './dynamic-generic.less?inline'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import AbstractProperty from '../../abstract-property/abstract-property'
import { type IGenericDynamicProperty } from '../../../../util/data-interfaces'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../../../util/config'

@customElement('dynamic-generic')
export default class DynamicGeneric extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IGenericDynamicProperty

  valueUpdated = debounce((formula: string) => {
    this.updateProperty({
      ...this.property,
      formula
    })
  }, functionDebounceTimer)

  render (): TemplateResult {
    return html`
      <div class="property-body">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="name-field" value="${this.property.name}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          <button class="remove-property" @click="${this.removeProperty}">
            ${icon(faTrash).node}
          </button>
        </div>
        <input id="value-editor" class="uk-input" value="${this.property.formula}"
               @input="${(e: Event) => { this.valueUpdated((e.target as HTMLInputElement).value) }}">
      </div>
      <hr>
    `
  }
}
