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
    variant: DynamicPropertyType.FixedPoint,
    dataset: undefined,
    observation: undefined
  }
}

export const trapSpaceDynamic = (id: string): ITrapSpaceDynamicProperty => {
  return {
    id,
    name: 'trap-space',
    variant: DynamicPropertyType.TrapSpace,
    dataset: undefined,
    observation: undefined,
    minimal: false,
    nonpercolable: false
  }
}

export const existsTrajectoryDynamic = (id: string): IExistsTrajectoryDynamicProperty => {
  return {
    id,
    name: 'exists-trajectory',
    variant: DynamicPropertyType.ExistsTrajectory,
    dataset: undefined
  }
}

export const attractorCountDynamic = (id: string): IAttractorCountDynamicProperty => {
  return {
    id,
    name: 'attractor-count',
    variant: DynamicPropertyType.AttractorCount,
    minimal: 1,
    maximal: 1
  }
}

export const hasAttractorDynamic = (id: string): IHasAttractorDynamicProperty => {
  return {
    id,
    name: 'has-attractor',
    variant: DynamicPropertyType.HasAttractor,
    dataset: undefined,
    observation: undefined
  }
}

export const genericDynamic = (id: string): IGenericDynamicProperty => {
  return {
    id,
    name: 'generic-dynamic',
    variant: DynamicPropertyType.Generic,
    formula: ''
  }
}

export const functionInputEssential = (id: string): IFunctionInputEssentialStaticProperty => {
  return {
    id,
    name: 'function-input-essential',
    variant: StaticPropertyType.FunctionInputEssential,
    target: 'func',
    input: 'var',
    value: Essentiality.FALSE,
    context: ''
  }
}

export const functionInputEssentialWithCondition = (id: string): IFunctionInputEssentialStaticProperty => {
  return {
    id,
    name: 'function-input-essential',
    variant: StaticPropertyType.FunctionInputEssentialWithCondition,
    target: 'func',
    input: 'var',
    value: Essentiality.TRUE,
    context: ''
  }
}

export const functionInputMonotonic = (id: string): IFunctionInputMonotonicStaticProperty => {
  return {
    id,
    name: 'function-input-monotonic',
    variant: StaticPropertyType.FunctionInputMonotonic,
    target: 'func',
    input: 'var',
    value: Monotonicity.ACTIVATION,
    context: ''
  }
}

export const functionInputMonotonicWithCondition = (id: string): IFunctionInputMonotonicStaticProperty => {
  return {
    id,
    name: 'function-input-monotonic',
    variant: StaticPropertyType.FunctionInputMonotonicWithCondition,
    target: undefined,
    input: undefined,
    value: Monotonicity.ACTIVATION,
    context: ''
  }
}

export const variableRegulationMonotonicWithCondition = (id: string): IVariableRegulatorMonotonicStaticProperty => {
  return {
    id,
    name: 'variable-regulation-monotonic',
    variant: StaticPropertyType.VariableRegulationMonotonicWithCondition,
    target: undefined,
    input: undefined,
    value: Monotonicity.ACTIVATION,
    context: ''
  }
}

export const variableRegulationEssentialWithCondition = (id: string): IVariableRegulatorEssentialStaticProperty => {
  return {
    id,
    name: 'variable-regulation-essential',
    variant: StaticPropertyType.VariableRegulationEssentialWithCondition,
    target: undefined,
    input: undefined,
    value: Essentiality.TRUE,
    context: ''
  }
}

export const genericStatic = (id: string): IGenericStaticProperty => {
  return {
    id,
    name: 'generic-static',
    variant: StaticPropertyType.Generic,
    formula: ''
  }
}
