import { Data } from 'dataclass'
import { html, type TemplateResult } from 'lit'
import { type IEdgeData, type INodeData } from '../component/regulations-editor/graph-interfaces'

export class TabData extends Data {
  id: number = -1
  name: string = 'unknown'
  pinned: boolean = false
  content: (contentData: ContentData) => TemplateResult<1> = () => html`unknown`
  active: boolean = false
  icon: string = 'question'
}

export class ContentData extends Data {
  nodes: INodeData[] = []
  edges: IEdgeData[] = []
}
