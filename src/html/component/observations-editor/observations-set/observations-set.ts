import { html, css, unsafeCSS, LitElement, type TemplateResult, type PropertyValues } from 'lit'
import { customElement, query, property, state } from 'lit/decorators.js'
import style_less from './observations-set.less?inline'
import style_tab from 'tabulator-tables/dist/css/tabulator_simple.min.css?inline'
import {
  Tabulator,
  SortModule,
  EditModule,
  PageModule,
  FilterModule,
  SelectRowModule,
  FormatModule,
  type ColumnDefinition, type CellComponent, InteractionModule
} from 'tabulator-tables'

import { type IVariableData } from '../../../util/data-interfaces'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/helpers/event'

@customElement('observations-set')
export default class ObservationsSet extends LitElement {
  static styles = [css`${unsafeCSS(style_less)}`, css`${unsafeCSS(style_tab)}`]
  @property() data: IVariableData[] = []
  @query('#table-wrapper') table: HTMLElement | undefined
  tabulator: Tabulator | undefined
  @state() dummy: IDummyObservation[] = Array(100001).fill(0).map((_, index) => {
    return {
      id: 'var' + String(index).padStart(4, '0'),
      name: 'var' + String(index).padStart(4, '0'),
      var0: Math.round(Math.random()),
      var1: Math.round(Math.random()),
      var2: Math.round(Math.random()),
      var3: Math.round(Math.random()),
      var4: Math.round(Math.random()),
      var5: Math.round(Math.random()),
      var6: Math.round(Math.random()),
      var7: Math.round(Math.random()),
      var8: Math.round(Math.random()),
      var9: Math.round(Math.random())
    }
  })

  dialogs: Record<string, WebviewWindow | undefined> = {}

  constructor () {
    super()
    Tabulator.registerModule(SortModule)
    Tabulator.registerModule(EditModule)
    Tabulator.registerModule(PageModule)
    Tabulator.registerModule(FilterModule)
    Tabulator.registerModule(SelectRowModule)
    Tabulator.registerModule(FormatModule)
    Tabulator.registerModule(InteractionModule)
  }

  protected async firstUpdated (_changedProperties: PropertyValues): Promise<void> {
    super.firstUpdated(_changedProperties)
    const dataCell = (field: string): ColumnDefinition => {
      return {
        title: field,
        field,
        editor: 'textarea',
        sorter: 'number',
        headerFilter: 'tickCross',
        hozAlign: 'center',
        headerFilterParams: { tristate: true }
      }
    }
    const columns: ColumnDefinition[] = [
      {
        title: '',
        formatter: 'rowSelection',
        titleFormatter: 'rowSelection',
        headerSort: false
      },
      {
        title: 'Name',
        field: 'name',
        width: 100,
        sorter: 'string',
        headerFilter: 'input'
      }
    ]
    for (let i = 0; i < 10; i++) {
      columns.push(dataCell('var' + i))
    }
    columns.push({
      title: '',
      formatter: (_cell, _params, _callback): string => {
        return "<button class='uk-button-small uk-button-primary'>Edit</button>"
      },
      width: 100,
      cellClick: (_e: UIEvent, _cell: CellComponent) => {
        console.log('test', _e, _cell.getData())
        void this.editObservation(_cell.getData() as IDummyObservation)
      }
    })
    if (this.table !== undefined) {
      this.tabulator = new Tabulator(this.table, {
        columns,
        data: this.dummy,
        layout: 'fitDataTable',
        responsiveLayout: false,
        pagination: true,
        renderVerticalBuffer: 300,
        sortMode: 'local',
        initialSort: [{ column: 'var0', dir: 'asc' }],
        headerSort: true,
        index: 'id',
        paginationSize: 20,
        selectable: 'highlight'
      }
      )
    }
  }

  private async editObservation (data: IDummyObservation): Promise<void> {
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
    void renameDialog.once('edit_observation_dialog', (event: TauriEvent<{ id: string, data: IDummyObservation }>) => {
      this.dialogs[data.id] = undefined
      const index = this.dummy.findIndex(observation => observation.id === data.id)
      if (index === -1) return
      void this.tabulator?.updateRow(event.payload.id, event.payload.data)
      console.log(event.payload)
    })
    void renameDialog.onCloseRequested(() => {
      this.dialogs[data.id] = undefined
    })
  }

  render (): TemplateResult {
    return html`
      <div id="table-wrapper"></div>
    `
  }
}

export interface IDummyObservation {
  id: string
  name: string
  var0: number
  var1: number
  var2: number
  var3: number
  var4: number
  var5: number
  var6: number
  var7: number
  var8: number
  var9: number
}
