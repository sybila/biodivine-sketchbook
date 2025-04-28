import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-generic.less?inline'
import { type IGenericStaticProperty } from '../../../../util/data-interfaces'
import abstractStaticProperty from '../abstract-static-property'

@customElement('static-generic')
export default class StaticGeneric extends abstractStaticProperty {
  static styles = css`${unsafeCSS(style_less)}`

  @property() declare property: IGenericStaticProperty

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
        <div class="uk-flex uk-flex-column uk-flex-left">
          <label class="value-label">FOL formula:</label>
          <div class="uk-flex uk-flex-row">
            <input id="value-editor" class="uk-input" .value="${this.property.formula}"
                   @focusout="${this.handleFocusOut}"/>
          </div>
        </div>
      </div>
    `
  }
}
