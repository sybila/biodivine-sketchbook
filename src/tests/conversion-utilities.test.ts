import { expect, test } from 'vitest'
import {
  convertToIFunction, convertFromIFunction, convertToIObservation,
  convertFromIObservation, convertToIObservationSet, convertFromIObservationSet,
  convertToIVariable, convertToILayout, convertToIRegulation
} from '../html/util/utilities'
import {
  Essentiality, Monotonicity, type IFunctionData, type IObservation, type IObservationSet,
  type IVariableData, type ILayoutData, type IRegulationData
} from '../html/util/data-interfaces'
import {
  type UninterpretedFnData, type ObservationData, type DatasetData, type VariableData, type LayoutNodeData,
  type RegulationData
} from '../aeon_state'

const mockUninterpretedFnData: UninterpretedFnData = {
  id: 'fn1',
  name: 'Function 1',
  annotation: 'Annotation for function 1',
  arguments: [
    [Monotonicity.ACTIVATION, Essentiality.TRUE],
    [Monotonicity.INHIBITION, Essentiality.FALSE]
  ],
  expression: 'var0 & var1'
}

const mockIFunctionData: IFunctionData = {
  id: 'fn1',
  name: 'Function 1',
  annotation: 'Annotation for function 1',
  function: 'var0 & var1',
  variables: [
    {
      id: '0',
      source: 'var0',
      target: 'fn1',
      monotonicity: Monotonicity.ACTIVATION,
      essential: Essentiality.TRUE
    },
    {
      id: '1',
      source: 'var1',
      target: 'fn1',
      monotonicity: Monotonicity.INHIBITION,
      essential: Essentiality.FALSE
    }
  ]
}

test('convertToIFunction', () => {
  const result = convertToIFunction(mockUninterpretedFnData)
  expect(result).toEqual(mockIFunctionData)
})

test('convertFromIFunction', () => {
  const result = convertFromIFunction(mockIFunctionData)
  expect(result).toEqual(mockUninterpretedFnData)
})

const mockObservationData: ObservationData = {
  id: 'obs1',
  name: 'Observation 1',
  annotation: 'Annotation for observation 1',
  dataset: 'dataset1',
  values: '01*'
}

const mockIObservation: IObservation = {
  id: 'obs1',
  name: 'Observation 1',
  annotation: 'Annotation for observation 1',
  selected: false,
  var0: '0',
  var1: '1',
  var2: ''
}

test('convertToIObservation', () => {
  const variables = ['var0', 'var1', 'var2']
  const result = convertToIObservation(mockObservationData, variables)
  expect(result).toEqual(mockIObservation)
})

test('convertFromIObservation', () => {
  const variables = ['var0', 'var1', 'var2']
  const result = convertFromIObservation(mockIObservation, 'dataset1', variables)
  expect(result).toEqual(mockObservationData)
})

const mockDatasetData: DatasetData = {
  id: 'dataset1',
  name: 'Dataset 1',
  annotation: 'Annotation for dataset 1',
  observations: [mockObservationData],
  variables: ['var0', 'var1', 'var2']
}

const mockIObservationSet: IObservationSet = {
  id: 'dataset1',
  name: 'Dataset 1',
  annotation: 'Annotation for dataset 1',
  observations: [mockIObservation],
  variables: ['var0', 'var1', 'var2']
}

test('convertToIObservationSet', () => {
  const result = convertToIObservationSet(mockDatasetData)
  expect(result).toEqual(mockIObservationSet)
})

test('convertFromIObservationSet', () => {
  const result = convertFromIObservationSet(mockIObservationSet)
  expect(result).toEqual(mockDatasetData)
})

const mockVariableData: VariableData = {
  id: 'var1',
  name: 'Variable 1',
  annotation: 'Annotation for variable 1',
  update_fn: 'var1 & var2'
}

const mockIVariableData: IVariableData = {
  id: 'var1',
  name: 'Variable 1',
  annotation: 'Annotation for variable 1',
  function: 'var1 & var2'
}

test('convertToIVariable', () => {
  const result = convertToIVariable(mockVariableData)
  expect(result).toEqual(mockIVariableData)
})

const mockLayoutNodes: LayoutNodeData[] = [
  { layout: 'layout1', variable: 'var1', px: 10, py: 20 },
  { layout: 'layout1', variable: 'var2', px: 30, py: 40 }
]

const mockILayoutData: ILayoutData = new Map([
  ['var1', { x: 10, y: 20 }],
  ['var2', { x: 30, y: 40 }]
])

test('convertToILayout', () => {
  const result = convertToILayout(mockLayoutNodes)
  expect(result).toEqual(mockILayoutData)
})

const mockRegulationData: RegulationData = {
  regulator: 'var1',
  target: 'var2',
  sign: Monotonicity.ACTIVATION,
  essential: Essentiality.TRUE
}

const mockIRegulationData: IRegulationData = {
  id: 'var1var2',
  source: 'var1',
  target: 'var2',
  monotonicity: Monotonicity.ACTIVATION,
  essential: Essentiality.TRUE
}

test('convertToIRegulation', () => {
  const result = convertToIRegulation(mockRegulationData)
  expect(result).toEqual(mockIRegulationData)
})
