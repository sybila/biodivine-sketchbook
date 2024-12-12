import { TabData } from './tab-data'
import { html } from 'lit'

/** Debounce time set for updating certain text fields. */
export const functionDebounceTimer = 1500
/** Time specifying how often does the backend send updates during inference. */
export const inferencePingTimer = 200

let index = 0

/** List with data for each tab of the editor window. */
export const tabList: TabData[] = [
  TabData.create({
    id: index++,
    name: 'Network',
    content: (contentData) => html`<regulations-editor .contentData=${contentData}></regulations-editor>`,
    icon: 'n',
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
    name: 'Annotations',
    content: (contentData) => html`<annotations-tab .contentData=${contentData}></annotations-tab>`,
    icon: 'a'
  }),
  TabData.create({
    id: index++,
    name: 'Analysis',
    content: (contentData) => html`<analysis-tab .contentData=${contentData}></analysis-tab>`,
    icon: 'i'
  })
]
