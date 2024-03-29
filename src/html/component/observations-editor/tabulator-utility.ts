import {
  AjaxModule,
  type ColumnDefinition,
  EditModule,
  FilterModule, FormatModule, InteractionModule, MenuModule,
  type Options,
  PageModule, ResizeColumnsModule, SelectRowModule,
  SortModule,
  Tabulator
} from 'tabulator-tables'

export const dataCell = (field: string): ColumnDefinition => {
  return {
    title: field,
    field,
    editor: 'number',
    sorter: 'number',
    headerFilter: 'tickCross',
    hozAlign: 'center',
    headerFilterParams: { tristate: true },
    headerMenu: [
      {
        label: 'Hide Column',
        action: function (_, column) {
          column.hide()
        }
      },
      {
        label: 'Delete Column',
        action: function (_, column) {
          void column.delete()
        }
      }
    ]
  }
}

export const checkboxColumn: ColumnDefinition = {
  title: '',
  formatter: 'rowSelection',
  titleFormatter: 'rowSelection',
  headerSort: false
}

export const nameColumn: ColumnDefinition = {
  title: 'Name',
  field: 'name',
  width: 100,
  sorter: 'string',
  headerFilter: 'input',
  editor: 'textarea'
}

export const tabulatorOptions: Options = {
  layout: 'fitData',
  resizableColumnFit: true,
  pagination: true,
  renderVerticalBuffer: 300,
  sortMode: 'local',
  initialSort: [{ column: 'name', dir: 'asc' }],
  headerSort: true,
  index: 'id',
  paginationSize: 20,
  paginationSizeSelector: true,
  rowContextMenu: [
    {
      label: 'Delete Row',
      action: function (_, row) {
        void row.delete()
      }
    }
  ]
}

export const loadTabulatorPlugins = (): void => {
  Tabulator.registerModule(SortModule)
  Tabulator.registerModule(EditModule)
  Tabulator.registerModule(PageModule)
  Tabulator.registerModule(FilterModule)
  Tabulator.registerModule(SelectRowModule)
  Tabulator.registerModule(FormatModule)
  Tabulator.registerModule(InteractionModule)
  Tabulator.registerModule(AjaxModule)
  Tabulator.registerModule(MenuModule)
  Tabulator.registerModule(ResizeColumnsModule)
}
