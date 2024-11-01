import { LitElement, html, css, unsafeCSS, type TemplateResult } from 'lit'
import { property, customElement } from 'lit/decorators.js'
import style_less from './annotation-tile.less?inline'
import type { IObservationSet } from '../../../util/data-interfaces'

@customElement('dataset-tile')
export class DatasetTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare data: IObservationSet

  render (): TemplateResult {
    return html`
      <div class="dataset">
          <p><b>${this.data.id}</b>: ${this.data.annotation}<br>
          <ul>
            ${this.data.observations
                .filter(observation => observation.annotation.trim() !== '')
                .map(observation => html`
                  <li><b>${observation.id}</b>: ${observation.annotation}<br></li>
                `)}
          </ul>
          <hr>
      </div>
    `
  }
}
