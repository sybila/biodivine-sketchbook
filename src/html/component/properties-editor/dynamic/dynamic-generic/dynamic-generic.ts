import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './dynamic-generic.less?inline'
import { type IGenericDynamicProperty } from '../../../../util/data-interfaces'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../../../util/config'
import AbstractDynamicProperty from '../abstract-dynamic-property'

@customElement('dynamic-generic')
export default class DynamicGeneric extends AbstractDynamicProperty {
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
        ${this.renderNameplate()}
        <input id="value-editor" class="uk-input" .value="${this.property.formula}"
               @input="${(e: Event) => {
                 this.valueUpdated((e.target as HTMLInputElement).value)
               }}">
      </div>
      <hr class="uk-margin-top uk-margin-bottom uk-margin-left uk-margin-right">
    `
  }
}
