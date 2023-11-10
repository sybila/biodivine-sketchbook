import { TabData } from './tab-data'

let index = 0

export const tabList: TabData[] = [
  TabData.create({
    id: index++,
    name: 'Regulations',
    data: 'Content of regulations tab',
    icon: 'r'
  }),
  TabData.create({
    id: index++,
    name: 'Functions',
    data: 'Content of functions tab',
    icon: 'f'
  }),
  TabData.create({
    id: index++,
    name: 'Observations',
    data: 'Content of observations tab',
    icon: 'o'
  }),
  TabData.create({
    id: index++,
    name: 'Properties',
    data: 'Content of properties tab',
    icon: 'p'
  }),
  TabData.create({
    id: index++,
    name: 'Analysis',
    data: 'Content of analysis tab',
    icon: 'a'
  })
]
