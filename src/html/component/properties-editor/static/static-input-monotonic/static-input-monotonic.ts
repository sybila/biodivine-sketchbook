import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-input-monotonic.less?inline'
import {
  type IFunctionInputMonotonicStaticProperty,
  Monotonicity,
  StaticPropertyType
} from '../../../../util/data-interfaces'
import { getMonotonicityClass, getNextMonotonicity } from '../../../../util/utilities'
import abstractStaticProperty from '../abstract-static-property'

@customElement('static-input-monotonic')
export default class StaticInputMonotonic extends abstractStaticProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IFunctionInputMonotonicStaticProperty

  toggleMonotonicity (): void {
    if (this.property.variant === StaticPropertyType.FunctionInputEssential) return
    let value = getNextMonotonicity((this.property).value)
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
        return '->'
      case Monotonicity.DUAL:
        return '-*'
      case Monotonicity.INHIBITION:
        return '-|'
      default:
        return '??'
    }
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        ${this.renderNameplate(false)}
        <div class="value-section">
          <div class="value-symbol">
            <div class="uk-margin-small-right">${this.property.input}</div>
            <div class="uk-margin-small-right">${this.getMonotonicitySymbol()}</div>
            <div class="uk-margin-small-right">${this.property.target}</div>
          </div>
          <div class="value-symbol">
            <span>(</span>
            <span class="monotonicity ${getMonotonicityClass(this.property.value)}">
              ${this.property.value.toLowerCase()}
            </span>
            <span>)</span>
          </div>
        </div>
      </div>
      </div>
      <hr>
    `
  }
}
