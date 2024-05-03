import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-input-monotonic.less?inline'
import AbstractProperty from '../../abstract-property/abstract-property'
import {
  type IFunctionInputMonotonicStaticProperty,
  Monotonicity,
  StaticPropertyType
} from '../../../../util/data-interfaces'
import { getMonotonicityClass, getNextMonotonicity } from '../../../../util/utilities'
import { when } from 'lit/directives/when.js'
import { choose } from 'lit/directives/choose.js'

@customElement('static-input-monotonic')
export default class StaticInputMonotonic extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IFunctionInputMonotonicStaticProperty

  toggleMonotonicity (): void {
    let monotonic = getNextMonotonicity(this.property.monotonic)
    if (monotonic === Monotonicity.UNSPECIFIED) {
      monotonic = getNextMonotonicity(monotonic)
    }
    this.updateProperty({
      ...this.property,
      monotonic
    })
  }

  conditionChanged (condition: string): void {
    this.updateProperty({
      ...this.property,
      condition
    })
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
        ${choose(this.property.type, [
          [StaticPropertyType.FunctionInputMonotonic, () => html`
            <div class="uk-flex uk-flex-row">
              <input id="name-field" class="name-field static-name-field" value="${this.property.name}" readonly/>
            </div>`],
          [StaticPropertyType.FunctionInputMonotonicWithCondition, () => html`
            <div class="uk-flex uk-flex-row">
              <input id="name-field" class="name-field" value="${this.property.name}" 
                     @change="${(e: Event) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
            </div>`]
        ])}
        <div class="value-section">
          <div class="value-symbol">
            <div class="uk-margin-small-right">${this.property.variable}</div>
            <div class="uk-margin-small-right">${this.getMonotonicitySymbol()}</div>
            <div class="uk-margin-small-right">${this.property.function}</div>
          </div>
          <div class="value-symbol" @click="${() => {
            this.toggleMonotonicity()
          }}">
            <span>(</span>
            <span class="monotonicity ${getMonotonicityClass(this.property.monotonic)}">
              ${this.property.monotonic.toLowerCase()}
            </span>
            <span>)</span>
          </div>
        </div>
        ${when(this.property.type === StaticPropertyType.FunctionInputMonotonicWithCondition,
            () => html`
              <div class="uk-flex uk-flex-column uk-flex-left">
                <label class="condition-label">Context formula:</label>
                <div class="uk-flex uk-flex-row">
                  <input id="condition-field" class="condition-field" value="${this.property.condition}"
                         @change="${(e: Event) => { this.conditionChanged((e.target as HTMLInputElement).value) }}"/>

                </div>
              </div>`
        )}
      </div>
      </div>
      <hr>
    `
  }
}
