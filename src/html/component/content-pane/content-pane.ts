import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './content-pane.less?inline'
import { ContentData, TabData } from '../../util/tab-data'
import { library, icon } from '@fortawesome/fontawesome-svg-core'
import '../regulations-editor/regulations-editor'
import '../functions-editor/functions-editor'
import { faLock, faLockOpen } from '@fortawesome/free-solid-svg-icons'
import { aeonState } from '../../../aeon_events'

library.add(faLock, faLockOpen)

@customElement('content-pane')
export class ContentPane extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  @property() private readonly tab: TabData = TabData.create()
  @property() private readonly data = ContentData.create()

  private pin (): void {
    if (this.tab.pinned) {
      aeonState.tab_bar.unpin(this.tab.id)
    } else {
      aeonState.tab_bar.pin(this.tab.id)
    }
  }

  protected render (): TemplateResult {
    return html`
      <div class="content-pane uk-container uk-container-expand">
        <button class="uk-button uk-button-small uk-button-secondary pin-button" @click="${this.pin}">
          ${this.tab.pinned ? icon(faLock).node : icon(faLockOpen).node}
        </button>
        ${this.tab.content(this.data)}
      </div>
    `
  }
}
