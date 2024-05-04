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
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import { icon } from '@fortawesome/fontawesome-svg-core'

@customElement('static-input-monotonic-condition')
export default class StaticInputMonotonicCondition extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare contentData: ContentData
  @property() declare property: IFunctionInputMonotonicStaticProperty | IVariableRegulatorMonotonicStaticProperty
  @state() selectedVariable: IVariableData | undefined
  @query('#second-selector') declare secondSelector: HTMLSelectElement

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

  firstChanged (event: Event): void {
    const value = (event.target as HTMLSelectElement).value
    console.log(value)
    if (this.property.variant === StaticPropertyType.FunctionInputMonotonicWithCondition) {
      this.updateProperty({
        ...this.property,
        target: value,
        input: undefined
      })
    } else if (this.property.variant === StaticPropertyType.VariableRegulationMonotonicWithCondition) {
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
    if (this.property.variant === StaticPropertyType.FunctionInputMonotonicWithCondition) {
      this.updateProperty({
        ...this.property,
        target: value
      })
    } else if (this.property.variant === StaticPropertyType.VariableRegulationMonotonicWithCondition) {
      this.updateProperty({
        ...this.property,
        input: value
      })
    }
  }

  getFirstSelectorItems (): string[] {
    if (this.property.variant === StaticPropertyType.FunctionInputMonotonicWithCondition) {
      return this.contentData.functions.map(func => func.id)
    } else if (this.property.variant === StaticPropertyType.VariableRegulationMonotonicWithCondition) {
      return this.contentData.variables.map(variable => variable.id)
    }
    return []
  }

  getSecondSelectorItems (): string[] {
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
