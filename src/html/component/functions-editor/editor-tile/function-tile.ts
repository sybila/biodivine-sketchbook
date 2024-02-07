import { html, type TemplateResult } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import { Essentiality, type IRegulationData, Monotonicity } from '../../../util/data-interfaces'
import { debounce } from 'lodash'
import { icon, library } from '@fortawesome/fontawesome-svg-core'
import { faMagnifyingGlass, faTrash } from '@fortawesome/free-solid-svg-icons'
import ace from 'ace-builds'
import langTools from 'ace-builds/src-noconflict/ext-language_tools'
import 'ace-builds/esm-resolver'
import { EditorTile } from './editor-tile'
import { getNextEssentiality, getNextMonotonicity } from '../../../util/utilities'

library.add(faTrash, faMagnifyingGlass)

@customElement('function-tile')
class FunctionTile extends EditorTile {
  @state() regulations: IRegulationData[] = []
  varIndex = 0
  constructor () {
    super()
    this.addEventListener('focus-function-field', () => { this.aceEditor.focus() })
    ace.require(langTools)
    console.log('test')
  }

  nameUpdated = debounce((name: string) => {
    this.dispatchEvent(new CustomEvent('rename-variable', {
      detail: {
        id: this.variables[this.variableIndex].id,
        name
      },
      bubbles: true,
      composed: true
    }))
  },
  300
  )

  functionUpdated = debounce(() => {
    this.dispatchEvent(new CustomEvent('set-variable-function', {
      detail: {
        id: this.variables[this.variableIndex].id,
        function: this.aceEditor.getValue()
      },
      bubbles: true,
      composed: true
    }))
  },
  300
  )

  removeVariable (): void {
    this.shadowRoot?.dispatchEvent(new CustomEvent('remove-variable', {
      detail: {
        id: this.variables[this.variableIndex].id
      },
      composed: true,
      bubbles: true
    }))
  }

  private addVariable (): void {
    const regs = [...this.regulations]
    regs.push({
      id: this.varIndex.toString(),
      source: 'var' + this.varIndex,
      target: '',
      essential: Essentiality.UNKNOWN,
      monotonicity: Monotonicity.UNSPECIFIED
    })
    this.varIndex++
    this.regulations = regs
  }

  toggleEssentiality (regulation: IRegulationData): void {
    const index = this.regulations.findIndex(reg => reg.id === regulation.id)
    if (index === -1) return
    const regs = [...this.regulations]
    regs[index] = {
      ...regs[index],
      essential: getNextEssentiality(regs[index].essential)
    }
    this.regulations = regs
  }

  toggleMonotonicity (regulation: IRegulationData): void {
    const index = this.regulations.findIndex(reg => reg.id === regulation.id)
    if (index === -1) return
    const regs = [...this.regulations]
    regs[index] = {
      ...regs[index],
      monotonicity: getNextMonotonicity(regs[index].monotonicity)
    }
    this.regulations = regs
  }

  removeRegulation (regulation: IRegulationData): void {
    const index = this.regulations.findIndex(reg => reg.id === regulation.id)
    if (index === -1) return
    const regs = [...this.regulations]
    regs.splice(index, 1)
    this.regulations = regs
  }

  protected render (): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="uk-input uk-text-center" value="${this.variables[this.variableIndex].name}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          <button class="uk-button uk-button-small" @click="${this.addVariable}">
            ${icon(faMagnifyingGlass).node}
          </button>
          <button class="uk-button uk-button-small" @click="${this.removeVariable}">
            ${icon(faTrash).node}
          </button>
        </div>
        <span class="uk-align-left uk-text-left uk-margin-remove">Regulators:</span>
        ${map(this.regulations, (regulation) => html`
          <div class="regulation uk-grid uk-grid-column-small uk-grid-row-large uk-child-width-1-4 uk-margin-remove uk-text-center uk-flex-right uk-text-nowrap">
            <button class="uk-width-1-6" @click="${() => {
              this.removeRegulation(regulation)
            }}">
              ${icon(faTrash).node}
            </button>
            <div class="uk-width-1-6">${regulation.source}</div>
            <div class="uk-width-1-6">${this.getRegulationSymbol(regulation.essential, regulation.monotonicity)}</div>
            <div class="regulation-property"
                 @click="${() => {
                   this.toggleEssentiality(regulation)
                 }}">
              ${this.getEssentialityText(regulation.essential)}
            </div>
            <div class="regulation-property ${this.monotonicityClass(regulation.monotonicity)}"
                 @click="${() => {
                   this.toggleMonotonicity(regulation)
                 }}">
              ${regulation.monotonicity.toLowerCase()}
            </div>
          </div>
        `)}
        <span class="uk-align-left uk-text-left uk-margin-remove">Update function:</span>
        <div id="function-editor"></div>
      </div>
      <hr>
    `
  }
}
