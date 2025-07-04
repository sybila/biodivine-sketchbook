import { css, html, LitElement, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'
import style_less from './properties-editor.less?inline'
import { map } from 'lit/directives/map.js'
import './abstract-property/abstract-property'
import './dynamic/dynamic-attractor-count/dynamic-attractor-count'
import './dynamic/dynamic-generic/dynamic-generic'
import './dynamic/dynamic-obs-selection/dynamic-obs-selection'
import './static/static-generic/static-generic'
import './static/static-essential/static-reg-essential'
import './static/static-essential/static-fn-essential'
import './static/static-essential-condition/static-reg-essential-condition'
import './static/static-essential-condition/static-fn-essential-condition'
import './static/static-monotonic/static-fn-monotonic'
import './static/static-monotonic/static-reg-monotonic'
import './static/static-monotonic-condition/static-fn-monotonic-condition'
import './static/static-monotonic-condition/static-reg-monotonic-condition'
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
import { formatTemplateName, getTemplateHelpText } from '../../util/utilities'

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
  // visibility of automatically generated function properties
  @state() showFunctionProperties = true
  // static prop edit dialogs
  dialogsStatic: Record<string, WebviewWindow | undefined> = {}
  // dynamic prop edit dialogs
  dialogsDynamic: Record<string, WebviewWindow | undefined> = {}

  addDynamicPropertyMenu: DynamicPropertyType[] = [
    DynamicPropertyType.TrapSpace,
    DynamicPropertyType.FixedPoint,
    DynamicPropertyType.ExistsTrajectory,
    DynamicPropertyType.AttractorCount,
    DynamicPropertyType.HasAttractor,
    DynamicPropertyType.Generic
  ]

  addStaticPropertyMenu: StaticPropertyType[] = [
    StaticPropertyType.FunctionInputEssentialWithCondition,
    StaticPropertyType.VariableRegulationEssentialWithCondition,
    StaticPropertyType.FunctionInputMonotonicWithCondition,
    StaticPropertyType.VariableRegulationMonotonicWithCondition,
    StaticPropertyType.Generic
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

    // refresh-event listeners (or listeners to events that update whole property sets)
    aeonState.sketch.properties.staticPropsRefreshed.addEventListener(this.#onStaticRefreshed.bind(this))
    aeonState.sketch.properties.dynamicPropsRefreshed.addEventListener(this.#onDynamicRefreshed.bind(this))
    aeonState.sketch.properties.allStaticUpdated.addEventListener(this.#onStaticRefreshed.bind(this))

    // note that the refresh events are automatically triggered or handled (after app refresh) directly
    // from the root component (due to some dependency issues between different components of the sketch)
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

  toggleFunctionPropertiesVisibility (): void {
    this.showFunctionProperties = !this.showFunctionProperties
  }

  numGeneratedFnProperties (): number {
    const generatedFnProps = this.contentData.staticProperties.filter(
      prop => prop.variant === StaticPropertyType.FunctionInputEssential || prop.variant === StaticPropertyType.FunctionInputMonotonic
    )
    return generatedFnProps.length
  }

  numGeneratedRegProperties (): number {
    const generatedRegProps = this.contentData.staticProperties.filter(
      prop => prop.variant === StaticPropertyType.VariableRegulationEssential || prop.variant === StaticPropertyType.VariableRegulationMonotonic
    )
    return generatedRegProps.length
  }

  render (): TemplateResult {
    return html`
      <div id="dynamic-property-menu" class="menu-content">
        ${when(this.addDynamicMenuVisible,
            () => html`
              <ul class="uk-nav">
                ${map(this.addDynamicPropertyMenu, (item) => html`
                  <li class="menu-item" @click="${() => {
                    this.itemClick(() => { this.addDynamicProperty(item) })
                  }}">
                    <a class="menu-tooltip">
                      ${formatTemplateName(item)}
                      <span class="menu-tooltiptext">${getTemplateHelpText(item)}</span>
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
                    this.itemClick(() => { this.addStaticProperty(item) })
                  }}">
                    <a class="menu-tooltip">
                      ${formatTemplateName(item)}
                      <span class="menu-tooltiptext">${getTemplateHelpText(item)}</span>
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
              ${this.numGeneratedRegProperties() > 0
? html`<div class="uk-margin-small">
                <button class="uk-button uk-button-small uk-button-primary uk-margin-bottom" @click="${this.toggleRegulationPropertiesVisibility}">
                  ${this.showRegulationProperties ? 'Hide' : 'Show'} Generated Regulation Properties
                </button>
              </div>`
: html``}

              ${this.numGeneratedFnProperties() > 0
? html`<div class="uk-margin-small">
                <button class="uk-button uk-button-small uk-button-primary uk-margin-bottom" @click="${this.toggleFunctionPropertiesVisibility}">
                  ${this.showFunctionProperties ? 'Hide' : 'Show'} Generated Function Properties
                </button>
              </div>`
: html``}`}
            <div class="section-list">
              ${map(this.contentData.staticProperties, (prop, index) => {
                let result = html``
                switch (prop.variant) {
                  case StaticPropertyType.Generic:
                    result = html`
                      <static-generic .index=${index}
                                      .property=${prop}
                                      .help=${prop}>
                      </static-generic>`
                    break
                  case StaticPropertyType.FunctionInputEssential:
                    // Only render this if showFunctionProperties is true
                    if (this.showFunctionProperties) {
                      result = html`
                        <static-fn-essential .index=${index}
                                             .property=${prop}>
                        </static-fn-essential>`
                        break
                    } else {
                      return html``
                    }
                  case StaticPropertyType.VariableRegulationEssential:
                    // Only render this if showRegulationProperties is true
                    if (this.showRegulationProperties) {
                      result = html`
                        <static-reg-essential .index=${index}
                                              .property=${prop}>
                        </static-reg-essential>`
                        break
                    } else {
                      return html``
                    }
                  case StaticPropertyType.FunctionInputEssentialWithCondition:
                    result = html`
                      <static-fn-essential-condition .index=${index}
                                                     .contentData=${this.contentData}
                                                     .property=${prop}>
                      </static-fn-essential-condition>`
                    break
                  case StaticPropertyType.VariableRegulationEssentialWithCondition:
                    result = html`
                      <static-reg-essential-condition .index=${index}
                                                      .contentData=${this.contentData}
                                                      .property=${prop}>
                      </static-reg-essential-condition>`
                    break
                  case StaticPropertyType.FunctionInputMonotonic:
                    // Only render this if showFunctionProperties is true
                    if (this.showFunctionProperties) {
                      result = html`
                        <static-fn-monotonic .index=${index}
                                             .property=${prop}>
                        </static-fn-monotonic>`
                        break
                    } else {
                      return html``
                    }
                  case StaticPropertyType.VariableRegulationMonotonic:
                    // Only render this if showRegulationProperties is true
                    if (this.showRegulationProperties) {
                      result = html`
                        <static-reg-monotonic .index=${index}
                                              .property=${prop}>
                        </static-reg-monotonic>`
                        break
                    } else {
                      return html``
                    }
                  case StaticPropertyType.FunctionInputMonotonicWithCondition:
                    result = html`
                      <static-fn-monotonic-condition .index=${index}
                                                     .contentData=${this.contentData}
                                                     .property=${prop}>
                      </static-fn-monotonic-condition>`
                    break
                  case StaticPropertyType.VariableRegulationMonotonicWithCondition:
                    result = html`
                      <static-reg-monotonic-condition .index=${index}
                                                      .contentData=${this.contentData}
                                                      .property=${prop}>
                      </static-reg-monotonic-condition>`
                    break
                }
                return html`${result}<hr class="uk-margin-top uk-margin-bottom uk-margin-left uk-margin-right">`
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
                let result = html``
                switch (prop.variant) {
                  case DynamicPropertyType.FixedPoint:
                  case DynamicPropertyType.HasAttractor:
                  case DynamicPropertyType.TrapSpace:
                  case DynamicPropertyType.ExistsTrajectory:
                    result = html`
                      <dynamic-obs-selection .index=${index}
                                           .property=${prop}
                                           .observations=${this.contentData.observations}>
                      </dynamic-obs-selection>`
                      break
                  case DynamicPropertyType.AttractorCount:
                    result = html`
                      <dynamic-attractor-count .index=${index}
                                               .property=${prop}>
                      </dynamic-attractor-count>`
                      break
                  case DynamicPropertyType.Generic:
                    result = html`
                      <dynamic-generic .index=${index}
                                       .property=${prop}>
                      </dynamic-generic>`
                      break
                }
                return html`${result}<hr class="uk-margin-top uk-margin-bottom uk-margin-left uk-margin-right">`
              })}
            </div>
          </div>
        </div>
      </div>`
  }
}
