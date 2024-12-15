import { type Position } from 'cytoscape'
import { Data } from 'dataclass'

/** Internally used structure to represent network variables. */
export interface IVariableData {
  id: string
  name: string
  annotation: string
  function: string
}

/** Internally used structure to represent type of entity in the regulation graph. */
export enum ElementType {
  NONE,
  EDGE,
  NODE
}

/** Enum of regulation monotonicity values. */
export enum Monotonicity {
  UNSPECIFIED = 'Unknown',
  ACTIVATION = 'Activation',
  INHIBITION = 'Inhibition',
  DUAL = 'Dual'
}

/** Enum of regulation essentiality values. */
export enum Essentiality {
  FALSE = 'False',
  TRUE = 'True',
  UNKNOWN = 'Unknown'
}

/** Internally used structure to represent regulations between variables. */
export interface IRegulationData {
  id: string
  source: string
  target: string
  essential: Essentiality
  monotonicity: Monotonicity
}

/** Structure mapping variables to their layout position. */
export type ILayoutData = Map<string, Position>

/**
 * Structure encompassing all the most important content data enconpassing
 * the whole sketch. This is propagated through the web-component hierarchy.
 */
export class ContentData extends Data {
  variables: IVariableData[] = []
  functions: IFunctionData[] = []
  layout: ILayoutData = new Map()
  regulations: IRegulationData[] = []
  observations: IObservationSet[] = []
  dynamicProperties: DynamicProperty[] = []
  staticProperties: StaticProperty[] = []
  annotation: string = ''
}

/** Internally used structure to represent update and uninterpreted functions. */
export interface IFunctionData {
  id: string
  name: string
  annotation: string
  function: string
  variables: IRegulationData[]
}

/** Internally used structure to represent observations. */
export interface IObservation {
  selected: boolean
  id: string
  name: string
  annotation: string

  [key: string]: string | number | boolean
}

/** Internally used structure to represent datasets (sets of observations). */
export interface IObservationSet {
  id: string
  name: string
  annotation: string
  observations: IObservation[]
  variables: string[]
}

/** Enum representing all supported types of static properties. */
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

/** Enum representing all supported types of dynamic properties. */
export enum DynamicPropertyType {
  Generic = 'GenericDynProp',
  FixedPoint = 'ExistsFixedPoint',
  TrapSpace = 'ExistsTrapSpace',
  ExistsTrajectory = 'ExistsTrajectory',
  AttractorCount = 'AttractorCount',
  HasAttractor = 'HasAttractor'
}

/** Typesafe representation of property template types. */
export type PropertyType = StaticPropertyType | DynamicPropertyType

/** Internally used structure to represent properties. */
export interface IProperty {
  id: string
  name: string
  annotation: string
  variant: PropertyType
}

/** Template dynamic property for fixed point existence. */
export interface IFixedPointDynamicProperty extends IProperty {
  dataset: string | null
  observation: string | null
}

/** Template dynamic property for trap space existence. */
export interface ITrapSpaceDynamicProperty extends IProperty {
  dataset: string | null
  observation: string | null
  minimal: boolean
  nonpercolable: boolean
}

/** Template dynamic property for trajectory existence. */
export interface IExistsTrajectoryDynamicProperty extends IProperty {
  dataset: string | null
}

/** Template dynamic property for attractor count. */
export interface IAttractorCountDynamicProperty extends IProperty {
  minimal: number
  maximal: number
}

/** Template dynamic property for attractor existence. */
export interface IHasAttractorDynamicProperty extends IProperty {
  dataset: string | null
  observation: string | null
}

/** Generic dynamic property given by an HCTL formula. */
export interface IGenericDynamicProperty extends IProperty {
  formula: string
}

/** Internally used structure to represent dynamic properties. */
export type DynamicProperty =
  IFixedPointDynamicProperty
  | ITrapSpaceDynamicProperty
  | IExistsTrajectoryDynamicProperty
  | IAttractorCountDynamicProperty
  | IHasAttractorDynamicProperty
  | IGenericDynamicProperty

/** Template static property for essentiality of a function's input. */
export interface IFunctionInputEssentialStaticProperty extends IProperty {
  input: string | null
  target: string | null
  value: Essentiality
  context: string | undefined
}

/** Template static property for monotonicity of a function's input. */
export interface IFunctionInputMonotonicStaticProperty extends IProperty {
  input: string | null
  target: string | null
  value: Monotonicity
  context: string | undefined
}

/** Generic static property given by a FOL formula. */
export interface IGenericStaticProperty extends IProperty {
  formula: string
}

/** Template static property for monotonicity of a regulation. */
export interface IVariableRegulatorMonotonicStaticProperty extends IProperty {
  input: string | null
  target: string | null
  value: Monotonicity
  context: string | undefined
}

/** Template static property for essentiality of a regulation. */
export interface IVariableRegulatorEssentialStaticProperty extends IProperty {
  input: string | null
  target: string | null
  value: Essentiality
  context: string | undefined
}

/** Internally used structure to represent static properties. */
export type StaticProperty =
  IFunctionInputEssentialStaticProperty
  | IFunctionInputMonotonicStaticProperty
  | IVariableRegulatorMonotonicStaticProperty
  | IVariableRegulatorEssentialStaticProperty
  | IGenericStaticProperty
