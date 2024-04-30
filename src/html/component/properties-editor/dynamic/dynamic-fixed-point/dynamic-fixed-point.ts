import { html, css, unsafeCSS, type TemplateResult } from 'lit'
import { customElement, property, query } from 'lit/decorators.js'
import style_less from './dynamic-fixed-point.less?inline'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import AbstractProperty from '../../abstract-property/abstract-property'
import { type IFixedPointDynamicProperty, type IObservationSet } from '../../../../util/data-interfaces'
import { map } from 'lit/directives/map.js'

@customElement('dynamic-fixed-point')
export default class DynamicFixedPoint extends AbstractProperty {
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
    this.observationSelector.selectedIndex = 0
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
      <div class="property-body">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="name-field" value="${this.property.name}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          <button class="remove-property" @click="${this.removeProperty}">
            ${icon(faTrash).node}
          </button>
        </div>
        <div class="uk-flex uk-flex-row uk-flex-around">
          <div class="uk-flex uk-flex-row uk-flex-around uk-flex-middle">
            <label for="dataset">Dataset:</label>
            <div class="uk-width-3-4">
              <select class="uk-select uk-margin-small-left" name="dataset" id="dataset" @change=${this.datasetChanged}>
                <option value=${undefined}>---</option>
                ${map(this.observations, (observationSet) => html`
                  <option value="${observationSet.id}">${observationSet.id}</option>
                `)}
              </select>
            </div>
          </div>
          <div class="uk-flex uk-flex-row uk-flex-around uk-flex-middle">
            <label for="observation">Observation:</label>
            <div class="uk-width-3-4">
              <select class="uk-select uk-margin-small-left" name="observation" id="observation"
                      @change=${this.observationChanged}>
                <option value=${undefined}>---</option>
                ${map(this.observations[this.observations.findIndex(dataset => dataset.id === this.property.dataset)]?.observations,
                    (observation) => html`
                      <option value="${observation.id}">${observation.id}</option>
                    `)}
              </select>
            </div>
          </div>
        </div>
      </div>
      <hr>
    `
  }
}
