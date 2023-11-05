class UndoRedo extends HTMLElement {
  shadow

  constructor () {
    super()
    const template = document.getElementById('undo-redo') as HTMLTemplateElement
    const content = template.content
    this.shadow = this.attachShadow({ mode: 'open' })
    this.shadow.appendChild(content.cloneNode(true))
  }
}

customElements.define('undo-redo', UndoRedo)
