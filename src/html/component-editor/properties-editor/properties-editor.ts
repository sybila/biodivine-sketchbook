import { css, html, LitElement, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'
import style_less from './properties-editor.less?inline'
import { map } from 'lit/directives/map.js'
import './abstract-property/abstract-property'
import './dynamic/dynamic-attractor-count/dynamic-attractor-count'
import './dynamic/dynamic-generic/dynamic-generic'
import './dynamic/dynamic-obs-selection/dynamic-obs-selection'
import './static/static-generic/static-generic'
import './static/static-input-essential/static-input-essential'
import './static/static-input-essential-condition/static-input-essential-condition'
import './static/static-input-monotonic/static-input-monotonic'
import './static/static-input-monotonic-condition/static-input-monotonic-condition'
import {
  ContentData,
  type DynamicProperty,
  DynamicPropertyType,
  type StaticProperty,
  StaticPropertyType
} from '../../util/data-interfaces'
import { when } from 'lit/directives/when.js'
import { computePosition, flip } from '@floating-ui/dom'
import { aeonState, type DynPropIdUpdateData, type StatPropIdUpdateData } from '../../../aeon_state'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/helpers/event'

/** Component responsible for the properties editor of the editor session. */
@customElement('properties-editor')
export default class PropertiesEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  @query('#dynamic-property-menu') declare dynamicPropertyMenuElement: HTMLElement
  @query('#add-dynamic-property-button') declare addDynamicPropertyElement: HTMLElement
  @query('#static-property-menu') declare staticPropertyMenuElement: HTMLElement
  @query('#add-static-property-button') declare addStaticPropertyElement: HTMLElement
  @state() addDynamicMenuVisible = false
  @state() addStaticMenuVisible = false
  // visibility of automatically generated regulation properties
  @state() showRegulationProperties = true
  // static prop edit dialogs
  dialogsStatic: Record<string, WebviewWindow | undefined> = {}
  // dynamic prop edit dialogs
  dialogsDynamic: Record<string, WebviewWindow | undefined> = {}

  addDynamicPropertyMenu: IAddPropertyItem[] = [
    {
      label: 'Exists trap space',
      help: 'Each selected observation exists in a trap space.',
      action: () => { this.addDynamicProperty(DynamicPropertyType.TrapSpace) }
    }, {
      label: 'Exists fixed point',
      help: 'Each selected observation exists in a fixed point.',
      action: () => { this.addDynamicProperty(DynamicPropertyType.FixedPoint) }
    }, {
      label: 'Exists trajectory',
      help: 'Observations of selected dataset lay on a trajectory.',
      action: () => { this.addDynamicProperty(DynamicPropertyType.ExistsTrajectory) }
    }, {
      label: 'Attractor count',
      help: 'Attractor count falls into a given range.',
      action: () => { this.addDynamicProperty(DynamicPropertyType.AttractorCount) }
    }, {
      label: 'Exists attractor',
      help: 'Each selected observation exists in an attractor.',
      action: () => { this.addDynamicProperty(DynamicPropertyType.HasAttractor) }
    }, {
      label: 'Generic',
      help: 'Generic HCTL property defined by the user.',
      action: () => { this.addDynamicProperty(DynamicPropertyType.Generic) }
    }
  ]

  addStaticPropertyMenu: IAddPropertyItem[] = [
    {
      label: 'Function input essential',
      help: 'Selected function input has given essentiality.',
      action: () => { this.addStaticProperty(StaticPropertyType.FunctionInputEssentialWithCondition) }
    }, {
      label: 'Regulation essential',
      help: 'Selected regulation has given essentiality.',
      action: () => { this.addStaticProperty(StaticPropertyType.VariableRegulationEssentialWithCondition) }
    }, {
      label: 'Function input monotonic',
      help: 'Selected function input has given monotonicity.',
      action: () => { this.addStaticProperty(StaticPropertyType.FunctionInputMonotonicWithCondition) }
    }, {
      label: 'Regulation monotonic',
      help: 'Selected regulation has given monotonicity.',
      action: () => { this.addStaticProperty(StaticPropertyType.VariableRegulationMonotonicWithCondition) }
    }, {
      label: 'Generic',
      help: 'Generic FOL property defined by the user.',
      action: () => { this.addStaticProperty(StaticPropertyType.Generic) }
    }
  ]

  constructor () {
    super()

    document.addEventListener('click', this.closeMenu.bind(this))

    // classical events
    aeonState.sketch.properties.dynamicCreated.addEventListener(this.#onDynamicCreated.bind(this))
    aeonState.sketch.properties.staticCreated.addEventListener(this.#onStaticCreated.bind(this))
    this.addEventListener('dynamic-property-removed', this.removeDynamicProperty)
    aeonState.sketch.properties.dynamicRemoved.addEventListener(this.#onDynamicRemoved.bind(this))
    this.addEventListener('static-property-removed', this.removeStaticProperty)
    aeonState.sketch.properties.staticRemoved.addEventListener(this.#onStaticRemoved.bind(this))
    this.addEventListener('dynamic-property-changed', this.changeDynamicProperty)
    aeonState.sketch.properties.dynamicContentChanged.addEventListener(this.#onDynamicChanged.bind(this))
    this.addEventListener('static-property-changed', this.changeStaticProperty)
    aeonState.sketch.properties.staticContentChanged.addEventListener(this.#onStaticChanged.bind(this))
    this.addEventListener('dynamic-property-id-changed', this.changeDynamicPropertyId)
    aeonState.sketch.properties.dynamicIdChanged.addEventListener(this.#onDynamicIdChanged.bind(this))
    this.addEventListener('static-property-id-changed', this.changeStaticPropertyId)
    aeonState.sketch.properties.staticIdChanged.addEventListener(this.#onStaticIdChanged.bind(this))
    this.addEventListener('dynamic-property-edited', (e) => { void this.editDynProperty(e) })
    this.addEventListener('static-property-edited', (e) => { void this.editStatProperty(e) })

    // refresh-event listeners (or listeners to events that update the whole list of props)
    aeonState.sketch.properties.staticPropsRefreshed.addEventListener(this.#onStaticRefreshed.bind(this))
    aeonState.sketch.properties.dynamicPropsRefreshed.addEventListener(this.#onDynamicRefreshed.bind(this))
    aeonState.sketch.properties.allStaticUpdated.addEventListener(this.#onStaticRefreshed.bind(this))

    // note that the refresh events are automatically triggered or handled (after app refresh) directly
    // from the root component (due to some dependency issues between different components)
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)
  }

  updateDynamicProperties (dynamicProperties: DynamicProperty[]): void {
    this.dispatchEvent(new CustomEvent('save-dynamic-properties', {
      detail: {
        dynamicProperties
      },
      bubbles: true,
      composed: true
    }))
  }

  updateStaticProperties (staticProperties: StaticProperty[]): void {
    this.dispatchEvent(new CustomEvent('save-static-properties', {
      detail: {
        staticProperties
      },
      bubbles: true,
      composed: true
    }))
  }

  private async editStatProperty (event: Event): Promise<void> {
    const detail = (event as CustomEvent).detail
    const propertyIndex = this.contentData.staticProperties.findIndex(p => p.id === detail.id)
    if (propertyIndex === -1) return
    const property = this.contentData.staticProperties[propertyIndex]

    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    if (this.dialogsStatic[property.id] !== undefined) {
      await this.dialogsStatic[property.id]?.setFocus()
      return
    }
    const editStaticDialog = new WebviewWindow(`editProperty${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component-editor/properties-editor/edit-property/edit-property.html',
      title: `Edit property (${property.id} / ${property.name})`,
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      height: 500,
      width: 400,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    this.dialogsStatic[property.id] = editStaticDialog
    void editStaticDialog.once('loaded', () => {
      void editStaticDialog.emit('edit_property_update', {
        ...property
      })
    })
    void editStaticDialog.once('edit_property_dialog', (event: TauriEvent<{ id: string, property: StaticProperty }>) => {
      this.dialogsStatic[property.id] = undefined
      const index = this.contentData.staticProperties.findIndex(p => p.id === property.id)
      if (index === -1) return
      this.finishEditStatic(property.id, event.payload.property)
    })
    void editStaticDialog.onCloseRequested(() => {
      this.dialogsStatic[property.id] = undefined
    })
  }

  private finishEditStatic (id: string, updatedProp: StaticProperty): void {
    const origProp = this.contentData.staticProperties.find(p => p.id === id)
    if (origProp === undefined) return

    // id might have changed
    if (origProp.id !== updatedProp.id) {
      aeonState.sketch.properties.setStaticId(origProp.id, updatedProp.id)
    }
    // name or annotation might have changed
    setTimeout(() => {
      aeonState.sketch.properties.setStaticContent(updatedProp.id, updatedProp)
    }, 50)
  }

  private async editDynProperty (event: Event): Promise<void> {
    const detail = (event as CustomEvent).detail
    const propertyIndex = this.contentData.dynamicProperties.findIndex(p => p.id === detail.id)
    if (propertyIndex === -1) return
    const property = this.contentData.dynamicProperties[propertyIndex]

    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    if (this.dialogsDynamic[property.id] !== undefined) {
      await this.dialogsDynamic[property.id]?.setFocus()
      return
    }
    const editDynamicDialog = new WebviewWindow(`editProperty${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component-editor/properties-editor/edit-property/edit-property.html',
      title: `Edit property (${property.id} / ${property.name})`,
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      height: 500,
      width: 400,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    this.dialogsDynamic[property.id] = editDynamicDialog
    void editDynamicDialog.once('loaded', () => {
      void editDynamicDialog.emit('edit_property_update', {
        ...property
      })
    })
    void editDynamicDialog.once('edit_property_dialog', (event: TauriEvent<{ id: string, property: DynamicProperty }>) => {
      this.dialogsDynamic[property.id] = undefined
      const index = this.contentData.dynamicProperties.findIndex(p => p.id === property.id)
      if (index === -1) return
      this.finishEditDynamic(property.id, event.payload.property)
    })
    void editDynamicDialog.onCloseRequested(() => {
      this.dialogsDynamic[property.id] = undefined
    })
  }

  private finishEditDynamic (id: string, updatedProp: DynamicProperty): void {
    const origProp = this.contentData.dynamicProperties.find(p => p.id === id)
    if (origProp === undefined) return

    // id might have changed
    if (origProp.id !== updatedProp.id) {
      aeonState.sketch.properties.setDynamicId(origProp.id, updatedProp.id)
    }
    // name or annotation might have changed
    setTimeout(() => {
      aeonState.sketch.properties.setDynamicContent(updatedProp.id, updatedProp)
    }, 50)
  }

  #onDynamicRefreshed (refreshedDynamic: DynamicProperty[]): void {
    this.updateDynamicProperties(refreshedDynamic)
    console.log('Refreshed ' + refreshedDynamic.length + ' properties.')
  }

  #onStaticRefreshed (refreshedStatic: StaticProperty[]): void {
    this.updateStaticProperties(refreshedStatic)
    console.log('Refreshed ' + refreshedStatic.length + ' properties.')
  }

  addDynamicProperty (type: DynamicPropertyType): void {
    aeonState.sketch.properties.addDefaultDynamic(type)
  }

  #onDynamicCreated (newDynamic: DynamicProperty): void {
    this.contentData.dynamicProperties.push(newDynamic)
    this.updateDynamicProperties(this.contentData.dynamicProperties)
    console.log('Created: ' + newDynamic.id)
  }

  addStaticProperty (type: StaticPropertyType): void {
    aeonState.sketch.properties.addDefaultStatic(type)
  }

  #onStaticCreated (newStatic: StaticProperty): void {
    this.contentData.staticProperties.push(newStatic)
    this.updateStaticProperties(this.contentData.staticProperties)
    console.log('Created: ' + newStatic.id)
  }

  changeDynamicProperty (event: Event): void {
    const detail = (event as CustomEvent).detail
    aeonState.sketch.properties.setDynamicContent(detail.property.id, detail.property)
  }

  #onDynamicChanged (changedProp: DynamicProperty): void {
    const id = changedProp.id
    const index = this.contentData.dynamicProperties.findIndex(prop => prop.id === id)
    if (index === -1) return
    const properties = [...this.contentData.dynamicProperties]
    properties[index] = changedProp
    console.log('Changed: ' + changedProp.id)
    this.updateDynamicProperties(properties)
  }

  changeStaticProperty (event: Event): void {
    const detail = (event as CustomEvent).detail
    aeonState.sketch.properties.setStaticContent(detail.property.id, detail.property)
  }

  #onStaticChanged (changedProp: StaticProperty): void {
    const id = changedProp.id
    const index = this.contentData.staticProperties.findIndex(prop => prop.id === id)
    if (index === -1) return
    const properties = [...this.contentData.staticProperties]
    properties[index] = changedProp
    console.log('Changed: ' + changedProp.id)
    this.updateStaticProperties(properties)
  }

  removeDynamicProperty (event: Event): void {
    const id = (event as CustomEvent).detail.id
    aeonState.sketch.properties.removeDynamic(id)
  }

  #onDynamicRemoved (removedProp: DynamicProperty): void {
    const id = removedProp.id
    const index = this.contentData.dynamicProperties.findIndex(prop => prop.id === id)
    if (index === -1) return
    const properties = [...this.contentData.dynamicProperties]
    console.log('Removed: ' + removedProp.id)
    this.updateDynamicProperties(properties.filter((p) => p.id !== removedProp.id))
  }

  removeStaticProperty (event: Event): void {
    const id = (event as CustomEvent).detail.id
    aeonState.sketch.properties.removeStatic(id)
  }

  #onStaticRemoved (removedProp: StaticProperty): void {
    const id = removedProp.id
    const index = this.contentData.staticProperties.findIndex(prop => prop.id === id)
    if (index === -1) return
    const properties = [...this.contentData.staticProperties]
    console.log('Removed: ' + removedProp.id)
    this.updateStaticProperties(properties.filter((p) => p.id !== removedProp.id))
  }

  changeDynamicPropertyId (event: Event): void {
    const detail = (event as CustomEvent).detail
    aeonState.sketch.properties.setDynamicId(detail.oldId, detail.newId)
  }

  #onDynamicIdChanged (data: DynPropIdUpdateData): void {
    const index = this.contentData.dynamicProperties.findIndex(d => d.id === data.original_id)
    if (index === -1) return
    const properties = [...this.contentData.dynamicProperties]
    properties[index] = {
      ...properties[index],
      id: data.new_id
    }
    this.updateDynamicProperties(properties)
  }

  changeStaticPropertyId (event: Event): void {
    const detail = (event as CustomEvent).detail
    aeonState.sketch.properties.setStaticId(detail.oldId, detail.newId)
  }

  #onStaticIdChanged (data: StatPropIdUpdateData): void {
    const index = this.contentData.staticProperties.findIndex(d => d.id === data.original_id)
    if (index === -1) return
    const properties = [...this.contentData.staticProperties]
    properties[index] = {
      ...properties[index],
      id: data.new_id
    }
    this.updateStaticProperties(properties)
  }

  async openAddDynamicPropertyMenu (): Promise<void> {
    this.addDynamicMenuVisible = true
    void computePosition(
      this.addDynamicPropertyElement,
      this.dynamicPropertyMenuElement,
      { middleware: [flip()], placement: 'bottom-end' }
    ).then(({ x, y }) => {
      this.dynamicPropertyMenuElement.style.left = x + 'px'
      this.dynamicPropertyMenuElement.style.top = y + 'px'
    })
  }

  async openAddStaticPropertyMenu (): Promise<void> {
    this.addStaticMenuVisible = true
    void computePosition(
      this.addStaticPropertyElement,
      this.staticPropertyMenuElement,
      { middleware: [flip()], placement: 'bottom-end' }
    ).then(({ x, y }) => {
      this.staticPropertyMenuElement.style.left = x + 'px'
      this.staticPropertyMenuElement.style.top = y + 'px'
    })
  }

  itemClick (action: () => void): void {
    this.addDynamicMenuVisible = false
    action()
  }

  closeMenu (event: Event): void {
    if (!(event.composedPath()[0] as HTMLElement).matches('.add-dynamic-property')) {
      this.addDynamicMenuVisible = false
    }
    if (!(event.composedPath()[0] as HTMLElement).matches('.add-static-property')) {
      this.addStaticMenuVisible = false
    }
  }

  toggleRegulationPropertiesVisibility (): void {
    this.showRegulationProperties = !this.showRegulationProperties
  }

  render (): TemplateResult {
    return html`
      <div id="dynamic-property-menu" class="menu-content">
        ${when(this.addDynamicMenuVisible,
            () => html`
              <ul class="uk-nav">
                ${map(this.addDynamicPropertyMenu, (item) => html`
                  <li class="menu-item" @click="${() => {
                    this.itemClick(item.action)
                  }}">
                    <a class="tooltip">
                      ${item.label}
                      <span class="tooltiptext">${item.help}</span>
                    </a>
                  </li>
                `)}
              </ul>`)}
      </div>
      <div id="static-property-menu" class="menu-content">
        ${when(this.addStaticMenuVisible,
            () => html`
              <ul class="uk-nav">
                ${map(this.addStaticPropertyMenu, (item) => html`
                  <li class="menu-item" @click="${() => {
                    this.itemClick(item.action)
                  }}">
                    <a class="tooltip">
                      ${item.label}
                      <span class="tooltiptext">${item.help}</span>
                    </a>
                  </li>
                `)}
              </ul>`)}
      </div>
      <div class="container">
        <div class="property-list">
          <div class="section" id="functions">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom">Static</h3>
              <button id="add-static-property-button" class="add-property add-static-property uk-button uk-button-small uk-button-primary"
                      @click="${this.openAddStaticPropertyMenu}">
                      + Add
              </button>
            </div>

            ${this.contentData?.staticProperties.length === 0
? html`
              <div class="uk-text-center uk-margin-bottom">
                  <span class="uk-label">No static properties defined</span>
              </div>`
: html`
              <div class="uk-margin-small">
                <button class="uk-button uk-button-small uk-button-primary uk-margin-bottom" @click="${this.toggleRegulationPropertiesVisibility}">
                  ${this.showRegulationProperties ? 'Hide' : 'Show'} Generated Regulation Properties
                </button>
              </div>`}
            <div class="section-list">
              ${map(this.contentData.staticProperties, (prop, index) => {
                switch (prop.variant) {
                  case StaticPropertyType.Generic:
                    return html`
                      <static-generic .index=${index}
                                      .property=${prop}>
                      </static-generic>`
                  case StaticPropertyType.FunctionInputEssential:
                    return html`
                      <static-input-essential .index=${index}
                                              .property=${prop}>
                      </static-input-essential>`
                  case StaticPropertyType.VariableRegulationEssential:
                    // Only render if showRegulationProperties is true
                    return this.showRegulationProperties
? html`
                      <static-input-essential .index=${index}
                                              .property=${prop}>
                      </static-input-essential>`
: ''
                  case StaticPropertyType.FunctionInputEssentialWithCondition:
                  case StaticPropertyType.VariableRegulationEssentialWithCondition:
                    return html`
                      <static-input-essential-condition .index=${index}
                                                        .contentData=${this.contentData}
                                                        .property=${prop}>
                      </static-input-essential-condition>`
                  case StaticPropertyType.FunctionInputMonotonic:
                    return html`
                      <static-input-monotonic .index=${index}
                                              .property=${prop}>
                      </static-input-monotonic>`
                  case StaticPropertyType.VariableRegulationMonotonic:
                    // Only render if showRegulationProperties is true
                    return this.showRegulationProperties
? html`
                      <static-input-monotonic .index=${index}
                                              .property=${prop}>
                      </static-input-monotonic>`
: ''
                  case StaticPropertyType.FunctionInputMonotonicWithCondition:
                  case StaticPropertyType.VariableRegulationMonotonicWithCondition:
                    return html`
                      <static-input-monotonic-condition .index=${index}
                                                        .contentData=${this.contentData}
                                                        .property=${prop}>
                      </static-input-monotonic-condition>`
                  default:
                    return ''
                }
              })}
            </div>
          </div>
          <div class="section" id="variables">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Dynamic</h3>
              <button id="add-dynamic-property-button" class="add-property add-dynamic-property uk-button uk-button-small uk-button-primary"
                      @click="${this.openAddDynamicPropertyMenu}">
                + Add
              </button>
            </div>
            ${this.contentData?.dynamicProperties.length === 0 ? html`<div class="uk-text-center uk-margin-bottom"><span class="uk-label">No dynamic properties defined</span></div>` : ''}
            <div class="section-list">
              ${map(this.contentData.dynamicProperties, (prop, index) => {
                switch (prop.variant) {
                  case DynamicPropertyType.FixedPoint:
                  case DynamicPropertyType.HasAttractor:
                  case DynamicPropertyType.TrapSpace:
                  case DynamicPropertyType.ExistsTrajectory:
                    return html`
                      <dynamic-obs-selection .index=${index}
                                           .property=${prop}
                                           .observations=${this.contentData.observations}>
                      </dynamic-obs-selection>`
                  case DynamicPropertyType.AttractorCount:
                    return html`
                      <dynamic-attractor-count .index=${index}
                                               .property=${prop}>
                      </dynamic-attractor-count>`
                  case DynamicPropertyType.Generic:
                    return html`
                      <dynamic-generic .index=${index}
                                       .property=${prop}>
                      </dynamic-generic>`
                  default:
                    return ''
                }
              })}
            </div>
          </div>
        </div>
      </div>`
  }
}

/**
 * Template for options in the 'Add property' menu.
 * Each entry has a displayed name, a hoverable tooltip help, and action.
 */
interface IAddPropertyItem {
  label: string
  help: string
  action: () => void
}
