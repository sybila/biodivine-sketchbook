import { html, type PropertyValues, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import { type IFunctionData, type IRegulationData, type IVariableData } from '../../../util/data-interfaces'
import { debounce } from 'lodash'
import { icon, library } from '@fortawesome/fontawesome-svg-core'
import { faMagnifyingGlass, faTrash } from '@fortawesome/free-solid-svg-icons'
import ace, { type Ace } from 'ace-builds'
import langTools from 'ace-builds/src-noconflict/ext-language_tools'
import 'ace-builds/esm-resolver'
import { EditorTile } from './editor-tile'
library.add(faTrash, faMagnifyingGlass)

@customElement('variable-tile')
class VariableTile extends EditorTile {
  @property() functions: IFunctionData[] = []
  @property() regulations: IRegulationData[] = []
  @property() variables: IVariableData[] = []

  constructor () {
    super()
    this.addEventListener('focus-function-field', () => { this.aceEditor.focus() })
    ace.require(langTools)
  }

  protected firstUpdated (_changedProperties: PropertyValues): void {
    super.firstUpdated(_changedProperties)
    this.init(this.index, this.variables)
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
    this.updateEditor(this.variables[this.index].name, this.variables[this.index].function)
    langTools.setCompleters([{
      getCompletions: (_editor: Ace.Editor, _session: Ace.EditSession, _point: Ace.Point, _prefix: string, callback: Ace.CompleterCallback) => {
        callback(null, this.getVariables().map((variable): Ace.Completion => ({ value: variable.id, meta: variable.name }))
          .concat(this.functions.map((f): Ace.Completion => ({ value: f.id, snippet: f.id + '()' }))))
      }
    }])
    // @ts-expect-error $highlightRules exists but not defined in the d.ts file
    this.aceEditor.session.getMode().$highlightRules.setKeywords({
      'constant.language': this.variables.map(v => v.id).join('|'),
      'support.function.dom': this.functions.map(f => f.id).join('|')
    })
  }

  private getVariables (): IVariableData[] {
    return this.variables
  }

  nameUpdated = debounce((name: string) => {
    this.dispatchEvent(new CustomEvent('rename-variable', {
      detail: {
        id: this.variables[this.index].id,
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
        id: this.variables[this.index].id,
        function: this.aceEditor.getValue()
      },
      bubbles: true,
      composed: true
    }))
  },
  300
  )

  toggleEssentiality (regulation: IRegulationData): void {
    this.dispatchEvent(new CustomEvent('toggle-regulation-essential', {
      detail: {
        id: regulation.id,
        source: regulation.source,
        target: regulation.target,
        essential: regulation.essential
      },
      bubbles: true,
      composed: true
    }))
  }

  toggleMonotonicity (regulation: IRegulationData): void {
    this.dispatchEvent(new CustomEvent('toggle-regulation-monotonicity', {
      detail: {
        ...regulation
      },
      bubbles: true,
      composed: true
    }))
  }

  removeVariable (): void {
    this.shadowRoot?.dispatchEvent(new CustomEvent('remove-variable', {
      detail: {
        id: this.variables[this.index].id
      },
      composed: true,
      bubbles: true
    }))
  }

  focusVariable (): void {
    window.dispatchEvent(new CustomEvent('focus-variable', {
      detail: {
        id: this.variables[this.index].id
      }
    }))
  }

  highlightRegulation (regulation: IRegulationData): void {
    window.dispatchEvent(new CustomEvent('highlight-regulation', {
      detail: {
        id: regulation.id
      }
    }))
  }

  resetHighlight (): void {
    window.dispatchEvent(new Event('reset-highlight'))
  }

  private getVariableText (variableId: string): string {
    const variable = this.variables.find(variable => variable.id === variableId)
    if (variable === undefined) return ''
    if (variable.id === variable.name) {
      return variable.id
    }
    return variable.name + ' (' + variable.id + ')'
  }

  protected render (): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="uk-input uk-text-center" value="${this.variables[this.index].name}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          <button class="uk-button uk-button-small" @click="${this.focusVariable}">
            ${icon(faMagnifyingGlass).node}
          </button>
          <button class="uk-button uk-button-small" @click="${this.removeVariable}">
            ${icon(faTrash).node}
          </button>
        </div>
        <span class="uk-align-left uk-text-left uk-margin-remove">Regulators:</span>
        ${map(this.regulations, (regulation) => html`
          <div
              class="regulation uk-grid uk-grid-column-small uk-grid-row-large uk-child-width-1-4 uk-margin-remove uk-text-center uk-flex-around uk-text-nowrap"
          @mouseenter="${() => {
            this.highlightRegulation(regulation)
          }}"
          @mouseleave="${this.resetHighlight}">
            <div class="uk-width-1-6">${this.getVariableText(regulation.source)}</div>
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
