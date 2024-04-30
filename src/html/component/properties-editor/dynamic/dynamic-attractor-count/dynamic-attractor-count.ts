import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './dynamic-attractor-count.less?inline'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import AbstractProperty from '../../abstract-property/abstract-property'
import { type IAttractorCountDynamicProperty } from '../../../../util/data-interfaces'
import { when } from 'lit/directives/when.js'

@customElement('dynamic-attractor-count')
export default class DynamicAttractorCount extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IAttractorCountDynamicProperty
  @state() exact = true

  setExact (exact: boolean): void {
    if (exact) {
      this.updateProperty({
        ...this.property,
        upper: this.property.lower
      })
    }
    this.exact = exact
  }

  exactChanged (event: Event): void {
    const value = +(event.target as HTMLInputElement).value
    this.updateProperty({
      ...this.property,
      upper: value,
      lower: value
    })
  }

  lowerChanged (event: Event): void {
    const value = +(event.target as HTMLInputElement).value
    this.updateProperty({
      ...this.property,
      lower: value
    })
  }

  upperChanged (event: Event): void {
    const value = +(event.target as HTMLInputElement).value
    this.updateProperty({
      ...this.property,
      upper: value
    })
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="name-field" value="${this.property.name}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          <button class="remove-property" @click="${this.removeProperty}">
            ${icon(faTrash).node}
          </button>
        </div>
        <div class="uk-flex uk-flex-row uk-flex-center">
          <button class="uk-button uk-button-small ${this.exact ? 'uk-button-primary' : 'uk-button-secondary'}"
                  @click="${() => {
                    this.setExact(true)
                  }}">Exact
          </button>
          <button class="uk-button uk-button-small ${!this.exact ? 'uk-button-primary' : 'uk-button-secondary'}"
                  @click="${() => {
                    this.setExact(false)
                  }}">Range
          </button>
        </div>
        ${when(this.exact,
            () => html`
              <div class="uk-flex uk-flex-row uk-flex-around uk-width-auto">
                <div class="uk-flex uk-flex-row uk-flex-middle uk-flex-center">
                  <label for="exact">Attractor count:</label>
                  <div class="uk-width-1-2">
                    <input class="uk-input uk-margin-small-left" id="exact" name="exact" type="number" min="0"
                           value="${this.property.lower}" @change="${this.exactChanged}">
                  </div>
                </div>
              </div>`,
            () => html`
              <div class="uk-flex uk-flex-row uk-flex-around uk-width-auto">
                <div class="uk-flex uk-flex-row uk-flex-middle uk-flex-center uk-width-1-2">
                  <label for="lower">Min:</label>
                  <div class="uk-width-1-2">
                    <input class="uk-input uk-margin-small-left" id="lower" name="lower" type="number" min="0" max="${this.property.upper}"
                           value="${this.property.lower}" @change="${this.lowerChanged}">
                  </div>
                </div>
                <div class="uk-flex uk-flex-row uk-flex-middle uk-flex-center uk-width-1-2">
                  <label for="upper">Max:</label>
                  <div class="uk-width-1-2">
                    <input class="uk-input uk-margin-small-left" id="upper" name="upper" type="number" min="${this.property.lower}"
                           value="${this.property.upper}" @change="${this.upperChanged}">
                  </div>
                </div>
              </div>`)}

      </div>
      <hr>
    `
  }
}
