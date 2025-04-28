import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-input-essential.less?inline'
import { Essentiality, type IFunctionInputEssentialStaticProperty } from '../../../../util/data-interfaces'
import { getEssentialityText, getNextEssentiality } from '../../../../util/utilities'
import abstractStaticProperty from '../abstract-static-property'

@customElement('static-input-essential')
export default class StaticInputEssential extends abstractStaticProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IFunctionInputEssentialStaticProperty

  private getEssentialitySymbol (): string {
    switch (this.property.value) {
      case Essentiality.TRUE:
        return '->'
      case Essentiality.FALSE:
        return '-/>'
      default:
        return '??'
    }
  }

  toggleEssentiality (): void {
    let value = getNextEssentiality((this.property).value)
    if (value === Essentiality.UNKNOWN) {
      value = getNextEssentiality(value)
    }
    this.updateProperty({
      ...this.property,
      value
    })
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        ${this.renderNameplate(false)}
        <div class="value-section">
          <div class="value-symbol">
            <div class="uk-margin-small-right">${this.property.input}</div>
            <div class="uk-margin-small-right">${this.getEssentialitySymbol()}</div>
            <div class="uk-margin-small-right">${this.property.target}</div>
          </div>
          <div class="value-symbol">
            <span class="essentiality">
              ${getEssentialityText(this.property.value)}
            </span>
          </div>
        </div>
      </div>
      </div>
    `
  }
}
