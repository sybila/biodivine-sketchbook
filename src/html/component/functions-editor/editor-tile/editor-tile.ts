import { css, LitElement, type PropertyValues, unsafeCSS } from 'lit'
import { property, query, state } from 'lit/decorators.js'
import style_less from './editor-tile.less?inline'
import { Essentiality, type IRegulationData, type IVariableData, Monotonicity } from '../../../util/data-interfaces'
import { library } from '@fortawesome/fontawesome-svg-core'
import { faMagnifyingGlass, faTrash } from '@fortawesome/free-solid-svg-icons'
import ace from 'ace-builds'
import langTools from 'ace-builds/src-noconflict/ext-language_tools'
import 'ace-builds/esm-resolver'
import AeonMode from './custom-ace.conf'
import type { DebouncedFunc } from 'lodash-es'

library.add(faTrash, faMagnifyingGlass)

export abstract class EditorTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() variableIndex = 0
  @property() regulations: IRegulationData[] = []
  @property() variables: IVariableData[] = []
  @state() variableFunction = ''
  @state() variableName = ''
  @query('#name-field') nameField: HTMLInputElement | undefined
  declare aceEditor: ace.Ace.Editor

  protected constructor () {
    super()
    ace.require(langTools)
  }

  protected firstUpdated (_changedProperties: PropertyValues): void {
    super.firstUpdated(_changedProperties)
    const editorElement = this.shadowRoot?.getElementById('function-editor')
    if (editorElement === null || editorElement === undefined) return
    this.aceEditor = ace.edit(editorElement, {
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
    if (!(_changedProperties.get('variables') === undefined || this.variables[this.variableIndex].function === this.aceEditor.getValue())) {
      this.aceEditor.getSession().off('change', this.functionUpdated)
      this.aceEditor.session.setValue(this.aceEditor.setValue(this.variables[this.variableIndex].function, this.variables[this.variableIndex].function.length - 1))
      this.aceEditor.getSession().on('change', this.functionUpdated)
    }
  }

  protected getRegulationSymbol (essential: Essentiality, monotonicity: Monotonicity): string {
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

  abstract readonly nameUpdated: DebouncedFunc<(name: string) => void>

  abstract readonly functionUpdated: DebouncedFunc<() => void>

  protected monotonicityClass (monotonicity: Monotonicity): string {
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

  abstract toggleEssentiality (regulation: IRegulationData): void
  abstract toggleMonotonicity (regulation: IRegulationData): void
  abstract removeVariable (): void

  protected getEssentialityText (essentiality: Essentiality): string {
    switch (essentiality) {
      case Essentiality.FALSE:
        return 'non-essential'
      case Essentiality.TRUE:
        return 'essential'
      default:
        return 'unknown'
    }
  }
}
