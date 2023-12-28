import { Monotonicity } from '../component/regulations-editor/element-type'
import { ContentData } from './tab-data'

export const dummyData: ContentData = ContentData.create({
  variables: [
    {
      id: 'YOX1',
      name: 'YOX1',
      position: { x: 297, y: 175 },
      function: ''
    },
    {
      id: 'CLN3',
      name: 'CLN3',
      position: { x: 128, y: 68 },
      function: ''
    },
    {
      id: 'YHP1',
      name: 'YHP1',
      position: { x: 286, y: 254 },
      function: ''
    },
    {
      id: 'ACE2',
      name: 'ACE2',
      position: { x: 74, y: 276 },
      function: ''
    },
    {
      id: 'SWI5',
      name: 'SWI5',
      position: { x: 47, y: 207 },
      function: ''
    },
    {
      id: 'MBF',
      name: 'MBF',
      position: { x: 219, y: 96 },
      function: ''
    },
    {
      id: 'SBF',
      name: 'SBF',
      position: { x: 281, y: 138 },
      function: ''
    },
    {
      id: 'HCM1',
      name: 'HCM1',
      position: { x: 305, y: 217 },
      function: ''
    },
    {
      id: 'SFF',
      name: 'SFF',
      position: { x: 186, y: 302 },
      function: ''
    }
  ],
  regulations: [
    { source: 'MBF', target: 'YOX1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '9ec9bb07-0a4d-4e76-8f42-bc6288fab957' },
    { source: 'SBF', target: 'YOX1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '591f6b76-581b-431b-ac28-ce3f82607de1' },
    { source: 'YOX1', target: 'CLN3', observable: true, monotonicity: Monotonicity.INHIBITION, id: 'ac97b30a-8881-4a19-a04a-fbd6a29c5d93' },
    { source: 'YHP1', target: 'CLN3', observable: true, monotonicity: Monotonicity.INHIBITION, id: '3cce9c9a-b4f0-4560-91d5-1a3aed277923' },
    { source: 'ACE2', target: 'CLN3', observable: true, monotonicity: Monotonicity.ACTIVATION, id: 'efab7cb8-38cc-4b21-abbb-9000102a7bbc' },
    { source: 'SWI5', target: 'CLN3', observable: true, monotonicity: Monotonicity.ACTIVATION, id: 'ffe4c0e7-a356-46f3-9699-0bb2cd23e1c8' },
    { source: 'MBF', target: 'YHP1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: 'ac4a075c-81d3-4ce0-b4df-969426d6cac0' },
    { source: 'SBF', target: 'YHP1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: 'bf7114cc-9208-4301-9800-cafe99f7264c' },
    { source: 'SFF', target: 'ACE2', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '98e2abb2-0539-489e-9c48-2672d0384853' },
    { source: 'SFF', target: 'SWI5', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '4b7a490a-ce5b-4370-8ec1-575a9c7197c9' },
    { source: 'CLN3', target: 'MBF', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '085e350e-c11b-4303-9bda-61ab3b1bd202' },
    { source: 'MBF', target: 'SBF', observable: true, monotonicity: Monotonicity.ACTIVATION, id: 'f321341a-fe12-4c72-9a63-9c287ed1ab34' },
    { source: 'YOX1', target: 'SBF', observable: true, monotonicity: Monotonicity.INHIBITION, id: '69f5f592-decd-4463-a2f0-2b315a09ab09' },
    { source: 'YHP1', target: 'SBF', observable: true, monotonicity: Monotonicity.INHIBITION, id: 'f100e0d2-7a4c-4f02-8cce-251611e0c2c5' },
    { source: 'CLN3', target: 'SBF', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '639a10a6-90e5-462d-bf1e-7067902fc29f' },
    { source: 'MBF', target: 'HCM1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: 'cf485f32-5572-4177-bf06-e03c269bfdc5' },
    { source: 'SBF', target: 'HCM1', observable: true, monotonicity: Monotonicity.ACTIVATION, id: '0d78b7e5-b566-4d0d-ad55-720db817ade7' },
    { source: 'SBF', target: 'SFF', observable: true, monotonicity: Monotonicity.ACTIVATION, id: 'd70b838b-ff29-48af-a9d7-326bdcea301d' },
    { source: 'HCM1', target: 'SFF', observable: true, monotonicity: Monotonicity.ACTIVATION, id: 'bde7998f-5532-4113-acc1-1518c0047e69' }
  ]

})
