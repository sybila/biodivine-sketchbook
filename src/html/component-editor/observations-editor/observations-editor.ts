import { css, html, LitElement, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './observations-editor.less?inline'
import './observations-set/observations-set'
import { ContentData, type IObservation, type IObservationSet } from '../../util/data-interfaces'
import { map } from 'lit/directives/map.js'
import { dialog } from '@tauri-apps/api'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/helpers/event'
import {
  aeonState,
  type DatasetMetaData,
  type DatasetData,
  type DatasetIdUpdateData,
  type ObservationData,
  type ObservationIdUpdateData
} from '../../../aeon_state'
import {
  convertFromIObservation, convertToIObservation,
  convertFromIObservationSet, convertToIObservationSet
} from '../../util/utilities'
import { when } from 'lit/directives/when.js'

/** Component responsible for the observation editor of the editor session. */
@customElement('observations-editor')
export default class ObservationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData = ContentData.create()
  @state() datasetRenameIndex = -1
  @state() shownDatasets: number[] = []
  // dataset edit dialogs
  dialogs: Record<string, WebviewWindow | undefined> = {}

  constructor () {
    super()

    // changes to whole datasets triggered by table buttons
    this.addEventListener('push-new-observation', this.pushNewObservation)
    this.addEventListener('remove-observation', this.removeObservation)
    this.addEventListener('remove-dataset', (e) => { void this.removeDataset(e) })
    this.addEventListener('add-dataset-variable', this.addVariable)
    this.addEventListener('edit-dataset', (e) => { void this.editDataset(e) })

    // changes to observations triggered by table edits
    this.addEventListener('change-observation', this.changeObservation)

    // event listeners for backend updates
    aeonState.sketch.observations.datasetLoaded.addEventListener(this.#onDatasetLoaded.bind(this))
    aeonState.sketch.observations.datasetCreated.addEventListener(this.#onDatasetCreated.bind(this))
    aeonState.sketch.observations.datasetRemoved.addEventListener(this.#onDatasetRemoved.bind(this))

    aeonState.sketch.observations.datasetContentChanged.addEventListener(this.#onDatasetContentChanged.bind(this))
    aeonState.sketch.observations.datasetIdChanged.addEventListener(this.#onDatasetIdChanged.bind(this))
    aeonState.sketch.observations.datasetMetadataChanged.addEventListener(this.#onDatasetMetadataChanged.bind(this))

    aeonState.sketch.observations.observationPushed.addEventListener(this.#onObservationPushed.bind(this))
    aeonState.sketch.observations.observationRemoved.addEventListener(this.#onObservationRemoved.bind(this))

    aeonState.sketch.observations.observationIdChanged.addEventListener(this.#onObservationIdChanged.bind(this))
    aeonState.sketch.observations.observationDataChanged.addEventListener(this.#onObservationContentChanged.bind(this))

    // refresh-event listeners
    aeonState.sketch.observations.datasetsRefreshed.addEventListener(this.#onDatasetsRefreshed.bind(this))

    // note that the refresh events are automatically triggered or handled (after app refresh) directly
    // from the root component (due to some dependency issues between different components)
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
  }

  #onDatasetsRefreshed (refreshedDatasets: DatasetData[]): void {
    const datasets = refreshedDatasets.map(d => convertToIObservationSet(d))
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
    aeonState.sketch.observations.loadDataset(fileName)
  }

  #onDatasetLoaded (data: DatasetData): void {
    const newDataset = convertToIObservationSet(data)
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
      aeonState.sketch.observations.setDatasetContent(name, convertFromIObservationSet(modifiedDataset))
    })

    // Handle the case when the dialog is closed/cancelled
    void importDialog.once('observations_import_cancelled', () => {
      console.log('Import dialog was closed or cancelled.')
      // the dataset was temporarily added in its original form, now we just remove it
      aeonState.sketch.observations.removeDataset(name)
    })
  }

  #onDatasetContentChanged (data: DatasetData): void {
    const observationSet = convertToIObservationSet(data)
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
    const newDataset = convertToIObservationSet(data)
    this.updateObservations(this.contentData.observations.concat(newDataset))
  }

  #onDatasetIdChanged (data: DatasetIdUpdateData): void {
    const index = this.contentData.observations.findIndex(d => d.id === data.original_id)
    if (index === -1) return
    const datasets = structuredClone(this.contentData.observations)
    datasets[index] = {
      ...datasets[index],
      id: data.new_id
    }
    this.updateObservations(datasets)
  }

  #onDatasetMetadataChanged (data: DatasetMetaData): void {
    const datasetIndex = this.contentData.observations.findIndex(d => d.id === data.id)
    if (datasetIndex === -1) return

    const datasets = structuredClone(this.contentData.observations)
    if (!this.areVariablesEqual(data.variables, datasets[datasetIndex].variables)) {
      // if variable names changed, we need to update the corresponding fields in all observations
      const oldVariables = datasets[datasetIndex].variables
      const newVariables = data.variables

      const observations = datasets[datasetIndex].observations.map((obs: IObservation) => {
        // Create a new observation object with updated keys
        const updatedObservation: IObservation = {
          selected: obs.selected,
          name: obs.name,
          annotation: obs.annotation,
          id: obs.id
        }

        oldVariables.forEach((oldVar: string, index: number) => {
          const newVar = newVariables[index]
          // If the variable name has changed, use the new name; otherwise, keep the original
          if (oldVar !== newVar) {
            updatedObservation[newVar] = obs[oldVar]
          } else {
            updatedObservation[oldVar] = obs[oldVar]
          }
        })
        return updatedObservation
      })

      datasets[datasetIndex] = {
        ...datasets[datasetIndex],
        name: data.name,
        annotation: data.annotation,
        variables: data.variables,
        observations
      }
    } else {
      // otherwise only change the name and annotation
      datasets[datasetIndex] = {
        ...datasets[datasetIndex],
        name: data.name,
        annotation: data.annotation
      }
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
    datasets[datasetIndex].observations.push(convertToIObservation(data, datasets[datasetIndex].variables))
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

    setTimeout(() => {
      aeonState.sketch.observations.refreshDatasets()
    }, 50)
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
    const origObservation = dataset.observations.find(o => o.id === detail.id)
    if (origObservation === undefined) return

    const newObsData = convertFromIObservation(detail.observation, dataset.id, dataset.variables)

    // id might have changed
    if (origObservation.id !== newObsData.id) {
      aeonState.sketch.observations.setObservationId(dataset.id, origObservation.id, newObsData.id)
    }
    // name, annotation, or one of the values might have changed
    setTimeout(() => {
      aeonState.sketch.observations.setObservationData(dataset.id, newObsData)
    }, 50)
  }

  #onObservationContentChanged (data: ObservationData): void {
    const datasetIndex = this.contentData.observations.findIndex(d => d.id === data.dataset)
    if (datasetIndex === -1) return
    const obsIndex = this.contentData.observations[datasetIndex].observations.findIndex(obs => obs.id === data.id)
    if (obsIndex === -1) return
    const datasets: IObservationSet[] = structuredClone(this.contentData.observations)
    datasets[datasetIndex].observations[obsIndex] = convertToIObservation(data, datasets[datasetIndex].variables)
    this.updateObservations(datasets)
  }

  #onObservationIdChanged (data: ObservationIdUpdateData): void {
    // data.metadata references the dataset id
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

  private changeDataset (id: string, updatedDataset: IObservationSet): void {
    const origDataset = this.contentData.observations.find(ds => ds.id === id)
    if (origDataset === undefined) return
    const datasetData = convertFromIObservationSet(updatedDataset)
    const datasetMetaData = {
      id: datasetData.id,
      annotation: datasetData.annotation,
      name: datasetData.name,
      variables: datasetData.variables
    }

    // id might have changed
    if (origDataset.id !== datasetData.id) {
      aeonState.sketch.observations.setDatasetId(origDataset.id, datasetData.id)
    }
    // name or annotation might have changed
    setTimeout(() => {
      aeonState.sketch.observations.setDatasetMetadata(datasetData.id, datasetMetaData)
    }, 50)
  }

  private async editDataset (event: Event): Promise<void> {
    const detail = (event as CustomEvent).detail
    const datasetIndex = this.contentData.observations.findIndex(d => d.id === detail.id)
    if (datasetIndex === -1) return
    const dataset = this.contentData.observations[datasetIndex]

    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    if (this.dialogs[dataset.id] !== undefined) {
      await this.dialogs[dataset.id]?.setFocus()
      return
    }
    const editDatasetDialog = new WebviewWindow(`editDataset${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component-editor/observations-editor/edit-dataset/edit-dataset.html',
      title: `Edit dataset (${dataset.id} / ${dataset.name})`,
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      height: 500,
      width: 400,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    this.dialogs[dataset.id] = editDatasetDialog
    void editDatasetDialog.once('loaded', () => {
      void editDatasetDialog.emit('edit_dataset_update', {
        ...dataset
      })
    })
    void editDatasetDialog.once('edit_dataset_dialog', (event: TauriEvent<{ id: string, data: IObservationSet }>) => {
      this.dialogs[dataset.id] = undefined
      const index = this.contentData.observations.findIndex(d => d.id === dataset.id)
      if (index === -1) return
      this.changeDataset(dataset.id, event.payload.data)
    })
    void editDatasetDialog.onCloseRequested(() => {
      this.dialogs[dataset.id] = undefined
    })
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

  // Helper method to compare the variable name arrays
  private areVariablesEqual (prev: string[] | undefined, current: string[]): boolean {
    if (prev === undefined || prev.length !== current.length) {
      return false
    }
    // Compare each element
    return prev.every((value, index) => value === current[index])
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
                ${html`${dataset.id}&nbsp;&nbsp;&nbsp;(${dataset.name})`}
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
