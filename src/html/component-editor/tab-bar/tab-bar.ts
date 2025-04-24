import { html, css, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './tab-bar.less?inline'
import { type TabData } from '../../util/tab-data'
import { faLock, fas, type IconName } from '@fortawesome/free-solid-svg-icons'
import { findIconDefinition, icon, library } from '@fortawesome/fontawesome-svg-core'
import { aeonState } from '../../../aeon_state'
library.add(faLock, fas)

@customElement('tab-bar')
export default class TabBar extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  @property() tabs: TabData[] = []

  switchTab (tabId: number) {
    return () => {
      // Some nodes in the Cytoscape editor may be selected during tab switch,
      // so we just send an event to deselect all elements before any switching
      // this sends an event processed in RegulationsEditor component
      this.dispatchEvent(new CustomEvent('unselect-cytoscape-nodes', {
        bubbles: true,
        composed: true,
        detail: {} // maybe include some information later
      }))

      aeonState.tabBar.active.emitValue(tabId)
    }
  }

  render (): TemplateResult {
    return html`
      <div class="tab-bar uk-button-group uk-flex uk-flex-row">
        ${map(this.tabs, (tab) => html`
            <button id="${tab.name}-tab-button" class="tab uk-button uk-padding-remove-vertical ${tab.active ? 'active uk-button-primary' : 'inactive uk-button-secondary'}" 
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
