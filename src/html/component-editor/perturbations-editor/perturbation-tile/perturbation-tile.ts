import { LitElement, html, css, unsafeCSS, type TemplateResult } from 'lit'
import { property, customElement, state } from 'lit/decorators.js'
import { map } from 'lit/directives/map.js'
import style_less from './perturbation-tile.less?inline'
import { type IVariableData, type IPerturbationData } from '../../../util/data-interfaces'
import { type PerturbationData } from '../../../../aeon_state'
import { functionDebounceTimer } from '../../../util/config'
import { debounce } from 'lodash'
import { icon, library } from '@fortawesome/fontawesome-svg-core'
import { faTrash, faPlus, faEdit } from '@fortawesome/free-solid-svg-icons'

library.add(faTrash, faPlus, faEdit)

@customElement('perturbation-tile')
export class PerturbationTile extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() perturbation?: IPerturbationData
  @property() variables: IVariableData[] = []
  @state() private isAddingVariable = false
  @state() private selectedVariableId = ''

  private getVariableText (variableId: string): string {
    const variable = this.variables.find(variable => variable.id === variableId)
    if (variable === undefined) return ''
    if (variable.id === variable.name) {
      return variable.id
    }
    return variable.name + ' (' + variable.id + ')'
  }

  private convertToPerturbationData (perturbation: IPerturbationData): PerturbationData {
    return {
      id: perturbation.id,
      name: perturbation.name,
      annotation: perturbation.annotation,
      perturbed_vars: Array.from(perturbation.perturbedVars.entries())
    }
  }

  private updatePerturbation (perturbation: IPerturbationData): void {
    this.dispatchEvent(new CustomEvent('set-perturbation-content', {
      detail: {
        id: this.perturbation?.id,
        perturbation: this.convertToPerturbationData(perturbation)
      },
      bubbles: true,
      composed: true
    }))
  }

  nameUpdated = debounce((name: string) => {
    if (this.perturbation === undefined) return
    const updatedPerturbation: IPerturbationData = {
      ...this.perturbation,
      name
    }
    this.updatePerturbation(updatedPerturbation)
  }, functionDebounceTimer
  )

  idUpdated = debounce((id: string) => {
    if (this.perturbation === undefined) return
    this.dispatchEvent(new CustomEvent('set-perturbation-id', {
      detail: {
        oldId: this.perturbation.id,
        newId: id
      },
      bubbles: true,
      composed: true
    }))
  }, functionDebounceTimer
  )

  removePerturbation (): void {
    this.dispatchEvent(new CustomEvent('remove-perturbation', {
      detail: { id: this.perturbation?.id },
      bubbles: true,
      composed: true
    }))
  }

  editPerturbation (): void {
    this.dispatchEvent(new CustomEvent('edit-perturbation', {
      detail: { id: this.perturbation?.id },
      bubbles: true,
      composed: true
    }))
  }

  togglePerturbedVariableState (varId: string): void {
    if (this.perturbation === undefined) return
    const updatedPerturbation = structuredClone(this.perturbation)
    const currentValue = updatedPerturbation.perturbedVars.get(varId)
    updatedPerturbation.perturbedVars.set(varId, currentValue !== true)
    this.updatePerturbation(updatedPerturbation)
  }

  removePerturbedVariable (varId: string): void {
    if (this.perturbation === undefined) return
    const updatedPerturbation = structuredClone(this.perturbation)
    updatedPerturbation.perturbedVars.delete(varId)
    this.updatePerturbation(updatedPerturbation)
  }

  private getAvailableVariables (): IVariableData[] {
    const perturbedIds = this.getPerturbedVariableIds()
    return this.variables.filter(variable => !perturbedIds.has(variable.id))
  }

  showAddVariableSelector (): void {
    const variable = this.getAvailableVariables()[0]
    if (variable === undefined) return
    this.selectedVariableId = variable.id
    this.isAddingVariable = true
  }

  cancelAddVariable (): void {
    this.isAddingVariable = false
    this.selectedVariableId = ''
  }

  addPerturbedVariable (): void {
    if (this.perturbation === undefined) return
    const availableVariables = this.getAvailableVariables()
    const variable = availableVariables.find(variable => variable.id === this.selectedVariableId) ?? availableVariables[0]
    if (variable === undefined) return

    const updatedPerturbation = structuredClone(this.perturbation)
    updatedPerturbation.perturbedVars.set(variable.id, true)
    this.cancelAddVariable()
    this.updatePerturbation(updatedPerturbation)
  }

  private getPerturbedVariableIds (): Set<string> {
    return new Set(this.perturbation?.perturbedVars?.keys() ?? [])
  }

  render (): TemplateResult {
    if (this.perturbation === null || this.perturbation === undefined) {
      return html`<div class="uk-text-muted uk-text-small">Loading...</div>`
    }

    return html`
      <div class="container uk-flex uk-flex-column uk-margin-small-bottom">
        <div class="perturbation-nameplate uk-flex uk-flex-row uk-flex-bottom uk-width-auto uk-margin-small-bottom">
          <div class="uk-flex uk-flex-column id-section">
            <label class="uk-form-label" for="id-field">ID</label>
            <input id="id-field" class="name-field property-id-field" .value="${this.perturbation.id}"
                   @input="${(e: InputEvent) => this.idUpdated((e.target as HTMLInputElement).value)}"/>
          </div>
          <div class="uk-flex uk-flex-column name-section">
            <label class="uk-form-label" for="name-field">NAME</label>
            <input id="name-field" class="name-field property-name-field" .value="${this.perturbation.name}"
                   @input="${(e: InputEvent) => this.nameUpdated((e.target as HTMLInputElement).value)}"/>
          </div>
          <button class="property-button uk-button uk-button-secondary uk-button-small" @click="${this.editPerturbation}">
            ${icon(faEdit).node}
          </button>
          <button class="property-button uk-button uk-button-secondary uk-button-small" @click="${this.removePerturbation}">
            ${icon(faTrash).node}
          </button>
        </div>

        <span class="uk-text-left uk-margin-small-top">Perturbed Variables:</span>

        <div class="perturbed-variables-list uk-margin-small-top uk-margin-small-bottom">
          ${this.perturbation.perturbedVars.size === 0
            ? html`<div class="uk-text-muted uk-text-small uk-margin-small-bottom">No perturbed variables selected.</div>`
            : map(this.perturbation.perturbedVars, ([varId, value]) => this.renderPerturbedVariableRow(varId, value))}
          ${this.renderAddVariableButton()}
        </div>

      </div>
      <hr class="uk-margin-top uk-margin-bottom uk-margin-left uk-margin-right">
    `
  }

  private renderPerturbedVariableRow (varId: string, value: boolean | undefined): TemplateResult {
    return html`
      <div class="perturbed-variable-row uk-flex uk-flex-row uk-flex-middle uk-margin-small-bottom">
        <div class="perturbed-variable-id uk-width-medium uk-text-truncate">
          ${this.getVariableText(varId)}
        </div>
        <div class="uk-margin-small-left uk-width-small">
          <span class="perturbation-value ${value === false ? 'perturbation-value-false' : 'perturbation-value-true'}"
                @click=${() => { this.togglePerturbedVariableState(varId) }}>
            ${value === false ? 'False' : 'True'}
          </span>
        </div>
        <div class="uk-margin-small-left">
          <button class="uk-button uk-button-danger uk-button-small" @click=${() => { this.removePerturbedVariable(varId) }}>
            ${icon(faTrash).node}
          </button>
        </div>
      </div>
    `
  }

  private renderAddVariableButton (): TemplateResult {
    const availableVariables = this.getAvailableVariables()
    const hasAvailableVariables = availableVariables.length > 0

    if (this.isAddingVariable && hasAvailableVariables) {
      return html`
        <div class="add-variable-row uk-flex uk-flex-row uk-flex-middle">
          <div class="uk-width-medium">
            <select class="uk-select uk-form-small"
                    .value=${this.selectedVariableId}
                    @change=${(e: Event) => {
                      this.selectedVariableId = (e.target as HTMLSelectElement).value
                    }}>
              ${map(availableVariables, (variable) => html`
                <option value="${variable.id}">${this.getVariableText(variable.id)}</option>
              `)}
            </select>
          </div>
          <button class="uk-button uk-button-secondary uk-button-small uk-margin-small-left"
                  @click=${this.addPerturbedVariable}>
            ${icon(faPlus).node}
            Add
          </button>
          <button class="uk-button uk-button-secondary uk-button-small uk-margin-small-left"
                  @click=${this.cancelAddVariable}>
            Cancel
          </button>
        </div>
      `
    }

    return html`
      <div class="add-variable-row">
        <button class="uk-button uk-button-secondary uk-button-small"
                ?disabled=${!hasAvailableVariables}
                @click=${this.showAddVariableSelector}>
          ${icon(faPlus).node}
          Add variable
        </button>
      </div>
    `
  }
}
