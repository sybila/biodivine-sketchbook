import { html, css, unsafeCSS, LitElement, type TemplateResult, type PropertyValues } from 'lit'
import { customElement, query, property } from 'lit/decorators.js'
import style_less from './observations-set.less?inline'
import style_tab from 'tabulator-tables/dist/css/tabulator_simple.min.css?inline'
import { Tabulator, SortModule, EditModule, PageModule, FilterModule, SelectRowModule, FormatModule } from 'tabulator-tables'

import { type IVariableData } from '../../../util/data-interfaces'

@customElement('observations-set')
class ObservationsSet extends LitElement {
  static styles = [css`${unsafeCSS(style_less)}`, css`${unsafeCSS(style_tab)}`]
  @property() data: IVariableData[] = []
  @query('#table-wrapper') table: HTMLElement | undefined
  tabulator: Tabulator | undefined
  dummy = Array(100001).fill(0).map((_, index) => {
    return {
      name: 'var' + index,
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

  constructor () {
    super()
    Tabulator.registerModule(SortModule)
    Tabulator.registerModule(EditModule)
    Tabulator.registerModule(PageModule)
    Tabulator.registerModule(FilterModule)
    Tabulator.registerModule(SelectRowModule)
    Tabulator.registerModule(FormatModule)
    // this.datatable = new Tabulator('#table-wrapper')
  }

  protected firstUpdated (_changedProperties: PropertyValues): void {
    super.firstUpdated(_changedProperties)
    if (this.table !== undefined) {
      this.tabulator = new Tabulator(this.table, {
        columns: [
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
          },
          {
            title: 'Var0',
            field: 'var0',
            editor: 'textarea',
            sorter: 'number',
            headerFilter: 'tickCross',
            hozAlign: 'center',
            headerFilterParams: { tristate: true }
          },
          {
            title: 'Var1',
            field: 'var1',
            editor: 'textarea',
            sorter: 'number',
            headerFilter: 'tickCross',
            hozAlign: 'center',
            headerFilterParams: { tristate: true }
          },
          {
            title: 'Var2',
            field: 'var2',
            editor: 'textarea',
            sorter: 'number',
            headerFilter: 'tickCross',
            hozAlign: 'center',
            headerFilterParams: { tristate: true }
          }, {
            title: 'Var3',
            field: 'var3',
            editor: 'textarea',
            sorter: 'number',
            headerFilter: 'tickCross',
            hozAlign: 'center',
            headerFilterParams: { tristate: true }
          }, {
            title: 'Var4',
            field: 'var4',
            editor: 'textarea',
            sorter: 'number',
            headerFilter: 'tickCross',
            hozAlign: 'center',
            headerFilterParams: { tristate: true }
          }, {
            title: 'Var5',
            field: 'var5',
            editor: 'textarea',
            sorter: 'number',
            headerFilter: 'tickCross',
            hozAlign: 'center',
            headerFilterParams: { tristate: true }
          }, {
            title: 'Var6',
            field: 'var6',
            editor: 'textarea',
            sorter: 'number',
            headerFilter: 'tickCross',
            hozAlign: 'center',
            headerFilterParams: { tristate: true }
          }, {
            title: 'Var7',
            field: 'var7',
            editor: 'textarea',
            sorter: 'number',
            headerFilter: 'tickCross',
            hozAlign: 'center',
            headerFilterParams: { tristate: true }
          }, {
            title: 'Var8',
            field: 'var8',
            editor: 'textarea',
            sorter: 'number',
            headerFilter: 'tickCross',
            hozAlign: 'center',
            headerFilterParams: { tristate: true }
          },
          {
            title: 'Var9',
            field: 'var9',
            editor: 'textarea',
            sorter: 'number',
            headerFilter: 'tickCross',
            hozAlign: 'center',
            headerFilterParams: { tristate: true }
          }
        ],
        data: this.dummy,
        layout: 'fitDataTable',
        pagination: true,
        renderVerticalBuffer: 300,
        sortMode: 'local',
        initialSort: [{ column: 'var0', dir: 'asc' }],
        headerSort: true,
        index: 'name',
        paginationSize: 20,
        selectable: 'highlight'
      }
      )
    }
  }

  render (): TemplateResult {
    return html`
      <div id="table-wrapper"></div>
    `
  }
}
