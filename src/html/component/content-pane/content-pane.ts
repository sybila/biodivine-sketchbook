import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './content-pane.less?inline'
import { type TabData } from '../../util/tab-data'
import { library, icon, findIconDefinition } from '@fortawesome/fontawesome-svg-core'
import { faLock, faLockOpen } from '@fortawesome/free-solid-svg-icons'
library.add(faLock, faLockOpen)

@customElement('content-pane')
export class ContentPane extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  @property() declare private readonly tab: TabData

  private pin (): void {
    this.dispatchEvent(new CustomEvent(this.tab.pinned ? 'unpin-tab' : 'pin-tab', {
      detail: {
        tabId: this.tab.id
      },
      bubbles: true,
      composed: true
    }))
  }

  protected render (): TemplateResult {
    const locked = icon(findIconDefinition({ prefix: 'fas', iconName: 'lock' })).node
    const unlocked = icon(findIconDefinition({ prefix: 'fas', iconName: 'lock-open' })).node
    return html`
        <div class="content-pane uk-container uk-container-expand">                
                <button class="uk-button uk-button-small uk-button-secondary pin-button" @click="${this.pin}">
                    ${this.tab.pinned ? locked : unlocked}
                </button>
                <h1 class="uk-heading uk-text-success">${this.tab.data}</h1>
            </div>
        `
  }
}
