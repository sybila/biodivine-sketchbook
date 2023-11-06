import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement } from 'lit/decorators.js'
import style_less from './menu.less?inline'

@customElement('hamburger-menu')
class Menu extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  render (): TemplateResult {
    return html`
      <button class="uk-button uk-button-small uk-button-secondary hamburger-menu">☰</button>
    `
  }
}
