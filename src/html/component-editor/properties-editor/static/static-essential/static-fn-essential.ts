import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-essential.less?inline'
import { type IFunctionInputEssentialStaticProperty } from '../../../../util/data-interfaces'
import { getEssentialityText } from '../../../../util/utilities'
import abstractStaticProperty from '../abstract-static-property'

@customElement('static-fn-essential')
export default class StaticFnEssential extends abstractStaticProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IFunctionInputEssentialStaticProperty

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
