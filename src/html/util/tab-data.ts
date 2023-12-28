import { Data } from 'dataclass'
import { html, type TemplateResult } from 'lit'
import { type IRegulationData, type IVariableData } from './data-interfaces'

export class TabData extends Data {
  id: number = -1
  name: string = 'unknown'
  pinned: boolean = false
  content: (contentData: ContentData) => TemplateResult<1> = () => html`unknown`
  active: boolean = false
  icon: string = 'question'
}

export class ContentData extends Data {
  variables: IVariableData[] = []
  regulations: IRegulationData[] = []
}
