import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-generic.less?inline'
import AbstractProperty from '../../abstract-property/abstract-property'
import { type IGenericStaticProperty } from '../../../../util/data-interfaces'

@customElement('static-generic')
export default class StaticGeneric extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IGenericStaticProperty

  valueChanged (value: string): void {
    this.updateProperty({
      ...this.property,
      value
    })
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="name-field" value="${this.property.name}" 
          @change="${(e: Event) => { this.nameUpdated((e.target as HTMLInputElement).value) }}" />
        </div>
        <div class="uk-flex uk-flex-column uk-flex-left">
          <label class="value-label">Context formula:</label>
          <div class="uk-flex uk-flex-row">
            <input id="value-editor" class="uk-input" value="${this.property.value}"
                   @change="${(e: Event) => { this.valueChanged((e.target as HTMLInputElement).value) }}"/>
          </div>
        </div>
      </div>
      <hr>
    `
  }
}
