import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './node-menu.less?inline'
import { findIconDefinition, icon, library } from '@fortawesome/fontawesome-svg-core'
import {
  faArrowTrendDown,
  faArrowTrendUp,
  faCalculator,
  faEye,
  faEyeSlash,
  faPen,
  faRightLeft,
  faTrash
} from '@fortawesome/free-solid-svg-icons'
import { type Position } from 'cytoscape'
import { map } from 'lit/directives/map.js'
import { ElementType, Monotonicity } from './element-type'

library.add(faRightLeft, faArrowTrendUp, faArrowTrendDown, faCalculator, faEye, faEyeSlash, faPen, faTrash)

@customElement('node-menu')
class NodeMenu extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() type = ElementType.NONE
  @property() position: Position = { x: 0, y: 0 }
  @property() zoom = 1.0
  @property() data: { id: string, observable: boolean, monotonicity: Monotonicity } | undefined
  @state() selectedButton: IButton | undefined = undefined

  nodeButtons: IButton[] = [
    {
      icon: () => icon(findIconDefinition({ prefix: 'fas', iconName: 'pen' })).node[0],
      label: () => 'Edit name (E)',
      click: () => {}
    },
    {
      icon: () => icon(findIconDefinition({ prefix: 'fas', iconName: 'calculator' })).node[0],
      label: () => 'Edit update function (F)',
      click: () => {}
    },
    {
      icon: () => icon(findIconDefinition({ prefix: 'fas', iconName: 'trash' })).node[0],
      label: () => 'Remove (⌫)',
      click: this.removeElement
    }
  ]

  edgeButtons: IButton[] = [
    {
      icon: () => icon(findIconDefinition({ prefix: 'fas', iconName: ((this.data?.observable) === true) ? 'eye-slash' : 'eye' })).node[0],
      label: () =>
        this.data === null || this.data?.observable === null
          ? 'Toggle observability (O)'
          : ((this.data?.observable) === true) ? 'Observability off (O)' : 'Observability on (O)',
      click: () => {
        this.dispatchEvent(new CustomEvent('update-edge', {
          detail: {
            edgeId: this.data?.id,
            observable: !(this.data?.observable ?? false),
            monotonicity: this.data?.monotonicity
          },
          bubbles: true,
          composed: true
        }))
        if (this.data !== undefined) this.data = { ...this.data, observable: !(this.data?.observable ?? false) }
      }
    },
    {
      icon: () => icon(findIconDefinition({
        prefix: 'fas',
        iconName: this.data?.monotonicity === Monotonicity.INHIBITION
          ? 'right-left'
          : this.data?.monotonicity === Monotonicity.ACTIVATION ? 'arrow-trend-down' : 'arrow-trend-up'
      })).node[0],
      label: () => {
        switch (this.data?.monotonicity) {
          case Monotonicity.OFF:
            return 'Make activating (M)'
          case Monotonicity.ACTIVATION:
            return 'Make inhibiting (M)'
          case Monotonicity.INHIBITION:
            return 'Monotonicity off (M)'
          default:
            return 'Toggle monotonicity (M)'
        }
      },
      click: () => {
        let monotonicity
        switch (this.data?.monotonicity) {
          case Monotonicity.ACTIVATION:
            monotonicity = Monotonicity.INHIBITION
            break
          case Monotonicity.INHIBITION:
            monotonicity = Monotonicity.OFF
            break
          default:
            monotonicity = Monotonicity.ACTIVATION
            break
        }
        if (this.data !== undefined) this.data = { ...this.data, monotonicity }
        this.dispatchEvent(new CustomEvent('update-edge', {
          detail: {
            edgeId: this.data?.id,
            observable: (this.data?.observable ?? true),
            monotonicity
          },
          bubbles: true,
          composed: true
        }))
      }
    },
    {
      icon: () => icon(findIconDefinition({ prefix: 'fas', iconName: 'trash' })).node[0],
      label: () => 'Remove (⌫)',
      click: this.removeElement
    }
  ]

  private removeElement (): void {
    this.dispatchEvent(new CustomEvent('remove-element', {
      detail: {
        id: this.data?.id
      },
      bubbles: true,
      composed: true
    }))
  }

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
                  const icon = buttonData.icon()
                    icon.classList.add('menu-icon')
                    return html`
                    <span class="float-button"
                         @mouseover=${() => { this.selectedButton = buttonData }} 
                         @mouseout=${() => { this.selectedButton = undefined }} 
                         @click=${buttonData.click}>
                        ${icon}
                    </span>
                `
})}
            </div>
            <span class="hint">${this.selectedButton?.label()}</span>
        </div>`
        }
    `
  }
}

interface IButton {
  icon: () => Element
  label: () => string
  click: (event: MouseEvent) => void
}
