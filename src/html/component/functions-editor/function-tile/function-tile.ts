import { css, html, LitElement, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'
import style_less from './function-tile.less?inline'
import { map } from 'lit/directives/map.js'
import { Essentiality, type IRegulationData, type IVariableData, Monotonicity } from '../../../util/data-interfaces'
import { debounce } from 'lodash'
import { icon, library } from '@fortawesome/fontawesome-svg-core'
import { faMagnifyingGlass, faTrash } from '@fortawesome/free-solid-svg-icons'
import ace, { type Ace } from 'ace-builds'
import langTools from 'ace-builds/src-noconflict/ext-language_tools'
import 'ace-builds/esm-resolver'
import AeonMode from './custom-ace.conf'

library.add(faTrash, faMagnifyingGlass)

@customElement('function-tile')
class FunctionTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() variableIndex = 0
  @property() regulations: IRegulationData[] = []
  @property() variables: IVariableData[] = []
  @state() variableFunction = ''
  @state() variableName = ''
  @query('#name-field') nameField: HTMLInputElement | undefined
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
      fontSize: 14
    })
    // @ts-expect-error ts seems to be ignoring inherited properties
    this.aceEditor.getSession().setMode(new AeonMode())
    this.aceEditor.container.style.lineHeight = '1.5em'
    this.aceEditor.container.style.fontSize = '1em'
    this.aceEditor.renderer.updateFontSize()
    this.aceEditor.getSession().on('change', this.functionUpdated)
    // @ts-expect-error $highlightRules exists but not defined in the d.ts file
    this.aceEditor.session.getMode().$highlightRules.setKeywords({ 'constant.language': this.variables.map(v => v.id).join('|') })
    this.aceEditor.renderer.attachToShadowRoot()
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
    if (this.nameField !== undefined) {
      this.nameField.value = this.variables[this.variableIndex].name
    }
    // @ts-expect-error $highlightRules exists but not defined in the d.ts file
    this.aceEditor.session.getMode().$highlightRules.setKeywords({ 'constant.language': this.variables.map(v => v.id).join('|') })
    if (_changedProperties.get('variables') === undefined || this.variables[this.variableIndex].function === this.aceEditor.getValue()) return
    this.aceEditor.getSession().off('change', this.functionUpdated)
    this.aceEditor.session.setValue(this.aceEditor.setValue(this.variables[this.variableIndex].function, this.variables[this.variableIndex].function.length - 1))
    this.aceEditor.getSession().on('change', this.functionUpdated)
  }

  private getRegulationSymbol (essential: Essentiality, monotonicity: Monotonicity): string {
    let res = '-'
    switch (essential) {
      case Essentiality.FALSE:
        res += '/'
        break
      case Essentiality.TRUE:
        res += ''
        break
      default:
        res += '?'
    }
    switch (monotonicity) {
      case Monotonicity.ACTIVATION:
        res += '>'
        break
      case Monotonicity.INHIBITION:
        res += '|'
        break
      case Monotonicity.DUAL:
        res += '*'
        break
      default:
        res += '?'
    }
    return res
  }

  private readonly nameUpdated = debounce((name: string) => {
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

  private readonly functionUpdated = debounce(() => {
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

  private monotonicityClass (monotonicity: Monotonicity): string {
    switch (monotonicity) {
      case Monotonicity.INHIBITION:
        return 'uk-text-danger'
      case Monotonicity.ACTIVATION:
        return 'uk-text-success'
      case Monotonicity.DUAL:
        return 'uk-text-primary'
      case Monotonicity.UNSPECIFIED:
        return 'uk-text-muted'
      default:
        return ''
    }
  }

  private toggleEssentiality (regulation: IRegulationData): void {
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

  private toggleMonotonicity (regulation: IRegulationData): void {
    let monotonicity
    switch (regulation.monotonicity) {
      case Monotonicity.ACTIVATION:
        monotonicity = Monotonicity.INHIBITION
        break
      case Monotonicity.INHIBITION:
        monotonicity = Monotonicity.DUAL
        break
      case Monotonicity.DUAL:
        monotonicity = Monotonicity.UNSPECIFIED
        break
      default:
        monotonicity = Monotonicity.ACTIVATION
        break
    }
    this.dispatchEvent(new CustomEvent('set-regulation-monotonicity', {
      detail: {
        id: regulation.id,
        source: regulation.source,
        target: regulation.target,
        monotonicity
      },
      bubbles: true,
      composed: true
    }))
  }

  private removeVariable (): void {
    this.shadowRoot?.dispatchEvent(new CustomEvent('remove-variable', {
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

  private getEssentialityText (essentiality: Essentiality): string {
    switch (essentiality) {
      case Essentiality.FALSE:
        return 'non-essential'
      case Essentiality.TRUE:
        return 'essential'
      default:
        return 'unknown'
    }
  }

  protected render (): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="uk-flex uk-flex-row">
          <input id="name-field" class="uk-input uk-text-center" value="${this.variables[this.variableIndex].name}"
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
