import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import style_less from './root-component.less?inline'
import '../content-pane/content-pane'
import '../nav-bar/nav-bar'
import { type ContentPane } from '../content-pane/content-pane'

const PIN_LIMIT = 1000

@customElement('root-component')
class RootComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @state() panes: ContentPane[] = []
  constructor () {
    super()
    this.addEventListener('switch-tab', this.switchTab)
    this.addEventListener('pin-pane', this.pinPane)
  }

  private pinPane (): void {
    console.log('pin')
    this.panes[this.panes[PIN_LIMIT].tabId] = this.panes[PIN_LIMIT]
    this.requestUpdate()
    this.panes[PIN_LIMIT] = document.createElement('content-pane') as ContentPane
  }

  private switchTab (e: Event): void {
    this.panes[PIN_LIMIT].dispatchEvent(new CustomEvent('switch-tab', {
      detail: {
        tabId: (e as CustomEvent).detail.tabId
      }
    }))
  }

  render (): TemplateResult {
    this.panes[PIN_LIMIT] = document.createElement('content-pane') as ContentPane

    return html`
      <div class="uk-container-expand">
        <nav-bar></nav-bar>
          <div></div>
          ${this.panes}
      </div>
    `
  }
}
