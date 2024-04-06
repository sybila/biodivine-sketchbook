import { html, css, unsafeCSS, LitElement, type TemplateResult, type PropertyValues } from 'lit'
import { customElement, query, property } from 'lit/decorators.js'
import style_less from './observations-set.less?inline'
import style_tab from '../tabulator-style.less?inline'
import { Tabulator, type ColumnDefinition, type CellComponent } from 'tabulator-tables'
import { type IObservation, type IObservationSet } from '../../../util/data-interfaces'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/helpers/event'
import {
  dataCell,
  loadTabulatorPlugins,
  nameColumn,
  tabulatorOptions
} from '../tabulator-utility'

@customElement('observations-set')
export default class ObservationsSet extends LitElement {
  static styles = [css`${unsafeCSS(style_less)}`, css`${unsafeCSS(style_tab)}`]
  @property() declare data: IObservationSet
  @property() index: number = -1
  @query('#table-wrapper') table: HTMLElement | undefined
  tabulator: Tabulator | undefined

  dialogs: Record<string, WebviewWindow | undefined> = {}

  constructor () {
    super()
    loadTabulatorPlugins()
  }

  protected async firstUpdated (_changedProperties: PropertyValues): Promise<void> {
    super.firstUpdated(_changedProperties)
    await this.init()
    this.tabulator?.on('cellEdited', (cell) => {
      const row = cell.getRow()
      this.observationEdited(row.getIndex(), row.getData() as IObservation)
    })
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
    void this.tabulator?.setData(this.data.observations)
  }

  private async init (): Promise<void> {
    const columns: ColumnDefinition[] = [
      this.checkboxColumn,
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
      cellClick: (_e: UIEvent, cell: CellComponent) => {
        const row = cell.getRow()
        void this.editObservation(row.getIndex(), row.getData() as IObservation)
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
      cellClick: (_e: UIEvent, cell: CellComponent) => {
        this.observationRemoved(cell.getRow().getIndex())
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
              this.dispatchEvent(new CustomEvent('add-observation', {
                detail: {
                  id: this.data.name
                },
                bubbles: true,
                composed: true
              }))
            }
          },
          {
            label: 'Edit Row',
            action: (_, row) => {
              void this.editObservation(row.getIndex(), row.getData() as IObservation)
            }
          },
          {
            label: 'Delete Row',
            action: (_, row) => {
              this.observationRemoved(row.getIndex())
            }
          }
        ]
      })
      this.tabulator.redraw(true)
      this.tabulator.on('rowSelectionChanged', function (data, rows, selectedRows, deselectedRows) {
        console.log(data, rows, selectedRows, deselectedRows)
      })
      this.tabulator.on('cellEdited', function (cell) {
        console.log(cell)
      })
    }
  }

  private observationEdited (obsID: string, data: IObservation): void {
    this.dispatchEvent(new CustomEvent('edit-observation', {
      detail: {
        id: this.index,
        obsID,
        data
      },
      bubbles: true,
      composed: true
    }))
  }

  private observationRemoved (obsID: string): void {
    this.dispatchEvent(new CustomEvent('remove-observation', {
      detail: {
        id: this.index,
        obsID
      },
      bubbles: true,
      composed: true
    }))
  }

  private async editObservation (obsID: string, data: IObservation): Promise<void> {
    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    if (this.dialogs[data.id] !== undefined) {
      await this.dialogs[data.id]?.setFocus()
      return
    }
    const renameDialog = new WebviewWindow(`editObservation${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component/observations-editor/edit-observation/edit-observation.html',
      title: `Edit observation (${data.id} / ${data.name})`,
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      height: 500,
      width: 400,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    this.dialogs[data.id] = renameDialog
    void renameDialog.once('loaded', () => {
      void renameDialog.emit('edit_observation_update', {
        ...data
      })
    })
    void renameDialog.once('edit_observation_dialog', (event: TauriEvent<{ id: string, data: IObservation }>) => {
      this.dialogs[data.id] = undefined
      this.observationEdited(obsID, event.payload.data)
    })
    void renameDialog.onCloseRequested(() => {
      this.dialogs[data.id] = undefined
    })
  }

  checkboxColumn: ColumnDefinition = {
    title: '<input type="checkbox" class="select-all-row" />',
    field: 'selected',
    formatter: (cell) => {
      const box = document.createElement('input')
      box.type = 'checkbox'
      box.classList.add('select-row')
      box.readOnly = true
      box.checked = cell.getData().selected
      return box
    },
    headerSort: false,
    cssClass: 'text-center',
    frozen: true,
    cellClick: (_e, cell) => {
      const row = cell.getRow()
      const data = row.getData() as IObservation
      this.observationEdited(row.getIndex(), {
        ...data,
        selected: !data.selected
      })
    },
    headerClick: function (_e, column) {
      if ((column.getElement().querySelector('.select-all-row') as HTMLInputElement).checked) {
        column.getTable().selectRow()
      } else {
        column.getTable().deselectRow()
      }
    }
  }

  render (): TemplateResult {
    return html`
      <div id="table-wrapper"></div>
    `
  }
}
