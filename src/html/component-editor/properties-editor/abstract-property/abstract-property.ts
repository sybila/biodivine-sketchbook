import { css, LitElement, unsafeCSS } from 'lit'
import { property } from 'lit/decorators.js'
import style_less from './abstract-property.less?inline'
import { type DynamicProperty, type StaticProperty } from '../../../util/data-interfaces'

export default class AbstractProperty extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: DynamicProperty | StaticProperty
  @property() declare index: number

  nameUpdated (name: string, eventName: string): void {
    this.dispatchEvent(new CustomEvent(eventName, {
      detail: {
        property: {
          ...this.property,
          name
        }
      },
      bubbles: true,
      composed: true
    }))
  }

  idUpdated (oldId: string, newId: string, eventName: string): void {
    this.dispatchEvent(new CustomEvent(eventName, {
      detail: {
        oldId,
        newId
      },
      bubbles: true,
      composed: true
    }))
  }

  removeProperty (eventName: string): void {
    this.dispatchEvent(new CustomEvent(eventName, {
      detail: {
        id: this.property.id
      },
      bubbles: true,
      composed: true
    }))
  }

  updateProperty (property: DynamicProperty | StaticProperty, eventName: string): void {
    this.dispatchEvent(new CustomEvent(eventName, {
      detail: {
        property
      },
      bubbles: true,
      composed: true
    }))
  }

  editProperty (id: string, eventName: string): void {
    this.dispatchEvent(new CustomEvent(eventName, {
      detail: {
        id
      },
      bubbles: true,
      composed: true
    }))
  }
}
