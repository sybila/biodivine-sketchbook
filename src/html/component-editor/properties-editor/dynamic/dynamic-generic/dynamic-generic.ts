import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './dynamic-generic.less?inline'
import { type IGenericDynamicProperty } from '../../../../util/data-interfaces'
import AbstractDynamicProperty from '../abstract-dynamic-property'

@customElement('dynamic-generic')
export default class DynamicGeneric extends AbstractDynamicProperty {
  static styles = css`${unsafeCSS(style_less)}`

  @property() declare property: IGenericDynamicProperty

  private handleFocusOut (e: Event): void {
    const inputElement = e.target as HTMLInputElement
    const newValue = inputElement.value

    // Update the property only if the value has changed
    if (newValue !== this.property.formula) {
      this.updateProperty({
        ...this.property,
        formula: newValue
      })
    }
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        ${this.renderNameplate()}
        <input id="value-editor" class="uk-input" .value="${this.property.formula}"
               @focusout="${this.handleFocusOut}">
      </div>
      <hr class="uk-margin-top uk-margin-bottom uk-margin-left uk-margin-right">
    `
  }
}
