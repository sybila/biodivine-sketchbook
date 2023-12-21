import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './function-tile.less?inline'
import { map } from 'lit/directives/map.js'
import { type IEdgeData, type INodeData } from '../../regulations-editor/graph-interfaces'
import { Monotonicity } from '../../regulations-editor/element-type'
import { debounce } from 'lodash'

@customElement('function-tile')
class FunctionTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare variable: INodeData
  @property() regulations: IEdgeData[] = []

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

  nameUpdated = debounce((name: string) => {
    this.dispatchEvent(new CustomEvent('rename-variable', {
      detail: {
        nodeId: this.variable.id,
        nodeName: name
      },
      bubbles: true,
      composed: true
    }))
  },
  300
  )

  functionUpdated = debounce((func: string) => {
    console.log(func)
  })

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
                <span>${regulation.observable ? 'observable' : 'non-observable'}</span>
                <span>${regulation.monotonicity}</span>
              </div>
            `)}
            <label>
              function:
              <input class="uk-input uk-text-center" placeholder="$f_${this.variable.name}(...)" @input="${(e: InputEvent) => this.functionUpdated((e.target as HTMLInputElement).value)}" />
            </label>
          </div>
          <hr>
    `
  }
}
