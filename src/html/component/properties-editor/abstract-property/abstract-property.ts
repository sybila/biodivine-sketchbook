import { css, LitElement, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './abstract-property.less?inline'
import { type DynamicProperty, type StaticProperty } from '../../../util/data-interfaces'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../../util/config'

@customElement('abstract-property')
export default class AbstractProperty extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: DynamicProperty | StaticProperty
  @property() declare index: number

  nameUpdated = debounce((name: string) => {
    this.dispatchEvent(new CustomEvent('property-changed', {
      detail: {
        property: {
          ...this.property,
          name
        }
      },
      bubbles: true,
      composed: true
    }))
  }, functionDebounceTimer)

  removeProperty (): void {
    this.dispatchEvent(new CustomEvent('property-removed', {
      detail: {
        index: this.index
      },
      bubbles: true,
      composed: true
    }))
  }

  updateProperty (property: DynamicProperty | StaticProperty): void {
    this.dispatchEvent(new CustomEvent('property-changed', {
      detail: {
        property,
        index: this.index
      },
      bubbles: true,
      composed: true
    }))
  }
}
