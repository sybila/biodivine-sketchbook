class ContentPane extends HTMLElement {
  private readonly _shadow
  private readonly _heading
  private _tabId = -1
  private isPinned

  constructor () {
    super()
    const template = document.getElementById('content-pane') as HTMLTemplateElement
    const content = template.content
    this._shadow = this.attachShadow({ mode: 'open' })
    this._shadow.appendChild(content.cloneNode(true))
    this._shadow.appendChild(this.createPinButton())
    this._heading = document.createElement('h1')
    this._heading.classList.add('uk-heading-large', 'uk-text-success')
    this._shadow.appendChild(this._heading)
    this.isPinned = false
  }

  connectedCallback (): void {
    this.addEventListener('switch-tab', (e) => {
      this._tabId = (e as CustomEvent).detail.tabId
      this._heading.innerText = `Content of tab ${this._tabId}`
    })
  }

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
}

customElements.define('content-pane', ContentPane)
