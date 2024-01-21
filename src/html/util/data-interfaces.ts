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

// TODO: add 'Dual' option
export enum Monotonicity {
  UNSPECIFIED = 'Unknown',
  ACTIVATION = 'Activation',
  INHIBITION = 'Inhibition',
}

export interface IRegulationData {
  id: string
  source: string
  target: string
  // TODO: add 'Observability' enum with three options instead of using bool
  observable: boolean
  monotonicity: Monotonicity
}
