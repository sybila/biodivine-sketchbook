import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { when } from 'lit/directives/when.js'
import { customElement, state } from 'lit/decorators.js'
import style_less from './observations-import.less?inline'
import { emit, type Event as TauriEvent, once } from '@tauri-apps/api/event'
import { appWindow } from '@tauri-apps/api/window'
import { type IObservation } from '../../../util/data-interfaces'
import style_tab from '../tabulator-style.less?inline'
import { type ColumnDefinition, Tabulator } from 'tabulator-tables'
import { checkboxColumn, dataCell, loadTabulatorPlugins, nameColumn, tabulatorOptions } from '../tabulator-utility'

@customElement('observations-import')
export default class ObservationsImport extends LitElement {
  static styles = [css`${unsafeCSS(style_less)}`, css`${unsafeCSS(style_tab)}`]
  @state() data: IObservation[] = []
  @state() variables: string[] = []
  @state() loaded = false
  @state() submitDisabled = false
  table = document.createElement('div')

  tabulator: Tabulator | undefined

  constructor () {
    super()
    loadTabulatorPlugins()
    this.table.id = 'table-wrapper'
  }

  async firstUpdated (): Promise<void> {
    await once('observations_import_update', (event: TauriEvent<{
      data: IObservation[]
      variables: string[]
    }>) => {
      this.data = event.payload.data
      this.variables = event.payload.variables
      void this.init()
    })
    await emit('loaded', {})
  }

  createColumns (): ColumnDefinition[] {
    const columns: ColumnDefinition[] = [
      checkboxColumn,
      nameColumn
    ]
    this.variables.forEach(v => {
      columns.push(dataCell(v))
    })
    return columns
  }

  private async init (): Promise<void> {
    if (this.table !== undefined) {
      this.tabulator = new Tabulator(this.table, {
        columns: this.createColumns(),
        data: this.data,
        popupContainer: this.table,
        ...tabulatorOptions
      })
      this.tabulator.on('tableBuilt', () => {
        this.tabulator?.selectRow()
        this.loaded = true
      })
    }
  }

  private async handleSubmit (event: Event): Promise<void> {
    event.preventDefault()
    this.submitDisabled = true
    await emit('observations_import_dialog', this.tabulator?.getSelectedData())
    await appWindow.close()
  }

  render (): TemplateResult {
    return html`${when(this.loaded,
        () => html`
          <div class="header uk-background-primary">
            <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Select rows to be imported</h3>
          </div>
          <div id="import-wrapper">
            ${this.table}
            <div class="footer uk-flex-row uk-text-center uk-margin-small">
              <button class="uk-button uk-button-primary" 
                      ?disabled="${this.submitDisabled}" 
                      @click="${this.handleSubmit}">
                ${this.submitDisabled ? 'Processing...' : 'Import'}
              </button>
            </div>
          </div>
      `,
      () => html`
        ${this.table}
          <h1 class="uk-text-center uk-margin-large-top">Loading data...</h1>
          <div class="sk-cube-grid">
            <div class="sk-cube sk-cube1"></div>
            <div class="sk-cube sk-cube2"></div>
            <div class="sk-cube sk-cube3"></div>
            <div class="sk-cube sk-cube4"></div>
            <div class="sk-cube sk-cube5"></div>
            <div class="sk-cube sk-cube6"></div>
            <div class="sk-cube sk-cube7"></div>
            <div class="sk-cube sk-cube8"></div>
            <div class="sk-cube sk-cube9"></div>
          </div>
      `)}`
  }
}
