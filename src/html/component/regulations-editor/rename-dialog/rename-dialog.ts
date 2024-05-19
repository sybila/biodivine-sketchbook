import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, query } from 'lit/decorators.js'
import style_less from './rename-dialog.less?inline'
import { emit, type Event as TauriEvent, once } from '@tauri-apps/api/event'
import { appWindow, LogicalSize } from '@tauri-apps/api/window'
import { type } from '@tauri-apps/api/os';

@customElement('rename-dialog')
export class RenameDialog extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @query('#node-name') nameField: HTMLInputElement | undefined
  @query('#node-id') variableIdField: HTMLInputElement | undefined
  variableId = ''
  name = ''

  async firstUpdated (): Promise<void> {
    await once('edit_node_update', (event: TauriEvent<{ id: string, name: string }>) => {
      this.variableId = event.payload.id
      this.name = event.payload.name
      if (this.variableIdField !== undefined) this.variableIdField.value = this.variableId
      if (this.nameField !== undefined) this.nameField.value = this.name
    })
    await emit('loaded', {})
    this.variableIdField?.focus()    
    let measured_width = document.querySelector('body')?.offsetWidth ?? 400
    let measured_height = (document.querySelector('body')?.offsetHeight ?? 200)
    if (await type() == 'Darwin') {
      // Currently, it seems that setSize includes the size of the title bar, but there
      // is no exact way to actually measure the title bar, so we just add 40 to the height.
      // Furthermore, this seems to be a difference in how window size is treated 
      // between macOS and Windows/Linux.
      // 
      // See also: https://github.com/tauri-apps/tauri/issues/6333
      measured_height += 40
    }
    await appWindow.setSize(new LogicalSize(measured_width, measured_height))
  }

  private async handleSubmit (event: Event): Promise<void> {
    event.preventDefault()
    if (this.variableId === '' || this.name === '') {
      this.nameField?.classList.remove('uk-form-danger')
      this.variableIdField?.classList.remove('uk-form-danger')
      if (this.variableId === '') {
        this.variableIdField?.classList.add('uk-form-danger')
      }
      if (this.name === '') {
        this.nameField?.classList.add('uk-form-danger')
      }

      return
    }
    await emit('edit_node_dialog', {
      id: this.variableId,
      name: this.name
    })
    await appWindow.close()
  }

  private handleIdUpdate (e: Event): void {
    this.variableId = (e.target as HTMLInputElement).value
  }

  private handleNameUpdate (e: Event): void {
    this.name = (e.target as HTMLInputElement).value
  }

  render (): TemplateResult {
    return html`
      <div class="uk-container">
        <form class="uk-form-horizontal">
          <div class="uk-margin-small uk-margin-small-top">
            <label class="uk-form-label" for="form-horizontal-text">Node ID</label>
            <div class="uk-form-controls">
              <input class="uk-input" @input="${this.handleIdUpdate}" id="node-id" type="text" placeholder="ID" />
            </div>
          </div>
          <div class="uk-margin-small">
            <label class="uk-form-label" for="form-horizontal-text">Node Name</label>
            <div class="uk-flex uk-flex-row">
              <input class="uk-input" @input="${this.handleNameUpdate}" id="node-name" type="text" placeholder="Name" />
            </div>
          </div>
          <button class="uk-button uk-button-primary uk-width-1-1" @click="${this.handleSubmit}">Submit</button>
        </form>
      </div>
    `
  }
}
