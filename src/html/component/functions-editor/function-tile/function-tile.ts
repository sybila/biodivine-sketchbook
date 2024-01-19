import { css, html, LitElement, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './function-tile.less?inline'
import { map } from 'lit/directives/map.js'
import { type IRegulationData, type IVariableData } from '../../../util/data-interfaces'
import { Monotonicity } from '../../regulations-editor/element-type'
import { debounce } from 'lodash'
import { icon, library } from '@fortawesome/fontawesome-svg-core'
import { faTrash, faMagnifyingGlass } from '@fortawesome/free-solid-svg-icons'
import ace, { type Ace, config } from 'ace-builds'
import langTools from 'ace-builds/src-noconflict/ext-language_tools'
import modeYaml from 'ace-builds/src-noconflict/mode-yaml?url'

library.add(faTrash, faMagnifyingGlass)
config.setModuleUrl('ace/mode/yaml', modeYaml)

@customElement('function-tile')
class FunctionTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() variableIndex = 0
  @property() regulations: IRegulationData[] = []
  @property() variables: IVariableData[] = []
  @state() variableFunction = ''
  @state() variableName = ''
  declare aceEditor: ace.Ace.Editor

  constructor () {
    super()
    this.addEventListener('focus-function-field', () => { this.aceEditor.focus() })
    ace.require(langTools)
  }

  protected firstUpdated (_changedProperties: PropertyValues): void {
    super.firstUpdated(_changedProperties)
    const editorElement = this.shadowRoot?.getElementById('function-editor')
    if (editorElement === null || editorElement === undefined) return
    this.aceEditor = ace.edit(editorElement, {
      // showGutter: false,
      enableBasicAutocompletion: [{
        getCompletions: (_editor, _session, _point, _prefix, callback) => {
          callback(null, this.variables.map((variable): Ace.Completion => ({ value: variable.id, meta: variable.name })))
        }
      }],
      enableSnippets: true,
      enableLiveAutocompletion: true,
      behavioursEnabled: true,
      value: this.variables[this.variableIndex].function,
      placeholder: '$f_' + this.variables[this.variableIndex].id + '(...)',
      minLines: 1,
      maxLines: Infinity,
      wrap: 'free',
      fontSize: 14,
      mode: 'ace/mode/yaml'
    })
    this.aceEditor.container.style.lineHeight = '1.5em'
    this.aceEditor.container.style.fontSize = '1em'
    this.aceEditor.renderer.updateFontSize()
    this.aceEditor.getSession().on('change', this.functionUpdated)
    this.aceEditor.renderer.attachToShadowRoot()
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
    if (_changedProperties.get('variables') === undefined || this.variables[this.variableIndex].function === this.aceEditor.getValue()) return
    this.aceEditor.getSession().off('change', this.functionUpdated)
    this.aceEditor.session.setValue(this.aceEditor.setValue(this.variables[this.variableIndex].function, this.variables[this.variableIndex].function.length - 1))
    this.aceEditor.getSession().on('change', this.functionUpdated)
  }

  private getRegulationSymbol (observable: boolean, monotonicity: Monotonicity): string {
    let res = '-'
    switch (monotonicity) {
      case Monotonicity.ACTIVATION:
        res += '>'
        break
      case Monotonicity.INHIBITION:
        res += '|'
        break
      default:
        res += '?'
    }
    return res + (observable ? '' : '?')
  }

  private readonly nameUpdated = debounce((name: string) => {
    this.dispatchEvent(new CustomEvent('update-variable', {
      detail: {
        ...this.variables[this.variableIndex],
        oldId: this.variables[this.variableIndex].id,
        name
      },
      bubbles: true,
      composed: true
    }))
  },
  300
  )

  private readonly functionUpdated = debounce(() => {
    this.dispatchEvent(new CustomEvent('update-variable', {
      detail: {
        ...this.variables[this.variableIndex],
        oldId: this.variables[this.variableIndex].id,
        function: this.aceEditor.getValue()
      },
      bubbles: true,
      composed: true
    }))
  },
  300
  )

  private monotonicityClass (monotonicity: Monotonicity): string {
    switch (monotonicity) {
      case Monotonicity.INHIBITION:
        return 'uk-text-danger'
      case Monotonicity.ACTIVATION:
        return 'uk-text-success'
      case Monotonicity.UNSPECIFIED:
        return 'uk-text-muted'
      default:
        return ''
    }
  }

  private toggleObservability (regulation: IRegulationData): void {
    // TODO: move higher up the component tree and merge with similar functions in float-menu
    this.dispatchEvent(new CustomEvent('update-regulation', {
      detail: {
        ...regulation,
        observable: !regulation.observable
      },
      bubbles: true,
      composed: true
    }))
  }

  private toggleMonotonicity (regulation: IRegulationData): void {
    // TODO: move higher up the component tree and merge with similar functions in float-menu
    let monotonicity
    switch (regulation.monotonicity) {
      case Monotonicity.ACTIVATION:
        monotonicity = Monotonicity.INHIBITION
        break
      case Monotonicity.INHIBITION:
        monotonicity = Monotonicity.UNSPECIFIED
        break
      default:
        monotonicity = Monotonicity.ACTIVATION
        break
    }
    this.dispatchEvent(new CustomEvent('update-regulation', {
      detail: {
        ...regulation,
        monotonicity
      },
      bubbles: true,
      composed: true
    }))
  }

  private removeVariable (): void {
    this.shadowRoot?.dispatchEvent(new CustomEvent('remove-element', {
      detail: {
        id: this.variables[this.variableIndex].id
      },
      composed: true,
      bubbles: true
    }))
  }

  private focusVariable (): void {
    window.dispatchEvent(new CustomEvent('focus-variable', {
      detail: {
        id: this.variables[this.variableIndex].id
      }
    }))
  }

  private highlightRegulation (regulation: IRegulationData): void {
    window.dispatchEvent(new CustomEvent('highlight-regulation', {
      detail: {
        id: regulation.id
      }
    }))
  }

  private resetHighlight (): void {
    window.dispatchEvent(new Event('reset-highlight'))
  }

  protected render (): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="uk-flex uk-flex-row">
          <input class="uk-input uk-text-center" value="${this.variables[this.variableIndex].name}"
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
              class="regulation uk-grid uk-grid-column-small uk-grid-row-large uk-child-width-1-4 uk-margin-remove uk-text-center uk-flex-right uk-text-nowrap"
          @mouseenter="${() => {
            this.highlightRegulation(regulation)
          }}"
          @mouseout="${this.resetHighlight}">
            <div class="uk-width-1-6">${regulation.source}</div>
            <div class="uk-width-1-6">${this.getRegulationSymbol(regulation.observable, regulation.monotonicity)}</div>
            <div class="regulation-property ${regulation.observable ? '' : 'uk-text-muted'}"
                 @click="${() => {
                   this.toggleObservability(regulation)
                 }}">
              ${regulation.observable ? 'observable' : 'non-observable'}
            </div>
            <div class="regulation-property ${this.monotonicityClass(regulation.monotonicity)}"
                 @click="${() => {
                   this.toggleMonotonicity(regulation)
                 }}">
              ${regulation.monotonicity}
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
