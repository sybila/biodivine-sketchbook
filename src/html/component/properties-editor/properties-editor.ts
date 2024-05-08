import { css, html, LitElement, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'
import style_less from './properties-editor.less?inline'
import { map } from 'lit/directives/map.js'
import './abstract-property/abstract-property'
import './dynamic/dynamic-attractor-count/dynamic-attractor-count'
import './dynamic/dynamic-fixed-point/dynamic-fixed-point'
import './dynamic/dynamic-generic/dynamic-generic'
import './dynamic/dynamic-has-attractor/dynamic-has-attractor'
import './dynamic/dynamic-trajectory/dynamic-trajectory'
import './dynamic/dynamic-trap-space/dynamic-trap-space'
import './static/static-generic/static-generic'
import './static/static-input-essential/static-input-essential'
import './static/static-input-essential-condition/static-input-essential-condition'
import './static/static-input-monotonic/static-input-monotonic'
import './static/static-input-monotonic-condition/static-input-monotonic-condition'
import {
  ContentData,
  type DynamicProperty,
  DynamicPropertyType,
  type PropertyType,
  type StaticProperty,
  StaticPropertyType
} from '../../util/data-interfaces'
import {
  attractorCountDynamic,
  existsTrajectoryDynamic,
  fixedPointDynamic,
  functionInputEssentialWithCondition,
  functionInputMonotonicWithCondition,
  genericDynamic,
  genericStatic,
  hasAttractorDynamic,
  trapSpaceDynamic, variableRegulationEssentialWithCondition, variableRegulationMonotonicWithCondition
} from './default-properties'
import { when } from 'lit/directives/when.js'
import { computePosition, flip } from '@floating-ui/dom'
import UIkit from 'uikit'
import { icon } from '@fortawesome/fontawesome-svg-core'
import { faAngleDown } from '@fortawesome/free-solid-svg-icons'
import { aeonState } from '../../../aeon_events'

@customElement('properties-editor')
export default class PropertiesEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  @query('#dynamic-property-menu') declare dynamicPropertyMenuElement: HTMLElement
  @query('#add-dynamic-property-button') declare addDynamicPropertyElement: HTMLElement
  @query('#static-property-menu') declare staticPropertyMenuElement: HTMLElement
  @query('#add-static-property-button') declare addStaticPropertyElement: HTMLElement
  @state() staticProperties: StaticProperty[] = []
  @state() dynamicProperties: DynamicProperty[] = []
  @state() addDynamicMenuVisible = false
  @state() addStaticMenuVisible = false
  propIndex = 0

  addDynamicPropertyMenu: IAddPropertyItem[] = [
    {
      label: 'Trap space',
      action: () => { this.addDynamicProperty(DynamicPropertyType.TrapSpace) }
    }, {
      label: 'Fixed point',
      action: () => { this.addDynamicProperty(DynamicPropertyType.FixedPoint) }
    }, {
      label: 'Exists trajectory',
      action: () => { this.addDynamicProperty(DynamicPropertyType.ExistsTrajectory) }
    }, {
      label: 'Attractor count',
      action: () => { this.addDynamicProperty(DynamicPropertyType.AttractorCount) }
    }, {
      label: 'Has attractor',
      action: () => { this.addDynamicProperty(DynamicPropertyType.HasAttractor) }
    }, {
      label: 'Generic',
      action: () => { this.addDynamicProperty(DynamicPropertyType.Generic) }
    }
  ]

  addStaticPropertyMenu: IAddPropertyItem[] = [
    {
      label: 'Essential function input',
      action: () => { this.addStaticProperty(StaticPropertyType.FunctionInputEssentialWithCondition) }
    }, {
      label: 'Essential variable regulation',
      action: () => { this.addStaticProperty(StaticPropertyType.VariableRegulationEssentialWithCondition) }
    }, {
      label: 'Monotonic function input',
      action: () => { this.addStaticProperty(StaticPropertyType.FunctionInputMonotonicWithCondition) }
    }, {
      label: 'Monotonic variable regulation',
      action: () => { this.addStaticProperty(StaticPropertyType.VariableRegulationMonotonicWithCondition) }
    }, {
      label: 'Generic',
      action: () => { this.addStaticProperty(StaticPropertyType.Generic) }
    }
  ]

  constructor () {
    super()

    this.addEventListener('static-property-changed', this.staticPropertyChanged)
    this.addEventListener('dynamic-property-changed', this.dynamicPropertyChanged)
    this.addEventListener('static-property-removed', this.staticPropertyRemoved)
    this.addEventListener('dynamic-property-removed', this.dynamicPropertyRemoved)

    document.addEventListener('click', this.closeMenu.bind(this))

    // refresh-event listeners
    aeonState.sketch.properties.staticPropsRefreshed.addEventListener(this.#onStaticRefreshed.bind(this))
    aeonState.sketch.properties.dynamicPropsRefreshed.addEventListener(this.#onDynamicRefreshed.bind(this))

    // refreshing content from backend - placeholders
    aeonState.sketch.properties.refreshDynamicProps()
    aeonState.sketch.properties.refreshStaticProps()
  }

  #onDynamicRefreshed (refreshedDynamic: DynamicProperty[]): void {
    this.dynamicProperties = refreshedDynamic
    console.log(refreshedDynamic)
  }

  #onStaticRefreshed (refreshedStatic: StaticProperty[]): void {
    this.staticProperties = refreshedStatic
    console.log(refreshedStatic)
  }

  protected firstUpdated (_changedProperties: PropertyValues): void {
    super.firstUpdated(_changedProperties)
    UIkit.sticky(this.shadowRoot?.querySelector('.header') as HTMLElement)
  }

  addDynamicProperty (type: PropertyType): void {
    const id = '' + this.propIndex++
    switch (type) {
      case DynamicPropertyType.Generic:
        this.dynamicProperties.push(genericDynamic(id))
        break
      case DynamicPropertyType.FixedPoint:
        this.dynamicProperties.push(fixedPointDynamic(id))
        break
      case DynamicPropertyType.TrapSpace:
        this.dynamicProperties.push(trapSpaceDynamic(id))
        break
      case DynamicPropertyType.ExistsTrajectory:
        this.dynamicProperties.push(existsTrajectoryDynamic(id))
        break
      case DynamicPropertyType.AttractorCount:
        this.dynamicProperties.push(attractorCountDynamic(id))
        break
      case DynamicPropertyType.HasAttractor:
        this.dynamicProperties.push(hasAttractorDynamic(id))
        break
    }
  }

  addStaticProperty (type: StaticPropertyType): void {
    const id = '' + this.propIndex++
    switch (type) {
      case StaticPropertyType.FunctionInputMonotonicWithCondition:
        this.staticProperties.push(functionInputMonotonicWithCondition(id))
        break
      case StaticPropertyType.FunctionInputEssentialWithCondition:
        this.staticProperties.push(functionInputEssentialWithCondition(id))
        break
      case StaticPropertyType.VariableRegulationEssentialWithCondition:
        this.staticProperties.push(variableRegulationEssentialWithCondition(id))
        break
      case StaticPropertyType.VariableRegulationMonotonicWithCondition:
        this.staticProperties.push(variableRegulationMonotonicWithCondition(id))
        break
      case StaticPropertyType.Generic:
        this.staticProperties.push(genericStatic(id))
    }
  }

  dynamicPropertyChanged (event: Event): void {
    const detail = (event as CustomEvent).detail
    console.log('property changed', detail.property)
    const props = [...this.dynamicProperties]
    props[detail.index] = detail.property
    this.dynamicProperties = props
  }

  staticPropertyChanged (event: Event): void {
    const detail = (event as CustomEvent).detail
    console.log('property changed', detail.property)
    const props = [...this.staticProperties]
    props[detail.index] = detail.property
    this.staticProperties = props
  }

  dynamicPropertyRemoved (event: Event): void {
    const detail = (event as CustomEvent).detail
    console.log('property removed', detail.index)
    const props = [...this.dynamicProperties]
    props.splice(detail.index, 1)
    this.dynamicProperties = props
  }

  staticPropertyRemoved (event: Event): void {
    const detail = (event as CustomEvent).detail
    console.log('property removed', detail.index)
    const props = [...this.staticProperties]
    props.splice(detail.index, 1)
    this.staticProperties = props
  }

  async openAddDynamicPropertyMenu (): Promise<void> {
    this.addDynamicMenuVisible = true
    void computePosition(this.addDynamicPropertyElement, this.dynamicPropertyMenuElement,
      {
        middleware: [flip()],
        placement: 'bottom-end'
      })
      .then(({ x, y }) => {
        this.dynamicPropertyMenuElement.style.left = x + 'px'
        this.dynamicPropertyMenuElement.style.top = y + 'px'
      })
  }

  async openAddStaticPropertyMenu (): Promise<void> {
    this.addStaticMenuVisible = true
    void computePosition(this.addStaticPropertyElement, this.staticPropertyMenuElement,
      {
        middleware: [flip()],
        placement: 'bottom-end'
      })
      .then(({ x, y }) => {
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
                    <a>
                      ${item.label}
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
                    <a>
                      ${item.label}
                    </a>
                  </li>
                `)}
              </ul>`)}
      </div>
      <div class="container">
        <div class="property-list">
          <div class="section" id="functions">
            <div class="header">
              <div></div>
              <h2 class="heading">Static</h2>
              <button id="add-static-property-button" class="add-property add-static-property"
                      @click="${this.openAddStaticPropertyMenu}">
                Add ${icon(faAngleDown).node}
              </button>
            </div>
            <div class="section-list">
              ${map(this.staticProperties, (prop, index) => {
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
            <div class="header">
              <div></div>
              <h2 class="heading">Dynamic</h2>
              <button id="add-dynamic-property-button" class="add-property add-dynamic-property"
                      @click="${this.openAddDynamicPropertyMenu}">
                Add ${icon(faAngleDown).node}
              </button>
            </div>
            <div class="section-list">
              ${map(this.dynamicProperties, (prop, index) => {
                switch (prop.variant) {
                  case DynamicPropertyType.FixedPoint:
                    return html`
                      <dynamic-fixed-point .index=${index}
                                           .property=${prop}
                                           .observations=${this.contentData.observations}>
                      </dynamic-fixed-point>`
                  case DynamicPropertyType.TrapSpace:
                    return html`
                      <dynamic-trap-space .index=${index}
                                          .property=${prop}
                                          .observations=${this.contentData.observations}>
                      </dynamic-trap-space>`
                  case DynamicPropertyType.ExistsTrajectory:
                    return html`
                      <dynamic-trajectory .index=${index}
                                          .property=${prop}
                                          .observations=${this.contentData.observations}>
                      </dynamic-trajectory>`
                  case DynamicPropertyType.AttractorCount:
                    return html`
                      <dynamic-attractor-count .index=${index}
                                               .property=${prop}>
                      </dynamic-attractor-count>`
                  case DynamicPropertyType.HasAttractor:
                    return html`
                      <dynamic-has-attractor .index=${index}
                                             .property=${prop}
                                             .observations=${this.contentData.observations}>
                      </dynamic-has-attractor>`
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

interface IAddPropertyItem {
  label: string
  action: () => void
}
