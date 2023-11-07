import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import style_less from './content-pane.less?inline'

@customElement('content-pane')
export class ContentPane extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  private _tabId = -1
  @state() private isPinned = false
  @state() private content = ''

  constructor () {
    super()
    this.addEventListener('switch-tab', (e) => {
      this._tabId = (e as CustomEvent).detail.tabId
      this.content = `Content of tab ${this._tabId}`
    })
  }

  private pin (): void {
    this.isPinned = !this.isPinned
    this.dispatchEvent(new CustomEvent(this.isPinned ? 'pin-pane' : 'unpin-pane', {
      detail: {
        tabId: this._tabId
      },
      bubbles: true,
      composed: true
    }))
  }

  get tabId (): number {
    return this._tabId
  }

  protected render (): TemplateResult {
    return html`
            <div class="content-pane uk-container uk-container-expand uk-margin-top">
                <button class="uk-button uk-button-small uk-button-secondary pin-button" @click="${this.pin}">${this.isPinned ? 'unpin' : 'pin'}</button>
                <h1 class="uk-heading-large uk-text-success">${this.content}</h1>
            </div>
        `
  }
}
