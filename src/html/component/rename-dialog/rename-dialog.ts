import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, query } from 'lit/decorators.js'
import style_less from './rename-dialog.less?inline'
import { emit, listen, type Event as TauriEvent } from '@tauri-apps/api/event'
import { appWindow } from '@tauri-apps/api/window'

@customElement('rename-dialog')
class RenameDialog extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @query('#name') nameField: HTMLInputElement | undefined
  @query('#nodeID') nodeIdField: HTMLInputElement | undefined
  nodeId = ''
  name = ''

  async connectedCallback (): Promise<void> {
    super.connectedCallback()

    await listen('edit_node_update', (event: TauriEvent<{ id: string, name: string }>) => {
      if (this.nodeIdField !== undefined) this.nodeIdField.value = event.payload.id
      if (this.nameField !== undefined) this.nameField.value = event.payload.name
    })
  }

  private async sendToParent (): Promise<void> {
    await emit('edit_node_dialog', {
      id: this.nodeId,
      name: this.name
    })
    await appWindow.close()
  }

  private handleIdUpdate (e: Event): void {
    this.nodeId = (e.target as HTMLInputElement).value
  }

  private handleNameUpdate (e: Event): void {
    this.name = (e.target as HTMLInputElement).value
  }

  render (): TemplateResult {
    return html`
        <div class="uk-flex uk-flex-column">
            <div class="uk-h2">Edit node</div>
            <input class="uk-input" @input="${this.handleIdUpdate}" id="nodeID" type="text" placeholder="ID" value="" />
            <input class="uk-input" @input="${this.handleNameUpdate}" id="name" type="text" placeholder="Name" value="" />
            <button class="uk-button" @click="${this.sendToParent}">Submit</button>
        </div>
    `
  }
}
