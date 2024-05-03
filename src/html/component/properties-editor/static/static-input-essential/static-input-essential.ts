import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-input-essential.less?inline'
import AbstractProperty from '../../abstract-property/abstract-property'
import {
  Essentiality,
  type IFunctionInputEssentialStaticProperty,
  StaticPropertyType
} from '../../../../util/data-interfaces'
import { getEssentialityText, getNextEssentiality } from '../../../../util/utilities'
import { when } from 'lit/directives/when.js'
import { choose } from 'lit/directives/choose.js'

@customElement('static-input-essential')
export default class StaticInputEssential extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IFunctionInputEssentialStaticProperty

  private getEssentialitySymbol (): string {
    switch (this.property.essential) {
      case Essentiality.TRUE:
        return '->'
      case Essentiality.FALSE:
        return '-/>'
      default:
        return '??'
    }
  }

  toggleEssentiality (): void {
    let essential = getNextEssentiality(this.property.essential)
    if (essential === Essentiality.UNKNOWN) {
      essential = getNextEssentiality(essential)
    }
    this.updateProperty({
      ...this.property,
      essential
    })
  }

  conditionChanged (condition: string): void {
    this.updateProperty({
      ...this.property,
      condition
    })
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        ${choose(this.property.type, [
          [StaticPropertyType.FunctionInputEssential, () => html`
            <div class="uk-flex uk-flex-row">
              <input id="name-field" class="name-field static-name-field" value="${this.property.name}" readonly/>
            </div>`],
          [StaticPropertyType.FunctionInputEssentialWithCondition, () => html`
            <div class="uk-flex uk-flex-row">
              <input id="name-field" class="name-field" value="${this.property.name}" 
                     @change="${(e: Event) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
            </div>`]
        ])}

        <div class="value-section">
          <div class="value-symbol">
            <div class="uk-margin-small-right">${this.property.variable}</div>
            <div class="uk-margin-small-right">${this.getEssentialitySymbol()}</div>
            <div class="uk-margin-small-right">${this.property.function}</div>
          </div>
          <div class="value-symbol" @click="${() => {
            this.toggleEssentiality()
          }}">
            <span>(</span>
            <span class="essentiality">
              ${getEssentialityText(this.property.essential)}
            </span>
            <span>)</span>
          </div>
        </div>
        ${when(this.property.type === StaticPropertyType.FunctionInputEssentialWithCondition,
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
