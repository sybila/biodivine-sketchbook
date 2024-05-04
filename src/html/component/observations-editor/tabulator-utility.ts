import {
  AjaxModule, type ColumnComponent,
  type ColumnDefinition,
  EditModule,
  FilterModule, FormatModule, InteractionModule, MenuModule, type MenuObject,
  type Options,
  PageModule, ReactiveDataModule, ResizeColumnsModule, SelectRowModule,
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
    editable: true,
    headerFilterParams: { tristate: true },
    headerMenu
  }
}

const headerMenu = function (): Array<MenuObject<ColumnComponent>> {
  const menu: Array<MenuObject<ColumnComponent>> = []
  const columns: ColumnComponent[] = this.getColumns()
  columns.forEach((column, index) => {
    if (index <= 1) return
    // create checkbox element using font awesome icons
    const checkbox = document.createElement('input')
    checkbox.type = 'checkbox'
    checkbox.checked = column.isVisible()

    // build label
    const label = document.createElement('span')
    const title = document.createElement('span')

    title.textContent = ' ' + column.getDefinition().title

    label.appendChild(checkbox)
    label.appendChild(title)

    // create menu item
    menu.push({
      label,
      action: function (e: Event) {
        // prevent menu closing
        e.stopPropagation()

        // toggle current column visibility
        column.toggle()

        // change menu item icon
        checkbox.checked = column.isVisible()
      }
    })
  })

  return menu
}

export const checkboxColumn2: ColumnDefinition = {
  title: '<input type="checkbox" class="select-all-row" />',
  field: 'selected',
  formatter: function (cell) {
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
  cellClick: function (_e, cell) {
    // const chkbox = cell.getElement().querySelector('.select-row') as Element
    // cell.getRow().toggleSelect()
    // const selected = cell.getData().selected as boolean
    // chkbox.prop('checked', !selected)
    dispatchEvent(new CustomEvent('toggle-selection', {
      detail: {
        obsID: cell.getData().id
      },
      bubbles: true,
      composed: true
    }))
    // cell.getData().selected = !selected
  },
  headerClick: function (_e, column) {
    console.log((column.getElement().querySelector('.select-all-row') as HTMLInputElement).checked)
    if ((column.getElement().querySelector('.select-all-row') as HTMLInputElement).checked) {
      column.getTable().selectRow()
    } else {
      column.getTable().deselectRow()
    }
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
  editable: true,
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
  Tabulator.registerModule(ReactiveDataModule)
}
