import { Data } from 'dataclass'
import { html, type TemplateResult } from 'lit'
import { type ContentData } from './data-interfaces'

export class TabData extends Data {
  id: number = -1
  name: string = 'unknown'
  pinned: boolean = false
  content: (contentData: ContentData) => TemplateResult<1> = () => html`unknown`
  active: boolean = false
  icon: string = 'question'
}
