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
import { convertFromIPerturbation, convertToIPerturbation } from '../../util/utilities'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/event'

/** Component responsible for the perturbations tab of the editor session. */
@customElement('perturbations-editor')
export class PerturbationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData = ContentData.create()
  dialogs: Record<string, WebviewWindow | undefined> = {}

  constructor () {
    super()

    // Listen to changes in perturbations and update content data accordingly.
    aeonState.sketch.perturbations.perturbationsRefreshed.addEventListener(this.#onPerturbationsRefreshed.bind(this))
    aeonState.sketch.perturbations.perturbationCreated.addEventListener(this.#onPerturbationCreated.bind(this))
    aeonState.sketch.perturbations.perturbationRemoved.addEventListener(this.#onPerturbationRemoved.bind(this))
    aeonState.sketch.perturbations.perturbationIdChanged.addEventListener(this.#onPerturbationIdChanged.bind(this))
    aeonState.sketch.perturbations.perturbationContentChanged.addEventListener(this.#onPerturbationContentChanged.bind(this))

    aeonState.sketch.perturbations.allPerturbationsUpdated.addEventListener(this.#onAllPerturbationsUpdated.bind(this))

    // Events from perturbation-tile that need to be processed and sent above.
    this.addEventListener('set-perturbation-content', (e) => { this.setPerturbationContent(e as CustomEvent) })
    this.addEventListener('set-perturbation-id', (e) => { this.setPerturbationId(e as CustomEvent) })
    this.addEventListener('remove-perturbation', (e) => { this.removePerturbation(e as CustomEvent) })
    this.addEventListener('edit-perturbation', (e) => { void this.editPerturbation(e) })
  }

  #onPerturbationsRefreshed (perturbations: PerturbationData[]): void {
    const iPerturbations = perturbations.map(p => convertToIPerturbation(p))
    this.updatePerturbations(iPerturbations)
  }

  #onPerturbationCreated (perturbation: PerturbationData): void {
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

  /** Open dialog to edit perturbation id/name/annotation, and propagate changes to backend. */
  private async editPerturbation (event: Event): Promise<void> {
    const detail = (event as CustomEvent).detail
    const perturbationIndex = this.contentData.perturbations.findIndex(p => p.id === detail.id)
    if (perturbationIndex === -1) return
    const perturbationData = this.contentData.perturbations[perturbationIndex]

    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    if (this.dialogs[perturbationData.id] !== undefined) {
      await this.dialogs[perturbationData.id]?.setFocus()
      return
    }

    const editPertDialog = new WebviewWindow(`editPerturbation${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component-editor/perturbations-editor/edit-pert-dialog/edit-pert-dialog.html',
      title: `Edit perturbation (${perturbationData.id} / ${perturbationData.name})`,
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      height: 500,
      width: 400,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    this.dialogs[perturbationData.id] = editPertDialog

    void editPertDialog.once('loaded', () => {
      void editPertDialog.emit('edit_pert_update', {
        id: perturbationData.id,
        name: perturbationData.name,
        annotation: perturbationData.annotation
      })
    })
    void editPertDialog.once('edit_pert_dialog', (event: TauriEvent<{ id: string, name: string, annotation: string }>) => {
      this.dialogs[perturbationData.id] = undefined
      const index = this.contentData.perturbations.findIndex(p => p.id === perturbationData.id)
      if (index === -1) return
      const updatedPerturbation: IPerturbationData = {
        id: event.payload.id,
        name: event.payload.name,
        annotation: event.payload.annotation,
        perturbedVars: perturbationData.perturbedVars
      }
      this.changePerturbation(perturbationData.id, updatedPerturbation)
    })
    void editPertDialog.onCloseRequested(() => {
      this.dialogs[perturbationData.id] = undefined
    })
  }

  /** Propagate potential changes to perturbation (from edit dialog) to backend. */
  private changePerturbation (id: string, updatedPerturbation: IPerturbationData): void {
    const origPerturbation = this.contentData.perturbations.find(p => p.id === id)
    if (origPerturbation === undefined) return

    const perturbationData = convertFromIPerturbation(updatedPerturbation)

    if (origPerturbation.id !== perturbationData.id) {
      aeonState.sketch.perturbations.setPerturbationId(origPerturbation.id, perturbationData.id)
    }
    setTimeout(() => {
      aeonState.sketch.perturbations.setPerturbationContent(perturbationData.id, perturbationData)
    }, 50)
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
