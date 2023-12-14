import { Monotonicity } from '../component/regulations-editor/element-type'
import { ContentData } from './tab-data'

export const dummyData: ContentData = ContentData.create({
  nodes: [
    {
      id: 'YOX1',
      name: 'YOX1',
      position: { x: 297, y: 175 }
    },
    {
      id: 'CLN3',
      name: 'CLN3',
      position: { x: 128, y: 68 }
    },
    {
      id: 'YHP1',
      name: 'YHP1',
      position: { x: 286, y: 254 }
    },
    {
      id: 'ACE2',
      name: 'ACE2',
      position: { x: 74, y: 276 }
    },
    {
      id: 'SWI5',
      name: 'SWI5',
      position: { x: 47, y: 207 }
    },
    {
      id: 'MBF',
      name: 'MBF',
      position: { x: 219, y: 96 }
    },
    {
      id: 'SBF',
      name: 'SBF',
      position: { x: 281, y: 138 }
    },
    {
      id: 'HCM1',
      name: 'HCM1',
      position: { x: 305, y: 217 }
    },
    {
      id: 'SFF',
      name: 'SFF',
      position: { x: 186, y: 302 }
    }
  ],
  edges: [
    { source: 'MBF', target: 'YOX1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'SBF', target: 'YOX1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'YOX1', target: 'CLN3', observable: true, monotonicity: Monotonicity.INHIBITION, id: '' },
    { source: 'YHP1', target: 'CLN3', observable: true, monotonicity: Monotonicity.INHIBITION, id: '' },
    { source: 'ACE2', target: 'CLN3', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'SWI5', target: 'CLN3', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'MBF', target: 'YHP1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'SBF', target: 'YHP1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'SFF', target: 'ACE2', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'SFF', target: 'SWI5', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'CLN3', target: 'MBF', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'MBF', target: 'SBF', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'YOX1', target: 'SBF', observable: true, monotonicity: Monotonicity.INHIBITION, id: '' },
    { source: 'YHP1', target: 'SBF', observable: true, monotonicity: Monotonicity.INHIBITION, id: '' },
    { source: 'CLN3', target: 'SBF', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'MBF', target: 'HCM1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'SBF', target: 'HCM1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'SBF', target: 'SFF', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' },
    { source: 'HCM1', target: 'SFF', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '' }
  ]

})
