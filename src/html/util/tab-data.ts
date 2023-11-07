import { Data } from 'dataclass'

export class TabData extends Data {
  id: number = -1
  name: string = 'unknown'
  pinned: boolean = false
  data: string = 'unknown'
  active: boolean = false
}
