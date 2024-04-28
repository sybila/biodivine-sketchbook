import { css, unsafeCSS, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './property-tile.less?inline'
import {
  type IExistsTrajectoryProperty,
  type IFixedPointDynamicProperty,
  type IProperty,
  type ITrapSpaceDynamicProperty
} from '../../../util/data-interfaces'
import { debounce } from 'lodash'
import { functionDebounceTimer } from '../../../util/config'

@customElement('property-tile')
export default class PropertyTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() declare property: IProperty
  @property() declare index: number

  nameUpdated = debounce((name: string) => {
    this.dispatchEvent(new CustomEvent('property-name-changed', {
      detail: {
        id: this.property.id,
        name
      },
      composed: true,
      bubbles: true
    }))
  }, functionDebounceTimer)

  // TODO: there has to be a better way to handle types
  updateProperty (property: IFixedPointDynamicProperty | ITrapSpaceDynamicProperty | IExistsTrajectoryProperty): void {
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
