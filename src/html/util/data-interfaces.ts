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

export enum Essentiality {
  FALSE = 'False',
  TRUE = 'True',
  UNKNOWN = 'Unknown'
}

export interface IRegulationData {
  id: string
  source: string
  target: string
  // TODO: add 'Observability' enum with three options instead of using bool
  essential: Essentiality
  monotonicity: Monotonicity
}

export type ILayoutData = Map<string, Position>

export class ContentData extends Data {
  variables: IVariableData[] = []
  layout: ILayoutData = new Map()
  regulations: IRegulationData[] = []
}
