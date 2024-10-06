import { css, html, LitElement, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './observations-editor.less?inline'
import './observations-set/observations-set'
import { ContentData, type IObservation, type IObservationSet } from '../../util/data-interfaces'
import { map } from 'lit/directives/map.js'
import { dialog } from '@tauri-apps/api'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/helpers/event'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../util/config'
import {
  aeonState,
  type DatasetMetaData,
  type DatasetData,
  type DatasetIdUpdateData,
  type ObservationData,
  type ObservationIdUpdateData
} from '../../../aeon_state'
import { when } from 'lit/directives/when.js'

@customElement('observations-editor')
export default class ObservationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData = ContentData.create()
  @state() datasetRenameIndex = -1
  @state() shownDatasets: number[] = []

  constructor () {
    super()

    // changes to whole datasets triggered by table buttons
    this.addEventListener('change-dataset-id', this.changeDatasetId)
    this.addEventListener('rename-dataset', this.renameDataset)
    this.addEventListener('push-new-observation', this.pushNewObservation)
    this.addEventListener('remove-observation', this.removeObservation)
    this.addEventListener('remove-dataset', (e) => { void this.removeDataset(e) })
    this.addEventListener('add-dataset-variable', this.addVariable)

    // changes to observations triggered by table edits
    this.addEventListener('change-observation', this.changeObservation)

    // event listeners for backend updates
    aeonState.sketch.observations.datasetLoaded.addEventListener(this.#onDatasetLoaded.bind(this))
    aeonState.sketch.observations.datasetCreated.addEventListener(this.#onDatasetCreated.bind(this))
    aeonState.sketch.observations.datasetRemoved.addEventListener(this.#onDatasetRemoved.bind(this))

    aeonState.sketch.observations.datasetContentChanged.addEventListener(this.#onDatasetContentChanged.bind(this))
    aeonState.sketch.observations.datasetIdChanged.addEventListener(this.#onDatasetIdChanged.bind(this))
    aeonState.sketch.observations.datasetNameChanged.addEventListener(this.#onDatasetNameChanged.bind(this))

    aeonState.sketch.observations.observationPushed.addEventListener(this.#onObservationPushed.bind(this))
    aeonState.sketch.observations.observationRemoved.addEventListener(this.#onObservationRemoved.bind(this))

    aeonState.sketch.observations.observationIdChanged.addEventListener(this.#onObservationIdChanged.bind(this))
    // these two handled the same way
    aeonState.sketch.observations.observationContentChanged.addEventListener(this.#onObservationContentChanged.bind(this))
    aeonState.sketch.observations.observationNameChanged.addEventListener(this.#onObservationContentChanged.bind(this))

    // refresh-event listeners
    aeonState.sketch.observations.datasetsRefreshed.addEventListener(this.#onDatasetsRefreshed.bind(this))

    // note that the refresh events are automatically triggered or handled (after app refresh) directly
    // from the root component (due to some dependency issues between different components)
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
  }

  private convertToIObservation (observationData: ObservationData, variables: string[]): IObservation {
    const obs: IObservation = {
      id: observationData.id,
      name: observationData.name,
      annotation: observationData.annotation,
      selected: false
    }
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
    return {
      id: observation.id,
      name: observation.name,
      annotation: observation.annotation,
      dataset: datasetId,
      values: valueString
    }
  }

  private convertToIObservationSet (datasetData: DatasetData): IObservationSet {
    const observations = datasetData.observations.map(
      observationData => this.convertToIObservation(observationData, datasetData.variables)
    )
    return {
      id: datasetData.id,
      name: datasetData.name,
      annotation: datasetData.annotation,
      observations,
      variables: datasetData.variables
    }
  }

  private convertFromIObservationSet (dataset: IObservationSet): DatasetData {
    const observations = dataset.observations.map(
      obs => this.convertFromIObservation(obs, dataset.id, dataset.variables)
    )
    return {
      id: dataset.id,
      name: dataset.name,
      annotation: dataset.annotation,
      observations,
      variables: dataset.variables
    }
  }

  #onDatasetsRefreshed (refreshedDatasets: DatasetData[]): void {
    const datasets = refreshedDatasets.map(d => this.convertToIObservationSet(d))
    this.updateObservations(datasets)
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
    // TODO: move dataset id generation to backend
    aeonState.sketch.observations.loadDataset(fileName)
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
      url: 'src/html/component-editor/observations-editor/observations-import/observations-import.html',
      title: 'Import observation set',
      alwaysOnTop: false,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })

    // Once loaded, show the dialog to edit and import the dataset.
    void importDialog.once('loaded', () => {
      void importDialog.emit('observations_import_update', {
        data,
        variables
      })
    })

    // Handle the case when data are successfully edited and imported
    void importDialog.once('observations_import_dialog', (event: TauriEvent<IObservation[]>) => {
      // this is just a placeholder, only the `observations` part will be used by backend
      const modifiedDataset: IObservationSet = {
        id: name,
        name,
        annotation: '',
        observations: event.payload,
        variables
      }
      // temporarily add the dataset in its current placeholder version, and send an event to backend with changes
      this.updateObservations(this.contentData.observations.concat(modifiedDataset))
      aeonState.sketch.observations.setDatasetContent(name, this.convertFromIObservationSet(modifiedDataset))
    })

    // Handle the case when the dialog is closed/cancelled
    void importDialog.once('observations_import_cancelled', () => {
      console.log('Import dialog was closed or cancelled.')
      // the dataset was temporarily added in its original form, now we just remove it
      aeonState.sketch.observations.removeDataset(name)
    })
  }

  #onDatasetContentChanged (data: DatasetData): void {
    const observationSet = this.convertToIObservationSet(data)
    const index = this.contentData.observations.findIndex(item => item.id === data.id)
    if (index === -1) return
    const datasets = structuredClone(this.contentData.observations)

    datasets[index] = observationSet
    this.updateObservations(datasets)
  }

  private createDataset (): void {
    aeonState.sketch.observations.addDefaultDataset()
  }

  #onDatasetCreated (data: DatasetData): void {
    console.log('Adding new dataset.')
    const newDataset = this.convertToIObservationSet(data)
    this.updateObservations(this.contentData.observations.concat(newDataset))
  }

  updateDatasetId = debounce((newId: string, index: number) => {
    const originalId = this.contentData.observations[index].id
    aeonState.sketch.observations.setDatasetId(originalId, newId)
  }, functionDebounceTimer
  )

  updateDatasetName = debounce((newName: string, index: number) => {
    const originalId = this.contentData.observations[index].id
    aeonState.sketch.observations.setDatasetName(originalId, newName)
  }, functionDebounceTimer
  )

  #onDatasetIdChanged (data: DatasetIdUpdateData): void {
    console.log(data)
    const index = this.contentData.observations.findIndex(d => d.id === data.original_id)
    if (index === -1) return
    const datasets = structuredClone(this.contentData.observations)
    datasets[index] = {
      ...datasets[index],
      id: data.new_id
    }
    this.updateObservations(datasets)
  }

  #onDatasetNameChanged (data: DatasetMetaData): void {
    console.log(data)
    const datasetIndex = this.contentData.observations.findIndex(d => d.id === data.id)
    if (datasetIndex === -1) return

    const datasets = structuredClone(this.contentData.observations)
    datasets[datasetIndex] = {
      ...datasets[datasetIndex],
      name: data.name
    }
    this.updateObservations(datasets)
  }

  private pushNewObservation (event: Event): void {
    // push new observation (placeholder) that is fully generated on backend
    const detail = (event as CustomEvent).detail
    aeonState.sketch.observations.pushObservation(detail.id)
  }

  #onObservationPushed (data: ObservationData): void {
    const datasetIndex = this.contentData.observations.findIndex(d => d.id === data.dataset)
    if (datasetIndex === -1) return
    const datasets = structuredClone(this.contentData.observations)
    datasets[datasetIndex].observations.push(this.convertToIObservation(data, datasets[datasetIndex].variables))
    this.updateObservations(datasets)
  }

  private removeObservation (event: Event): void {
    // push new observation (placeholder) that is fully generated on backend
    const detail = (event as CustomEvent).detail
    aeonState.sketch.observations.removeObservation(detail.dataset, detail.id)
  }

  private addVariable (event: Event): void {
    // add new variable (placeholder) that is fully generated on backend
    const detail = (event as CustomEvent).detail
    aeonState.sketch.observations.addDatasetVariable(detail.id)
    aeonState.sketch.observations.refreshDatasets()
  }

  #onObservationRemoved (data: ObservationData): void {
    const datasetIndex = this.contentData.observations.findIndex(d => d.id === data.dataset)
    if (datasetIndex === -1) return
    const datasets: IObservationSet[] = structuredClone(this.contentData.observations)
    datasets[datasetIndex].observations = datasets[datasetIndex].observations.filter(obs => obs.id !== data.id)
    this.updateObservations(datasets)
  }

  private changeObservation (event: Event): void {
    const detail = (event as CustomEvent).detail
    const dataset = this.contentData.observations.find(ds => ds.id === detail.dataset)
    if (dataset === undefined) return
    if (detail.id !== detail.observation.id) {
      aeonState.sketch.observations.setObservationId(dataset.id, detail.id, detail.observation.id)
    }
    const obsData = this.convertFromIObservation(detail.observation, dataset.id, dataset.variables)
    if (detail.name !== obsData.name) {
      aeonState.sketch.observations.setObservationName(dataset.id, obsData)
    }
    aeonState.sketch.observations.setObservationContent(dataset.id, obsData)
  }

  #onObservationContentChanged (data: ObservationData): void {
    const datasetIndex = this.contentData.observations.findIndex(d => d.id === data.dataset)
    if (datasetIndex === -1) return
    const obsIndex = this.contentData.observations[datasetIndex].observations.findIndex(obs => obs.id === data.id)
    if (obsIndex === -1) return
    const datasets: IObservationSet[] = structuredClone(this.contentData.observations)
    datasets[datasetIndex].observations[obsIndex] = this.convertToIObservation(data, datasets[datasetIndex].variables)
    this.updateObservations(datasets)
  }

  #onObservationIdChanged (data: ObservationIdUpdateData): void {
    // data.metadata seems to be dataset todo: confirm with ondrej
    const datasetIndex = this.contentData.observations.findIndex(d => d.id === data.metadata)
    if (datasetIndex === -1) return
    const obsIndex = this.contentData.observations[datasetIndex].observations.findIndex(obs => obs.id === data.original_id)
    if (obsIndex === -1) return
    const datasets: IObservationSet[] = structuredClone(this.contentData.observations)
    datasets[datasetIndex].observations[obsIndex].id = data.new_id
    this.updateObservations(datasets)
  }

  updateObservations (datasets: IObservationSet[]): void {
    this.dispatchEvent(new CustomEvent('save-observations', {
      detail: {
        datasets
      },
      bubbles: true,
      composed: true
    }))
  }

  renameDataset (event: Event): void {
    const detail = (event as CustomEvent).detail
    this.datasetRenameIndex = this.contentData.observations.findIndex(d => d.id === detail.id);
    (this.shadowRoot?.querySelector('#set-name-' + this.datasetRenameIndex) as HTMLInputElement)?.focus()
  }

  changeDatasetId (event: Event): void {
    const detail = (event as CustomEvent).detail
    this.datasetRenameIndex = this.contentData.observations.findIndex(d => d.id === detail.id);
    (this.shadowRoot?.querySelector('#set-data-id-' + this.datasetRenameIndex) as HTMLInputElement)?.focus()
  }

  async removeDataset (event: Event): Promise<void> {
    if (!await dialog.confirm('Remove dataset?')) return
    const detail = (event as CustomEvent).detail
    aeonState.sketch.observations.removeDataset(detail.id)
  }

  #onDatasetRemoved (data: DatasetData): void {
    const datasets = this.contentData.observations.filter(d => d.id !== data.id)
    this.updateObservations(datasets)
  }

  toggleDataset (index: number): void {
    const dsIndex = this.shownDatasets.indexOf(index)
    if (dsIndex === -1) {
      this.shownDatasets = this.shownDatasets.concat([index])
      return
    }
    const shownDatasets = [...this.shownDatasets]
    shownDatasets.splice(dsIndex, 1)
    this.shownDatasets = shownDatasets
  }

  render (): TemplateResult {
    return html`
      <div class="observations">
        <div class="header uk-background-primary uk-margin-bottom">
          <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Observations</h3>
          <div class="buttons-container">
            <button @click="${this.createDataset}" class="uk-button uk-button-primary uk-button-small create-button">+ Create</button>
            <button @click="${this.loadDataset}" class="uk-button uk-button-primary uk-button-small import-button">+ Import</button>
          </div>
        </div>
        ${this.contentData?.observations.length === 0 ? html`<div class="uk-text-center"><span class="uk-label uk-margin-bottom">No observations loaded</span></div>` : ''}
        <div class="accordion-body">
          <div class="accordion uk-margin-small-left uk-margin-small-right">
            ${map(this.contentData.observations, (dataset, index) => html`
              <div class="container ${this.shownDatasets.includes(index) ? 'active' : ''}" id="${'container' + index}">
                <div class="label name-id-container" @click="${() => {
                  this.toggleDataset(index)
                }}">
                  <input
                    @input="${(e: InputEvent) => {
                      this.updateDatasetName((e.target as HTMLInputElement).value, index)
                    }}"
                    ?readonly="${this.datasetRenameIndex !== index}"
                    @keydown="${(e: KeyboardEvent) => {
                      if (e.key === 'Enter') {
                        this.datasetRenameIndex = -1
                      }
                    }}"
                    @blur="${() => { this.datasetRenameIndex = -1 }}"
                    class="set-name heading uk-input uk-form-blank" id="${'set-name-' + index}"
                    .value="${dataset.name}"/>
                  ID = 
                  <input
                    @input="${(e: InputEvent) => {
                      this.updateDatasetId((e.target as HTMLInputElement).value, index)
                    }}"
                    ?readonly="${this.datasetRenameIndex !== index}"
                    @keydown="${(e: KeyboardEvent) => {
                      if (e.key === 'Enter') {
                        this.datasetRenameIndex = -1
                      }
                    }}"
                    @blur="${() => { this.datasetRenameIndex = -1 }}"
                    class="set-data-id heading uk-input uk-form-blank" id="${'set-data-id-' + index}"
                    .value="${dataset.id}"/>
                </div>
                ${when(this.shownDatasets.includes(index), () => html`
                  <div class="content">
                    <observations-set
                        .data="${dataset}">
                    </observations-set>
                  </div>
                `)}
              </div>
              <hr class="uk-margin-bottom uk-margin-left uk-margin-right">
            `)}
          </div>
        </div>
      </div>
      
    `
  }
}
