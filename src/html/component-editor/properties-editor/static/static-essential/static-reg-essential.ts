import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-essential.less?inline'
import { Essentiality, type IVariableRegulatorEssentialStaticProperty } from '../../../../util/data-interfaces'
import { getEssentialityText } from '../../../../util/utilities'
import abstractStaticProperty from '../abstract-static-property'

@customElement('static-reg-essential')
export default class StaticRegEssential extends abstractStaticProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IVariableRegulatorEssentialStaticProperty

  private getEssentialitySymbol (): string {
    switch (this.property.value) {
      case Essentiality.TRUE:
        return '->'
      case Essentiality.FALSE:
        return '-/>'
      default:
        return '-?>'
    }
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        ${this.renderNameplate(false)}
        <div class="value-section">
          <div class="value-symbol">
            <div class="uk-margin-small-right uk-text-bold">${this.property.input}</div>
            <div class="uk-margin-small-right">${this.getEssentialitySymbol()}</div>
            <div class="uk-margin-small-right uk-text-bold">${this.property.target}</div>
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
