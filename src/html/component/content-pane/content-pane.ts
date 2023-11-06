import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement } from 'lit/decorators.js'
import style_less from './content-pane.less'

@customElement('content-pane')
export class ContentPane extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  private readonly _heading = document.createElement('h1')
  private _tabId = -1
  private isPinned = false

  private createPinButton (): HTMLButtonElement {
    const button = document.createElement('button')
    button.classList.add('uk-button', 'uk-button-small', 'uk-button-secondary', 'pin-button')
    button.innerText = 'pin'
    button.onclick = () => {
      this.isPinned = !this.isPinned
      if (this.isPinned) {
        this.dispatchEvent(new CustomEvent('pin-pane', {
          detail: {
            tabId: this._tabId
          },
          bubbles: true,
          composed: true
        }))
      } else {
        this.dispatchEvent(new CustomEvent('unpin-pane', {
          detail: {
            tabId: this._tabId
          },
          bubbles: true,
          composed: true
        }))
      }
      button.innerText = this.isPinned ? 'unpin' : 'pin' // todo: icons
    }
    return button
  }

  get tabId (): number {
    return this._tabId
  }

  protected render (): TemplateResult {
    this.addEventListener('switch-tab', (e) => {
      this._tabId = (e as CustomEvent).detail.tabId
      this._heading.innerText = `Content of tab ${this._tabId}`
    })
    const pinButton = this.createPinButton()
    this._heading.classList.add('uk-heading-large', 'uk-text-success')
    return html`
      <div class="content-pane uk-container uk-container-expand uk-margin-top">
        ${pinButton}
        ${this._heading}
      </div>
    `
  }
}
