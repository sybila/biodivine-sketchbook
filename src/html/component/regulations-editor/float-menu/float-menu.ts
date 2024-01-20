import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './float-menu.less?inline'
import { findIconDefinition, icon, library } from '@fortawesome/fontawesome-svg-core'
import {
  faArrowTrendDown,
  faArrowTrendUp,
  faCalculator,
  faEye,
  faEyeSlash,
  faPen,
  faPlus,
  faRightLeft,
  faTrash
} from '@fortawesome/free-solid-svg-icons'
import { type Position } from 'cytoscape'
import { map } from 'lit/directives/map.js'
import { ElementType, type IRegulationData, type IVariableData, Monotonicity } from '../../../util/data-interfaces'

library.add(faRightLeft, faArrowTrendUp, faArrowTrendDown, faCalculator, faEye, faEyeSlash, faPen, faTrash, faPlus)

@customElement('float-menu')
class FloatMenu extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() type = ElementType.NONE
  @property() position: Position = { x: 0, y: 0 }
  @property() zoom = 1.0
  @property() data: (IRegulationData & IVariableData) | undefined
  @state() selectedButton: IButton | undefined = undefined

  connectedCallback (): void {
    super.connectedCallback()
    document.addEventListener('keydown', this._handleKeyDown.bind(this))
  }

  private _handleKeyDown (event: KeyboardEvent): void {
    switch (this.type) {
      case ElementType.NODE:
        switch (event.key.toUpperCase()) {
          case 'E':
            this.renameNode()
            break
          case 'A':
            this.addEdge()
            break
          case 'F':
            this.focusRegulation()
            break
          case 'DELETE':
          case 'BACKSPACE':
            this.removeElement()
            break
        }
        break
      case ElementType.EDGE:
        switch (event.key.toUpperCase()) {
          case 'O':
            this.toggleObservability()
            break
          case 'M':
            this.toggleMonotonicity()
            break
          case 'DELETE':
            this.removeElement()
            break
        }
    }
  }

  nodeButtons: IButton[] = [
    {
      icon: () => icon(faPen).node[0],
      label: () => 'Edit name (E)',
      click: this.renameNode
    },
    {
      icon: () => icon(faPlus).node[0],
      label: () => 'Add Edge (A)',
      click: this.addEdge
    },
    {
      icon: () => icon(faCalculator).node[0],
      label: () => 'Edit update function (F)',
      click: this.focusRegulation
    },
    {
      icon: () => icon(faTrash).node[0],
      label: () => 'Remove (⌫)',
      click: this.removeElement
    }
  ]

  edgeButtons: IButton[] = [
    {
      icon: () => icon(this.data?.observable === true ? faEyeSlash : faEye).node[0],
      label: () =>
        this.data === null || this.data?.observable === null
          ? 'Toggle observability (O)'
          : ((this.data?.observable) === true) ? 'Observability off (O)' : 'Observability on (O)',
      click: this.toggleObservability
    },
    {
      icon: () => icon(findIconDefinition(
        this.data?.monotonicity === Monotonicity.INHIBITION
          ? faRightLeft
          : this.data?.monotonicity === Monotonicity.ACTIVATION
            ? faArrowTrendDown
            : faArrowTrendUp
      )).node[0],
      label: () => {
        switch (this.data?.monotonicity) {
          case Monotonicity.UNSPECIFIED:
            return 'Make activating (M)'
          case Monotonicity.ACTIVATION:
            return 'Make inhibiting (M)'
          case Monotonicity.INHIBITION:
            return 'Monotonicity off (M)'
          default:
            return 'Toggle monotonicity (M)'
        }
      },
      click: this.toggleMonotonicity
    },
    {
      icon: () => icon(faTrash).node[0],
      label: () => 'Remove (⌫)',
      click: this.removeElement
    }
  ]

  private removeElement (): void {
    switch (this.type) {
      case ElementType.EDGE:
        this.dispatchEvent(new CustomEvent('remove-regulation', {
          detail: {
            source: this.data?.source,
            target: this.data?.target
          },
          bubbles: true,
          composed: true
        }))
        break
      case ElementType.NODE:
        this.dispatchEvent(new CustomEvent('remove-variable', {
          detail: {
            id: this.data?.id
          },
          bubbles: true,
          composed: true
        }))
        break
    }
  }

  private toggleObservability (): void {
    this.dispatchEvent(new CustomEvent('set-regulation-observable', {
      detail: {
        id: this.data?.id,
        source: this.data?.source,
        target: this.data?.target,
        observable: !(this.data?.observable ?? false)
      },
      bubbles: true,
      composed: true
    }))
    if (this.data !== undefined) this.data = { ...this.data, observable: !(this.data?.observable ?? false) }
  }

  private toggleMonotonicity (): void {
    let monotonicity
    switch (this.data?.monotonicity) {
      case Monotonicity.ACTIVATION:
        monotonicity = Monotonicity.INHIBITION
        break
      case Monotonicity.INHIBITION:
        monotonicity = Monotonicity.UNSPECIFIED
        break
      default:
        monotonicity = Monotonicity.ACTIVATION
        break
    }
    if (this.data !== undefined) this.data = { ...this.data, monotonicity }
    this.dispatchEvent(new CustomEvent('set-regulation-monotonicity', {
      detail: {
        id: this.data?.id,
        source: this.data?.source,
        target: this.data?.target,
        monotonicity
      },
      bubbles: true,
      composed: true
    }))
  }

  private renameNode (): void {
    this.dispatchEvent(new CustomEvent('rename-node', {
      detail: {
        id: this.data?.id,
        name: this.data?.name
      },
      bubbles: true,
      composed: true
    }))
  }

  private addEdge (): void {
    this.dispatchEvent(new CustomEvent('add-edge', {
      detail: {
        id: this.data?.id
      },
      bubbles: true,
      composed: true
    }))
  }

  private focusRegulation (): void {
    this.dispatchEvent(new CustomEvent('focus-function', {
      detail: {
        variableId: this.data?.id
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
                                       top: ${this.position.y + 8 + yOffset}px; 
                                       transform: scale(${this.zoom})">
            <div class="button-row uk-flex uk-flex-row" style="width: ${buttons.length * 2}em">
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
