import { type Position } from 'cytoscape'
import { Data } from 'dataclass'

export interface IVariableData {
  id: string
  name: string
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
  DUAL = 'Dual'
}

export interface IRegulationData {
  id: string
  source: string
  target: string
  // TODO: add 'Observability' enum with three options instead of using bool
  observable: boolean
  monotonicity: Monotonicity
}

export type ILayoutData = Record<string, Position>

export class ContentData extends Data {
  variables: IVariableData[] = []
  layout: ILayoutData = {}
  regulations: IRegulationData[] = []
}
