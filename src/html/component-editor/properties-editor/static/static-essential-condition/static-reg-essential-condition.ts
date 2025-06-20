import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-essential-condition.less?inline'
import {
  Essentiality,
  type IVariableRegulatorEssentialStaticProperty
} from '../../../../util/data-interfaces'
import { getEssentialityText, getNextEssentiality } from '../../../../util/utilities'
import { map } from 'lit/directives/map.js'
import StaticRegSelectorsProperty from '../static-reg-selectors-property'

@customElement('static-reg-essential-condition')
export default class StaticRegEssentialCondition extends StaticRegSelectorsProperty {
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

  toggleEssentiality (): void {
    const value = getNextEssentiality(this.property.value)
    // if (value === Essentiality.UNKNOWN) {
    //  value = getNextEssentiality(value)
    // }
    this.updateProperty({
      ...this.property,
      value
    })
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        ${this.renderNameplate()}
        <div class="value-section">
          <div class="value-symbol uk-width-3-5 gap">
            <select id="input-selector" class="uk-select" @change="${this.inputChanged}">
              <option value="${null}">---</option>
              ${map(this.getInputSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
            <span>${this.getEssentialitySymbol()}</span>
            <select id="target-selector" class="uk-select" @change="${this.targetChanged}"
                    ?disabled="${this.property.input === null}">
              <option value="${null}">---</option>
              ${map(this.getTargetSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
          </div>
          <div class="value-symbol uk-width-2-5" @click="${() => {
            this.toggleEssentiality()
          }}">
            <span class="essentiality">
              ${getEssentialityText(this.property.value)}
            </span>
          </div>
        </div>
        ${this.renderConditionField()}
      </div>
      </div>
    `
  }
}
