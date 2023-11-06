import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement } from 'lit/decorators.js'
import style_less from './undo-redo.less?inline'

@customElement('undo-redo')
class UndoRedo extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  render (): TemplateResult {
    return html`
      <div class="uk-flex-nowrap">
        <button class="uk-button uk-button-secondary uk-button-small">&#8249;</button>
        <button class="uk-button uk-button-secondary uk-button-small" disabled>&#8250;</button>
      </div>
    `
  }
}
