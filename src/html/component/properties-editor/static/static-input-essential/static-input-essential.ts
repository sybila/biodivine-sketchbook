import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-input-essential.less?inline'
import AbstractProperty from '../../abstract-property/abstract-property'
import { Essentiality, type IFunctionInputEssentialStaticProperty } from '../../../../util/data-interfaces'
import { getEssentialityText } from '../../../../util/utilities'

@customElement('static-input-essential')
export default class StaticInputEssential extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IFunctionInputEssentialStaticProperty

  private getEssentialitySymbol (): string {
    switch (this.property.essential) {
      case Essentiality.TRUE:
        return '->'
      case Essentiality.FALSE:
        return '-/>'
      default:
        return '??'
    }
  }

  toggleEssentiality (): void {

  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="name-field static-name-field" value="${this.property.name}" readonly/>
        </div>
        <div
            class="regulation uk-grid uk-grid-column-small uk-grid-row-large uk-child-width-1-4 uk-margin-remove uk-text-center uk-flex-around uk-text-nowrap">
          <div class="uk-width-1-6">${this.property.variable}</div>
          <div class="uk-width-1-6">${this.getEssentialitySymbol()}</div>
          <div class="uk-width-1-6">${this.property.function}</div>
          <div class="regulation-property"
               @click="${() => {
                 this.toggleEssentiality()
               }}">
            (<span class="essentiality">${getEssentialityText(this.property.essential)}</span>)
          </div>
        </div>
      </div>
      </div>
      <hr>
    `
  }
}
