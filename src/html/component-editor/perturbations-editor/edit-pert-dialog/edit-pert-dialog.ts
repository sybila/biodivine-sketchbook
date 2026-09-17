import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, query } from 'lit/decorators.js'
import style_less from './edit-pert-dialog.less?inline'
import { emit, type Event as TauriEvent, once } from '@tauri-apps/api/event'
import { appWindow, LogicalSize } from '@tauri-apps/api/window'
import { type } from '@tauri-apps/api/os'

@customElement('edit-pert-dialog')
export class EditPertDialog extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @query('#pert-name') nameField: HTMLInputElement | undefined
  @query('#pert-id') idField: HTMLInputElement | undefined
  @query('#pert-annotation') annotationField: HTMLTextAreaElement | undefined
  id = ''
  name = ''
  annotation = ''

  async firstUpdated (): Promise<void> {
    await once('edit_pert_update', (event: TauriEvent<{ id: string, name: string, annotation: string }>) => {
      this.id = event.payload.id
      this.name = event.payload.name
      this.annotation = event.payload.annotation
      if (this.idField !== undefined) this.idField.value = this.id
      if (this.nameField !== undefined) this.nameField.value = this.name
      if (this.annotationField !== undefined) this.annotationField.value = this.annotation
    })
    await emit('loaded', {})
    this.idField?.focus()
    const measuredWidth = document.querySelector('body')?.offsetWidth ?? 500
    let measuredHeight = (document.querySelector('body')?.offsetHeight ?? 300)
    if (await type() === 'Darwin') {
      measuredHeight += 40
    }
    await appWindow.setSize(new LogicalSize(measuredWidth, measuredHeight))
  }

  private async handleSubmit (event: Event): Promise<void> {
    event.preventDefault()
    if (this.id === '' || this.name === '') {
      this.nameField?.classList.remove('uk-form-danger')
      this.idField?.classList.remove('uk-form-danger')
      if (this.id === '') {
        this.idField?.classList.add('uk-form-danger')
      }
      if (this.name === '') {
        this.nameField?.classList.add('uk-form-danger')
      }
      return
    }
    await emit('edit_pert_dialog', {
      id: this.id,
      name: this.name,
      annotation: this.annotation
    })
    await appWindow.close()
  }

  private handleIdUpdate (e: Event): void {
    this.id = (e.target as HTMLInputElement).value
  }

  private handleNameUpdate (e: Event): void {
    this.name = (e.target as HTMLInputElement).value
  }

  private handleAnnotationUpdate (e: Event): void {
    this.annotation = (e.target as HTMLTextAreaElement).value
  }

  render (): TemplateResult {
    return html`
      <div class="uk-container">
        <form class="uk-form-horizontal">
          <div class="uk-margin-small uk-margin-small-top">
            <label class="uk-form-label" for="pert-id">Perturbation ID</label>
            <div class="uk-form-controls">
              <input class="uk-input" @input="${this.handleIdUpdate}" id="pert-id" type="text" placeholder="ID" />
            </div>
          </div>
          <div class="uk-margin-small">
            <label class="uk-form-label" for="pert-name">Perturbation Name</label>
            <div class="uk-flex uk-flex-row">
              <input class="uk-input" @input="${this.handleNameUpdate}" id="pert-name" type="text" placeholder="Name" />
            </div>
          </div>
          <div class="uk-margin-small">
            <label class="uk-form-label" for="pert-annotation">Perturbation Annotation</label>
            <div class="uk-flex uk-flex-row">
              <textarea class="uk-textarea" @input="${this.handleAnnotationUpdate}" id="pert-annotation" placeholder="Annotation"></textarea>
            </div>
          </div>
          <button class="uk-button uk-button-primary uk-width-1-1" @click="${this.handleSubmit}">Submit</button>
        </form>
      </div>
    `
  }
}
