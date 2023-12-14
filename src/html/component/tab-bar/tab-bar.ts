import { html, css, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './tab-bar.less?inline'
import { type TabData } from '../../util/tab-data'
import { faLock, fas, type IconName } from '@fortawesome/free-solid-svg-icons'
import { findIconDefinition, icon, library } from '@fortawesome/fontawesome-svg-core'
import { aeonState } from '../../../aeon_events'
library.add(faLock, fas)

@customElement('tab-bar')
class TabBar extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  @property() tabs: TabData[] = []

  switchTab (tabId: number) {
    return () => {
      aeonState.tab_bar.active.emitValue(tabId)
    }
  }

  render (): TemplateResult {
    return html`
      <div class="tabs uk-flex uk-flex-row">
        ${map(this.tabs, (tab) => html`
            <button class="tab uk-button uk-padding-remove-vertical ${tab.active ? 'uk-button-primary' : 'uk-button-secondary'}" 
                    @click=${this.switchTab(tab.id)}>
                ${tab.pinned ? icon(faLock).node : ''}
                ${icon(findIconDefinition({ prefix: 'fas', iconName: `${tab.icon as IconName}` })).node}
                <span class="tab-name">${tab.name}</span>
            </button>
        `)}
      </div>
    `
  }
}
