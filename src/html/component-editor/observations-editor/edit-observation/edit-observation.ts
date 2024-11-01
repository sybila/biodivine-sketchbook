import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, query, state } from 'lit/decorators.js'
import style_less from './edit-observation.less?inline'
import { emit, type Event as TauriEvent, once } from '@tauri-apps/api/event'
import { appWindow } from '@tauri-apps/api/window'
import { map } from 'lit/directives/map.js'
import { type IObservation } from '../../../util/data-interfaces'

@customElement('edit-observation')
export default class EditObservation extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @query('#node-id') idField: HTMLInputElement | undefined
  @state() data: IObservation | undefined
  id = ''

  async firstUpdated (): Promise<void> {
    await once('edit_observation_update', (event: TauriEvent<IObservation>) => {
      this.id = event.payload.id
      this.data = event.payload
    })
    await emit('loaded', {})
    this.idField?.focus()
  }

  private async handleSubmit (event: Event): Promise<void> {
    event.preventDefault()
    await emit('edit_observation_dialog', {
      id: this.id,
      data: this.data
    })
    await appWindow.close()
  }

  private getValue<T>(data: T, key: string): T[keyof T] {
    const newKey = key as keyof typeof data
    return data[newKey]
  }

  private setValue<T>(data: T, key: string, value: string): void {
    const newKey = key as keyof typeof data
    data[newKey] = value as T[keyof T]
  }

  render (): TemplateResult {
    return html`
      <div class="uk-container uk-margin-medium-bottom">
        <form class="uk-form-horizontal uk-flex uk-flex-column uk-flex-between">
          <div class="fields">
            ${map(Object.keys(this.data ?? {}).filter(key => key !== 'index' && key !== 'selected'), (key) => {
              return html`
            <div class="uk-margin-small">
              <label class="uk-form-label uk-text-bold" for="form-horizontal-text">${key.toUpperCase()}</label>
              <div class="uk-form-controls">
                ${key === 'annotation'
                  ? html`<textarea class="uk-textarea" .value="${this.getValue(this.data, key)}" @input="${(e: InputEvent) => { this.setValue(this.data, key, (e.target as HTMLTextAreaElement).value) }}" id="node-${key}" placeholder="${key}"></textarea>`
                  : html`<input class="uk-input" .value="${this.getValue(this.data, key)}" @input="${(e: InputEvent) => { this.setValue(this.data, key, (e.target as HTMLInputElement).value) }}" id="node-${key}" type="text" placeholder="${key}"/>`}
              </div>
            </div>`
            })}
          </div>
          <button class="uk-button uk-button-primary uk-width-1-1 uk-margin-small-top" @click="${this.handleSubmit}">Save</button>
        </form>
      </div>
    `
  }
}
