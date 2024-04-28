import {
  DynamicPropertyType,
  type IFixedPointDynamicProperty,
  type ITrapSpaceDynamicProperty
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
