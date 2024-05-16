import { css, html, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query } from 'lit/decorators.js'
import style_less from './static-input-monotonic-condition.less?inline'
import {
  type ContentData,
  type IFunctionInputMonotonicStaticProperty,
  type IVariableRegulatorMonotonicStaticProperty,
  Monotonicity,
  StaticPropertyType
} from '../../../../util/data-interfaces'
import { getMonotonicityClass, getNextMonotonicity } from '../../../../util/utilities'
import { map } from 'lit/directives/map.js'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../../../util/config'
import abstractStaticProperty from '../abstract-static-property'

@customElement('static-input-monotonic-condition')
export default class StaticInputMonotonicCondition extends abstractStaticProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare contentData: ContentData
  @property() declare property: IFunctionInputMonotonicStaticProperty | IVariableRegulatorMonotonicStaticProperty
  @query('#target-selector') declare targetSelector: HTMLSelectElement
  @query('#input-selector') declare inputSelector: HTMLSelectElement

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

  conditionChanged = debounce((context: string): void => {
    this.updateProperty({
      ...this.property,
      context
    })
  }, functionDebounceTimer)

  private getMonotonicitySymbol (): string {
    switch (this.property.value) {
      case Monotonicity.ACTIVATION:
        return '<-'
      case Monotonicity.DUAL:
        return '*-'
      case Monotonicity.INHIBITION:
        return '|-'
      default:
        return '??'
    }
  }

  targetChanged (event: Event): void {
    let value: string | null = (event.target as HTMLSelectElement).value
    value = value === '' ? null : value
    console.log(value)
    if (this.property.variant === StaticPropertyType.FunctionInputMonotonicWithCondition) {
      this.updateProperty({
        ...this.property,
        target: value,
        input: null
      })
    } else if (this.property.variant === StaticPropertyType.VariableRegulationMonotonicWithCondition) {
      this.updateProperty({
        ...this.property,
        target: value,
        input: null
      })
    }
    this.inputSelector.selectedIndex = 0
  }

  inputChanged (event: Event): void {
    let value: string | null = (event.target as HTMLSelectElement).value
    value = value === '' ? null : value
    if (this.property.variant === StaticPropertyType.FunctionInputMonotonicWithCondition) {
      this.updateProperty({
        ...this.property,
        input: value
      })
    } else if (this.property.variant === StaticPropertyType.VariableRegulationMonotonicWithCondition) {
      this.updateProperty({
        ...this.property,
        input: value
      })
    }
  }

  getTargetSelectorItems (): string[] {
    if (this.property.variant === StaticPropertyType.FunctionInputMonotonicWithCondition) {
      return this.contentData.functions.map(func => func.id)
    } else if (this.property.variant === StaticPropertyType.VariableRegulationMonotonicWithCondition) {
      return this.contentData.variables.map(variable => variable.id)
    }
    return []
  }

  getInputSelectorItems (): string[] {
    if (this.property.variant === StaticPropertyType.FunctionInputMonotonicWithCondition) {
      return this.contentData.functions
        .find(func => func.id === (this.property as IFunctionInputMonotonicStaticProperty).target)
        ?.variables.map(variable => variable.source) ?? []
    } else if (this.property.variant === StaticPropertyType.VariableRegulationMonotonicWithCondition) {
      return this.contentData.regulations
        .filter(regulation => regulation.target === (this.property as IVariableRegulatorMonotonicStaticProperty).target)
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
            <span>${this.getMonotonicitySymbol()}</span>
            <select id="input-selector" class="uk-select" @change="${this.inputChanged}"
                    ?disabled="${this.property.target === null}">
            <option value="${null}">---</option>
              ${map(this.getInputSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
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
