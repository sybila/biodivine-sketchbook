import { css, html, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query } from 'lit/decorators.js'
import style_less from './dynamic-obs-selection.less?inline'
import {
  DynamicPropertyType,
  type IObservationSet,
  type ITrapSpaceDynamicProperty
} from '../../../../util/data-interfaces'
import { map } from 'lit/directives/map.js'
import AbstractDynamicProperty from '../abstract-dynamic-property'
import { when } from 'lit/directives/when.js'

const ALL = '*'

@customElement('dynamic-obs-selection')
export default class DynamicObsSelection extends AbstractDynamicProperty {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: ITrapSpaceDynamicProperty
  @property() declare observations: IObservationSet[]
  @query('#dataset') declare datasetSelector: HTMLSelectElement
  @query('#observation') declare observationSelector: HTMLSelectElement

  datasetChanged (event: Event): void {
    const datasetId = (event.target as HTMLSelectElement).value
    this.updateProperty({
      ...this.property,
      dataset: datasetId === '' ? null : datasetId,
      observation: this.property.variant === DynamicPropertyType.HasAttractor ? ALL : null
    })
    if (this.property.variant !== DynamicPropertyType.ExistsTrajectory) {
      this.observationSelector.selectedIndex = 0
    }
  }

  observationChanged (event: Event): void {
    const observation = (event.target as HTMLSelectElement).value
    this.updateProperty({
      ...this.property,
      observation: observation === '' ? null : observation
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

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
    const obsIndex = this.observations.findIndex(dataset => dataset.id === this.property.dataset)
    this.datasetSelector.selectedIndex = obsIndex + 1
    this.observationSelector.selectedIndex = this.observations[obsIndex]?.observations.findIndex(obs => obs.id === this.property.observation) + 1
  }

  render (): TemplateResult {
    return html`
      <div class="property-body">
        ${this.renderNameplate()}
        <div class="uk-flex uk-flex-row uk-flex-around">
          <div class="uk-flex uk-flex-row uk-flex-around uk-flex-middle">
            <label for="dataset">Dataset:</label>
            <div class="uk-width-3-4">
              <select class="uk-select uk-margin-small-left" name="dataset" id="dataset" @change=${this.datasetChanged}>
                <option value=${null}>---</option>
                ${map(this.observations, (observationSet) => html`
                  <option value="${observationSet.id}">${observationSet.id}</option>
                `)}
              </select>
            </div>
          </div>

          ${when(this.property.variant !== DynamicPropertyType.ExistsTrajectory, () => html`
            <div class="uk-flex uk-flex-row uk-flex-around uk-flex-middle">
              <label for="observation">Observation:</label>
              <div class="uk-width-3-4">
                <select class="uk-select uk-margin-small-left" name="observation" id="observation"
                        @change=${this.observationChanged}
                        ?disabled="${this.property.dataset === null}">
                  ${when(this.property.variant === DynamicPropertyType.HasAttractor,
                      () => html`
                        <option value=${'*'}>all</option>`,
                      () => html`
                        <option value=${null}>---</option>`)}
                  ${map(this.observations[this.observations.findIndex(dataset => dataset.id === this.property.dataset)]?.observations,
                      (observation) => html`
                        <option value="${observation.id}">${observation.id}</option>
                      `)}
                </select>
              </div>
            </div>`)}
        </div>

        ${when(this.property.variant === DynamicPropertyType.TrapSpace, () => html`
          <div class="uk-flex uk-flex-row uk-flex-around">
            <div class="toggle">
              <input class="uk-checkbox" type="checkbox" id="minimal" name="minimal" ?checked=${this.property.minimal}
                     @change=${this.minimalChanged} />
              <label class="pointer" for="minimal">minimal</label>
            </div>
            <div class="pointer">
              <input class="uk-checkbox" type="checkbox" id="non-percolable" name="non-percolable"
                     ?checked=${this.property.nonpercolable} @change=${this.nonpercolableChanged} />
              <label class="pointer" for="non-percolable">non-percolable</label>
            </div>
          </div>`)}
      </div>
      <hr>
    `
  }
}
