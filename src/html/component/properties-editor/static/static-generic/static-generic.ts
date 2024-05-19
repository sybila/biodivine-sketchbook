import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-generic.less?inline'
import { type IGenericStaticProperty } from '../../../../util/data-interfaces'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../../../util/config'
import abstractStaticProperty from '../abstract-static-property'

@customElement('static-generic')
export default class StaticGeneric extends abstractStaticProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IGenericStaticProperty

  valueChanged = debounce((formula: string): void => {
    this.updateProperty({
      ...this.property,
      formula
    })
  }, functionDebounceTimer)

  render (): TemplateResult {
    return html`
      <div class="property-body">
        ${this.renderNameplate()}
        <div class="uk-flex uk-flex-column uk-flex-left">
          <label class="value-label">Context formula:</label>
          <div class="uk-flex uk-flex-row">
            <input id="value-editor" class="uk-input" .value="${this.property.formula}"
                   @input="${(e: Event) => { this.valueChanged((e.target as HTMLInputElement).value) }}"/>
          </div>
        </div>
      </div>
      <hr class="uk-margin-top uk-margin-bottom uk-margin-left uk-margin-right">
    `
  }
}
