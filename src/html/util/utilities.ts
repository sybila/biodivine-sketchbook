import {
  Essentiality, Monotonicity, type IFunctionData,
  type IObservationSet, type IObservation, type ILayoutData,
  type IRegulationData, type IVariableData
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
