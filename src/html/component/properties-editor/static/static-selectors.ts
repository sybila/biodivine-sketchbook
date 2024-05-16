import { type PropertyValues } from 'lit'
import { property, query } from 'lit/decorators.js'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../../util/config'
import AbstractStaticProperty from './abstract-static-property'
import {
  type ContentData,
  type IFunctionInputEssentialStaticProperty,
  type IFunctionInputMonotonicStaticProperty,
  type IVariableRegulatorEssentialStaticProperty,
  type IVariableRegulatorMonotonicStaticProperty,
  StaticPropertyType
} from '../../../util/data-interfaces'

export default class StaticSelectors extends AbstractStaticProperty {
  @property() declare contentData: ContentData
  @property() declare property: IFunctionInputMonotonicStaticProperty | IVariableRegulatorMonotonicStaticProperty | IFunctionInputEssentialStaticProperty | IVariableRegulatorEssentialStaticProperty
  @query('#target-selector') declare targetSelector: HTMLSelectElement
  @query('#input-selector') declare inputSelector: HTMLSelectElement

  conditionChanged = debounce((context: string): void => {
    this.updateProperty({
      ...this.property,
      context
    })
  }, functionDebounceTimer)

  private isFunctionInput (): boolean {
    return this.property.variant === StaticPropertyType.FunctionInputMonotonicWithCondition ||
      this.property.variant === StaticPropertyType.FunctionInputEssentialWithCondition
  }

  private isVariableRegulator (): boolean {
    return this.property.variant === StaticPropertyType.VariableRegulationMonotonicWithCondition ||
      this.property.variant === StaticPropertyType.VariableRegulationEssentialWithCondition
  }

  targetChanged (event: Event): void {
    let value: string | null = (event.target as HTMLSelectElement).value
    value = value === '' ? null : value
    console.log(value)
    if (this.isFunctionInput()) {
      this.updateProperty({
        ...this.property,
        target: value,
        input: null
      })
    } else if (this.isVariableRegulator()) {
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
    if (this.isFunctionInput()) {
      this.updateProperty({
        ...this.property,
        input: value
      })
    } else if (this.isVariableRegulator()) {
      this.updateProperty({
        ...this.property,
        input: value
      })
    }
  }

  getTargetSelectorItems (): string[] {
    if (this.isFunctionInput()) {
      return this.contentData.functions.map(func => func.id)
    } else if (this.isVariableRegulator()) {
      return this.contentData.variables.map(variable => variable.id)
    }
    return []
  }

  getInputSelectorItems (): string[] {
    if (this.isFunctionInput()) {
      return this.contentData.functions
        .find(func => func.id === (this.property as IFunctionInputMonotonicStaticProperty).target)
        ?.variables.map(variable => variable.source) ?? []
    } else if (this.isVariableRegulator()) {
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
}
