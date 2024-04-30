import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
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
import './static/static-input-monotonic/static-input-monotonic'
import {
  ContentData,
  DynamicPropertyType,
  Essentiality,
  type IProperty,
  Monotonicity,
  StaticPropertyType
} from '../../util/data-interfaces'
import {
  attractorCountDynamic,
  existsTrajectoryDynamic,
  fixedPointDynamic,
  functionInputEssential,
  functionInputMonotonic,
  genericDynamic,
  genericStatic,
  hasAttractorDynamic,
  trapSpaceDynamic
} from './default-properties'

@customElement('properties-editor')
export default class PropertiesEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  @state() properties: IProperty[] = []

  constructor () {
    super()

    this.addEventListener('property-changed', this.propertyChanged)
    this.addEventListener('property-removed', this.propertyRemoved)

    this.addDynamicProperty(DynamicPropertyType.FixedPoint)
    this.addDynamicProperty(DynamicPropertyType.TrapSpace)
    this.addDynamicProperty(DynamicPropertyType.ExistsTrajectory)
    this.addDynamicProperty(DynamicPropertyType.AttractorCount)
    this.addDynamicProperty(DynamicPropertyType.HasAttractor)
    this.addDynamicProperty(DynamicPropertyType.Generic)

    this.addStaticProperty(StaticPropertyType.Generic)
    this.addStaticProperty(StaticPropertyType.FunctionInputEssential)
    this.addStaticProperty(StaticPropertyType.FunctionInputMonotonic)
  }

  addStaticProperty (type: StaticPropertyType): void {
    const id = '' + this.properties.length
    switch (type) {
      case StaticPropertyType.Generic:
        this.properties.push(genericStatic(id))
        break
      case StaticPropertyType.FunctionInputEssential:
        this.properties.push(functionInputEssential(id, 'func', 'var', Essentiality.TRUE))
        break
      case StaticPropertyType.FunctionInputMonotonic:
        this.properties.push(functionInputMonotonic(id, 'func', 'var', Monotonicity.ACTIVATION))
        break
    }
  }

  addDynamicProperty (type: DynamicPropertyType): void {
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

  render (): TemplateResult {
    return html`
      <div class="property-list">
        <div class="section" id="functions">
          <div class="header">
            <div></div>
            <h2 class="heading">Static</h2>
            <div></div>
          </div>
          <div class="uk-list uk-list-divider uk-text-center">
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
                case StaticPropertyType.FunctionInputMonotonic:
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
            <button class="add-dynamic-property">+</button>
          </div>
          <div class="uk-list uk-list-divider uk-text-center">
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
      </div>    `
  }
}
