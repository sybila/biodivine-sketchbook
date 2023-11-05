const SAVE_KEY = 'tabs'
class TabBar extends HTMLElement {
  tabs: HTMLButtonElement[] = []
  tabIndex = 0
  shadow

  constructor () {
    super()
    const template = document.getElementById('tab-bar') as HTMLTemplateElement
    const content = template.content
    this.shadow = this.attachShadow({ mode: 'open' })
    this.shadow.appendChild(content.cloneNode(true))
  }

  connectedCallback (): void {
    this.loadTabs()
    this.addTab()
    this.resetButton()
  }

  private addTab (): void {
    const addTabButton = this.shadow.querySelector('.new-tab-button')
    addTabButton?.addEventListener('click', () => {
      this.tabIndex++
      this.createTab(this.tabIndex, `Tab ${this.tabIndex}`, ['tab', 'uk-button', 'uk-button-secondary'])
    })
  }

  private resetButton (): void {
    const resetButton = document.createElement('button')
    resetButton.classList.add('tab', 'uk-button-small', 'uk-button-secondary', 'uk-margin-left')
    resetButton.textContent = '\u21bb'
    resetButton.onclick = () => {
      this.tabIndex = 0
      this.tabs.forEach(tab => { tab.remove() })
      this.tabs = []
      localStorage.removeItem(SAVE_KEY)
    }
    this.shadow.querySelector('.tabs')?.appendChild(resetButton)
  }

  private saveTabs (): void {
    const tabData = this.tabs.map((tab) => ({
      classList: Array.from(tab.classList),
      textContent: tab.textContent,
      index: tab.dataset.index
    }))
    localStorage.setItem(SAVE_KEY, JSON.stringify(tabData))
  }

  private loadTabs (): void {
    const tabData = JSON.parse(localStorage.getItem(SAVE_KEY) ?? '[]')
    tabData.forEach((data: { classList: string[], textContent: string, index: string }) => {
      this.createTab(+data.index, data.textContent, data.classList)
      this.tabIndex = Math.max(this.tabIndex, +data.index)
    })
    console.log('tabs loaded')
    console.log('index: ', this.tabIndex)
  }

  private createTab (index: number, title: string, classList: string[]): void {
    const newTabButton = document.createElement('button')
    newTabButton.classList.add(...classList)
    newTabButton.onclick = this.switchTab(newTabButton)
    newTabButton.dataset.index = index.toString()
    newTabButton.textContent = title
    this.tabs.push(newTabButton)

    this.saveTabs()

    const addTabButton = this.shadow.querySelector('.new-tab-button')
    this.shadow.querySelector('.tabs')?.insertBefore(newTabButton, addTabButton)
  }

  private switchTab (newTabButton: HTMLButtonElement) {
    return () => {
      this.tabs.forEach((tab) => {
        tab.classList.remove('uk-button-primary')
        tab.classList.add('uk-button-secondary')
      })
      newTabButton.classList.remove('uk-button-secondary')
      newTabButton.classList.add('uk-button-primary')
      const tabIndex = +(newTabButton.dataset.index ?? 0)
      this.dispatchEvent(new CustomEvent('switch-tab', {
        detail: {
          tabId: tabIndex
        },
        bubbles: true,
        composed: true
      }))
      console.log(`clicked tab ${tabIndex}`)
    }
  }
}

customElements.define('tab-bar', TabBar)
