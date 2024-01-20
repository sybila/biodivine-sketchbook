import { type Position } from 'cytoscape'

export interface IVariableData {
  id: string
  name: string
  position: Position
  function: string
}

export enum ElementType {
  NONE,
  EDGE,
  NODE
}

export enum Monotonicity {
  UNSPECIFIED = 'Unknown',
  ACTIVATION = 'Activation',
  INHIBITION = 'Inhibition',
}

export interface IRegulationData {
  id: string
  source: string
  target: string
  observable: boolean
  monotonicity: Monotonicity
}
