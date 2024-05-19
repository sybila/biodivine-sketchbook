import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './nav-bar.less?inline'
import '../menu/menu'
import '../tab-bar/tab-bar'
import '../undo-redo/undo-redo'
import '../search/search'
import { type TabData } from '../../util/tab-data'

@customElement('nav-bar')
export default class NavBar extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() tabs: TabData[] = []

  render (): TemplateResult {
    return html`
      <div class="nav-bar uk-container uk-container-expand uk-margin-remove uk-padding-remove-horizontal uk-flex-nowrap">
        <nav class="uk-navbar-container uk-navbar-transparent uk-flex-nowrap uk-margin-left uk-margin-right">
          <div class="gap uk-navbar uk-flex-nowrap">
            <div class="uk-navbar-left uk-flex-nowrap">
              <hamburger-menu></hamburger-menu>
              <tab-bar .tabs=${this.tabs}></tab-bar>
            </div>

            <div class="uk-navbar-right uk-flex-nowrap">
              <undo-redo></undo-redo>
<!--              <search-bar></search-bar>-->
            </div>

          </div>
        </nav>
      </div>
    `
  }
}
