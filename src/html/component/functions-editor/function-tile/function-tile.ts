import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'
import style_less from './function-tile.less?inline'
import { map } from 'lit/directives/map.js'
import { type IRegulationData, type IVariableData } from '../../../util/data-interfaces'
import { Monotonicity } from '../../regulations-editor/element-type'
import { debounce } from 'lodash'
import { icon, library } from '@fortawesome/fontawesome-svg-core'
import { faTrash, faMagnifyingGlass } from '@fortawesome/free-solid-svg-icons'

library.add(faTrash, faMagnifyingGlass)

@customElement('function-tile')
class FunctionTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare variable: IVariableData
  @property() regulations: IRegulationData[] = []
  @query('#function-input') functionInput: HTMLInputElement | undefined
  @state() variableFunction = ''
  @state() variableName = ''

  constructor () {
    super()
    this.addEventListener('focus-function-field', () => this.functionInput?.focus())
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
        oldId: this.variable.id,
        ...this.variable,
        name
      },
      bubbles: true,
      composed: true
    }))
  },
  300
  )

  private readonly functionUpdated = debounce((func: string) => {
    this.dispatchEvent(new CustomEvent('update-variable', {
      detail: {
        ...this.variable,
        oldId: this.variable.id,
        function: func
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
        id: this.variable.id
      },
      composed: true,
      bubbles: true
    }))
  }

  private focusVariable (): void {
    this.dispatchEvent(new CustomEvent('focus-variable', {
      detail: {
        variableId: this.variable.id
      },
      bubbles: true,
      composed: true
    }))
  }

  protected render (): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="uk-flex uk-flex-row">
          <input class="uk-input uk-text-center" value="${this.variable.name}"
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
              class="regulation uk-grid uk-grid-column-small uk-grid-row-large uk-child-width-1-4 uk-margin-remove uk-text-center uk-flex-right uk-text-nowrap">
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
        <input id="function-input"
               class="uk-input uk-text-center"
               value="${this.variable.function}"
               placeholder="$f_${this.variable.name}(...)"
               @input="${(e: InputEvent) => this.functionUpdated((e.target as HTMLInputElement).value)}"/>
      </div>
      <hr>
    `
  }
}
