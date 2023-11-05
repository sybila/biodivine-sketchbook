class ContentPane extends HTMLElement {
  private readonly _shadow
  private readonly _heading
  private _tabId = -1

  constructor () {
    super()
    const template = document.getElementById('content-pane') as HTMLTemplateElement
    const content = template.content
    this._shadow = this.attachShadow({ mode: 'open' })
    this._shadow.appendChild(content.cloneNode(true))
    const linkElem = document.createElement('link')
    linkElem.setAttribute('rel', 'stylesheet')
    linkElem.setAttribute('href', 'component/content-pane/content-pane.less')
    this._shadow.appendChild(linkElem)
    this._heading = document.createElement('h1')
    this._heading.classList.add('uk-heading-large', 'uk-text-success')
    this._shadow.appendChild(this._heading)
  }

  connectedCallback (): void {
    this.addEventListener('switch-tab', (e) => {
      this._tabId = (e as CustomEvent).detail.tabId
      this._heading.innerText = `Content of tab ${this._tabId}`
    })
  }

  get tabId (): number {
    return this._tabId
  }
}

customElements.define('content-pane', ContentPane)
