import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './analysis-component.less?inline'
import {
  aeonState,
  type SketchData,
  type InferenceResults,
  type StaticCheckResults
} from '../../../aeon_events'
import {
  AnalysisType
} from '../../util/analysis-interfaces'
import { dialog } from '@tauri-apps/api'

@customElement('analysis-component')
export default class AnalysisComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() sketchData: SketchData | null = null

  @state() selected_analysis: AnalysisType | null = null
  @state() results: InferenceResults | StaticCheckResults | null = null
  @state() isRandomizeChecked: boolean = false // Track the state of the "Randomize" checkbox

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

    // updates regarding analyses received
    aeonState.analysis.inferenceStarted.addEventListener(
      this.#onInferenceStarted.bind(this)
    )
    aeonState.analysis.staticCheckStarted.addEventListener(
      this.#onStaticCheckStarted.bind(this)
    )
    aeonState.analysis.inferenceResultsReceived.addEventListener(
      this.#onInferenceResultsReceived.bind(this)
    )
    aeonState.analysis.staticCheckResultsReceived.addEventListener(
      this.#onStaticCheckResultsReceived.bind(this)
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
      console.log('DUMMY MESSAGE: Inference analysis sucessfully started.')
    } else {
      console.log('Error starting inference analysis.')
    }
  }

  #onStaticCheckStarted (success: boolean): void {
    if (success) {
      console.log('DUMMY MESSAGE: Static check analysis sucessfully started.')
    } else {
      console.log('Error starting static check analysis.')
    }
  }

  #onInferenceResultsReceived (results: InferenceResults): void {
    this.results = results
    console.log('Received full inference results.')
    console.log('-> There are ' + results.num_sat_networks + ' satisfying networks.')
    console.log('-> The computation took ' + results.comp_time + ' seconds.')
  }

  #onStaticCheckResultsReceived (results: StaticCheckResults): void {
    this.results = results
    console.log('Received static check results.')
    console.log('-> There are ' + results.num_sat_networks + ' satisfying networks.')
    console.log('-> The computation took ' + results.comp_time + ' seconds.')
  }

  private async confirmDialog (): Promise<boolean> {
    return await dialog.ask('Are you sure?', {
      type: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Keep',
      title: 'Delete'
    })
  }

  private runInference (): void {
    console.log('Initiating inference analysis, wait a bit...')
    aeonState.analysis.startFullInference()
    this.selected_analysis = AnalysisType.Inference
  }

  private runStaticCheck (): void {
    console.log('Initiating static check, wait a bit...')
    aeonState.analysis.startStaticCheck()
    this.selected_analysis = AnalysisType.StaticCheck
  }

  // Method to format the results for display
  private formatResults (results: InferenceResults | StaticCheckResults): string {
    return `Number of satisfying networks: ${results.num_sat_networks}\n` +
      `Computation time: ${results.comp_time} seconds\n\n\n` +
      'Computation metadata:\n' +
      '--------------\n' +
      `${results.metadata_log}\n`
  }

  private resetAnalysis (): void {
    // Reset analysis settings and clear the results
    console.log('Resetting analysis.')
    aeonState.analysis.resetAnalysis()

    this.selected_analysis = null
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
  
            <!-- Buttons for resetting analysis -->
            <div class="reset-buttons">
              <button class="uk-button uk-button-large uk-button-secondary"
                      @click="${() => {
                        this.resetAnalysis()
                      }}">Restart the analysis
              </button>
            </div>
  
            <!-- Conditionally render analysis buttons only if no analysis is selected -->
            ${this.selected_analysis === null
? html`
              <div class="uk-flex uk-flex-row uk-flex-center">
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
                          this.runStaticCheck()
                        }}">Static check
                </button>
              </div>
            `
: ''}
  
            <!-- Conditionally render results window when analysis starts, centered on screen -->
            ${this.selected_analysis !== null
? html`
              <div class="results-window" style="display: flex; justify-content: center; align-items: center; flex-direction: column;">
                <textarea rows="12" cols="70" readonly style="text-align: center;">${this.results !== null ? this.formatResults(this.results) : 'Initiating analysis, wait a bit...'}</textarea>
  
                <!-- Conditionally render "Sample network" section if results are set -->
                ${this.results !== null
? html`
                  <div class="sample-options" style="margin-top: 20px; text-align: center;">
                    <label>Witness networks:</label>
                    <div style="display: flex; align-items: center; justify-content: center;">
                      <label>Witness count</label>
                      <input type="number" min="1" .value="${1}" id="witness-count" style="width: 50px; text-align: center; margin-left: 5px; margin-right: 15px;">
  
                      <label>Randomize</label>
                      <input type="checkbox" id="randomize" .checked="${this.isRandomizeChecked}" @change="${this.handleRandomizeChange}" style="margin-left: 5px;">
                      
                      <!-- Conditionally render random seed input based on the state -->
                      ${this.isRandomizeChecked
? html`
                        <label style="margin-left: 15px;">Random seed</label>
                        <input type="number" id="random-seed" .value="${0}" style="width: 50px; text-align: center; margin-left: 5px;">
                      `
: ''}
                    </div>
                    <button class="uk-button uk-button-large uk-button-secondary"
                            @click="${async () => {
                              await this.sampleNetworks()
                            }}">Generate network(s)
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
