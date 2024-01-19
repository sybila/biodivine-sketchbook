import { type Position } from 'cytoscape'
import { type Monotonicity } from '../component/regulations-editor/element-type'

export interface IVariableData {
  id: string
  name: string
  position: Position
  function: string
}

export interface IRegulationData {
  id: string
  source: string
  target: string
  observable: boolean
  monotonicity: Monotonicity
}
