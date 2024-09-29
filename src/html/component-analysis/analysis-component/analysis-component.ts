import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './analysis-component.less?inline'
import {
  aeonState,
  type SketchData,
  type InferenceResults
} from '../../../aeon_events'
import {
  AnalysisType
} from '../../util/analysis-interfaces'
import { dialog } from '@tauri-apps/api'
import { inferencePingTimer } from '../../util/config'

@customElement('analysis-component')
export default class AnalysisComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() sketchData: SketchData | null = null

  // Type of the analysis we are running
  @state() selected_analysis: AnalysisType | null = null
  // Results of analysis
  @state() results: InferenceResults | null = null
  // Track the state of the "Randomize" checkbox for sampling
  @state() isRandomizeChecked: boolean = false
  // ID of the `setInterval` we use for pinging backend to get results
  @state() pingIntervalId: ReturnType<typeof setInterval> | undefined = undefined
  // Number of times backend was pinged already (for current computation)
  @state() pingCounter: number = 0
  // Main HTML text displayed when waiting for analysis results (can depend on analysis type
  // and can be updated during computation)
  @state() waitingMainMessage: string = ''
  // Intermediate progress report when waiting for analysis results (can be updated during computation)
  @state() waitingProgressReport: string = ''

  constructor () {
    super()

    // error event listener
    aeonState.error.errorReceived.addEventListener((e) => {
      void this.#onErrorMessage(e)
    })

    // underlying sketch data updated (should only happen at the beginning)
    aeonState.analysis.sketchRefreshed.addEventListener((sketch) => {
      void this.#onSketchRefreshed(sketch)
    })

    // updates regarding analysis start received
    aeonState.analysis.inferenceStarted.addEventListener(
      this.#onInferenceStarted.bind(this)
    )

    // updates regarding analysis results
    aeonState.analysis.inferenceResultsReceived.addEventListener(
      this.#onInferenceResultsReceived.bind(this)
    )

    // updates regarding analysis progress or errors
    aeonState.analysis.computationUpdated.addEventListener(
      this.#onComputationUpdateMessageReceived.bind(this)
    )
    aeonState.analysis.computationErrorReceived.addEventListener(
      this.#onComputationErrorMessageReceived.bind(this)
    )

    // ask for sketch data during initiation (just in case the automatic transfer fails)
    aeonState.analysis.refreshSketch()
  }

  async #onSketchRefreshed (sketchData: SketchData): Promise<void> {
    // currently we only accept the sketch data once, and it is frozen later
    // if this changes and we want to allow re-writing sketch data, update this function

    if (this.sketchData === null) {
      this.sketchData = sketchData
      const numVars = sketchData.model.variables.length
      console.log('Received sketch data. The sketch has ' + numVars.toString() + ' variables.')
    } else {
      console.log('Can\'t accept sketch data. Sketch was already set before.')
    }
  }

  async #onErrorMessage (errorMessage: string): Promise<void> {
    await dialog.message(errorMessage, {
      type: 'error',
      title: 'Error'
    })
  }

  #onInferenceStarted (success: boolean): void {
    if (success) {
      console.log('Inference analysis sucessfully started. Starting interval pinging backend.')
    } else {
      console.log('Error starting inference analysis.')
    }

    this.waitingMainMessage = 'Computation is running. Waiting for the results.<br>'
    this.waitingProgressReport += 'Intermediate progress report:\n--------------\n'

    // start pinging backend
    this.pingIntervalId = setInterval(() => {
      this.pingCounter += 1
      aeonState.analysis.pingForInferenceResults()
    }, inferencePingTimer)
  }

  #onComputationUpdateMessageReceived (message: string): void {
    console.log(message)
    this.waitingProgressReport += message
  }

  #onComputationErrorMessageReceived (message: string): void {
    console.log(message)
    this.waitingMainMessage = 'Inference computation ended with an error.<br>'
    this.waitingProgressReport = 'Error running inference:\n\n' + message

    // stop pinging backend
    clearInterval(this.pingIntervalId)
    this.pingIntervalId = undefined
    this.pingCounter = 0
  }

  #onInferenceResultsReceived (results: InferenceResults): void {
    // stop pinging backend
    clearInterval(this.pingIntervalId)
    this.pingIntervalId = undefined
    this.pingCounter = 0

    this.results = results
    console.log('Received full inference results.')
  }

  // TODO: use this dialog when restarting inference that did not finish yet
  private async confirmDialog (): Promise<boolean> {
    return await dialog.ask('Are you sure?', {
      type: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Keep',
      title: 'Delete'
    })
  }

  private runInference (): void {
    console.log('Initiating inference analysis.')
    aeonState.analysis.startFullInference()
    this.selected_analysis = AnalysisType.FullInference
  }

  private runStaticInference (): void {
    console.log('Initiating inference with static properties.')
    aeonState.analysis.startStaticInference()
    this.selected_analysis = AnalysisType.StaticInference
  }

  // Method to format the results for display
  private formatResultsOverview (results: InferenceResults): string {
    const compTime = Math.max(results.comp_time, 1) // just in case, to not have "0 seconds"
    return 'Analysis finished!<br><br>' +
      `Number of satisfying networks: ${results.num_sat_networks}<br>` +
      `Computation time: ${compTime} seconds<br>`
  }

  // Method to format the results for display
  private formatResultsMetadata (results: InferenceResults): string {
    return 'Computation metadata:\n' +
      '--------------\n' +
      `Analysis type: ${results.analysis_type}\n` +
      `${results.metadata_log}\n`
  }

  private resetAnalysis (): void {
    console.log('Resetting analysis.')

    // stop pinging backend
    clearInterval(this.pingIntervalId)
    this.pingIntervalId = undefined
    this.pingCounter = 0

    // reset event to backend
    aeonState.analysis.resetAnalysis()

    // clear analysis settings and results
    this.selected_analysis = null
    this.waitingMainMessage = ''
    this.waitingProgressReport = ''
    this.results = null
  }

  private async sampleNetworks (): Promise<void> {
    const witnessCountInput = this.shadowRoot?.getElementById('witness-count') as HTMLInputElement
    if (witnessCountInput === null) {
      console.error('Failed to get input elements to sample networks.')
      return
    }

    const witnessCount = parseInt(witnessCountInput.value, 10)
    const randomSeedInput = this.shadowRoot?.getElementById('random-seed') as HTMLInputElement | null
    const randomSeed = this.isRandomizeChecked && randomSeedInput !== null ? parseInt(randomSeedInput.value, 10) : null

    console.log(`Sampling networks - witness count: ${witnessCount}, randomize: ${this.isRandomizeChecked}, random seed: ${randomSeed}`)

    const archiveName = `sat_networks_${witnessCount}.zip`
    const handle = await dialog.save({
      defaultPath: archiveName,
      filters: [{
        name: 'ZIP',
        extensions: ['zip']
      }]
    })
    if (handle === null) return

    let fileName
    if (Array.isArray(handle)) {
      fileName = handle.pop() ?? 'unknown'
    } else {
      fileName = handle
    }

    console.log(`Generating network archive at: ${fileName}`)
    aeonState.analysis.sampleNetworks(witnessCount, randomSeed, fileName)
  }

  // Add a handler to update the checkbox state
  private handleRandomizeChange (event: Event): void {
    const checkbox = event.target as HTMLInputElement
    this.isRandomizeChecked = checkbox.checked
  }

  render (): TemplateResult {
    return html`
      <div class="container">
        <div class="inference">
          <div class="section" id="inference">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom">Inference</h3>
            </div>
  
            <!-- Conditionally render analysis buttons only if no analysis is selected,
                 Otherwise, render a button for resetting analysis. -->
            ${this.selected_analysis === null
? html`
              <div class="uk-flex uk-flex-row uk-flex-center" style="margin-top: 90px">
                <button class="uk-button uk-button-large uk-button-secondary"
                        @click="${() => {
                          this.runInference()
                        }}">Run full inference
                </button>
              </div>
              
              <!-- Space between the buttons -->
              <div style="height: 10px;"></div>
              
              <div class="uk-flex uk-flex-row uk-flex-center">
                <button class="uk-button uk-button-large uk-button-secondary"
                        @click="${() => {
                          this.runStaticInference()
                        }}">Run static inference
                </button>
              </div>
            `
: html`
              <div class="reset-buttons">
                <button class="uk-button uk-button-large uk-button-secondary"
                        @click="${() => {
                          this.resetAnalysis()
                        }}">Start again
                </button>
              </div>
            `
}
            <!-- Conditionally render results window when analysis starts, centered on screen -->
            ${this.selected_analysis !== null
? html`
              <div class="results-window">
                <div class="overview-message"
                  .innerHTML="${this.results !== null ? this.formatResultsOverview(this.results) : this.waitingMainMessage + '.'.repeat(this.pingCounter % 4 + 1) + '<br>'}">
                </div>

                <textarea rows="10" cols="70" readonly style="text-align: center;">${this.results !== null ? this.formatResultsMetadata(this.results) : this.waitingProgressReport}</textarea>
  
                <!-- Conditionally render "Sample network" section if results are set -->
                ${this.results !== null
? html`
                  <div class="sample-options">
                    <label>Candidate networks sampling:</label>
                    <div style="display: flex; align-items: center; justify-content: center;">
                      <label>Network count</label>
                      <input  type="number" min="1" .value="${1}" id="witness-count">
  
                      <label>Randomize</label>
                      <input type="checkbox" id="randomize" .checked="${this.isRandomizeChecked}" @change="${this.handleRandomizeChange}" style="margin-left: 5px;">
                      
                      <!-- Conditionally render random seed input based on the state -->
                      ${this.isRandomizeChecked
? html`
                        <label style="margin-left: 15px;">Random seed</label>
                        <input type="number" id="random-seed" .value="${0}">
                      `
: ''}
                    </div>
                    <button id="generate-network-button" class="uk-button uk-button-large uk-button-secondary"
                            @click="${async () => {
                              await this.sampleNetworks()
                            }}">Sample network(s)
                    </button>
                  </div>
                `
: ''}
              </div>
            `
: ''}
          </div>
        </div>
      </div>
    `
  }
}
