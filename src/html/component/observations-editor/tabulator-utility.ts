import {
  AjaxModule,
  type ColumnDefinition,
  EditModule,
  FilterModule, FormatModule, InteractionModule,
  type Options,
  PageModule, SelectRowModule,
  SortModule,
  Tabulator
} from 'tabulator-tables'

export const dataCell = (field: string): ColumnDefinition => {
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
  headerFilter: 'input'
}

export const tabulatorOptions: Options = {
  layout: 'fitDataTable',
  responsiveLayout: false,
  pagination: true,
  renderVerticalBuffer: 300,
  sortMode: 'local',
  initialSort: [{ column: 'name', dir: 'asc' }],
  headerSort: true,
  index: 'id',
  paginationSize: 20,
  selectable: 'highlight'
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
}
