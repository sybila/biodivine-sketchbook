import { html, css, unsafeCSS, LitElement, type TemplateResult, type PropertyValues } from 'lit'
import { customElement, query, property } from 'lit/decorators.js'
import style_less from './observations-set.less?inline'
import style_tab from '../tabulator-style.less?inline'
import { Tabulator, type ColumnDefinition, type CellComponent } from 'tabulator-tables'
import { type IObservation, type IObservationSet } from '../../../util/data-interfaces'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/helpers/event'
import { checkboxColumn, dataCell, loadTabulatorPlugins, nameColumn, tabulatorOptions } from '../tabulator-utility'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faAdd, faEdit, faTrash } from '@fortawesome/free-solid-svg-icons'

@customElement('observations-set')
export default class ObservationsSet extends LitElement {
  static styles = [css`${unsafeCSS(style_less)}`, css`${unsafeCSS(style_tab)}`]
  @property() declare data: IObservationSet
  @query('#table-wrapper') table: HTMLElement | undefined
  tabulator: Tabulator | undefined
  tabulatorReady = false

  dialogs: Record<string, WebviewWindow | undefined> = {}

  constructor () {
    super()
    loadTabulatorPlugins()
    // todo: add events properly
  }

  protected async firstUpdated (_changedProperties: PropertyValues): Promise<void> {
    super.firstUpdated(_changedProperties)
    await this.init()
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
    if (this.tabulatorReady) void this.tabulator?.setData(this.data.observations)
  }

  private async init (): Promise<void> {
    const columns: ColumnDefinition[] = [
      checkboxColumn,
      nameColumn
    ]
    this.data.variables.forEach(v => {
      columns.push(dataCell(v))
    })
    // edit button
    columns.push({
      title: '',
      formatter: (_cell, _params, _callback): string => {
        return "<button class='uk-button-small uk-button-primary'>Edit</button>"
      },
      width: 70,
      headerSort: false,
      hozAlign: 'center',
      cellClick: (_e: UIEvent, _cell: CellComponent) => {
        void this.editObservation(_cell.getData() as IObservation)
      }
    })
    // delete button
    columns.push({
      title: '',
      formatter: (_cell, _params, _callback): string => {
        return "<button class='uk-button-small uk-button-danger'>Delete</button>"
      },
      width: 80,
      headerSort: false,
      hozAlign: 'center',
      cellClick: (_e: UIEvent, _cell: CellComponent) => {
        this.removeObservation(_cell.getRow().getData() as IObservation)
      }
    })
    if (this.table !== undefined) {
      this.tabulator = new Tabulator(this.table, {
        ...tabulatorOptions,
        columns,
        data: this.data.observations,
        popupContainer: this.table,
        reactiveData: true,
        rowContextMenu: [
          {
            label: 'Add Row',
            action: () => {
              this.pushNewObservation()
            }
          },
          {
            label: 'Edit Row',
            action: (_, row) => {
              void this.editObservation(row.getData() as IObservation)
            }
          },
          {
            label: 'Delete Row',
            action: (_, row) => {
              this.removeObservation((row.getData() as IObservation))
            }
          }
        ]
      })

      this.tabulator.on('dataLoaded', () => {
        this.tabulatorReady = true
        this.tabulator?.on('cellEdited', (cell) => {
          const data = cell.getData() as IObservation
          this.changeObservation(data.id, data)
        })
      })
    }
  }

  private pushNewObservation (): void {
    this.dispatchEvent(new CustomEvent('push-new-observation', {
      detail: {
        id: this.data.id
      },
      bubbles: true,
      composed: true
    }))
  }

  private removeObservation (obs: IObservation): void {
    this.dispatchEvent(new CustomEvent('remove-observation', {
      detail: {
        dataset: this.data.id,
        id: obs.id
      },
      bubbles: true,
      composed: true
    }))
  }

  private changeObservation (id: string, observation: IObservation): void {
    this.dispatchEvent(new CustomEvent('change-observation', {
      detail: {
        dataset: this.data.id,
        id,
        observation
      },
      bubbles: true,
      composed: true
    }))
  }

  private async editObservation (obs: IObservation): Promise<void> {
    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    if (this.dialogs[obs.id] !== undefined) {
      await this.dialogs[obs.id]?.setFocus()
      return
    }
    const renameDialog = new WebviewWindow(`editObservation${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component/observations-editor/edit-observation/edit-observation.html',
      title: `Edit observation (${obs.id} / ${obs.name})`,
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      height: 500,
      width: 400,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    this.dialogs[obs.id] = renameDialog
    void renameDialog.once('loaded', () => {
      void renameDialog.emit('edit_observation_update', {
        ...obs
      })
    })
    void renameDialog.once('edit_observation_dialog', (event: TauriEvent<{ id: string, data: IObservation }>) => {
      this.dialogs[obs.id] = undefined
      const index = this.data.observations.findIndex(observation => observation.id === obs.id)
      if (index === -1) return
      this.changeObservation(obs.id, event.payload.data)
    })
    void renameDialog.onCloseRequested(() => {
      this.dialogs[obs.id] = undefined
    })
  }

  removeDataset (): void {
    this.dispatchEvent(new CustomEvent('remove-dataset', {
      detail: {
        id: this.data.id
      },
      bubbles: true,
      composed: true
    }))
  }

  renameDataset (): void {
    this.dispatchEvent(new CustomEvent('rename-dataset', {
      detail: {
        id: this.data.id
      },
      bubbles: true,
      composed: true
    }))
  }

  render (): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-row uk-flex-right uk-margin-small-bottom">
        <button class="uk-button uk-button-small uk-button-secondary"
                @click=${this.renameDataset}>
          <div class="button-label">
            ${icon(faEdit).node}
            Rename dataset
          </div>
          
        </button>
        <button class="uk-button uk-button-small uk-button-secondary"
                @click=${this.pushNewObservation}>
          <div class="button-label">
            ${icon(faAdd).node}
            Add observation
          </div>
        </button>
        <button class="uk-button uk-button-small uk-button-danger"
                @click=${this.removeDataset}>
          <div class="button-label">
            ${icon(faTrash).node}
            Delete dataset
          </div>
          
        </button>
      </div>
      <div id="table-wrapper"></div>
    `
  }
}
