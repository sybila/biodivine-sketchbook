import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-input-monotonic-condition.less?inline'
import {
  type IFunctionInputMonotonicStaticProperty,
  type IVariableRegulatorMonotonicStaticProperty,
  Monotonicity
} from '../../../../util/data-interfaces'
import { getMonotonicityClass, getNextMonotonicity } from '../../../../util/utilities'
import { map } from 'lit/directives/map.js'
import StaticSelectorsProperty from '../static-selectors-property'

@customElement('static-input-monotonic-condition')
export default class StaticInputMonotonicCondition extends StaticSelectorsProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IFunctionInputMonotonicStaticProperty | IVariableRegulatorMonotonicStaticProperty

  toggleMonotonicity (): void {
    let value = getNextMonotonicity(this.property.value)
    if (value === Monotonicity.UNSPECIFIED) {
      value = getNextMonotonicity(value)
    }
    this.updateProperty({
      ...this.property,
      value
    })
  }

  private getMonotonicitySymbol (): string {
    switch (this.property.value) {
      case Monotonicity.ACTIVATION:
        return '<-'
      case Monotonicity.DUAL:
        return '*-'
      case Monotonicity.INHIBITION:
        return '|-'
      default:
        return '??'
    }
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        ${this.renderNameplate()}
        <div class="value-section">
          <div class="value-symbol gap">
            <select id="target-selector" class="uk-select" @change="${this.targetChanged}">
              <option value="${null}">---</option>
              ${map(this.getTargetSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
            <span>${this.getMonotonicitySymbol()}</span>
            <select id="input-selector" class="uk-select" @change="${this.inputChanged}"
                    ?disabled="${this.property.target === null}">
              <option value="${null}">---</option>
              ${map(this.getInputSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
          </div>
          <div class="value-symbol" @click="${() => {
            this.toggleMonotonicity()
          }}">
            <span>(</span>
            <span class="monotonicity ${getMonotonicityClass(this.property.value)}">
              ${this.property.value.toLowerCase()}
            </span>
            <span>)</span>
          </div>
        </div>
        ${this.renderConditionField()}
      </div>
      </div>
      <hr class="uk-margin-top uk-margin-bottom uk-margin-left uk-margin-right">
    `
  }
}
