import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './analysis-component.less?inline'
import {
  aeonState,
  type SketchData
} from '../../../aeon_state'
import {
  type InferenceStatusReport,
  InferenceType,
  type InferenceResults
} from '../../util/analysis-interfaces'
import { dialog } from '@tauri-apps/api'
import { inferencePingTimer } from '../../util/config'

/** The main component responsible for the inference anylis window. */
@customElement('analysis-component')
export default class AnalysisComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() sketchData: SketchData | null = null

  /** Type of the inference analysis we are running */
  @state() selected_inference: InferenceType | null = null
  /** Results of the inference */
  @state() results: InferenceResults | null = null
  /** Track the state of the "Randomize" checkbox for sampling */
  @state() isRandomizeChecked: boolean = false
  /** ID of the `setInterval` we use for pinging backend to get results */
  @state() pingIntervalId: ReturnType<typeof setInterval> | undefined = undefined
  /** Number of times backend was pinged already (for current computation) */
  @state() pingCounter: number = 0
  /** Main HTML text displayed when waiting for inference results (can depend on inference
  /** type and can be updated during computation) */
  @state() waitingMainMessage: string = ''
  /** Intermediate progress report when waiting for inference results (updated during computation) */
  @state() waitingProgressReport: string = ''
  /** Number of already evaluated static properties */
  @state() staticDone: number = 0
  /** Number of already evaluated dynamic properties */
  @state() dynamicDone: number = 0

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

    // updates regarding inference start received
    aeonState.analysis.inferenceStarted.addEventListener(
      this.#onInferenceStarted.bind(this)
    )

    // updates regarding inference results
    aeonState.analysis.inferenceResultsReceived.addEventListener(
      this.#onInferenceResultsReceived.bind(this)
    )

    // updates regarding inference progress or errors
    aeonState.analysis.computationUpdated.addEventListener(
      this.#onComputationUpdateReceived.bind(this)
    )
    aeonState.analysis.computationErrorReceived.addEventListener(
      this.#onComputationErrorMessageReceived.bind(this)
    )

    // ask for sketch data during initiation (just in case the automatic transfer fails)
    aeonState.analysis.refreshSketch()
  }

  /** Process sketch data sent by backend. */
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

  /** Process error message sent by backend, display error dialog. */
  async #onErrorMessage (errorMessage: string): Promise<void> {
    await dialog.message(errorMessage, {
      type: 'error',
      title: 'Error'
    })
  }

  /** Process confirmation that inference started on the backend, start displaying progress. */
  #onInferenceStarted (success: boolean): void {
    if (success) {
      console.log('Inference computation sucessfully started. Starting pinging backend.')
    } else {
      console.log('Error starting the inference computation.')
    }

    this.waitingProgressReport += '--------------\nDetailed progress report:\n--------------\n'
    this.waitingMainMessage = this.formatWaitingOverview()

    // start pinging backend
    this.pingIntervalId = setInterval(() => {
      this.pingCounter += 1
      aeonState.analysis.pingForInferenceResults()
    }, inferencePingTimer)
  }

  /** Format the message shown during computation, with an overview of the computation
   * progress (into HTML string with HTML newline tags). */
  private formatWaitingOverview (): string {
    const staticTotal = this.sketchData?.stat_properties.length
    const dynamicTotal = this.sketchData?.dyn_properties.length

    let message = 'Computation is running. Waiting for the results.<br>' +
      `- processed ${this.staticDone} static properties (out of ${staticTotal})<br>`
    if (this.selected_inference === InferenceType.FullInference) {
      message += `- processed ${this.dynamicDone} dynamic properties (out of ${dynamicTotal})<br>`
    }
    return message
  }

  /** Process computation update sent from the backend, updating progress message. */
  #onComputationUpdateReceived (progressReports: InferenceStatusReport[]): void {
    progressReports.forEach((progressUpdate) => {
      console.log(progressUpdate)
      if (typeof progressUpdate.status === 'object' && 'EvaluatedStatic' in progressUpdate.status) {
        this.staticDone += 1
      }
      if (typeof progressUpdate.status === 'object' && 'EvaluatedDynamic' in progressUpdate.status) {
        this.dynamicDone += 1
      }
      this.waitingMainMessage = this.formatWaitingOverview()
      this.waitingProgressReport += progressUpdate.message + '\n'
    })
  }

  /** Process computation error message sent from the backend, logging it and finishing computation. */
  #onComputationErrorMessageReceived (message: string): void {
    console.log(message)
    this.waitingMainMessage = 'Inference computation ended with an error.<br>'
    this.waitingProgressReport = 'Internal solver error during inference:\n\n' + message

    // stop pinging backend
    clearInterval(this.pingIntervalId)
    this.pingIntervalId = undefined
    this.pingCounter = 0
  }

  /** Process inference results sent from the backend, stop pinging the backend. */
  #onInferenceResultsReceived (results: InferenceResults): void {
    // stop pinging backend
    clearInterval(this.pingIntervalId)
    this.pingIntervalId = undefined
    this.pingCounter = 0

    this.results = results
    console.log('Received inference results.')
  }

  /** Show the dialog to confirm restarting the inference. */
  private async confirmInferenceRestartDialog (): Promise<boolean> {
    return await dialog.ask('Restarting the inference will erase the current progress and results. Do you want to proceed?', {
      type: 'warning',
      okLabel: 'Restart',
      cancelLabel: 'Keep',
      title: 'Restart Inference'
    })
  }

  /** Invoke the backend to start the inference. */
  private runInference (): void {
    console.log('Initiating full inference with all properties.')
    aeonState.analysis.startFullInference()
    this.selected_inference = InferenceType.FullInference
  }

  /** Invoke the backend to start the inference with static properties. */
  private runStaticInference (): void {
    console.log('Initiating inference with static properties.')
    aeonState.analysis.startStaticInference()
    this.selected_inference = InferenceType.StaticInference
  }

  /** Helper function to format computation time (given in milliseconds). */
  private formatCompTime (ms: number): string {
    if (ms >= 1000) {
      const seconds = Math.floor(ms / 1000)
      const milliseconds = ms % 1000
      return `${seconds}.${milliseconds.toString().padStart(3, '0')} seconds`
    } else {
      return `${ms} milliseconds`
    }
  }

  /** Format the results overview (into string with HTML tag newlines). */
  private formatResultsOverview (results: InferenceResults): string {
    /// format time (from pure milliseconds)
    const compTimeStr = this.formatCompTime(results.comp_time)

    // different message if sketch is satisfiable/unsatisfiable
    if (results.num_sat_networks > 0) {
      return 'Inference finished!<br><br>' +
        `Number of satisfying candidates: ${results.num_sat_networks}<br>` +
        `Computation time: ${compTimeStr}<br>`
    } else {
      return 'Inference finished!<br><br>' +
        'There are no satisfying candidates.<br>' +
        `Computation time: ${compTimeStr}<br>`
    }
  }

  /** Format the results messages and summary for display. */
  private formatResultsMetadata (results: InferenceResults): string {
    const progressSummary = results
      .progress_statuses
      .slice(1) // skip the first status
      .map(statusReport => statusReport.message)
      .join('\n')

    let resultsMessage = '--------------\nExtended summary:\n--------------\n' +
      `${results.summary_message}\n`
    if (results.num_sat_networks > 0) {
      // prepare the summary with update functions per variable, sorted by var name
      const updateFnsSummary = Object.entries(results.num_update_fns_per_var)
        .sort(([varNameA], [varNameB]) => varNameA.localeCompare(varNameB))
        .map(([varName, count]) => {
          const countDisplay = count >= 1000 ? 'more than 1000' : count.toString()
          return `${varName}: ${countDisplay}`
        })
        .join('\n')

      resultsMessage += '--------------\nNumber of admissible update functions per variable:\n--------------\n' +
        updateFnsSummary + '\n\n'
    }
    resultsMessage += '--------------\nDetailed progress report:\n--------------\n' +
      progressSummary
    return resultsMessage
  }

  /** Initiate the inference reset. First, user is asked to confirm, the we invoke backend. */
  private async resetInference (): Promise<void> {
    if (!await this.confirmInferenceRestartDialog()) return
    console.log('Resetting inference.')

    // stop pinging backend
    clearInterval(this.pingIntervalId)
    this.pingIntervalId = undefined
    this.pingCounter = 0

    // reset event to backend
    aeonState.analysis.resetInference()

    // clear inference settings and results
    this.selected_inference = null
    this.waitingMainMessage = ''
    this.waitingProgressReport = ''
    this.results = null
    this.staticDone = 0
    this.dynamicDone = 0
  }

  /** Invoke the backend to sample and export BNs according to user selected parameters and paths. */
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

  /** Invoke the backend to export results, and let the user select the path. */
  private async dumpFullResults (): Promise<void> {
    const handle = await dialog.save({
      defaultPath: 'results.zip',
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

    console.log(`Dumping results at: ${fileName}`)
    aeonState.analysis.dumpFullResults(fileName)
  }

  /** Handle changes to the checkbox state for randomized sampling. */
  private handleRandomizeChange (event: Event): void {
    const checkbox = event.target as HTMLInputElement
    this.isRandomizeChecked = checkbox.checked
  }

  /** Render the page for inference, using conditional rendering where possible. */
  render (): TemplateResult {
    return html`
      <div class="width-container">
        <div class="width-section" id="inference">
          <div class="header uk-background-primary uk-margin-bottom">
            <h3 class="uk-heading-bullet uk-margin-remove-bottom">Inference</h3>
          </div>

          <!-- Conditionally render inference buttons only if no inference is selected yet,
                Otherwise, render a button for resetting inference. -->
          ${this.selected_inference === null
? html`
            <div class="uk-flex uk-flex-row uk-flex-center" style="margin-top: 90px">
              <button id="full-inference-button" class="uk-button uk-button-large uk-button-secondary"
                      @click="${() => {
                        this.runInference()
                      }}">Run full inference
              </button>
            </div>
            
            <!-- Space between the buttons -->
            <div style="height: 10px;"></div>
            
            <div class="uk-flex uk-flex-row uk-flex-center">
              <button id="static-inference-button" class="uk-button uk-button-large uk-button-secondary"
                      @click="${() => {
                        this.runStaticInference()
                      }}">Run static inference
              </button>
            </div>
          `
: html`
            <div class="reset-buttons">
              <button id="reset-inference-button" class="uk-button uk-button-large uk-button-secondary"
                      @click="${() => {
                        void this.resetInference()
                      }}">Start again
              </button>
            </div>
          `
}
          <!-- Conditionally render results window when inference starts, centered on screen -->
          ${this.selected_inference !== null
? html`
            <div class="results-window">
              <div class="overview-message uk-text" 
                .innerHTML="${this.results !== null ? this.formatResultsOverview(this.results) : this.waitingMainMessage + '.'.repeat(this.pingCounter % 4) + '<br>'}">
              </div>

              <textarea rows="12" cols="120" readonly style="text-align: left;">${this.results !== null ? this.formatResultsMetadata(this.results) : this.waitingProgressReport}</textarea>

              <!-- Conditionally render dumping/sampling sections if results are set (and there are >0 candiates) -->
              ${this.results !== null && this.results.num_sat_networks > 0
? html`
                <div class="results-options uk-container">
                  <button id="dump-bdd-button" class="uk-button uk-button-large uk-button-secondary"
                          @click="${async () => {
                            await this.dumpFullResults()
                          }}">Save full results
                  </button>
                </div>

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
    `
  }
}
