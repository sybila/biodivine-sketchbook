import { type Position } from 'cytoscape'
import { type Monotonicity } from './element-type'

export interface INodeData {
  id: string
  name: string
  position: Position
}

export interface IEdgeData {
  id: string
  source: string
  target: string
  observable: boolean
  monotonicity: Monotonicity
}
