class Menu extends HTMLElement {
  shadow

  constructor () {
    super()
    const template = document.getElementById('menu') as HTMLTemplateElement
    const content = template.content
    this.shadow = this.attachShadow({ mode: 'open' })
    this.shadow.appendChild(content.cloneNode(true))
  }
}

customElements.define('hamburger-menu', Menu)
