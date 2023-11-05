class RootComponent extends HTMLElement {
  shadow
  panes: ContentPane[] = []
  dynamicPane: ContentPane

  constructor () {
    super()
    const template = document.getElementById('root-component') as HTMLTemplateElement
    const content = template.content
    this.shadow = this.attachShadow({ mode: 'open' })
    this.shadow.appendChild(content.cloneNode(true))
    this.dynamicPane = this.newPane()
  }

  connectedCallback (): void {
    this.addEventListener('switch-tab', this.switchTab)
    this.addEventListener('pin-pane', this.pinPane)
  }

  private pinPane (): void {
    this.panes[this.dynamicPane.tabId] = this.dynamicPane
    const nextPane = this.panes.slice(this.dynamicPane.tabId).find((p) => p !== undefined)
    if (nextPane !== undefined) {
      this.shadow.insertBefore(this.dynamicPane, nextPane)
    }
    this.dynamicPane = this.newPane()
  }

  private switchTab (e: Event): void {
    this.dynamicPane.dispatchEvent(new CustomEvent('switch-tab', {
      detail: {
        tabId: (e as CustomEvent).detail.tabId
      }
    }))
  }

  private newPane (): ContentPane {
    const pane = document.createElement('content-pane') as ContentPane
    this.shadow.appendChild(pane)
    return pane
  }
}

customElements.define('root-component', RootComponent)
