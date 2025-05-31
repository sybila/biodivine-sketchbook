import { html, type PropertyValues, type TemplateResult } from 'lit'
import { property, query } from 'lit/decorators.js'
import AbstractStaticProperty from './abstract-static-property'
import {
  type ContentData,
  type IFunctionInputEssentialStaticProperty,
  type IFunctionInputMonotonicStaticProperty
} from '../../../util/data-interfaces'

export default class StaticFnSelectorsProperty extends AbstractStaticProperty {
  @property() declare contentData: ContentData
  @property() declare property: IFunctionInputMonotonicStaticProperty | IFunctionInputEssentialStaticProperty
  @query('#target-selector') declare targetSelector: HTMLSelectElement
  @query('#input-selector') declare inputSelector: HTMLSelectElement

  conditionChanged (context: string): void {
    this.updateProperty({
      ...this.property,
      context
    })
  }

  targetChanged (event: Event): void {
    // if target (function) changes, invalidate the input
    let value: string | null = (event.target as HTMLSelectElement).value
    value = value === '' ? null : value
    this.updateProperty({
      ...this.property,
      target: value,
      input: null
    })
    this.inputSelector.selectedIndex = 0
  }

  inputChanged (event: Event): void {
    // if input changes, function stays the same
    let value: string | null = (event.target as HTMLSelectElement).value
    value = value === '' ? null : value
    this.updateProperty({
      ...this.property,
      input: value
    })
  }

  getTargetSelectorItems (): string[] {
    // get all functions in the model
    return this.contentData.functions.map(func => func.id)
  }

  getInputSelectorItems (): string[] {
    // get all inputs of the given function
    return this.contentData.functions
      .find(func => func.id === (this.property as IFunctionInputMonotonicStaticProperty).target)
      ?.variables.map(variable => variable.source) ?? []
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
    this.targetSelector.selectedIndex = this.getTargetSelectorItems().indexOf(this.property.target ?? '') + 1
    this.inputSelector.selectedIndex = this.getInputSelectorItems().indexOf(this.property.input ?? '') + 1
  }

  renderConditionField (): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-column uk-flex-left">
        <label class="condition-label">Context formula:</label>
        <div class="uk-flex uk-flex-row">
          <input id="condition-field" class="condition-field" .value="${this.property.context}"
                 @change="${(e: Event) => {
                   this.conditionChanged((e.target as HTMLInputElement).value)
                 }}"/>
        </div>
      </div>`
  }
}
