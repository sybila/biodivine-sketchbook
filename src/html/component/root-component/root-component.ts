import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement } from 'lit/decorators.js'
import style_less from './root-component.less?inline'
import '../content-pane/content-pane'
import '../nav-bar/nav-bar'
import { type ContentPane } from '../content-pane/content-pane'

@customElement('root-component')
class RootComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  private panes: ContentPane[] = []
  private dynamicPane: ContentPane = document.createElement('content-pane') as ContentPane

  private pinPane (): void {
    console.log('pin')
    this.panes[this.dynamicPane.tabId] = this.dynamicPane
    const nextPane = this.panes.slice(this.dynamicPane.tabId).find((p) => p !== undefined)
    if (nextPane !== undefined) {
      this.shadowRoot?.insertBefore(this.dynamicPane, nextPane)
    }
    this.dynamicPane = document.createElement('content-pane') as ContentPane
  }

  private switchTab (e: Event): void {
    this.dynamicPane.dispatchEvent(new CustomEvent('switch-tab', {
      detail: {
        tabId: (e as CustomEvent).detail.tabId
      }
    }))
  }

  render (): TemplateResult {
    this.addEventListener('switch-tab', this.switchTab)
    this.addEventListener('pin-pane', this.pinPane)
    this.dynamicPane = document.createElement('content-pane') as ContentPane

    return html`
      <div class="uk-container-expand">
        <nav-bar></nav-bar>
        ${this.dynamicPane}
      </div>
    `
  }
}
