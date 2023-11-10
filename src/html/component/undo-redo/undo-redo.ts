import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement } from 'lit/decorators.js'
import style_less from './undo-redo.less?inline'
import { faArrowLeft, faArrowRight } from '@fortawesome/free-solid-svg-icons'
import { findIconDefinition, icon, library } from '@fortawesome/fontawesome-svg-core'
library.add(faArrowLeft, faArrowRight)

@customElement('undo-redo')
class UndoRedo extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  render (): TemplateResult {
    return html`
      <div class="undo-redo uk-flex-nowrap">
        <button class="uk-button uk-button-secondary uk-button-small">${icon(findIconDefinition({ prefix: 'fas', iconName: 'arrow-left' })).node}</button>
        <button class="uk-button uk-button-secondary uk-button-small" disabled>${icon(findIconDefinition({ prefix: 'fas', iconName: 'arrow-right' })).node}</button>
      </div>
    `
  }
}
