import { expect, test } from 'vitest'
import {
  getNextEssentiality,
  getEssentialityText,
  getNextMonotonicity,
  getMonotonicityClass
} from '../html/util/utilities'
import { Essentiality, Monotonicity } from '../html/util/data-interfaces'

// Test essentility toggling.
test('getNextEssentiality', () => {
  expect(getNextEssentiality(Essentiality.FALSE)).toEqual(Essentiality.TRUE)
  expect(getNextEssentiality(Essentiality.TRUE)).toEqual(Essentiality.UNKNOWN)
  expect(getNextEssentiality(Essentiality.UNKNOWN)).toEqual(Essentiality.FALSE)
})

// Test essentility classes.
test('getEssentialityText', () => {
  expect(getEssentialityText(Essentiality.FALSE)).toEqual('non-essential')
  expect(getEssentialityText(Essentiality.TRUE)).toEqual('essential')
  expect(getEssentialityText(Essentiality.UNKNOWN)).toEqual('unknown')
})

// Test monotonicity toggling.
test('getNextMonotonicity', () => {
  expect(getNextMonotonicity(Monotonicity.ACTIVATION)).toEqual(Monotonicity.INHIBITION)
  expect(getNextMonotonicity(Monotonicity.INHIBITION)).toEqual(Monotonicity.DUAL)
  expect(getNextMonotonicity(Monotonicity.DUAL)).toEqual(Monotonicity.UNSPECIFIED)
  expect(getNextMonotonicity(Monotonicity.UNSPECIFIED)).toEqual(Monotonicity.ACTIVATION)
})

// Test monotonicity classes.
test('getMonotonicityClass', () => {
  expect(getMonotonicityClass(Monotonicity.ACTIVATION)).toEqual('monotonicity-activation')
  expect(getMonotonicityClass(Monotonicity.INHIBITION)).toEqual('monotonicity-inhibition')
  expect(getMonotonicityClass(Monotonicity.DUAL)).toEqual('monotonicity-dual')
  expect(getMonotonicityClass(Monotonicity.UNSPECIFIED)).toEqual('monotonicity-unspecified')
})
