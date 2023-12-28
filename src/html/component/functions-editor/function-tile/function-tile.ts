import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'
import style_less from './function-tile.less?inline'
import { map } from 'lit/directives/map.js'
import { type IRegulationData, type IVariableData } from '../../../util/data-interfaces'
import { Monotonicity } from '../../regulations-editor/element-type'
import { debounce } from 'lodash'

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
    this.addEventListener('focus-function', () => this.functionInput?.focus())
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
    this.dispatchEvent(new CustomEvent('rename-variable', {
      detail: {
        variableId: this.variable.id,
        nodeName: name
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
        variableId: this.variable.id,
        variableName: this.variable.name,
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

  protected render (): TemplateResult {
    return html`
          <div class="uk-flex uk-flex-column uk-margin-small-bottom">
            <div class="uk-inline">
              <span class="uk-form-icon" uk-icon="icon: user"></span>
              <input class="uk-input uk-text-center" value="${this.variable.name}" @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}" />
            </div>
            ${map(this.regulations, (regulation) => html`
              <div class="uk-flex uk-flex-row uk-flex-around">
                <span>${regulation.target}</span>
                <span>${this.getRegulationSymbol(regulation.observable, regulation.monotonicity)}</span>
                <span class="${regulation.observable ? '' : 'uk-text-muted'}"
                      @click="${() => { this.toggleObservability(regulation) }}">
                  ${regulation.observable ? 'observable' : 'non-observable'}
                </span>
                <span class="${this.monotonicityClass(regulation.monotonicity)}"
                      @click="${() => { this.toggleMonotonicity(regulation) }}">
                  ${regulation.monotonicity}
                </span>
              </div>
            `)}
            <label>
              function:
              <input id="function-input"
                     class="uk-input uk-text-center" 
                     value="${this.variable.function}" 
                     placeholder="$f_${this.variable.name}(...)"
                     @input="${(e: InputEvent) => this.functionUpdated((e.target as HTMLInputElement).value)}" />
            </label>
          </div>
          <hr>
    `
  }
}
