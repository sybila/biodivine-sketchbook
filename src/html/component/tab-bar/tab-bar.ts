import { html, css, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './tab-bar.less?inline'
import { type TabData } from '../../util/tab-data'
import { faLock, faGhost } from '@fortawesome/free-solid-svg-icons'
import { findIconDefinition, icon, library } from '@fortawesome/fontawesome-svg-core'
library.add(faLock, faGhost)

@customElement('tab-bar')
class TabBar extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  @property() tabs: TabData[] = []

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

  render (): TemplateResult {
    return html`
      <div class="tabs uk-flex">
        ${map(this.tabs, (tab) => html`
            <button class="tab uk-button uk-padding-small uk-padding-remove-vertical ${tab.active ? 'uk-button-primary' : 'uk-button-secondary'}" 
                    @click=${this.switchTab(tab.id)} 
                    ${tab.pinned ? 'disabled' : ''}>
                ${icon(findIconDefinition({ prefix: 'fas', iconName: 'ghost' })).node}
                <span class="tab-name">${tab.name}</span>
                ${tab.pinned ? icon(findIconDefinition({ prefix: 'fas', iconName: 'lock' })).node : ''}
            </button>
        `)}
      </div>
    `
  }
}
