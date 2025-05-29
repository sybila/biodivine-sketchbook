import { html, type PropertyValues, type TemplateResult } from 'lit'
import { property, query } from 'lit/decorators.js'
import AbstractStaticProperty from './abstract-static-property'
import {
  type ContentData,
  type IVariableRegulatorEssentialStaticProperty,
  type IVariableRegulatorMonotonicStaticProperty
} from '../../../util/data-interfaces'

export default class StaticRegSelectorsProperty extends AbstractStaticProperty {
  @property() declare contentData: ContentData
  @property() declare property: IVariableRegulatorMonotonicStaticProperty | IVariableRegulatorEssentialStaticProperty
  @query('#target-selector') declare targetSelector: HTMLSelectElement
  @query('#input-selector') declare inputSelector: HTMLSelectElement

  conditionChanged (context: string): void {
    this.updateProperty({
      ...this.property,
      context
    })
  }

  targetChanged (event: Event): void {
    // if the target of the regulation changes, input stays the same
    let value: string | null = (event.target as HTMLSelectElement).value
    value = value === '' ? null : value
    this.updateProperty({
      ...this.property,
      target: value
    })
  }

  inputChanged (event: Event): void {
    // if the input of the regulation changes, invalidate the target
    let value: string | null = (event.target as HTMLSelectElement).value
    value = value === '' ? null : value
    this.updateProperty({
      ...this.property,
      input: value,
      target: null
    })
    this.targetSelector.selectedIndex = 0
  }

  getTargetSelectorItems (): string[] {
    // get all target variables regulated by the selected source variable
    return this.contentData.regulations
      .filter(regulation => regulation.source === (this.property as IVariableRegulatorMonotonicStaticProperty).input)
      .map(regulation => regulation.target)
  }

  getInputSelectorItems (): string[] {
    // get all variables in the model
    return this.contentData.variables.map(variable => variable.id)
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
