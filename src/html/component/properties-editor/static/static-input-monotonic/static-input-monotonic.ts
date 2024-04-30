import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-input-monotonic.less?inline'
import AbstractProperty from '../../abstract-property/abstract-property'
import { type IFunctionInputMonotonicStaticProperty, Monotonicity } from '../../../../util/data-interfaces'
import { getMonotonicityClass } from '../../../../util/utilities'

@customElement('static-input-monotonic')
export default class StaticInputMonotonic extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IFunctionInputMonotonicStaticProperty

  toggleMonotonicity (): void {

  }

  private getMonotonicitySymbol (): string {
    switch (this.property.monotonic) {
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
        <div class="uk-flex uk-flex-row uk-flex-center">
          <input id="name-field" class="name-field static-name-field" value="${this.property.name}" readonly/>
        </div>
        <div
            class="regulation uk-grid uk-grid-column-small uk-grid-row-large uk-child-width-1-4 uk-margin-remove uk-text-center uk-flex-around uk-text-nowrap">
          <div class="uk-width-1-6">${this.property.variable}</div>
          <div class="uk-width-1-6">${this.getMonotonicitySymbol()}</div>
          <div class="uk-width-1-6">${this.property.function}</div>
          <div @click="${() => {
            this.toggleMonotonicity()
          }}">
            (<span class="monotonicity ${getMonotonicityClass(this.property.monotonic)}">${this.property.monotonic.toLowerCase()}</span>)
          </div>
        </div>
      </div>
      </div>
      <hr>
    `
  }
}
