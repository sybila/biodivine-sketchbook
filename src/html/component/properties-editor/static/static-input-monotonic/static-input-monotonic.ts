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
    let value = getNextMonotonicity(this.property.value)
    if (value === Monotonicity.UNSPECIFIED) {
      value = getNextMonotonicity(value)
    }
    this.updateProperty({
      ...this.property,
      value
    })
  }

  conditionChanged (context: string): void {
    this.updateProperty({
      ...this.property,
      context
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
        ${choose(this.property.variant, [
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
            <div class="uk-margin-small-right">${this.property.input}</div>
            <div class="uk-margin-small-right">${this.getMonotonicitySymbol()}</div>
            <div class="uk-margin-small-right">${this.property.target}</div>
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
        ${when(this.property.variant === StaticPropertyType.FunctionInputMonotonicWithCondition,
            () => html`
              <div class="uk-flex uk-flex-column uk-flex-left">
                <label class="condition-label">Context formula:</label>
                <div class="uk-flex uk-flex-row">
                  <input id="condition-field" class="condition-field" value="${this.property.context}"
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
