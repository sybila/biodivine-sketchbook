import { css, LitElement, unsafeCSS } from 'lit'
import { property, query, state } from 'lit/decorators.js'
import style_less from './editor-tile.less?inline'
import { Essentiality, type IFunctionData, type IRegulationData, Monotonicity } from '../../../util/data-interfaces'
import { library } from '@fortawesome/fontawesome-svg-core'
import { faMagnifyingGlass, faTrash } from '@fortawesome/free-solid-svg-icons'
import ace, { type Ace } from 'ace-builds'
import langTools from 'ace-builds/src-noconflict/ext-language_tools'
import 'ace-builds/esm-resolver'
import AeonMode from './custom-ace.conf'
import type { DebouncedFunc } from 'lodash-es'

library.add(faTrash, faMagnifyingGlass)

export abstract class EditorTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() index = 0
  @property() functions: IFunctionData[] = []
  @state() variableFunction = ''
  @state() variableName = ''
  @query('#name-field') nameField: HTMLInputElement | undefined
  declare aceEditor: ace.Ace.Editor

  protected constructor () {
    super()
    ace.require(langTools)
  }

  protected init (index: number, objectList: Array<{ id: string, function: string }>): void {
    const editorElement = this.shadowRoot?.getElementById('function-editor')
    if (editorElement === null || editorElement === undefined) return
    this.aceEditor = ace.edit(editorElement, {
      enableSnippets: true,
      enableLiveAutocompletion: true,
      behavioursEnabled: true,
      value: objectList[index].function,
      placeholder: '$f_' + objectList[index].id + '(...)',
      minLines: 1,
      maxLines: Infinity,
      wrap: 'free',
      fontSize: 14,
      theme: 'ace/theme/cloud_editor'
    })
    // @ts-expect-error ts seems to be ignoring inherited properties
    this.aceEditor.getSession().setMode(new AeonMode())
    this.aceEditor.container.style.lineHeight = '1.5em'
    this.aceEditor.container.style.fontSize = '1em'
    this.aceEditor.renderer.updateFontSize()
    this.aceEditor.on('blur', this.functionUpdated)
    // @ts-expect-error $highlightRules exists but not defined in the d.ts file
    this.aceEditor.session.getMode().$highlightRules.setKeywords({ 'constant.language': objectList.map(v => v.id).join('|') })
    this.aceEditor.renderer.attachToShadowRoot()
  }

  protected updateEditor (name: string, func: string): void {
    if (this.nameField !== undefined) {
      this.nameField.value = name
    }
    if (func !== this.aceEditor.getValue()) {
      this.aceEditor.off('blur', this.functionUpdated)
      this.aceEditor.session.setValue(this.aceEditor.setValue(func, func.length - 1))
      this.aceEditor.on('blur', this.functionUpdated)
    }
    langTools.setCompleters([{
      getCompletions: (_editor: Ace.Editor, _session: Ace.EditSession, _point: Ace.Point, _prefix: string, callback: Ace.CompleterCallback) => {
        callback(null, this.functions.map((v): Ace.ValueCompletion => ({
          meta: v.id + '(' + v.variables.map(fv => fv.source).join(', ') + ')',
          value: v.id + '(' + ')',
          caption: v.id
        })))
      }
    }])
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

  abstract toggleEssentiality (regulation: IRegulationData): void
  abstract toggleMonotonicity (regulation: IRegulationData): void
  abstract removeVariable (): void
}
