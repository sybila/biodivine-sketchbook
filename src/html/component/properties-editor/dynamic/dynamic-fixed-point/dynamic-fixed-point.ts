import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property, query } from 'lit/decorators.js'
import style_less from './dynamic-fixed-point.less?inline'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import PropertyTile from '../../property-tile/property-tile'
import { type IFixedPointDynamicProperty, type IObservationSet } from '../../../../util/data-interfaces'
import { map } from 'lit/directives/map.js'

@customElement('dynamic-fixed-point')
export default class DynamicFixedPoint extends PropertyTile {
  static styles = css`${unsafeCSS(style_less)}`
  @query('#observation') declare observationSelector: HTMLSelectElement
  @property() declare property: IFixedPointDynamicProperty
  @property() observations: IObservationSet[] = []

  datasetChanged (event: Event): void {
    const datasetId = (event.target as HTMLSelectElement).value
    this.updateProperty({
      ...this.property,
      dataset: datasetId,
      observation: ''
    })
  }

  observationChanged (event: Event): void {
    const observation = (event.target as HTMLSelectElement).value
    this.updateProperty({
      ...this.property,
      observation
    })
  }

  render (): TemplateResult {
    return html`
      <div class="property-body uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="uk-input uk-text-center" value="${this.property.name}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          <button class="uk-button uk-button-small">
            ${icon(faTrash).node}
          </button>
        </div>
        <div class="uk-flex uk-flex-row uk-flex-around uk-width-auto">
          <div>
            <label for="dataset">Dataset:</label>
            <select class="uk-select uk-width-max-content" name="dataset" id="dataset" @change=${this.datasetChanged}>
              <option value=${undefined}>---</option>
              ${map(this.observations, (observationSet) => html`
                <option value="${observationSet.id}">${observationSet.id}</option>
              `)}
            </select>
          </div>
          <div>
            <label for="observation">Observation:</label>
            <select class="uk-select uk-width-max-content" name="observation" id="observation"
                    @change=${this.observationChanged}>
              <option value=${undefined}>---</option>
              ${map(this.observations[this.observations.findIndex(dataset => dataset.id === this.property.dataset)]?.observations, (observation) => html`
                <option value="${observation.id}">${observation.id}</option>
              `)}
            </select>
          </div>
        </div>
      </div>
      <hr>
    `
  }
}
