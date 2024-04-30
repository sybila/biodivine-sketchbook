import {
  DynamicPropertyType,
  type Essentiality,
  type IAttractorCountDynamicProperty,
  type IExistsTrajectoryDynamicProperty,
  type IFixedPointDynamicProperty,
  type IFunctionInputEssentialStaticProperty,
  type IFunctionInputMonotonicStaticProperty,
  type IGenericDynamicProperty,
  type IGenericStaticProperty,
  type IHasAttractorDynamicProperty,
  type ITrapSpaceDynamicProperty,
  type Monotonicity,
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

export const functionInputEssential = (id: string, func: string, variable: string, essential: Essentiality): IFunctionInputEssentialStaticProperty => {
  return {
    id,
    name: 'function-input-essential',
    type: StaticPropertyType.FunctionInputEssential,
    function: func,
    variable,
    essential
  }
}

export const functionInputMonotonic = (id: string, func: string, variable: string, monotonic: Monotonicity): IFunctionInputMonotonicStaticProperty => {
  return {
    id,
    name: 'function-input-monotonic',
    type: StaticPropertyType.FunctionInputMonotonic,
    function: func,
    variable,
    monotonic
  }
}

export const genericStatic = (id: string): IGenericStaticProperty => {
  return {
    id,
    name: 'generic-static',
    type: StaticPropertyType.Generic,
    value: 'generic-static-value'
  }
}
