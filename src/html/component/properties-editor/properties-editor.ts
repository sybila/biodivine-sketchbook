import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './properties-editor.less?inline'
import { map } from 'lit/directives/map.js'
import './property-tile/property-tile'
import './dynamic/dynamic-fixed-point/dynamic-fixed-point'
import './dynamic/dynamic-trap-space/dynamic-trap-space'
import './static/static-generic/static-generic'
import { ContentData, DynamicPropertyType, type IProperty, StaticPropertyType } from '../../util/data-interfaces'
import { fixedPointDynamic, trapSpaceDynamic } from './default-properties'

@customElement('properties-editor')
export default class PropertiesEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  @state() properties: IProperty[] = []

  constructor () {
    super()

    this.addEventListener('property-changed', this.propertyChanged)

    this.addDynamicProperty(DynamicPropertyType.FixedPoint)
    this.addDynamicProperty(DynamicPropertyType.TrapSpace)
  }

  addDynamicProperty (type: DynamicPropertyType): void {
    switch (type) {
      case DynamicPropertyType.Generic:
        break
      case DynamicPropertyType.FixedPoint:
        this.properties.push(fixedPointDynamic('' + this.properties.length))
        break
      case DynamicPropertyType.TrapSpace:
        this.properties.push(trapSpaceDynamic('' + this.properties.length))
        break
      case DynamicPropertyType.ExistsTrajectory:
        break
      case DynamicPropertyType.AttractorCount:
        break
      case DynamicPropertyType.HasAttractor:
        break
    }
  }

  propertyChanged (event: Event): void {
    const detail = (event as CustomEvent).detail
    const props = [...this.properties]
    props[detail.index] = detail.property
    this.properties = props
  }

  render (): TemplateResult {
    return html`
      <div class="property-list">
        <div class="section" id="functions">
          <h2 class="heading uk-text-center">Static</h2>
          <div class="uk-list uk-list-divider uk-text-center">
            ${map(this.properties, (prop, index) => {
              switch (prop.type) {
                case StaticPropertyType.Generic:
                  return html`<static-generic .index=${index}
                                              .property=${prop}
                  ></static-generic>`
                default:
                  return ''
              }
            })}
          </div>
        </div>
        <div class="section" id="variables">
          <h2 class="heading uk-text-center">Dynamic</h2>
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
                default:
                  return ''
              }
            })}
          </div>
        </div>
      </div>    `
  }
}
