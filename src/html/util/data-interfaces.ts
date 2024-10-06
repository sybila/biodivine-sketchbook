import { type Position } from 'cytoscape'
import { Data } from 'dataclass'

export interface IVariableData {
  id: string
  name: string
  annotation: string
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
  essential: Essentiality
  monotonicity: Monotonicity
}

export type ILayoutData = Map<string, Position>

export class ContentData extends Data {
  variables: IVariableData[] = []
  functions: IFunctionData[] = []
  layout: ILayoutData = new Map()
  regulations: IRegulationData[] = []
  observations: IObservationSet[] = []
  dynamicProperties: DynamicProperty[] = []
  staticProperties: StaticProperty[] = []
}

export interface IFunctionData {
  id: string
  function: string
  variables: IRegulationData[]
}

export interface IObservation {
  selected: boolean
  id: string
  name: string
  annotation: string

  [key: string]: string | number | boolean
}

export interface IObservationSet {
  id: string
  name: string
  annotation: string
  observations: IObservation[]
  variables: string[]
}

export enum StaticPropertyType {
  Generic = 'GenericStatProp',
  FunctionInputEssential = 'FnInputEssential',
  FunctionInputEssentialWithCondition = 'FnInputEssentialContext',
  VariableRegulationEssential = 'RegulationEssential',
  VariableRegulationEssentialWithCondition = 'RegulationEssentialContext',
  FunctionInputMonotonic = 'FnInputMonotonic',
  FunctionInputMonotonicWithCondition = 'FnInputMonotonicContext',
  VariableRegulationMonotonic = 'RegulationMonotonic',
  VariableRegulationMonotonicWithCondition = 'RegulationMonotonicContext'
}

export enum DynamicPropertyType {
  Generic = 'GenericDynProp',
  FixedPoint = 'ExistsFixedPoint',
  TrapSpace = 'ExistsTrapSpace',
  ExistsTrajectory = 'ExistsTrajectory',
  AttractorCount = 'AttractorCount',
  HasAttractor = 'HasAttractor'
}

export type PropertyType = StaticPropertyType | DynamicPropertyType

export interface IProperty {
  id: string
  name: string
  annotation: string
  variant: PropertyType
}

export interface IFixedPointDynamicProperty extends IProperty {
  dataset: string | null
  observation: string | null
}

export interface ITrapSpaceDynamicProperty extends IProperty {
  dataset: string | null
  observation: string | null
  minimal: boolean
  nonpercolable: boolean
}

export interface IExistsTrajectoryDynamicProperty extends IProperty {
  dataset: string | null
}

export interface IAttractorCountDynamicProperty extends IProperty {
  minimal: number
  maximal: number
}

export interface IHasAttractorDynamicProperty extends IProperty {
  dataset: string | null
  observation: string | null
}

export interface IGenericDynamicProperty extends IProperty {
  formula: string
}

export type DynamicProperty =
  IFixedPointDynamicProperty
  | ITrapSpaceDynamicProperty
  | IExistsTrajectoryDynamicProperty
  | IAttractorCountDynamicProperty
  | IHasAttractorDynamicProperty
  | IGenericDynamicProperty

export interface IFunctionInputEssentialStaticProperty extends IProperty {
  input: string | null
  target: string | null
  value: Essentiality
  context: string | undefined
}

export interface IFunctionInputMonotonicStaticProperty extends IProperty {
  input: string | null
  target: string | null
  value: Monotonicity
  context: string | undefined
}

export interface IGenericStaticProperty extends IProperty {
  formula: string
}

export interface IVariableRegulatorMonotonicStaticProperty extends IProperty {
  input: string | null
  target: string | null
  value: Monotonicity
  context: string | undefined
}

export interface IVariableRegulatorEssentialStaticProperty extends IProperty {
  input: string | null
  target: string | null
  value: Essentiality
  context: string | undefined
}

export type StaticProperty =
  IFunctionInputEssentialStaticProperty
  | IFunctionInputMonotonicStaticProperty
  | IVariableRegulatorMonotonicStaticProperty
  | IVariableRegulatorEssentialStaticProperty
  | IGenericStaticProperty
