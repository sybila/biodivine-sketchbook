import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-monotonic-condition.less?inline'
import {
  type IVariableRegulatorMonotonicStaticProperty,
  Monotonicity
} from '../../../../util/data-interfaces'
import { getMonotonicityClass, getNextMonotonicity } from '../../../../util/utilities'
import { map } from 'lit/directives/map.js'
import StaticRegSelectorsProperty from '../static-reg-selectors-property'

@customElement('static-reg-monotonic-condition')
export default class StaticRegMonotonicCondition extends StaticRegSelectorsProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IVariableRegulatorMonotonicStaticProperty

  toggleMonotonicity (): void {
    const value = getNextMonotonicity(this.property.value)
    // if (value === Monotonicity.UNSPECIFIED) {
    //  value = getNextMonotonicity(value)
    // }
    this.updateProperty({
      ...this.property,
      value
    })
  }

  private getMonotonicitySymbol (): string {
    switch (this.property.value) {
      case Monotonicity.ACTIVATION:
        return '->'
      case Monotonicity.DUAL:
        return '-*'
      case Monotonicity.INHIBITION:
        return '-|'
      default:
        return '-?'
    }
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
            <span>${this.getMonotonicitySymbol()}</span>
            <select id="target-selector" class="uk-select" @change="${this.targetChanged}"
                    ?disabled="${this.property.input === null}">
              <option value="${null}">---</option>
              ${map(this.getTargetSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
          </div>
          <div class="value-symbol uk-width-2-5" @click="${() => {
            this.toggleMonotonicity()
          }}">
            <span class="monotonicity ${getMonotonicityClass(this.property.value)}">
              ${this.property.value.toLowerCase()}
            </span>
          </div>
        </div>
        ${this.renderConditionField()}
      </div>
      </div>
    `
  }
}
