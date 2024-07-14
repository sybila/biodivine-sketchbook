import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './analysis-tab.less?inline'
import { ContentData } from '../../util/data-interfaces'
import {
  aeonState
} from '../../../aeon_events'

@customElement('analysis-tab')
export class AnalysisTab extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()

  constructor () {
    super()

    // just a placeholder event so that eslint does not complain because of empty constructor for now
    this.addEventListener('run-inference', () => { this.runInference() })
  }

  runInference (): void {
    aeonState.new_session.createNewAnalysisSession()
  }

  protected render (): TemplateResult {
    return html`
      <div class="container">
        <div class="inference">
          <div class="section" id="inference">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Inference</h3>
            </div>
            <div class="uk-flex uk-flex-row uk-flex-center">
              <button class="uk-button uk-button-large uk-button-secondary"
                      @click="${() => {
                        this.runInference()
                      }}">Start analysis workflow
              </button>
            </div>
          </div>
        </div> 
      </div>
    `
  }
}
