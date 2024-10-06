import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './annotations-tab.less?inline'
import { ContentData } from '../../util/data-interfaces'

@customElement('annotations-tab')
export class AnnotationsTab extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()

  constructor () {
    super()
    console.log('debug print')
  }

  protected render (): TemplateResult {
    return html`
      <div class="container">
        <div class="annotations-list">
          <div class="section" id="whole-sketch">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Whole sketch</h3>
            </div>
            TODO: sketch annotation
          </div>
          <div class="section" id="variables">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Variables</h3>
            </div>
            TODO: list of variable annotations
          </div>
          <div class="section" id="functions">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Uninterpreted functions</h3>
            </div>
            TODO: list of function annotations
          </div>
          <div class="section" id="datasets">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Datasets</h3>
            </div>
            TODO: list of dataset annotations + inner lists with observations
          </div>
          <div class="section" id="static">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Static properties</h3>
            </div>
            TODO: list of property annotations
          </div>
          <div class="section" id="dynamic">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Dynamic properties</h3>
            </div>
            TODO: list of property annotations
          </div>
        </div> 
      </div>
    `
  }
}
