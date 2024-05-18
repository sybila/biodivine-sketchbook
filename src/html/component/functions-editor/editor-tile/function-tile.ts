import { html, type PropertyValues, type TemplateResult } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import { type IRegulationData } from '../../../util/data-interfaces'
import { debounce } from 'lodash'
import { icon, library } from '@fortawesome/fontawesome-svg-core'
import { faMagnifyingGlass, faTrash, faPlus, faAngleDown, faAngleUp } from '@fortawesome/free-solid-svg-icons'
import ace, { type Ace } from 'ace-builds'
import langTools from 'ace-builds/src-noconflict/ext-language_tools'
import 'ace-builds/esm-resolver'
import { EditorTile } from './editor-tile'
import { functionDebounceTimer } from '../../../util/config'
import { getEssentialityText, getMonotonicityClass } from '../../../util/utilities'

library.add(faTrash, faMagnifyingGlass, faAngleDown, faAngleUp)

@customElement('function-tile')
export class FunctionTile extends EditorTile {
  @state() bodyVisible = false
  varIndex = 0

  constructor () {
    super()
    ace.require(langTools)
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
    this.updateEditor(this.functions[this.index].id, this.functions[this.index].function)
    // @ts-expect-error $highlightRules exists but not defined in the d.ts file
    this.aceEditor.session.getMode().$highlightRules.setKeywords({
      'constant.language': this.functions[this.index].variables.map(r => r.source).join('|'),
      'support.function.dom': this.functions.map(v => v.id).join('|')
    })
    this.aceEditor.completers = this.aceEditor.completers.concat({
      getCompletions: (_editor: Ace.Editor, _session: Ace.EditSession, _point: Ace.Point, _prefix: string, callback: Ace.CompleterCallback) => {
        callback(null, this.functions[this.index].variables.map((variable): Ace.Completion => ({
          caption: variable.source,
          value: variable.source,
          meta: variable.source
        })))
      }
    })
    this.aceEditor.setOption('placeholder', '$f_' + this.functions[this.index].id + '(...)')
  }

  protected firstUpdated (_changedProperties: PropertyValues): void {
    super.firstUpdated(_changedProperties)
    this.init(this.index, this.functions)
  }

  nameUpdated = debounce((name: string) => {
    this.dispatchEvent(new CustomEvent('rename-function-definition', {
      detail: {
        oldId: this.functions[this.index].id,
        newId: name
      },
      bubbles: true,
      composed: true
    }))
  }, functionDebounceTimer
  )

  functionUpdated = debounce(() => {
    this.dispatchEvent(new CustomEvent('set-uninterpreted-function-expression', {
      detail: {
        id: this.functions[this.index].id,
        function: this.aceEditor.getValue()
      },
      bubbles: true,
      composed: true
    }))
  }, functionDebounceTimer
  )

  async removeVariable (): Promise<void> {
    this.dispatchEvent(new CustomEvent('remove-function-definition', {
      detail: {
        id: this.functions[this.index].id
      },
      composed: true,
      bubbles: true
    }))
  }

  private addVariable (): void {
    this.dispatchEvent(new CustomEvent('add-function-variable', {
      detail: {
        id: this.functions[this.index].id,
        variable: 'var' + this.varIndex
      },
      bubbles: true,
      composed: true
    }))
    this.varIndex++
  }

  toggleEssentiality (regulation: IRegulationData): void {
    const index = this.functions[this.index].variables.findIndex(f => f === regulation)
    if (index === -1) return
    this.dispatchEvent(new CustomEvent('toggle-function-variable-essentiality', {
      detail: {
        id: this.functions[this.index].id,
        index,
        essentiality: this.functions[this.index].variables[index].essential
      },
      bubbles: true,
      composed: true
    }))
  }

  toggleMonotonicity (regulation: IRegulationData): void {
    const index = this.functions[this.index].variables.findIndex(f => f === regulation)
    if (index === -1) return
    this.dispatchEvent(new CustomEvent('toggle-function-variable-monotonicity', {
      detail: {
        id: this.functions[this.index].id,
        index,
        monotonicity: this.functions[this.index].variables[index].monotonicity
      },
      bubbles: true,
      composed: true
    }))
  }

  async removeRegulation (regulation: IRegulationData): Promise<void> {
    const index = this.functions[this.index].variables.findIndex(f => f === regulation)
    if (index === -1) return
    this.dispatchEvent(new CustomEvent('remove-function-variable', {
      detail: {
        id: this.functions[this.index].id,
        index
      },
      bubbles: true,
      composed: true
    }))
  }

  toggleBody (): void {
    this.bodyVisible = !this.bodyVisible
  }

  protected render (): TemplateResult {
    return html`
      <div class="container uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="uk-input uk-text-center" value="${this.functions[this.index].id}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          
          <button class="uk-button uk-button-small" @click="${this.addVariable}">
            ${icon(faPlus).node}
          </button>
          
          <button class="uk-button uk-button-small" @click="${this.removeVariable}">
            ${icon(faTrash).node}
          </button>
          
          <button class="uk-button uk-button-small" @click="${this.toggleBody}">
            <div class="uk-flex uk-flex-column">
              <span class="var-count">${this.functions[this.index].variables.length}</span>
              ${(this.bodyVisible ? icon(faAngleUp) : icon(faAngleDown)).node}
            </div>
          </button>
        </div>
        <div class="functions-body" style="display: ${this.bodyVisible ? 'flex' : 'none'}">
          <span class="uk-align-left uk-text-left uk-margin-remove">Regulators:</span>
          ${map(this.functions[this.index].variables, (variable) => html`
            <div
                class="regulation uk-grid uk-grid-column-small uk-grid-row-large uk-child-width-1-4 uk-margin-remove uk-text-center uk-flex-around uk-text-nowrap">
              <button class="remove-reg uk-width-1-6 uk-button uk-button-small uk-text-center" @click="${() => {
                void this.removeRegulation(variable)
              }}">
                ${icon(faTrash).node}
              </button>
              <div class="uk-width-1-6">${variable.source}</div>
              <div class="uk-width-1-6">${this.getRegulationSymbol(variable.essential, variable.monotonicity)}</div>
              <div class="regulation-property"
                   @click="${() => {
                     this.toggleEssentiality(variable)
                   }}">
                ${getEssentialityText(variable.essential)}
              </div>
              <div class="regulation-property ${getMonotonicityClass(variable.monotonicity)}"
                   @click="${() => {
                     this.toggleMonotonicity(variable)
                   }}">
                ${variable.monotonicity.toLowerCase()}
              </div>
            </div>
          `)}
          <span class="uk-align-left uk-text-left uk-margin-remove">Update function:</span>
          <div id="function-editor"></div>
        </div>
      </div>
      <hr>
    `
  }
}
