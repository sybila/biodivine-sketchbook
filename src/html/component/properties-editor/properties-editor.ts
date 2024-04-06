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
    this.properties = [{
      name: 'static property',
      value: 'static property value',
      static: true
    }, {
      name: 'dynamic property',
      value: 'dynamic property value',
      static: false
    }]
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
