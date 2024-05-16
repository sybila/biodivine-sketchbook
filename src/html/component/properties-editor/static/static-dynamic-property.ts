import { type DynamicProperty, type StaticProperty } from '../../../util/data-interfaces'
import { debounce } from 'lodash'
import AbstractProperty from '../abstract-property/abstract-property'
import { html, type TemplateResult } from 'lit'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash } from '@fortawesome/free-solid-svg-icons'
import { when } from 'lit/directives/when.js'

const EVENT_PROPERTY_CHANGED = 'static-property-changed'
const EVENT_PROPERTY_REMOVED = 'static-property-removed'

export default class StaticDynamicProperty extends AbstractProperty {
  nameUpdated = debounce((name: string) => {
    super.nameUpdated(name, EVENT_PROPERTY_CHANGED)
  }, 0)

  removeProperty (): void {
    super.removeProperty(EVENT_PROPERTY_REMOVED)
  }

  updateProperty (property: DynamicProperty | StaticProperty): void {
    super.updateProperty(property, EVENT_PROPERTY_CHANGED)
  }

  renderNameplate (removeButton: boolean = true): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-row">
        <input id="name-field" class="name-field static-name-field" value="${this.property.name}" readonly/>
        ${when(removeButton, () => html`
          <button class="remove-property" @click="${this.removeProperty}">
            ${icon(faTrash).node}
          </button>
        `)}
        
      </div>
    `
  }
}
