import { css, html, LitElement, unsafeCSS, type TemplateResult, type PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './analysis-tab.less?inline'
import { ContentData } from '../../util/data-interfaces'
import { aeonState } from '../../../aeon_state'

/** Component responsible for the analysis tab of the editor session. */
@customElement('analysis-tab')
export class AnalysisTab extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData: ContentData = ContentData.create()
  /** String message of consistency check (null if currently not fetched). */
  @state() consistencyResults: string | null = null
  /** Number of params of the PSBN component (null if currently not fetched). */
  @state() numPSBNParams: number | null = null

  constructor () {
    super()

    // listen for the consistency check results event
    aeonState.sketch.consistencyResults.addEventListener(
      this.#onConsistencyResults.bind(this)
    )
    // listen for the fetched number of psbn params
    aeonState.sketch.numPSBNParamsFetched.addEventListener(
      this.#onPSBNParamsNumFetched.bind(this)
    )
  }

  protected updated (_changedProperties: PropertyValues): void {
    super.updated(_changedProperties)

    // Once some part of the actual sketch is updated, hide the consistency check results
    // and number of PSBN params as they may no longer be valid.
    if (_changedProperties.has('contentData')) {
      this.consistencyResults = null
      this.numPSBNParams = null
    }
  }

  /** Initiate the inference window by calling the backend. */
  runInference (): void {
    aeonState.new_session.createNewInferenceSession()
  }

  /** Run the consistency check by calling the backend. */
  checkConsistency (): void {
    aeonState.sketch.checkConsistency()
  }

  /** Process and save the consistency check results. */
  #onConsistencyResults (results: string): void {
    this.consistencyResults = results
    console.log('Received consistency check results.')
  }

  closeConsistencyResults (): void {
    this.consistencyResults = null
  }

  /** Request backend to send num of PSBN params. */
  getPSBNParamsNum (): void {
    aeonState.sketch.fetchNumPSBNParams()
  }

  /** Process and save the number of psbn params. */
  #onPSBNParamsNumFetched (num: number): void {
    this.numPSBNParams = num
    console.log('Received number of PSBN params.')
  }

  /** Compute maximal node indegree in the influence graph. */
  getMaxNetworkIndegree (): number {
    let maxIndegree = 0
    for (const variable of this.contentData.variables) {
      // Count regulations where this variable is the target
      const indegree = this.contentData.regulations.filter(
        reg => reg.target === variable.id
      ).length
      if (indegree > maxIndegree) {
        maxIndegree = indegree
      }
    }
    return maxIndegree
  }

  /**
   * Compute how many parametrizations there are. Simply multiply arities of all
   * function symbols and in-degrees of variables without update functions.
   *
   * TODO: This does not into account that some function symbols may be unused or may
   * have their expressions partially specified.
   */
  estimateNumPSBNCandidates (): number {
    return this.contentData.functions.reduce(
      (product, fn) => product * (fn.variables.length),
      1
    )
  }

  protected render (): TemplateResult {
    return html`
      <div class="container">
        <div class="analyses-list">
          <div class="section" id="inference">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Inference workflow</h3>
            </div>
            <div class="uk-flex uk-flex-column uk-flex-left uk-margin-bottom" style="padding: 20px 20px 20px 20px;">
              <span style="font-size:larger;">Sketch overview:</span>
              <table id="model-stats" style="padding: 20px 20px 5px 20px;">
                <colgroup>
                  <col style="width: 35%;">
                  <col style="width: 15%;">
                  <col style="width: 35%;">
                  <col style="width: 15%;">
                </colgroup>
                <tbody>
                  <tr class="row">    
                    <td>Variables: </td>       
                    <td class="value">${this.contentData.variables.length}</td>     
                    <td>Suppl. functions: </td> 
                    <td class="value">${this.contentData.functions.length}</td> 
                  </tr>
                  <tr class="row">    
                    <td>Regulations: </td>      
                    <td class="value">${this.contentData.regulations.length}</td>     
                    <td>Static properties: </td> 
                    <td class="value">${this.contentData.staticProperties.length}</td> 
                  </tr>
                  <tr class="row">    
                    <td>Max. in-degree: </td>   
                    <td class="value">${this.getMaxNetworkIndegree()}</td>     
                    <td>Dynamic properties: </td> 
                    <td class="value">${this.contentData.dynamicProperties.length}</td> 
                  </tr>
                  <tr class="row">    
                    <td>PSBN interpretations: </td>       
                    <td class="value">
                      ${this.numPSBNParams === null
                          ? html`
                          <button id="compute-psbn-params-button" class="uk-button uk-button-small uk-button-secondary"
                            @click="${() => {
                              this.getPSBNParamsNum()
                            }}">Fetch
                          </button>
                          `
: html`2^${this.numPSBNParams}`}
                    </td>     
                    <td></td>    
                    <td></td>          
                  </tr>
                </tbody>
              </table>
            </div>
            <div class="uk-flex uk-flex-row uk-flex-center">
              <button id="open-inference-button" class="uk-button uk-button-large uk-button-secondary uk-margin-bottom"
                      @click="${() => {
                        this.runInference()
                      }}">Start inference session
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
              ${this.consistencyResults !== null
                  ? html`
                    <div class="results-window" style="display: flex; justify-content: center; align-items: center; flex-direction: column;">
                      <textarea rows="12" cols="60" readonly style="text-align: center;">${this.consistencyResults}</textarea>
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
