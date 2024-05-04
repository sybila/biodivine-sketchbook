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

@customElement('properties-editor')
export default class PropertiesEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  @query('#dynamic-property-menu') declare dynamicPropertyMenuElement: HTMLElement
  @query('#add-dynamic-property-button') declare addDynamicPropertyElement: HTMLElement
  @query('#static-property-menu') declare staticPropertyMenuElement: HTMLElement
  @query('#add-static-property-button') declare addStaticPropertyElement: HTMLElement
  @state() properties: Array<DynamicProperty | StaticProperty> = []
  @state() addDynamicMenuVisible = false
  @state() addStaticMenuVisible = false

  addDynamicPropertyMenu: IAddPropertyItem[] = [
    {
      label: 'Trap space',
      action: () => { this.addProperty(DynamicPropertyType.TrapSpace) }
    }, {
      label: 'Fixed point',
      action: () => { this.addProperty(DynamicPropertyType.FixedPoint) }
    }, {
      label: 'Exists trajectory',
      action: () => { this.addProperty(DynamicPropertyType.ExistsTrajectory) }
    }, {
      label: 'Attractor count',
      action: () => { this.addProperty(DynamicPropertyType.AttractorCount) }
    }, {
      label: 'Has attractor',
      action: () => { this.addProperty(DynamicPropertyType.HasAttractor) }
    }, {
      label: 'Generic',
      action: () => { this.addProperty(DynamicPropertyType.Generic) }
    }
  ]

  addStaticPropertyMenu: IAddPropertyItem[] = [
    {
      label: 'Essential function input',
      action: () => { this.addProperty(StaticPropertyType.FunctionInputEssentialWithCondition) }
    }, {
      label: 'Essential variable regulation',
      action: () => { this.addProperty(StaticPropertyType.VariableRegulationEssentialWithCondition) }
    }, {
      label: 'Monotonic function input',
      action: () => { this.addProperty(StaticPropertyType.FunctionInputMonotonicWithCondition) }
    }, {
      label: 'Monotonic variable regulation',
      action: () => { this.addProperty(StaticPropertyType.VariableRegulationMonotonicWithCondition) }
    }, {
      label: 'Generic',
      action: () => { this.addProperty(StaticPropertyType.Generic) }
    }
  ]

  constructor () {
    super()

    this.addEventListener('property-changed', this.propertyChanged)
    this.addEventListener('property-removed', this.propertyRemoved)

    document.addEventListener('click', this.closeMenu.bind(this))
    // seed dummy data
    this.addProperty(StaticPropertyType.FunctionInputMonotonicWithCondition)
    this.addProperty(StaticPropertyType.VariableRegulationMonotonicWithCondition)
    this.addProperty(StaticPropertyType.FunctionInputEssentialWithCondition)
    this.addProperty(StaticPropertyType.VariableRegulationEssentialWithCondition)
    console.log(this.properties)
    // this.properties.push(functionInputEssential('a'))
    // this.properties.push(functionInputMonotonic('b'))
  }

  protected firstUpdated (_changedProperties: PropertyValues): void {
    super.firstUpdated(_changedProperties)
    UIkit.sticky(this.shadowRoot?.querySelector('.header') as HTMLElement)
  }

  addProperty (type: PropertyType): void {
    const id = '' + this.properties.length
    switch (type) {
      case DynamicPropertyType.Generic:
        this.properties.push(genericDynamic(id))
        break
      case DynamicPropertyType.FixedPoint:
        this.properties.push(fixedPointDynamic(id))
        break
      case DynamicPropertyType.TrapSpace:
        this.properties.push(trapSpaceDynamic(id))
        break
      case DynamicPropertyType.ExistsTrajectory:
        this.properties.push(existsTrajectoryDynamic(id))
        break
      case DynamicPropertyType.AttractorCount:
        this.properties.push(attractorCountDynamic(id))
        break
      case DynamicPropertyType.HasAttractor:
        this.properties.push(hasAttractorDynamic(id))
        break
      case StaticPropertyType.FunctionInputMonotonicWithCondition:
        this.properties.push(functionInputMonotonicWithCondition(id))
        break
      case StaticPropertyType.FunctionInputEssentialWithCondition:
        this.properties.push(functionInputEssentialWithCondition(id))
        break
      case StaticPropertyType.VariableRegulationEssentialWithCondition:
        this.properties.push(variableRegulationEssentialWithCondition(id))
        break
      case StaticPropertyType.VariableRegulationMonotonicWithCondition:
        this.properties.push(variableRegulationMonotonicWithCondition(id))
        break
      case StaticPropertyType.Generic:
        this.properties.push(genericStatic(id))
    }
  }

  propertyChanged (event: Event): void {
    const detail = (event as CustomEvent).detail
    console.log('property changed', detail.property)
    const props = [...this.properties]
    props[detail.index] = detail.property
    this.properties = props
  }

  propertyRemoved (event: Event): void {
    const detail = (event as CustomEvent).detail
    console.log('property removed', detail.index)
    const props = [...this.properties]
    props.splice(detail.index, 1)
    this.properties = props
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
              ${map(this.properties, (prop, index) => {
                switch (prop.type) {
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
                  case StaticPropertyType.FunctionInputMonotonicWithCondition:
                    return html`
                      <static-input-monotonic .index=${index}
                                              .property=${prop}>
                      </static-input-monotonic>`
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
              ${map(this.properties, (prop, index) => {
                switch (prop.type) {
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
