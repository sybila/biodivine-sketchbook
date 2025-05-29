import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-monotonic.less?inline'
import {
  type IFunctionInputMonotonicStaticProperty
} from '../../../../util/data-interfaces'
import { getMonotonicityClass } from '../../../../util/utilities'
import abstractStaticProperty from '../abstract-static-property'

@customElement('static-fn-monotonic')
export default class StaticFnMonotonic extends abstractStaticProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IFunctionInputMonotonicStaticProperty

  render (): TemplateResult {
    return html`
      <div class="property-body">
        ${this.renderNameplate(false)}
        <div class="value-section">
          <div class="value-symbol gap">
            <label for="target">Fn: </label>
            <div id="target" class="uk-margin-small-right uk-text-bold">${this.property.target}</div>
            <label for="input">Input: </label>
            <div id="input" class="uk-margin-small-right uk-text-bold">${this.property.input}</div>
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
