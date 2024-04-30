import { css, html, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query } from 'lit/decorators.js'
import style_less from './dynamic-trajectory.less?inline'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import AbstractProperty from '../../abstract-property/abstract-property'
import { map } from 'lit/directives/map.js'

import { type IExistsTrajectoryDynamicProperty, type IObservationSet } from '../../../../util/data-interfaces'

@customElement('dynamic-trajectory')
export default class DynamicTrajectory extends AbstractProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @query('#observation') declare observationSelector: HTMLSelectElement
  @property() declare property: IExistsTrajectoryDynamicProperty
  @property() observations: IObservationSet[] = []

  datasetChanged (event: Event): void {
    const datasetId = (event.target as HTMLSelectElement).value
    this.updateProperty({
      ...this.property,
      dataset: datasetId
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
        </div>
      </div>
      <hr>
    `
  }
}
