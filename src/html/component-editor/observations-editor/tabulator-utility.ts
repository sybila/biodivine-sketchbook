import {
  AjaxModule,
  type ColumnDefinition,
  EditModule,
  FilterModule,
  FormatModule,
  InteractionModule,
  MenuModule,
  MoveColumnsModule,
  type Options,
  PageModule,
  ReactiveDataModule,
  ResizeColumnsModule,
  SelectRowModule,
  SortModule,
  Tabulator
} from 'tabulator-tables'

/** Column template for variable data during importing.
 * It is a simplified version of the column used later in Observations
 * editor, not having any context menu.
 */
export const variableImportColumn = (field: string): ColumnDefinition => {
  return {
    title: field,
    field,
    editor: 'number',
    sorter: 'number',
    hozAlign: 'center',
    editable: true,
    headerFilter: 'tickCross',
    headerFilterParams: { tristate: true }
  }
}

export const checkboxColumn: ColumnDefinition = {
  title: '',
  formatter: 'rowSelection',
  titleFormatter: 'rowSelection',
  headerSort: false
}

export const nameColumn = (editable: boolean): ColumnDefinition => {
  return {
    title: 'Name',
    field: 'name',
    width: 100,
    sorter: 'string',
    headerFilter: 'input',
    editable,
    editor: 'textarea'
  }
}

export const idColumn = (editable: boolean): ColumnDefinition => {
  return {
    title: 'ID',
    field: 'id',
    width: 100,
    sorter: 'string',
    headerFilter: 'input',
    editable,
    editor: 'textarea'
  }
}

export const indexColumn = (): ColumnDefinition => {
  return {
    title: 'Index',
    field: 'index',
    width: 75,
    sorter: 'number',
    headerFilter: 'input',
    editable: false
  }
}

export const tabulatorOptions: Options = {
  layout: 'fitData',
  // resizableColumnFit: true,
  pagination: true,
  renderVerticalBuffer: 300,
  sortMode: 'local',
  initialSort: [{
    column: 'index',
    dir: 'asc'
  }],
  movableColumns: true,
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
  Tabulator.registerModule(ReactiveDataModule)
  Tabulator.registerModule(MoveColumnsModule)
}
