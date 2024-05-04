import {
  DynamicPropertyType,
  Essentiality,
  type IAttractorCountDynamicProperty,
  type IExistsTrajectoryDynamicProperty,
  type IFixedPointDynamicProperty,
  type IFunctionInputEssentialStaticProperty,
  type IFunctionInputMonotonicStaticProperty,
  type IGenericDynamicProperty,
  type IGenericStaticProperty,
  type IHasAttractorDynamicProperty,
  type ITrapSpaceDynamicProperty, type IVariableRegulatorEssentialStaticProperty, type IVariableRegulatorMonotonicStaticProperty,
  Monotonicity,
  StaticPropertyType
} from '../../util/data-interfaces'

export const fixedPointDynamic = (id: string): IFixedPointDynamicProperty => {
  return {
    id,
    name: 'fixed-point',
    type: DynamicPropertyType.FixedPoint,
    dataset: '',
    observation: ''
  }
}

export const trapSpaceDynamic = (id: string): ITrapSpaceDynamicProperty => {
  return {
    id,
    name: 'dynamic-trap-space',
    type: DynamicPropertyType.TrapSpace,
    dataset: '',
    observation: '',
    minimal: false,
    nonpercolable: false
  }
}

export const existsTrajectoryDynamic = (id: string): IExistsTrajectoryDynamicProperty => {
  return {
    id,
    name: 'exists-trajectory-dynamic',
    type: DynamicPropertyType.ExistsTrajectory,
    dataset: ''
  }
}

export const attractorCountDynamic = (id: string): IAttractorCountDynamicProperty => {
  return {
    id,
    name: 'attractor-count-dynamic',
    type: DynamicPropertyType.AttractorCount,
    lower: 0,
    upper: 0
  }
}

export const hasAttractorDynamic = (id: string): IHasAttractorDynamicProperty => {
  return {
    id,
    name: 'has-attractor-dynamic',
    type: DynamicPropertyType.HasAttractor,
    dataset: '',
    observation: ''
  }
}

export const genericDynamic = (id: string): IGenericDynamicProperty => {
  return {
    id,
    name: 'generic-dynamic',
    type: DynamicPropertyType.Generic,
    value: ''
  }
}

export const functionInputEssential = (id: string): IFunctionInputEssentialStaticProperty => {
  return {
    id,
    name: 'function-input-essential',
    type: StaticPropertyType.FunctionInputEssential,
    function: 'func',
    variable: 'var',
    essential: Essentiality.FALSE,
    condition: ''
  }
}

export const functionInputEssentialWithCondition = (id: string): IFunctionInputEssentialStaticProperty => {
  return {
    id,
    name: 'function-input-essential',
    type: StaticPropertyType.FunctionInputEssentialWithCondition,
    function: 'func',
    variable: 'var',
    essential: Essentiality.TRUE,
    condition: ''
  }
}

export const functionInputMonotonic = (id: string): IFunctionInputMonotonicStaticProperty => {
  return {
    id,
    name: 'function-input-monotonic',
    type: StaticPropertyType.FunctionInputMonotonic,
    function: 'func',
    variable: 'var',
    monotonic: Monotonicity.ACTIVATION,
    condition: ''
  }
}

export const functionInputMonotonicWithCondition = (id: string): IFunctionInputMonotonicStaticProperty => {
  return {
    id,
    name: 'function-input-monotonic',
    type: StaticPropertyType.FunctionInputMonotonicWithCondition,
    function: undefined,
    variable: undefined,
    monotonic: Monotonicity.ACTIVATION,
    condition: ''
  }
}

export const variableRegulationMonotonicWithCondition = (id: string): IVariableRegulatorMonotonicStaticProperty => {
  return {
    id,
    name: 'variable-regulation-monotonic',
    type: StaticPropertyType.VariableRegulationMonotonicWithCondition,
    variable: undefined,
    regulator: undefined,
    monotonic: Monotonicity.ACTIVATION,
    condition: ''
  }
}

export const variableRegulationEssentialWithCondition = (id: string): IVariableRegulatorEssentialStaticProperty => {
  return {
    id,
    name: 'variable-regulation-essential',
    type: StaticPropertyType.VariableRegulationEssentialWithCondition,
    variable: undefined,
    regulator: undefined,
    essential: Essentiality.TRUE,
    condition: ''
  }
}

export const genericStatic = (id: string): IGenericStaticProperty => {
  return {
    id,
    name: '',
    type: StaticPropertyType.Generic,
    value: ''
  }
}
