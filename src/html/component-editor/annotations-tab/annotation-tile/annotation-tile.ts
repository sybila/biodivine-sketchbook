import { LitElement, html, css, unsafeCSS, type TemplateResult } from 'lit'
import { property, customElement } from 'lit/decorators.js'
import style_less from './annotation-tile.less?inline'

@customElement('annotation-tile')
export class AnnotationTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare id: string
  @property() declare content: string

  render (): TemplateResult {
    return html`<b>${this.id}</b>: ${this.content}<br><hr>`
  }
}
