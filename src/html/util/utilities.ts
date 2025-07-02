import {
  Essentiality, Monotonicity, type IFunctionData,
  type IObservationSet, type IObservation, type ILayoutData,
  type IRegulationData, type IVariableData,
  type PropertyType,
  StaticPropertyType,
  DynamicPropertyType
} from './data-interfaces'
import {
  type UninterpretedFnData, type VariableData, type ObservationData,
  type DatasetData, type RegulationData, type LayoutNodeData
} from '../../aeon_state'

/** Toggling essentiality. */
export function getNextEssentiality (essentiality: Essentiality): Essentiality {
  switch (essentiality) {
    case Essentiality.FALSE:
      return Essentiality.TRUE
    case Essentiality.TRUE:
      return Essentiality.UNKNOWN
    default:
      return Essentiality.FALSE
  }
}

/** Get text describing each essentiality class. */
export function getEssentialityText (essentiality: Essentiality): string {
  switch (essentiality) {
    case Essentiality.FALSE:
      return 'non-essential'
    case Essentiality.TRUE:
      return 'essential'
    default:
      return 'unknown'
  }
}

/** Toggling monotonicity. */
export function getNextMonotonicity (monotonicity: Monotonicity): Monotonicity {
  switch (monotonicity) {
    case Monotonicity.ACTIVATION:
      return Monotonicity.INHIBITION
    case Monotonicity.INHIBITION:
      return Monotonicity.DUAL
    case Monotonicity.DUAL:
      return Monotonicity.UNSPECIFIED
    default:
      return Monotonicity.ACTIVATION
  }
}

/** Get text describing each monotonicity class. */
export function getMonotonicityClass (monotonicity: Monotonicity): string {
  switch (monotonicity) {
    case Monotonicity.INHIBITION:
      return 'monotonicity-inhibition'
    case Monotonicity.ACTIVATION:
      return 'monotonicity-activation'
    case Monotonicity.DUAL:
      return 'monotonicity-dual'
    case Monotonicity.UNSPECIFIED:
      return 'monotonicity-unspecified'
    default:
      return ''
  }
}

/** Convert UninterpretedFnData instance into internally used IFunction. */
export function convertToIFunction (fnData: UninterpretedFnData): IFunctionData {
  const variables = fnData.arguments.map(
    (arg, index) => {
      return {
        id: index.toString(),
        source: 'var' + index.toString(),
        target: fnData.id,
        monotonicity: arg[0],
        essential: arg[1]
      }
    })
  return {
    id: fnData.id,
    name: fnData.name,
    annotation: fnData.annotation,
    function: fnData.expression,
    variables
  }
}

/** Convert internally used IFunctionData instance into UninterpretedFnData (used by backend). */
export function convertFromIFunction (iFunction: IFunctionData): UninterpretedFnData {
  const fnArguments = iFunction.variables.map(varData => {
    return [varData.monotonicity, varData.essential] as [Monotonicity, Essentiality]
  })
  return {
    id: iFunction.id,
    name: iFunction.name,
    annotation: iFunction.annotation,
    arguments: fnArguments,
    expression: iFunction.function
  }
}

/** Convert ObservationData instance into internally used IObservation. */
export function convertToIObservation (observationData: ObservationData, variables: string[]): IObservation {
  const obs: IObservation = {
    id: observationData.id,
    name: observationData.name,
    annotation: observationData.annotation,
    selected: false
  }
  variables.forEach(((v, idx) => {
    const value = observationData.values[idx]
    obs[v] = (value === '*') ? '' : value
  }))
  return obs
}

/** Convert internally used IObservation instance into ObservationData (used by backend). */
export function convertFromIObservation (
  observation: IObservation,
  datasetId: string,
  variables: string[]
): ObservationData {
  const valueString = variables.map(v => {
    return (observation[v] === '') ? '*' : observation[v]
  }).join('')
  return {
    id: observation.id,
    name: observation.name,
    annotation: observation.annotation,
    dataset: datasetId,
    values: valueString
  }
}

/** Convert DatasetData instance into internally used IObservationSet. */
export function convertToIObservationSet (datasetData: DatasetData): IObservationSet {
  const observations = datasetData.observations.map(
    observationData => convertToIObservation(observationData, datasetData.variables)
  )
  return {
    id: datasetData.id,
    name: datasetData.name,
    annotation: datasetData.annotation,
    observations,
    variables: datasetData.variables
  }
}

/** Convert internally used IObservationSet instance into DatasetData (used by backend). */
export function convertFromIObservationSet (dataset: IObservationSet): DatasetData {
  const observations = dataset.observations.map(
    obs => convertFromIObservation(obs, dataset.id, dataset.variables)
  )
  return {
    id: dataset.id,
    name: dataset.name,
    annotation: dataset.annotation,
    observations,
    variables: dataset.variables
  }
}

/** Convert VariableData instance into internally used IVariableData. */
export function convertToIVariable (variable: VariableData): IVariableData {
  return {
    id: variable.id,
    name: variable.name,
    annotation: variable.annotation,
    function: variable.update_fn
  }
}

/** Convert LayoutNodeData instance into internally used ILayoutData. */
export function convertToILayout (layoutNodes: LayoutNodeData[]): ILayoutData {
  const layout: ILayoutData = new Map()
  layoutNodes.forEach(layoutNode => {
    layout.set(layoutNode.variable, { x: layoutNode.px, y: layoutNode.py })
  })
  return layout
}

/** Convert RegulationData instance into internally used IRegulationData. */
export function convertToIRegulation (regulation: RegulationData): IRegulationData {
  return {
    id: regulation.regulator + regulation.target,
    source: regulation.regulator,
    target: regulation.target,
    essential: regulation.essential,
    monotonicity: regulation.sign
  }
}

/** Make a readable name string for each property template. These will be displayed
 * on the UI side.
 */
export function formatTemplateName (propertyType: PropertyType): string {
  switch (propertyType) {
    case StaticPropertyType.FunctionInputEssential:
    case StaticPropertyType.FunctionInputEssentialWithCondition:
      return 'Function input essential'
    case StaticPropertyType.FunctionInputMonotonic:
    case StaticPropertyType.FunctionInputMonotonicWithCondition:
      return 'Function input monotonic'
    case StaticPropertyType.VariableRegulationEssential:
    case StaticPropertyType.VariableRegulationEssentialWithCondition:
      return 'Regulation essential'
    case StaticPropertyType.VariableRegulationMonotonic:
    case StaticPropertyType.VariableRegulationMonotonicWithCondition:
      return 'Regulation monotonic'
    case StaticPropertyType.Generic:
      return 'Generic static property'
    case DynamicPropertyType.AttractorCount:
      return 'Attractor count'
    case DynamicPropertyType.ExistsTrajectory:
      return 'Exists trajectory'
    case DynamicPropertyType.FixedPoint:
      return 'Exist fixed points'
    case DynamicPropertyType.TrapSpace:
      return 'Exist trap spaces'
    case DynamicPropertyType.HasAttractor:
      return 'Exist attractors'
    case DynamicPropertyType.Generic:
      return 'Generic dynamic property'
  }
}

/** Provide help text for each property template. These can be displayed
 * as tooltips or help messages on the UI side.
 */
export function getTemplateHelpText (propertyType: PropertyType): string {
  switch (propertyType) {
    case StaticPropertyType.FunctionInputEssential:
    case StaticPropertyType.FunctionInputEssentialWithCondition:
      return 'Specifies whether input is essential.'
    case StaticPropertyType.FunctionInputMonotonic:
    case StaticPropertyType.FunctionInputMonotonicWithCondition:
      return 'Specifies whether input has monotonic effect.'
    case StaticPropertyType.VariableRegulationEssential:
    case StaticPropertyType.VariableRegulationEssentialWithCondition:
      return 'Specifies whether regulation is essential.'
    case StaticPropertyType.VariableRegulationMonotonic:
    case StaticPropertyType.VariableRegulationMonotonicWithCondition:
      return 'Specifies whether regulation has monotonic effect.'
    case StaticPropertyType.Generic:
      return 'A generic static property defined by the user.'
    case DynamicPropertyType.AttractorCount:
      return 'Attractor count falls into given range.'
    case DynamicPropertyType.ExistsTrajectory:
      return 'Observations of selected dataset lay on trajectory.'
    case DynamicPropertyType.FixedPoint:
      return 'Each selected observation corresponds to a fixed point.'
    case DynamicPropertyType.TrapSpace:
      return 'Each selected observation corresponds to a trap space.'
    case DynamicPropertyType.HasAttractor:
      return 'Each selected observation exists in an attractor.'
    case DynamicPropertyType.Generic:
      return 'A generic HCTL property defined by the user.'
  }
}
