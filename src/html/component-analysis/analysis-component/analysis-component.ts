import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './analysis-component.less?inline'
import {
  aeonState,
  type SketchData,
  type InferenceResults,
  type StaticCheckResults
} from '../../../aeon_events'
import { dialog } from '@tauri-apps/api'

@customElement('analysis-component')
export default class AnalysisComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() sketchData: SketchData | null = null

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
    console.log('Received full inference results.')
    console.log('-> There are ' + results.num_sat_networks + ' satisfying networks.')
    console.log('-> The computation took ' + results.comp_time + ' seconds.')
  }

  #onStaticCheckResultsReceived (results: StaticCheckResults): void {
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
    // TODO
  }

  private runStaticCheck (): void {
    console.log('Initiating static check, wait a bit...')
    aeonState.analysis.startStaticCheck()
    // TODO
  }

  render (): TemplateResult {
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
                      }}">Run full inference
              </button>
            </div>
            
            <div class="uk-flex uk-flex-row uk-flex-center">
              <button class="uk-button uk-button-large uk-button-secondary"
                      @click="${() => {
                        this.runStaticCheck()
                      }}">Static check
              </button>
            </div>
          </div>
        </div>
      </div>
    `
  }
}
