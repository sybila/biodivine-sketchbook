import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-essential-condition.less?inline'
import {
  type IFunctionInputEssentialStaticProperty
} from '../../../../util/data-interfaces'
import { getEssentialityText, getNextEssentiality } from '../../../../util/utilities'
import { map } from 'lit/directives/map.js'
import StaticFnSelectorsProperty from '../static-fn-selectors-property'

@customElement('static-fn-essential-condition')
export default class StaticFnEssentialCondition extends StaticFnSelectorsProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IFunctionInputEssentialStaticProperty

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
            <label for="target-selector">Fn:</label>
            <select id="target-selector" class="uk-select" @change="${this.targetChanged}">
              <option value="${null}">---</option>
              ${map(this.getTargetSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
            <label for="input-selector">Input:</label>
            <select id="input-selector" class="uk-select" @change="${this.inputChanged}"
                    ?disabled="${this.property.target === null}">
              <option value="${null}">---</option>
              ${map(this.getInputSelectorItems(), (item) => html`
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
