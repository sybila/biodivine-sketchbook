import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './node-menu.less?inline'
import { findIconDefinition, icon, library } from '@fortawesome/fontawesome-svg-core'
import { faArrowRightArrowLeft, faCalculator, faEye, faPen, faTrash } from '@fortawesome/free-solid-svg-icons'
import { type Position } from 'cytoscape'
import { map } from 'lit/directives/map.js'
import { ElementType } from './element-type'

library.add(faPen, faTrash, faCalculator, faEye, faArrowRightArrowLeft)

@customElement('node-menu')
class NodeMenu extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() type = ElementType.NONE
  @property() position: Position = { x: 0, y: 0 }
  @property() zoom = 1.0
  @state() hint = ''

  nodeButtons: IButton[] = [
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

  edgeButtons: IButton[] = [
    {
      icon: icon(findIconDefinition({ prefix: 'fas', iconName: 'eye' })).node[0],
      label: 'Toggle observability (O)'
    },
    {
      icon: icon(findIconDefinition({ prefix: 'fas', iconName: 'arrow-right-arrow-left' })).node[0],
      label: 'Toggle monotonicity (M)'
    },
    {
      icon: icon(findIconDefinition({ prefix: 'fas', iconName: 'trash' })).node[0],
      label: 'Remove (⌫)'
    }
  ]

  render (): TemplateResult {
    let buttons: IButton[]
    let yOffset = 0
    switch (this.type) {
      case ElementType.NODE:
        buttons = this.nodeButtons
        yOffset = 30 * this.zoom
        break
      case ElementType.EDGE:
        buttons = this.edgeButtons
        break
      case ElementType.NONE:
      default:
        buttons = []
        break
    }
    return html`
        ${this.type !== ElementType.NONE && html`
        <div class="float-menu" style="left: ${this.position.x + 8 - 90 * this.zoom}px; 
                                       top: ${this.position.y + 58 + yOffset}px; 
                                       transform: scale(${this.zoom})">
            <div class="button-row uk-flex uk-flex-row">
                ${map(buttons, (buttonData) => {
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
        </div>
        <!--        <div id="edge-menu" class="float-menu invisible">-->
        <!--            <div class="button-row">-->
        <!--                <img alt="Toggle observability (O)" id="edge-menu-observability" class="button" src="img/visibility_off-24px.svg"-->
        <!--                     src-on="img/visibility_off-24px.svg" src-off="img/visibility_on-24px.svg"-->
        <!--                     alt-on="Observability off (O)" alt-off="Observability on (O)"-->
        <!--                     state=""-->
        <!--                >-->
        <!--                <img alt="Toggle monotonicity (M)" id="edge-menu-monotonicity" class="button" src="img/trending_up-24px.svg"-->
        <!--                     src-unspecified="img/trending_up-24px.svg" alt-unspecified="Make activating (M)"-->
        <!--                     src-activation="img/trending_down-24px.svg" alt-activation="Make inhibiting (M)"-->
        <!--                     src-inhibition="img/swap_vert-24px.svg" alt-inhibition="Monotonicity off (M)"-->
        <!--                     state=""-->
        <!--                >-->
        <!--                <img alt="Remove (⌫)" id="edge-menu-remove" class="button" src="img/delete-24px.svg">-->
        <!--            </div>-->
        <!--            <br>-->
        <!--            <span class="hint invisible">Hint</span>-->
        <!--        </div>-->`
        }
    `
  }
}

interface IButton {
  icon: Element
  label: string
}
