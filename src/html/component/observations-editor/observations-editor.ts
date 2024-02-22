import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './observations-editor.less?inline'
import './observations-set/observations-set'
import { ContentData, type IVariableData } from '../../util/data-interfaces'
import { map } from 'lit/directives/map.js'
import { dummyData } from '../../util/dummy-data'
import { dialog } from '@tauri-apps/api'

@customElement('observations-editor')
class ObservationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData = ContentData.create()
  @state() sets: IObservationSet[] = []

  constructor () {
    super()
    this.addSet('TEST')
  }

  private async import (): Promise<void> {
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
    if (Array.isArray(handle)) {
      this.addSet(handle.pop() ?? 'unknown')
    } else {
      this.addSet(handle)
    }
  }

  private addSet (filePath: string): void {
    this.sets = this.sets.concat({
      name: filePath,
      data: dummyData.variables
    })
  }

  render (): TemplateResult {
    return html`
      <div class="observations">
        <div class="header">
          <div></div>
          <h1 class="heading uk-heading-line uk-text-center">Observations</h1>
          <button @click="${this.import}" class="uk-button uk-button-primary uk-button-small import-button">+ Import</button>
        </div>
        <div class="accordion-body">
          <div class="accordion">
            ${map(this.sets, (set) => html`
          <div class="container">
            <div class="label" @click="${(e: Event) => { e.target?.parentElement.classList.toggle('active') }}" >
              ${set.name}
            </div>
            <div class="content">
              <observations-set
                  .data="${set.data}">
              </observations-set>
            </div>
          </div>
          <hr>
        `)}
          </div>
        </div>
      </div>
      
    `
  }
}

interface IObservationSet {
  name: string
  data: IVariableData[]
}
