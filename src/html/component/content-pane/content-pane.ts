import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './content-pane.less?inline'
import { type TabData } from '../../util/tab-data'
import UIkit from 'uikit'
import Icons from 'uikit/dist/js/uikit-icons'

UIkit.use(Icons)

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
    return html`
            <div class="content-pane uk-container uk-container-expand uk-margin-top">
                <button class="uk-button uk-button-small uk-button-secondary pin-button" @click="${this.pin}">${this.tab.pinned ? 'unpin' : 'pin'}</button>
                <span class="uk-margin-small-right uk-icon" uk-icon="lock"></span>
                <span uk-icon="icon: home; ratio: 2"></span>
                <span uk-icon="icon: settings; ratio: 2"></span>
                <h1 class="uk-heading-large uk-text-success">${this.tab.data}</h1>
            </div>
        `
  }
}
