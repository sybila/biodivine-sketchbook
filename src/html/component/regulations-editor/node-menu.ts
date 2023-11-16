import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './node-menu.less?inline'
import { library, icon, findIconDefinition } from '@fortawesome/fontawesome-svg-core'
import { faPen, faTrash, faCalculator } from '@fortawesome/free-solid-svg-icons'
import { type Position } from 'cytoscape'
import { map } from 'lit/directives/map.js'
library.add(faPen, faTrash, faCalculator)

@customElement('node-menu')
class NodeMenu extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() visible = false
  @property() position: Position = { x: 0, y: 0 }
  @property() zoom = 1.0
  @state() hint = ''

  buttons = [
    {
      icon: icon(findIconDefinition({ prefix: 'fas', iconName: 'pen' })).node[0],
      label: 'Edit name (E)'
    },
    {
      icon: icon(findIconDefinition({ prefix: 'fas', iconName: 'calculator' })).node[0],
      label: 'Edit update function (F)'
    },
    {
      icon: icon(findIconDefinition({ prefix: 'fas', iconName: 'trash' })).node[0],
      label: 'Remove (⌫)'
    }
  ]

  render (): TemplateResult {
    return html`
        ${this.visible && html`
        <div class="float-menu" style="left: ${this.position.x}px; 
                                       top: ${this.position.y + (70 * this.zoom)}px; 
                                       transform: scale(${this.zoom * 0.75}) translate(-50%, -50%)">
            <div class="button-row uk-flex uk-flex-row">
                ${map(this.buttons, (buttonData) => {
                  buttonData.icon.classList.add('menu-icon')
                    return html`
                    <div class="float-button" 
                       @mouseover=${() => { this.hint = buttonData.label }} 
                       @mouseout=${() => { this.hint = '' }}>
                        ${buttonData.icon}
                    </div>
                `
})}
            </div>
            <span class="hint">${this.hint}</span>
        </div>`
        }
    `
  }
}
