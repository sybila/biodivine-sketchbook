import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'
import style_less from './static-input-monotonic-condition.less?inline'
import AbstractProperty from '../../abstract-property/abstract-property'
import {
  type ContentData,
  type IFunctionInputMonotonicStaticProperty,
  type IVariableData,
  type IVariableRegulatorMonotonicStaticProperty,
  Monotonicity,
  StaticPropertyType
} from '../../../../util/data-interfaces'
import { getMonotonicityClass, getNextMonotonicity } from '../../../../util/utilities'
import { map } from 'lit/directives/map.js'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../../../util/config'

@customElement('static-input-monotonic-condition')
export default class StaticInputMonotonicCondition extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare contentData: ContentData
  @property() declare property: IFunctionInputMonotonicStaticProperty | IVariableRegulatorMonotonicStaticProperty
  @state() selectedVariable: IVariableData | undefined
  @query('#second-selector') declare secondSelector: HTMLSelectElement

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

  conditionChanged = debounce((condition: string): void => {
    this.updateProperty({
      ...this.property,
      condition
    })
  }, functionDebounceTimer)

  private getMonotonicitySymbol (): string {
    switch (this.property.monotonic) {
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

  firstChanged (event: Event): void {
    const value = (event.target as HTMLSelectElement).value
    console.log(value)
    if (this.property.type === StaticPropertyType.FunctionInputMonotonicWithCondition) {
      this.updateProperty({
        ...this.property,
        function: value,
        variable: undefined
      })
    } else if (this.property.type === StaticPropertyType.VariableRegulationMonotonicWithCondition) {
      this.updateProperty({
        ...this.property,
        variable: value,
        regulator: undefined
      })
    }
    this.secondSelector.selectedIndex = 0
  }

  secondChanged (event: Event): void {
    const value = (event.target as HTMLSelectElement).value
    if (this.property.type === StaticPropertyType.FunctionInputMonotonicWithCondition) {
      this.updateProperty({
        ...this.property,
        variable: value
      })
    } else if (this.property.type === StaticPropertyType.VariableRegulationMonotonicWithCondition) {
      this.updateProperty({
        ...this.property,
        regulator: value
      })
    }
  }

  getFirstSelectorItems (): string[] {
    if (this.property.type === StaticPropertyType.FunctionInputMonotonicWithCondition) {
      return this.contentData.functions.map(func => func.id)
    } else if (this.property.type === StaticPropertyType.VariableRegulationMonotonicWithCondition) {
      return this.contentData.variables.map(variable => variable.id)
    }
    return []
  }

  getSecondSelectorItems (): string[] {
    if (this.property.type === StaticPropertyType.FunctionInputMonotonicWithCondition) {
      return this.contentData.functions
        .find(func => func.id === (this.property as IFunctionInputMonotonicStaticProperty).function)
        ?.variables.map(variable => variable.source) ?? []
    } else if (this.property.type === StaticPropertyType.VariableRegulationMonotonicWithCondition) {
      return this.contentData.regulations
        .filter(regulation => regulation.target === (this.property as IVariableRegulatorMonotonicStaticProperty).variable)
        .map(regulation => regulation.source)
    }
    return []
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="name-field static-name-field" value="${this.property.name}" readonly/>
        </div>
        <div class="value-section">
          <div class="value-symbol">
            <select id="first-selector" class="uk-select" @change="${this.firstChanged}">
              <option value="${undefined}">---</option>
              ${map(this.getFirstSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
            <span>${this.getMonotonicitySymbol()}</span>
            <select id="second-selector" class="uk-select" @change="${this.secondChanged}">
              <option value="${undefined}">---</option>
              ${map(this.getSecondSelectorItems(), (item) => html`
                <option value="${item}">${item}</option>
              `)}
            </select>
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
        <div class="uk-flex uk-flex-column uk-flex-left">
          <label class="condition-label">Context formula:</label>
          <div class="uk-flex uk-flex-row">
            <input id="condition-field" class="condition-field" value="${this.property.condition}"
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
