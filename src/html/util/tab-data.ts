import { Data } from 'dataclass'
import { html, type TemplateResult } from 'lit'

export class TabData extends Data {
  id: number = -1
  name: string = 'unknown'
  pinned: boolean = false
  data: TemplateResult<1> = html`unknown`
  active: boolean = false
  icon: string = 'question'
}
