import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './properties-editor.less?inline'
import { map } from 'lit/directives/map.js'
import './property-tile/property-tile'

import { ContentData, type IProperty } from '../../util/data-interfaces'

@customElement('properties-editor')
export default class PropertiesEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  @state() properties: IProperty[] = []

  constructor () {
    super()
    this.addEventListener('property-name-changed', this.propNameChanged)
    this.addEventListener('property-value-changed', this.propValueChanged)

    this.properties = [{
      id: '1',
      name: 'static property',
      value: 'static property value',
      static: true
    }, {
      id: '2',
      name: 'dynamic property',
      value: 'dynamic property value',
      static: false
    }]
  }

  propNameChanged (event: Event): void {
    const detail = (event as CustomEvent).detail
    const properties = [...this.properties]
    const index = properties.findIndex(prop => prop.id === detail.id)
    if (index === -1) return
    properties[index].name = detail.name
    this.properties = properties
  }

  propValueChanged (event: Event): void {
    const detail = (event as CustomEvent).detail
    const properties = [...this.properties]
    const index = properties.findIndex(prop => prop.id === detail.id)
    if (index === -1) return
    properties[index].value = detail.value
    this.properties = properties
  }

  render (): TemplateResult {
    return html`
      <div class="property-list">
        <div class="section" id="functions">
          <h2 class="heading uk-text-center">Static</h2>
          <div class="uk-list uk-list-divider uk-text-center">
            ${map(this.properties.filter(p => p.static), (prop) => html`
              <property-tile .prop="${prop}">
              </property-tile>
            `)}
          </div>
        </div>
        <div class="section" id="variables">
          <h2 class="heading uk-text-center">Dynamic</h2>
          <div class="uk-list uk-list-divider uk-text-center">
            ${map(this.properties.filter(p => !p.static), (prop) => html`
              <property-tile .prop=${prop}>
              </property-tile>
            `)}
          </div>
        </div>
      </div>    `
  }
}
