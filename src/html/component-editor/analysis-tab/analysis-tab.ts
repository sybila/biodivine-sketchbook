import { css, html, LitElement, unsafeCSS, type TemplateResult, type PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './analysis-tab.less?inline'
import { ContentData } from '../../util/data-interfaces'
import { aeonState } from '../../../aeon_state'

@customElement('analysis-tab')
export class AnalysisTab extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  @state() consistency_results: string | null = null

  constructor () {
    super()

    aeonState.sketch.consistencyResults.addEventListener(
      this.#onConsistencyResults.bind(this)
    )
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)

    // Once some part of the actual sketch is updated, hide the consistency check results
    // as they are no longer valid.
    if (_changedProperties.has('contentData')) {
      this.consistency_results = null
    }
  }

  runInference (): void {
    aeonState.new_session.createNewAnalysisSession()
  }

  checkConsistency (): void {
    aeonState.sketch.checkConsistency()
  }

  #onConsistencyResults (results: string): void {
    this.consistency_results = results
    console.log('Received consistency check results.')
  }

  closeConsistencyResults (): void {
    this.consistency_results = null
  }

  protected render (): TemplateResult {
    return html`
      <div class="container">
        <div class="analyses-list">
          <div class="section" id="inference">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Inference workflow</h3>
            </div>
            <div class="uk-flex uk-flex-row uk-flex-center">
              <button id="open-inference-button" class="uk-button uk-button-large uk-button-secondary uk-margin-bottom"
                      @click="${() => {
                        this.runInference()
                      }}">Start inference workflow
              </button>
            </div>
          </div>
          <div class="section" id="consistency-check">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Consistency check</h3>
            </div>
            <div class="uk-flex uk-flex-row uk-flex-center">
              <button id="consistency-check-button" class="uk-button uk-button-large uk-button-secondary uk-margin-bottom"
                      @click="${() => {
                        this.checkConsistency()
                      }}">Run consistency check
              </button>
            </div>
            <div class="uk-flex uk-flex-row uk-flex-center">
                <!-- Conditionally render consistency results window when check starts, centered on screen -->
              ${this.consistency_results !== null
                  ? html`
                    <div class="results-window" style="display: flex; justify-content: center; align-items: center; flex-direction: column;">
                      <textarea rows="12" cols="60" readonly style="text-align: center;">${this.consistency_results}</textarea>
                      <button class="uk-button uk-button-small uk-button-danger uk-margin-top"
                              @click="${this.closeConsistencyResults}">Close</button>
                    </div>
                  `
: ''}
            </div>
          </div>
          </div>
        </div> 
      </div>
    `
  }
}
