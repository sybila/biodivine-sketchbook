import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './static-generic.less?inline'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import AbstractProperty from '../../abstract-property/abstract-property'
import { type IGenericStaticProperty } from '../../../../util/data-interfaces'

@customElement('static-generic')
export default class StaticGeneric extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IGenericStaticProperty

  render (): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="uk-input uk-text-center" readonly value="${this.property.name}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          <button class="uk-button uk-button-small">
            ${icon(faTrash).node}
          </button>
        </div>
        <span class="uk-align-left uk-text-left uk-margin-remove">Value:</span>
        <input id="value-editor" class="uk-input" value="${this.property.value}">
      </div>
      <hr>
    `
  }
}
