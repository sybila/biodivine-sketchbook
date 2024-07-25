import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './analysis-component.less?inline'
import {
  aeonState,
  type SketchData
} from '../../../aeon_events'
import {
  ContentData
} from '../../util/data-interfaces'
import { dialog } from '@tauri-apps/api'

@customElement('analysis-component')
export default class AnalysisComponent extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() data: ContentData = ContentData.create()

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
  }

  async #onSketchRefreshed (sketchData: SketchData): Promise<void> {
    const numVars = sketchData.model.variables.length
    await dialog.message('Received sketch data, yay! It has ' + numVars.toString() + ' vars.', {
      type: 'info',
      title: 'Sketch received.'
    })
  }

  async #onErrorMessage (errorMessage: string): Promise<void> {
    await dialog.message(errorMessage, {
      type: 'error',
      title: 'Error'
    })
  }

  private async confirmDialog (): Promise<boolean> {
    return await dialog.ask('Are you sure?', {
      type: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Keep',
      title: 'Delete'
    })
  }

  private async dummyDialog (): Promise<boolean> {
    return await dialog.ask('Hello there.', {
      type: 'info',
      okLabel: 'OK',
      cancelLabel: 'Cancel',
      title: 'Hello there'
    })
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
                        void this.dummyDialog()
                      }}">Step-by-step workflow
              </button>
            </div>
            
            <div class="uk-flex uk-flex-row uk-flex-center">
              <button class="uk-button uk-button-large uk-button-secondary"
                      @click="${() => {
                        void this.dummyDialog()
                      }}">Run full inference
              </button>
            </div>
          </div>
        </div>
      </div>
    `
  }
}
