import { css, html, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'
import style_less from './static-input-essential-condition.less?inline'
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
import StaticDynamicProperty from '../static-dynamic-property'

@customElement('static-input-essential-condition')
export default class StaticInputEssentialCondition extends StaticDynamicProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare contentData: ContentData
  @property() declare property: IFunctionInputEssentialStaticProperty | IVariableRegulatorEssentialStaticProperty
  @state() selectedVariable: IVariableData | undefined
  @query('#target-selector') declare targetSelector: HTMLSelectElement
  @query('#input-selector') declare inputSelector: HTMLSelectElement

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

  targetChanged (event: Event): void {
    let value: null | string = (event.target as HTMLSelectElement).value
    value = value === '' ? null : value
    if (this.property.variant === StaticPropertyType.FunctionInputEssentialWithCondition) {
      this.updateProperty({
        ...this.property,
        target: value,
        input: null
      })
    } else if (this.property.variant === StaticPropertyType.VariableRegulationEssentialWithCondition) {
      this.updateProperty({
        ...this.property,
        target: value,
        input: null
      })
    }
    this.targetSelector.selectedIndex = 0
  }

  inputChanged (event: Event): void {
    let value: null | string = (event.target as HTMLSelectElement).value
    value = value === '' ? null : value
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

  getTargetSelectorItems (): string[] {
    if (this.property.variant === StaticPropertyType.FunctionInputEssentialWithCondition) {
      return this.contentData.functions.map(func => func.id)
    } else if (this.property.variant === StaticPropertyType.VariableRegulationEssentialWithCondition) {
      return this.contentData.variables.map(variable => variable.id)
    }
    return []
  }

  getInputSelectorItems (): string[] {
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

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
    this.targetSelector.selectedIndex = this.getTargetSelectorItems().indexOf(this.property.target ?? '') + 1
    this.inputSelector.selectedIndex = this.getInputSelectorItems().indexOf(this.property.input ?? '') + 1
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        ${this.renderNameplate()}
        <div class="value-section">
          <div class="value-symbol gap">
            <select id="target-selector" class="uk-select" @change="${this.targetChanged}">
              <option value="${null}">---</option>
              ${map(this.getTargetSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
            <span>${this.getEssentialitySymbol()}</span>
            <select id="input-selector" class="uk-select" @change="${this.inputChanged}"
                    ?disabled="${this.property.target === null}">
              <option value="${null}">---</option>
              ${map(this.getInputSelectorItems(), (item) => html`
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
