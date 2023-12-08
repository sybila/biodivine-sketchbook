import { TabData } from './tab-data'
import { html } from 'lit'

let index = 0

export const tabList: TabData[] = [
  TabData.create({
    id: index++,
    name: 'Regulations',
    data: html`<regulations-editor></regulations-editor>`,
    icon: 'r',
    active: true
  }),
  TabData.create({
    id: index++,
    name: 'Functions',
    data: html`<h1 class="uk-heading uk-text-success">Content of functions tab</h1>`,
    icon: 'f'
  }),
  TabData.create({
    id: index++,
    name: 'Observations',
    data: html`<h1 class="uk-heading uk-text-success">Content of observations tab</h1>`,
    icon: 'o'
  }),
  TabData.create({
    id: index++,
    name: 'Properties',
    data: html`<h1 class="uk-heading uk-text-success">Content of properties tab</h1>`,
    icon: 'p'
  }),
  TabData.create({
    id: index++,
    name: 'Analysis',
    data: html`<h1 class="uk-heading uk-text-success">Content of analysis tab</h1>`,
    icon: 'a'
  })
]
