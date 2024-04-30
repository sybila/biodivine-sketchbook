import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-generic.less?inline'
import AbstractProperty from '../../abstract-property/abstract-property'
import { type IGenericStaticProperty } from '../../../../util/data-interfaces'

@customElement('static-generic')
export default class StaticGeneric extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IGenericStaticProperty

  render (): TemplateResult {
    return html`
      <div class="property-body">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="name-field static-name-field" value="${this.property.name}" readonly/>
        </div>
        <input id="value-editor" class="uk-input" value="${this.property.value}" readonly>
      </div>
      <hr>
    `
  }
}
