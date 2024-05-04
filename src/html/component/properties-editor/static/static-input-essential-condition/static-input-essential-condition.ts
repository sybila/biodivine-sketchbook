import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'
import style_less from './static-input-essential-condition.less?inline'
import AbstractProperty from '../../abstract-property/abstract-property'
import {
  type ContentData,
  Essentiality,
  type IFunctionInputEssentialStaticProperty,
  type IVariableData,
  type IVariableRegulatorEssentialStaticProperty,
  StaticPropertyType
} from '../../../../util/data-interfaces'
import { getEssentialityText, getNextEssentiality } from '../../../../util/utilities'
import { map } from 'lit/directives/map.js'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../../../util/config'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import { icon } from '@fortawesome/fontawesome-svg-core'

@customElement('static-input-essential-condition')
export default class StaticInputEssentialCondition extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare contentData: ContentData
  @property() declare property: IFunctionInputEssentialStaticProperty | IVariableRegulatorEssentialStaticProperty
  @state() selectedVariable: IVariableData | undefined
  @query('#second-selector') declare secondSelector: HTMLSelectElement

  private getEssentialitySymbol (): string {
    switch (this.property.value) {
      case Essentiality.TRUE:
        return '<-'
      case Essentiality.FALSE:
        return '</-'
      default:
        return '??'
    }
  }

  toggleEssentiality (): void {
    let value = getNextEssentiality(this.property.value)
    if (value === Essentiality.UNKNOWN) {
      value = getNextEssentiality(value)
    }
    this.updateProperty({
      ...this.property,
      value
    })
  }

  conditionChanged = debounce((context: string): void => {
    this.updateProperty({
      ...this.property,
      context
    })
  }, functionDebounceTimer)

  firstChanged (event: Event): void {
    const value = (event.target as HTMLSelectElement).value
    if (this.property.variant === StaticPropertyType.FunctionInputEssentialWithCondition) {
      this.updateProperty({
        ...this.property,
        target: value,
        input: undefined
      })
    } else if (this.property.variant === StaticPropertyType.VariableRegulationEssentialWithCondition) {
      this.updateProperty({
        ...this.property,
        target: value,
        input: undefined
      })
    }
    this.secondSelector.selectedIndex = 0
  }

  secondChanged (event: Event): void {
    const value = (event.target as HTMLSelectElement).value
    if (this.property.variant === StaticPropertyType.FunctionInputEssentialWithCondition) {
      this.updateProperty({
        ...this.property,
        input: value
      })
    } else if (this.property.variant === StaticPropertyType.VariableRegulationEssentialWithCondition) {
      this.updateProperty({
        ...this.property,
        input: value
      })
    }
  }

  getFirstSelectorItems (): string[] {
    if (this.property.variant === StaticPropertyType.FunctionInputEssentialWithCondition) {
      return this.contentData.functions.map(func => func.id)
    } else if (this.property.variant === StaticPropertyType.VariableRegulationEssentialWithCondition) {
      return this.contentData.variables.map(variable => variable.id)
    }
    return []
  }

  getSecondSelectorItems (): string[] {
    if (this.property.variant === StaticPropertyType.FunctionInputEssentialWithCondition) {
      return this.contentData.functions
        .find(func => func.id === (this.property as IFunctionInputEssentialStaticProperty).target)
        ?.variables.map(variable => variable.source) ?? []
    } else if (this.property.variant === StaticPropertyType.VariableRegulationEssentialWithCondition) {
      return this.contentData.regulations
        .filter(regulation => regulation.target === (this.property as IVariableRegulatorEssentialStaticProperty).target)
        .map(regulation => regulation.source)
    }
    return []
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="name-field static-name-field" value="${this.property.name}" readonly/>
          <button class="remove-property" @click="${this.removeProperty}">
            ${icon(faTrash).node}
          </button>
        </div>
        <div class="value-section">
          <div class="value-symbol">
            <select id="first-selector" class="uk-select" @change="${this.firstChanged}">
              <option value="${undefined}">---</option>
              ${map(this.getFirstSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
            <span>${this.getEssentialitySymbol()}</span>
            <select id="second-selector" class="uk-select" @change="${this.secondChanged}">
              <option value="${undefined}">---</option>
              ${map(this.getSecondSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
          </div>
          <div class="value-symbol" @click="${() => {
            this.toggleEssentiality()
          }}">
            <span>(</span>
            <span class="essentiality">
              ${getEssentialityText(this.property.value)}
            </span>
            <span>)</span>
          </div>
        </div>
        <div class="uk-flex uk-flex-column uk-flex-left">
          <label class="condition-label">Context formula:</label>
          <div class="uk-flex uk-flex-row">
            <input id="condition-field" class="condition-field" value="${this.property.context}"
                   @input="${(e: Event) => {
                     this.conditionChanged((e.target as HTMLInputElement).value)
                   }}"/>
          </div>
        </div>
      </div>
      </div>
      <hr>
    `
  }
}
