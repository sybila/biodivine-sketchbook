import { Essentiality, Monotonicity } from './data-interfaces'

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
