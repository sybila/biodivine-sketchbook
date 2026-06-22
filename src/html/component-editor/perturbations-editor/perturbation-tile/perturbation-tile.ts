import { LitElement, html, css, unsafeCSS, type TemplateResult } from 'lit'
import { property, customElement } from 'lit/decorators.js'
import style_less from './perturbation-tile.less?inline'
import { type IPerturbationData } from '../../../util/data-interfaces'

@customElement('perturbation-tile')
export class PerturbationTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare perturbation?: IPerturbationData

  render (): TemplateResult {
    if (this.perturbation === null || this.perturbation === undefined) {
      return html`<div class="perturbation-tile loading">Loading...</div>`
    }

    return html`
      <div class="perturbation-tile">
        <div class="perturbation-header">
          <h3 class="perturbation-name">${this.perturbation.name}</h3>
          <span class="perturbation-id">${this.perturbation.id}</span>
        </div>
        ${this.perturbation.annotation !== null && this.perturbation.annotation !== undefined && this.perturbation.annotation !== ''
          ? html`<p class="perturbation-annotation">${this.perturbation.annotation}</p>`
          : html``}
        <div class="perturbation-vars">
          <h4>Perturbed Variables:</h4>
          ${(this.perturbation.perturbedVars?.size ?? 0) > 0
            ? html`
                <ul>
                  ${Array.from(this.perturbation.perturbedVars.entries()).map(
                    ([varId, value]) => html`
                      <li>
                        <span class="var-id">${varId}</span>
                        <span class="var-value ${value ? 'active' : 'inactive'}">
                          ${value ? '1' : '0'}
                        </span>
                      </li>
                    `
                  )}
                </ul>
              `
            : html`<p class="no-vars">No perturbed variables</p>`}
        </div>
      </div>
    `
  }
}
