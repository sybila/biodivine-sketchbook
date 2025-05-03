import { type DynamicProperty, type StaticProperty } from '../../../util/data-interfaces'
import { debounce } from 'lodash'
import AbstractProperty from '../abstract-property/abstract-property'
import { html, type TemplateResult } from 'lit'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faTrash, faEdit } from '@fortawesome/free-solid-svg-icons'
import { when } from 'lit/directives/when.js'
import { functionDebounceTimer } from '../../../util/config'

const EVENT_PROPERTY_CHANGED = 'static-property-changed'
const EVENT_PROPERTY_ID_CHANGED = 'static-property-id-changed'
const EVENT_PROPERTY_REMOVED = 'static-property-removed'
const EVENT_PROPERTY_EDITED = 'static-property-edited'

export default class abstractStaticProperty extends AbstractProperty {
  nameUpdated = debounce((name: string) => {
    super.nameUpdated(name, EVENT_PROPERTY_CHANGED)
  }, functionDebounceTimer)

  idUpdated = debounce((id: string) => {
    super.idUpdated(this.property.id, id, EVENT_PROPERTY_ID_CHANGED)
  }, functionDebounceTimer)

  removeProperty (): void {
    super.removeProperty(EVENT_PROPERTY_REMOVED)
  }

  updateProperty (property: DynamicProperty | StaticProperty): void {
    super.updateProperty(property, EVENT_PROPERTY_CHANGED)
  }

  editStatProperty (): void {
    super.editProperty(this.property.id, EVENT_PROPERTY_EDITED)
  }

  renderNameplate (removeButton: boolean = true): TemplateResult {
    return html`
      <div class="uk-flex uk-flex-row uk-flex-bottom uk-width-auto">
        <div class="uk-flex uk-flex-column">
          <label class="uk-form-label" for="id-field">ID</label>
          <input id="id-field" class="uk-input static-name-field" .value="${this.property.id}" readonly/>
        </div>

        <div class="uk-flex uk-flex-column name-section">
          <label class="uk-form-label" for="name-field">NAME</label>
          <input id="name-field" class="name-field static-name-field" .value="${this.property.name}" readonly/>
        </div>
        
        ${when(removeButton, () => html`
          <button class="remove-property uk-button uk-button-secondary uk-button-small" @click="${this.editStatProperty}">
            ${icon(faEdit).node}
          </button>
          <button class="remove-property uk-button uk-button-secondary uk-button-small" @click="${this.removeProperty}">
            ${icon(faTrash).node}
          </button>
        `)}
      </div>
    `
  }
}
