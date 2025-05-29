import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-monotonic.less?inline'
import {
  type IVariableRegulatorMonotonicStaticProperty,
  Monotonicity
} from '../../../../util/data-interfaces'
import { getMonotonicityClass } from '../../../../util/utilities'
import abstractStaticProperty from '../abstract-static-property'

@customElement('static-reg-monotonic')
export default class StaticRegMonotonic extends abstractStaticProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IVariableRegulatorMonotonicStaticProperty

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
        ${this.renderNameplate(false)}
        <div class="value-section">
          <div class="value-symbol">
            <div class="uk-margin-small-right uk-text-bold">${this.property.input}</div>
            <div class="uk-margin-small-right">${this.getMonotonicitySymbol()}</div>
            <div class="uk-margin-small-right uk-text-bold">${this.property.target}</div>
          </div>
          <div class="value-symbol">
            <span class="monotonicity ${getMonotonicityClass(this.property.value)}">
              ${this.property.value.toLowerCase()}
            </span>
          </div>
        </div>
      </div>
      </div>
    `
  }
}
