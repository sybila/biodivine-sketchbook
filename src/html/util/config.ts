import { TabData } from './tab-data'
import { html } from 'lit'

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
    content: () => html`<h1 class="uk-heading uk-text-success">Content of observations tab</h1>`,
    icon: 'o'
  }),
  TabData.create({
    id: index++,
    name: 'Properties',
    content: () => html`<h1 class="uk-heading uk-text-success">Content of properties tab</h1>`,
    icon: 'p'
  }),
  TabData.create({
    id: index++,
    name: 'Analysis',
    content: () => html`<h1 class="uk-heading uk-text-success">Content of analysis tab</h1>`,
    icon: 'a'
  })
]
