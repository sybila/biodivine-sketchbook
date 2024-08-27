import { TabData } from './tab-data'
import { html } from 'lit'

export const functionDebounceTimer = 1500
export const inferencePingTimer = 500

let index = 0

export const tabList: TabData[] = [
  TabData.create({
    id: index++,
    name: 'Regulations',
    content: (contentData) => html`<regulations-editor .contentData=${contentData}></regulations-editor>`,
    icon: 'r',
    active: true
  }),
  TabData.create({
    id: index++,
    name: 'Functions',
    content: (contentData) => html`<functions-editor .contentData=${contentData}></functions-editor>`,
    icon: 'f'
  }),
  TabData.create({
    id: index++,
    name: 'Observations',
    content: (contentData) => html`<observations-editor .contentData=${contentData}></observations-editor>`,
    icon: 'o'
  }),
  TabData.create({
    id: index++,
    name: 'Properties',
    content: (contentData) => html`<properties-editor .contentData=${contentData}></properties-editor>`,
    icon: 'p'
  }),
  TabData.create({
    id: index++,
    name: 'Analysis',
    content: (contentData) => html`<analysis-tab .contentData=${contentData}></analysis-tab>`,
    icon: 'a'
  })
]
