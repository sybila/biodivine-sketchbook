import { html, css, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './tab-bar.less?inline'
import { type TabData } from '../../util/tab-data'

@customElement('tab-bar')
class TabBar extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  @property() tabs: TabData[] = []

  private addTab (): void {
    this.dispatchEvent(new Event('add-tab', { bubbles: true, composed: true }))
    this.requestUpdate()
  }

  switchTab (tabId: number) {
    return () => {
      this.dispatchEvent(new CustomEvent('switch-tab', {
        detail: {
          tabId
        },
        bubbles: true,
        composed: true
      }))
      this.requestUpdate()
    }
  }

  private reset (): void {
    this.dispatchEvent(new Event('reset', { bubbles: true, composed: true }))
    this.requestUpdate()
  }

  render (): TemplateResult {
    return html`
      <div class="tabs">
        ${map(this.tabs, (tab) => html`
            <button class="tab uk-button ${tab.active ? 'uk-button-primary' : 'uk-button-secondary'}" 
                    @click=${this.switchTab(tab.id)} 
                    ${tab.pinned ? 'disabled' : ''}>
                ${tab.name}
            </button>
        `)}
        <button class="uk-button uk-button-default uk-button-small new-tab-button" @click=${this.addTab}><span class="plus-symbol">+</span></button>
        <button class="uk-button-small uk-button-secondary uk-margin-left" @click=${this.reset}>\u21bb</button>
      </div>
    `
  }
}
