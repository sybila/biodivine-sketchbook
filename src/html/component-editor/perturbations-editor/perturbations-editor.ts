import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './perturbations-editor.less?inline'
import { ContentData, type IPerturbationData } from '../../util/data-interfaces'
import { map } from 'lit/directives/map.js'
import './perturbation-tile/perturbation-tile'
import {
  aeonState,
  type PerturbationData,
  type PerturbationIdUpdateData
} from '../../../aeon_state'
import { convertToIPerturbation } from '../../util/utilities'

/** Component responsible for the perturbations tab of the editor session. */
@customElement('perturbations-editor')
export class PerturbationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData = ContentData.create()

  constructor () {
    super()

    // Listen to changes in perturbations and update content data accordingly.
    aeonState.sketch.perturbations.perturbationsRefreshed.addEventListener(this.#onPerturbationsRefreshed.bind(this))
    aeonState.sketch.perturbations.perturbationCreated.addEventListener(this.#onPerturbationCreated.bind(this))
    aeonState.sketch.perturbations.perturbationAdded.addEventListener(this.#onPerturbationAdded.bind(this))
    aeonState.sketch.perturbations.perturbationRemoved.addEventListener(this.#onPerturbationRemoved.bind(this))
    aeonState.sketch.perturbations.perturbationIdChanged.addEventListener(this.#onPerturbationIdChanged.bind(this))
    aeonState.sketch.perturbations.perturbationContentChanged.addEventListener(this.#onPerturbationContentChanged.bind(this))

    aeonState.sketch.perturbations.allPerturbationsUpdated.addEventListener(this.#onAllPerturbationsUpdated.bind(this))

    // Events from perturbation-tile that need to be processed and sent above.
    this.addEventListener('set-perturbation-content', (e) => { this.setPerturbationContent(e as CustomEvent) })
    this.addEventListener('set-perturbation-id', (e) => { this.setPerturbationId(e as CustomEvent) })
    this.addEventListener('remove-perturbation', (e) => { this.removePerturbation(e as CustomEvent) })
  }

  #onPerturbationsRefreshed (perturbations: PerturbationData[]): void {
    const iPerturbations = perturbations.map(p => convertToIPerturbation(p))
    this.updatePerturbations(iPerturbations)
  }

  #onPerturbationCreated (perturbation: PerturbationData): void {
    const iPerturbation = convertToIPerturbation(perturbation)
    this.updatePerturbations(this.contentData.perturbations.concat(iPerturbation))
  }

  #onPerturbationAdded (perturbation: PerturbationData): void {
    const iPerturbation = convertToIPerturbation(perturbation)
    this.updatePerturbations(this.contentData.perturbations.concat(iPerturbation))
  }

  #onPerturbationRemoved (perturbation: PerturbationData): void {
    const perturbations = this.contentData.perturbations.filter(p => p.id !== perturbation.id)
    this.updatePerturbations(perturbations)
  }

  #onPerturbationIdChanged (data: PerturbationIdUpdateData): void {
    const index = this.contentData.perturbations.findIndex(p => p.id === data.original_id)
    if (index === -1) return
    const perturbations = structuredClone(this.contentData.perturbations)
    perturbations[index] = {
      ...perturbations[index],
      id: data.new_id
    }
    this.updatePerturbations(perturbations)
  }

  #onPerturbationContentChanged (perturbation: PerturbationData): void {
    const index = this.contentData.perturbations.findIndex(p => p.id === perturbation.id)
    if (index === -1) return
    const perturbations = structuredClone(this.contentData.perturbations)
    perturbations[index] = convertToIPerturbation(perturbation)
    this.updatePerturbations(perturbations)
  }

  #onAllPerturbationsUpdated (perturbations: PerturbationData[]): void {
    const iPerturbations = perturbations.map(p => convertToIPerturbation(p))
    this.updatePerturbations(iPerturbations)
  }

  updatePerturbations (perturbations: IPerturbationData[]): void {
    this.dispatchEvent(new CustomEvent('save-perturbations', {
      detail: {
        perturbations
      },
      bubbles: true,
      composed: true
    }))
  }

  /** Create empty perturbation (no perturbed variables). */
  private createPerturbation (): void {
    console.log('Creating perturbation')
    aeonState.sketch.perturbations.addDefaultPerturbation()
  }

  private removePerturbation (event: CustomEvent): void {
    const detail = event.detail
    aeonState.sketch.perturbations.removePerturbation(detail.id)
  }

  private setPerturbationContent (event: CustomEvent): void {
    const detail = event.detail
    aeonState.sketch.perturbations.setPerturbationContent(detail.id, detail.perturbation)
  }

  private setPerturbationId (event: CustomEvent): void {
    const detail = event.detail
    aeonState.sketch.perturbations.setPerturbationId(detail.oldId, detail.newId)
  }

  render (): TemplateResult {
    return html`
      <!-- Single-section container to limit the max width of the tab content. -->
      <div class="width-container">
        <div class="width-section">
          <div class="perturbations">
            <div class="header uk-background-primary uk-margin-bottom">
              <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Perturbations</h3>
              <div class="buttons-container">
                <button @click="${this.createPerturbation}" class="uk-button uk-button-primary uk-button-small create-button uk-border-rounded">+ Create</button>
              </div>
            </div>
            ${this.contentData?.perturbations.length === 0 ? html`<div class="uk-text-center"><span class="uk-label uk-margin-bottom">No perturbations yet</span></div>` : ''}
            <div class="accordion-body">
              <div class="accordion perturbations-list-container uk-margin-small-left uk-margin-small-right">
                ${map(this.contentData.perturbations, (perturbation) => html`
                  <perturbation-tile .perturbation="${perturbation}" .variables="${this.contentData.variables}"></perturbation-tile>`)}
              </div>
            </div>
          </div>
        </div>
      </div>
    `
  }
}
