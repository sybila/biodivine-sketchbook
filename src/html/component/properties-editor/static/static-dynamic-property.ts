import { type DynamicProperty, type StaticProperty } from '../../../util/data-interfaces'
import { debounce } from 'lodash'
import AbstractProperty from '../abstract-property/abstract-property'

const EVENT_PROPERTY_CHANGED = 'static-property-changed'
const EVENT_PROPERTY_REMOVED = 'dynamic-property-removed'

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
}
