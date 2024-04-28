import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './dynamic-fixed-point.less?inline'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import PropertyTile from '../../property-tile/property-tile'
import { type IObservationSet } from '../../../../util/data-interfaces'

@customElement('dynamic-fixed-point')
export default class DynamicFixedPoint extends PropertyTile {
  static styles = css`${unsafeCSS(style_less)}`
  @property() observations: IObservationSet[] = []

  render (): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="uk-input uk-text-center" value="${this.prop.name}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          <button class="uk-button uk-button-small">
            ${icon(faTrash).node}
          </button>
        </div>
        <br>
        <div class="uk-flex uk-flex-row uk-flex-between uk-width-auto">
          <div>
            <label for="dataset">Dataset:</label>
            <select class="uk-select" name="dataset" id="dataset">
              <option value="x">x</option>
              <option value="y">y</option>
            </select>
          </div>
          <div>
            <label for="observation">Observation</label>
            <select class="uk-select" name="observation" id="observation">
              <option value="x">x</option>
              <option value="y">y</option>
            </select>
          </div>
        </div>
      </div>
      <hr>
    `
  }
}
