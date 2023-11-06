import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement } from 'lit/decorators.js'
import style_less from './search.less'

@customElement('search-bar')
class SearchBar extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  render (): TemplateResult {
    return html`
      <form class="uk-search uk-search-default" aria-label="Search">
        <input class="uk-search-input" type="search" placeholder="Search..." aria-label="">
      </form>
    `
  }
}
