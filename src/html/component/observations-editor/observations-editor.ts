import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './observations-editor.less?inline'
import './observations-set/observations-set'
import { ContentData, type IObservation, type IObservationSet, DataCategory } from '../../util/data-interfaces'
import { map } from 'lit/directives/map.js'
import { dialog } from '@tauri-apps/api'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/helpers/event'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../util/config'
import {
  aeonState,
  type DatasetData,
  type DatasetIdUpdateData,
  type ObservationData,
  type ObservationIdUpdateData
} from '../../../aeon_events'

@customElement('observations-editor')
export default class ObservationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData = ContentData.create()
  @state() datasets: IObservationSet[] = []
  index = 0

  constructor () {
    super()

    // observations-related event listeners
    aeonState.observations.datasetLoaded.addEventListener(this.#onDatasetLoaded.bind(this))
    aeonState.observations.datasetContentChanged.addEventListener(this.#onDatasetContentChanged.bind(this))
    aeonState.observations.datasetIdChanged.addEventListener(this.#onDatasetIdChanged.bind(this))
    this.addEventListener('push-new-observation', this.pushNewObservation)
    aeonState.observations.observationPushed.addEventListener(this.#onObservationPushed.bind(this))
    this.addEventListener('remove-observation', this.removeObservation)
    aeonState.observations.observationRemoved.addEventListener(this.#onObservationRemoved.bind(this))
    this.addEventListener('change-observation', this.changeObservation)
    aeonState.observations.observationContentChanged.addEventListener(this.#onObservationContentChanged.bind(this))
    aeonState.observations.observationIdChanged.addEventListener(this.#onObservationIdChanged.bind(this))
    // TODO add all other events

    // refresh-event listeners
    aeonState.observations.datasetsRefreshed.addEventListener(this.#onDatasetsRefreshed.bind(this))

    // refreshing content from backend
    aeonState.observations.refreshDatasets()
  }

  private convertToIObservation (observationData: ObservationData, variables: string[]): IObservation {
    const obs: IObservation = { id: observationData.id, name: observationData.id }
    variables.forEach(((v, idx) => {
      const value = observationData.values[idx]
      obs[v] = (value === '*') ? '' : value
    }))
    return obs
  }

  private convertFromIObservation (observation: IObservation, datasetId: string, variables: string[]): ObservationData {
    const valueString = variables.map(v => {
      return (observation[v] === '') ? '*' : observation[v]
    }).join('')
    return { id: observation.id, dataset: datasetId, values: valueString }
  }

  private convertToIObservationSet (datasetData: DatasetData): IObservationSet {
    const observations = datasetData.observations.map(
      observationData => this.convertToIObservation(observationData, datasetData.variables)
    )
    return {
      id: datasetData.id,
      observations,
      variables: datasetData.variables,
      category: datasetData.category
    }
  }

  private convertFromIObservationSet (dataset: IObservationSet): DatasetData {
    const observations = dataset.observations.map(
      obs => this.convertFromIObservation(obs, dataset.id, dataset.variables)
    )
    return {
      id: dataset.id,
      observations,
      variables: dataset.variables,
      category: dataset.category
    }
  }

  #onDatasetsRefreshed (refreshedDatasets: DatasetData[]): void {
    const datasets = refreshedDatasets.map(d => this.convertToIObservationSet(d))
    this.index = datasets.length
    this.datasets = datasets
  }

  private async loadDataset (): Promise<void> {
    const handle = await dialog.open({
      title: 'Import observation set...',
      multiple: false,
      filters: [
        {
          name: 'Comma-separated values',
          extensions: ['csv']
        },
        {
          name: 'Tab-separated values',
          extensions: ['tsv', 'tab']
        },
        {
          name: 'All',
          extensions: ['*']
        }
      ]
    })
    if (handle === null) return
    let fileName
    if (Array.isArray(handle)) {
      fileName = handle.pop() ?? 'unknown'
    } else {
      fileName = handle
    }

    aeonState.observations.loadDataset(fileName, 'dataset' + this.index)
  }

  #onDatasetLoaded (data: DatasetData): void {
    const newDataset = this.convertToIObservationSet(data)
    // just call import dialog, dataset will be filtered and then added
    void this.importObservations(newDataset.id, newDataset.observations, newDataset.variables)
  }

  private async importObservations (name: string, data: IObservation[], variables: string[]): Promise<void> {
    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    const importDialog = new WebviewWindow(`editObservation${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component/observations-editor/observations-import/observations-import.html',
      title: 'Import observation set',
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    void importDialog.once('loaded', () => {
      void importDialog.emit('observations_import_update', {
        data,
        variables
      })
    })
    void importDialog.once('observations_import_dialog', (event: TauriEvent<IObservation[]>) => {
      const modifiedDataset: IObservationSet = {
        id: name,
        observations: event.payload,
        variables,
        category: DataCategory.UNSPECIFIED
      }
      // temporarily add the dataset in its current version, but also send an event to backend with changes
      this.datasets = this.datasets.concat(modifiedDataset)
      this.index++
      aeonState.observations.setDatasetContent(name, this.convertFromIObservationSet(modifiedDataset))
    })
  }

  #onDatasetContentChanged (data: DatasetData): void {
    const observationSet = this.convertToIObservationSet(data)
    const index = this.datasets.findIndex(item => item.id === data.id)
    if (index === -1) return
    const datasets = structuredClone(this.datasets)

    datasets[index] = observationSet
    this.datasets = datasets
  }

  updateDatasetId = debounce((newId: string, index: number) => {
    const originalId = this.datasets[index].id
    aeonState.observations.setDatasetId(originalId, newId)
  }, functionDebounceTimer
  )

  #onDatasetIdChanged (data: DatasetIdUpdateData): void {
    console.log(data)
    const index = this.datasets.findIndex(d => d.id === data.original_id)
    if (index === -1) return
    const datasets = structuredClone(this.datasets)
    datasets[index] = {
      ...datasets[index],
      id: data.new_id
    }
    this.datasets = datasets
  }

  private pushNewObservation (event: Event): void {
    // push new observation (placeholder) that is fully generated on backend
    const detail = (event as CustomEvent).detail
    aeonState.observations.pushObservation(detail.id)
  }

  #onObservationPushed (data: ObservationData): void {
    const datasetIndex = this.datasets.findIndex(d => d.id === data.dataset)
    if (datasetIndex === -1) return
    const datasets = structuredClone(this.datasets)
    datasets[datasetIndex].observations.push(this.convertToIObservation(data, datasets[datasetIndex].variables))
    this.datasets = datasets
  }

  private removeObservation (event: Event): void {
    // push new observation (placeholder) that is fully generated on backend
    const detail = (event as CustomEvent).detail
    aeonState.observations.removeObservation(detail.dataset, detail.id)
  }

  #onObservationRemoved (data: ObservationData): void {
    const datasetIndex = this.datasets.findIndex(d => d.id === data.dataset)
    if (datasetIndex === -1) return
    const datasets: IObservationSet[] = structuredClone(this.datasets)
    datasets[datasetIndex].observations = datasets[datasetIndex].observations.filter(obs => obs.id !== data.id)
    this.datasets = datasets
  }

  private changeObservation (event: Event): void {
    const detail = (event as CustomEvent).detail
    const dataset = this.datasets.find(ds => ds.id === detail.dataset)
    if (dataset === undefined) return
    if (detail.id !== detail.observation.id) {
      aeonState.observations.setObservationId(dataset.id, detail.id, detail.observation.id)
    }
    const obsData = this.convertFromIObservation(detail.observation, dataset.id, dataset.variables)
    aeonState.observations.setObservationContent(detail.dataset, obsData)
  }

  #onObservationContentChanged (data: ObservationData): void {
    const datasetIndex = this.datasets.findIndex(d => d.id === data.dataset)
    if (datasetIndex === -1) return
    const obsIndex = this.datasets[datasetIndex].observations.findIndex(obs => obs.id === data.id)
    if (obsIndex === -1) return
    const datasets: IObservationSet[] = structuredClone(this.datasets)
    datasets[datasetIndex].observations[obsIndex] = this.convertToIObservation(data, datasets[datasetIndex].variables)
    this.datasets = datasets
  }

  #onObservationIdChanged (data: ObservationIdUpdateData): void {
    // data.metadata seems to be dataset todo: confirm with ondrej
    const datasetIndex = this.datasets.findIndex(d => d.id === data.metadata)
    if (datasetIndex === -1) return
    const obsIndex = this.datasets[datasetIndex].observations.findIndex(obs => obs.id === data.original_id)
    if (obsIndex === -1) return
    const datasets: IObservationSet[] = structuredClone(this.datasets)
    datasets[datasetIndex].observations[obsIndex].id = data.new_id
    datasets[datasetIndex].observations[obsIndex].name = data.new_id
    this.datasets = datasets
  }

  render (): TemplateResult {
    return html`
      <div class="observations">
        <div class="header">
          <div></div>
          <h1 class="heading uk-heading-line uk-text-center">Observations</h1>
          <button @click="${this.loadDataset}" class="uk-button uk-button-primary uk-button-small import-button">+ Import</button>
        </div>
        <div class="accordion-body">
          <div class="accordion">
            ${map(this.datasets, (dataset, index) => html`
          <div class="container" id="${'container' + index}">
            <div class="label" @click="${() => { this.shadowRoot?.getElementById('container' + index)?.classList.toggle('active') }}" >
              <input 
                  @input="${(e: InputEvent) => {
                    this.updateDatasetId((e.target as HTMLInputElement).value, index)
                  }}"
                  ?readonly="${true}"
                  @dblclick="${(e: InputEvent) => {
                    (e.target as HTMLInputElement).readOnly = !(e.target as HTMLInputElement).readOnly
                  }}"
                  class="set-name heading uk-input uk-form-blank"
                  value="${dataset.id}"/>
            </div>
            <div class="content">
              <observations-set
                  .data="${dataset}">
              </observations-set>
            </div>
          </div>
          <hr>
        `)}
          </div>
        </div>
      </div>
      
    `
  }
}
