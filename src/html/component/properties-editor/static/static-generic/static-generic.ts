import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-generic.less?inline'
import { type IGenericStaticProperty } from '../../../../util/data-interfaces'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../../../util/config'
import StaticDynamicProperty from '../static-dynamic-property'

@customElement('static-generic')
export default class StaticGeneric extends StaticDynamicProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IGenericStaticProperty

  valueChanged = debounce((formula: string): void => {
    this.updateProperty({
      ...this.property,
      formula
    })
  }, functionDebounceTimer)

  render (): TemplateResult {
    return html`
      <div class="property-body">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="name-field static-name-field" value="${this.property.name}" readonly />
          <button class="remove-property" @click="${this.removeProperty}">
            ${icon(faTrash).node}
          </button>
        </div>
        <div class="uk-flex uk-flex-column uk-flex-left">
          <label class="value-label">Context formula:</label>
          <div class="uk-flex uk-flex-row">
            <input id="value-editor" class="uk-input" value="${this.property.formula}"
                   @input="${(e: Event) => { this.valueChanged((e.target as HTMLInputElement).value) }}"/>
          </div>
        </div>
      </div>
      <hr>
    `
  }
}
