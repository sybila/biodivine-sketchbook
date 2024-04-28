import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './properties-editor.less?inline'
import { map } from 'lit/directives/map.js'
import './property-tile/property-tile'
import './dynamic/dynamic-fixed-point/dynamic-fixed-point'
import './dynamic/dynamic-trap-space/dynamic-trap-space'
import './static/static-generic/static-generic'
import {
  ContentData,
  type IFixedPointDynamicProperty,
  type IProperty,
  type ITrapSpaceDynamicProperty,
  PropertyType
} from '../../util/data-interfaces'

@customElement('properties-editor')
export default class PropertiesEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  @state() properties: IProperty[] = []

  constructor () {
    super()

    this.addEventListener('property-changed', this.propertyChanged)

    const trapSpace: ITrapSpaceDynamicProperty = {
      id: '1',
      name: 'dynamic-trap-space',
      type: PropertyType.TrapSpaceDynamic,
      dataset: '',
      observation: '',
      minimal: false,
      nonpercolable: false
    }
    this.properties.push(trapSpace)

    const fixedPoint: IFixedPointDynamicProperty = {
      id: '0',
      name: 'fixed-point',
      type: PropertyType.FixedPointDynamic,
      dataset: '',
      observation: ''
    }
    this.properties.push(fixedPoint)
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
                case PropertyType.GenericStatic:
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
                case PropertyType.FixedPointDynamic:
                  return html`
                    <dynamic-fixed-point .index=${index} 
                                         .property=${prop}
                                         .observations=${this.contentData.observations}>
                    </dynamic-fixed-point>`
                case PropertyType.TrapSpaceDynamic:
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
