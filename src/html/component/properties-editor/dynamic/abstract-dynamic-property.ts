import { type DynamicProperty, type StaticProperty } from '../../../util/data-interfaces'
import { debounce } from 'lodash'
import AbstractProperty from '../abstract-property/abstract-property'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { html, type TemplateResult } from 'lit'

const EVENT_PROPERTY_CHANGED = 'dynamic-property-changed'
const EVENT_PROPERTY_REMOVED = 'dynamic-property-removed'

export default class AbstractDynamicProperty extends AbstractProperty {
  nameUpdated = debounce((name: string) => {
    super.nameUpdated(name, EVENT_PROPERTY_CHANGED)
  }, 0)

  removeProperty (): void {
    super.removeProperty(EVENT_PROPERTY_REMOVED)
  }

  updateProperty (property: DynamicProperty | StaticProperty): void {
    super.updateProperty(property, EVENT_PROPERTY_CHANGED)
  }

  renderNameplate (): TemplateResult {
    return html`
            <div class="uk-flex uk-flex-row">
          <input id="name-field" class="name-field" value="${this.property.name}"
                 @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          <button class="remove-property" @click="${this.removeProperty}">
            ${icon(faTrash).node}
          </button>
        </div>
        `
  }
}
