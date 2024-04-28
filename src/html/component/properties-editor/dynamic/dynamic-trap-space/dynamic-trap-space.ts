import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query } from 'lit/decorators.js'
import style_less from './dynamic-trap-space.less?inline'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import PropertyTile from '../../property-tile/property-tile'
import { type IObservationSet, type ITrapSpaceDynamicProperty } from '../../../../util/data-interfaces'
import { map } from 'lit/directives/map.js'

@customElement('dynamic-trap-space')
export default class DynamicTrapSpace extends PropertyTile {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: ITrapSpaceDynamicProperty
  @property() declare observations: IObservationSet[]
  @query('#dataset') declare datasetSelector: HTMLSelectElement

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

  minimalChanged (): void {
    this.updateProperty({
      ...this.property,
      minimal: !this.property.minimal
    })
  }

  nonpercolableChanged (): void {
    this.updateProperty({
      ...this.property,
      nonpercolable: !this.property.nonpercolable
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
                    value="${this.property.observation}"
                    @change=${this.observationChanged}>
              <option value=${undefined}>---</option>
              ${map(this.observations[this.observations.findIndex(dataset => dataset.id === this.property.dataset)]?.observations,
                  (observation) => html`
                <option value="${observation.id}">${observation.id}</option>
              `)}
            </select>
          </div>
        </div>
        <div class="uk-flex uk-flex-row uk-flex-around">
          <div>
            <input class="uk-checkbox" type="checkbox" id="minimal" name="minimal" value=${this.property.minimal}
                   @change=${this.minimalChanged}  />
            <label for="minimal">minimal</label>
          </div>
          <div>
            <input class="uk-checkbox" type="checkbox" id="non-percolable" name="non-percolable"
                   value=${this.property.nonpercolable} @change=${this.nonpercolableChanged} />
            <label for="non-percolable">non-percolable</label>
          </div>
        </div>
      </div>
      <hr>
    `
  }
}
