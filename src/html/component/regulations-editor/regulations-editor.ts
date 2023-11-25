import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, query, state } from 'lit/decorators.js'
import style_less from './regulations-editor.less?inline'
import cytoscape, { type Core, type EdgeSingular, type NodeSingular, type Position } from 'cytoscape'
import dagre from 'cytoscape-dagre'
import edgeHandles, { type EdgeHandlesInstance } from 'cytoscape-edgehandles'
import dblclick from 'cytoscape-dblclick'
import './node-menu'
import { edgeOptions, initOptions } from './regulations-editor.config'
import { ElementType, Monotonicity } from './element-type'
import { dialog } from '@tauri-apps/api'

const SAVE_NODES = 'nodes'
const SAVE_EDGES = 'edges'

@customElement('regulations-editor')
class RegulationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`

  @query('#node-menu')
    nodeMenu!: HTMLElement

  @query('#edge-menu')
    edgeMenu!: HTMLElement

  editorElement
  cy: Core | undefined
  edgeHandles: EdgeHandlesInstance | undefined
  lastTabCount = 1
  @state() _nodes: INodeData[] = []
  @state() _edges: IEdgeData[] = []
  @state() menuType = ElementType.NONE
  @state() menuPosition = { x: 0, y: 0 }
  @state() menuZoom = 1.0
  @state() menuData = undefined
  @state() drawMode = false

  constructor () {
    super()
    cytoscape.use(dagre)
    cytoscape.use(edgeHandles)
    cytoscape.use(dblclick)
    this.addEventListener('update-edge', this.updateEdge)
    this.addEventListener('adjust-graph', this.pan)
    this.addEventListener('add-edge', this.addEdge)
    this.addEventListener('mousemove', this.hoverFix)
    this.addEventListener('remove-element', (e) => { void (async () => { await this.removeElement(e) })() })

    this.editorElement = document.createElement('div')
    this.editorElement.id = 'cytoscape-editor'
  }

  render (): TemplateResult {
    return html`
        <button @click=${this.loadDummyData} class="uk-button uk-button-danger uk-button-small uk-margin-large-left uk-position-absolute uk-position-z-index-high">reset (debug)</button>
        ${this.editorElement}
        <node-menu .type=${this.menuType} .position=${this.menuPosition} .zoom=${this.menuZoom} .data=${this.menuData}></node-menu>
    `
  }

  hoverFix (): void {
    // TODO
  }

  addEdge (event: Event): void {
    this.cy?.nodes().deselect()
    this.toggleMenu(ElementType.NONE)
    const nodeID = (event as CustomEvent).detail.id

    // start attribute wrongly typed - added weird typecast to avoid tslint error
    this.edgeHandles?.start((this.cy?.nodes(`#${nodeID}`) as unknown as string))
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-expect-error
    this.cy.renderer().hoverData.capture = true
  }

  firstUpdated (): void {
    this.init()
    if (!this.loadCachedNodes() || !this.loadCachedEdges()) this.loadDummyData()
  }

  init (): void {
    this.cy = cytoscape(initOptions(this.editorElement))
    this.edgeHandles = this.cy.edgehandles(edgeOptions)

    this.cy.on('add remove position', this.saveState)

    this.cy.on('zoom', () => {
      this.renderMenuForSelectedNode()
      this.renderMenuForSelectedEdge()
    })
    this.cy.on('pan', () => {
      this.renderMenuForSelectedNode()
      this.renderMenuForSelectedEdge()
    })
    this.cy.on('dblclick', (e) => {
      if (e.target !== this.cy) return // dont trigger when mouse is over cy elements
      const name = (Math.random() + 1).toString(36).substring(8).toUpperCase()
      this.addNode(name, name, e.position)
    })
    this.cy.on('mouseover', 'node', function (e) {
      e.target.addClass('hover')
    })

    this.cy.on('mouseover', 'node', (e) => {
      e.target.addClass('hover')
      // node.addClass('hover')
      // this._modelEditor.hoverVariable(id, true)
    })
    this.cy.on('mouseout', 'node', (e) => {
      e.target.removeClass('hover')
      // this._modelEditor.hoverVariable(id, false)
    })
    this.cy.on('select', 'node', (e) => {
      // deselect any previous selection - we don't support multiselection yet
      // this.cy?.$(':selected').forEach((selected) => {
      //   if (selected.data().id !== id) {
      //     selected.unselect()
      //   }
      // })
      this.renderMenuForSelectedNode(e.target)
      // this._modelEditor.selectVariable(id, true)
    })
    this.cy.on('unselect', 'node', () => {
      this.toggleMenu(ElementType.NONE)
      // this._modelEditor.selectVariable(id, false)
    })
    // node.on('click', () => {
    //   this._lastClickTimestamp = 0 // ensure that we cannot double-click inside the node
    // })
    this.cy.on('drag', (e) => {
      if ((e.target as NodeSingular).selected()) this.renderMenuForSelectedNode(e.target)
      this.renderMenuForSelectedEdge()
    })

    this.cy.on('select', 'edge', (e) => {
      this.renderMenuForSelectedEdge(e.target)
    })
    this.cy.on('unselect', 'edge', () => {
      this.toggleMenu(ElementType.NONE) // hide menu
    })
    this.cy.on('mouseover', 'edge', (e) => {
      e.target.addClass('hover')
      // ModelEditor.hoverRegulation(edge.data().source, edge.data().target, true);
    })
    this.cy.on('mouseout', 'edge', (e) => {
      e.target.removeClass('hover')
      // ModelEditor.hoverRegulation(edge.data().source, edge.data().target, false);
    })

    this.cy.on('mousemove', (e) => {
      console.log(e)
    })

    this.cy.ready(() => {
      this.cy?.center()
      this.cy?.fit(undefined, 50)
      this.cy?.resize()
    })
  }

  loadDummyData (): void {
    console.log('loading dummy data')
    this.cy?.remove('node')
    this.cy?.edges().remove()
    this.addNodes(dummyData.nodes)
    this.addEdges(dummyData.edges)
    this.saveState()

    this.cy?.ready(() => {
      this.cy?.center()
      this.cy?.fit(undefined, 50)
      this.cy?.resize()
    })
  }

  pan (event: Event): void {
    const tabCount = (event as CustomEvent).detail.tabCount
    if (tabCount === this.lastTabCount) return
    const toLeft = this.lastTabCount < tabCount
    this.lastTabCount = tabCount
    this.cy?.panBy({ x: (toLeft ? -1 : 1) * (this.cy?.width() / (tabCount * 2)), y: 0 })
  }

  renderMenuForSelectedNode (node: NodeSingular | undefined = undefined): void {
    if (node === undefined) {
      node = this.cy?.nodes(':selected').first()
      if (node === undefined || node.length === 0) return // nothing selected
    }
    const zoom = this.cy?.zoom()
    const position = node.renderedPosition()
    this.toggleMenu(ElementType.NODE, position, zoom, node.data())
  }

  renderMenuForSelectedEdge (edge: EdgeSingular | undefined = undefined): void {
    if (edge === undefined) {
      edge = this.cy?.edges(':selected').first()
      if (edge === undefined || edge.length === 0) return // nothing selected
    }
    const zoom = this.cy?.zoom()
    const boundingBox = edge.renderedBoundingBox()
    const position = {
      x: (boundingBox.x1 + boundingBox.x2) / 2,
      y: (boundingBox.y1 + boundingBox.y2) / 2
    }
    this.toggleMenu(ElementType.EDGE, position, zoom, edge.data())
  }

  addNode (id: string, name: string, position = { x: 0, y: 0 }): void {
    this.cy?.add({
      data: { id, name },
      position: { ...position }
    })
  }

  toggleMenu (type: ElementType, position: Position | undefined = undefined, zoom = 1.0, data = undefined): void {
    this.menuType = type
    this.menuPosition = position ?? { x: 0, y: 0 }
    this.menuZoom = zoom
    this.menuData = data
    this.saveState()
  }

  ensureRegulation (regulation: IRegulation): void {
    // const currentEdge = this._findRegulationEdge(regulation.regulator, regulation.target)
    // if (currentEdge !== undefined) {
    //   // Edge exists - just make sure to update data
    //   const data = currentEdge.data()
    //   data.observable = regulation.observable
    //   data.monotonicity = regulation.monotonicity
    //   this.cy?.style().update() // redraw graph
    //   if (currentEdge.selected()) {
    //     // if the edge is selected, we also redraw the edge menu
    //     this._renderMenuForSelectedEdge(currentEdge)
    //   }
    // } else {
    // Edge does not exist - create a new one
    this.cy?.add({
      group: 'edges',
      data: {
        source: regulation.source,
        target: regulation.target,
        observable: regulation.observable,
        monotonicity: regulation.monotonicity
      }
    })
  }

  updateEdge (event: Event): void {
    const e = (event as CustomEvent)
    this.cy?.$id(e.detail.edgeId)
      .data('observable', e.detail.observable)
      .data('monotonicity', e.detail.monotonicity)
    this.menuData = this.cy?.$id(e.detail.edgeId).data()
  }

  async removeElement (event: Event): Promise<void> {
    if (!await dialog.confirm('Are you sure?', {
      type: 'warning',
      okLabel: 'Delete',
      cancelLabel: 'Keep',
      title: 'Delete'
    })) return
    this.cy?.$id((event as CustomEvent).detail.id).remove()
    this.toggleMenu(ElementType.NONE)
  }

  saveState (): void {
    const nodes = ((this.cy?.nodes()) ?? []).map((node): INodeData => {
      return {
        id: node.data().id,
        name: node.data().name,
        position: node.position()
      }
    })
    const edges: IEdgeData[] = ((this.cy?.edges()) ?? []).map((edge): IEdgeData => {
      return {
        id: edge.id(),
        source: edge.source().id(),
        target: edge.target().id(),
        observable: edge.data().observable as boolean,
        monotonicity: edge.data().monotonicity as Monotonicity
      }
    })
    if (nodes.length > 0) {
      this._nodes = nodes
      localStorage.setItem(SAVE_NODES, JSON.stringify(nodes))
    }
    if (edges.length > 0) {
      this._edges = edges
      localStorage.setItem(SAVE_EDGES, JSON.stringify(edges))
    }
  }

  loadCachedNodes (): boolean {
    try {
      const parsed = (JSON.parse(localStorage.getItem(SAVE_NODES) ?? '[]') as INodeData[])
      if (parsed.length === 0) return false
      this.addNodes(parsed)
    } catch (e) {
      return false
    }
    console.log('nodes loaded')
    return true
  }

  loadCachedEdges (): boolean {
    try {
      const parsed = (JSON.parse(localStorage.getItem(SAVE_EDGES) ?? '[]') as IEdgeData[])
      if (parsed.length === 0) return false
      this.addEdges(parsed)
    } catch (e) {
      return false
    }
    console.log('edges loaded')
    return true
  }

  addNodes (nodes: INodeData[]): void {
    nodes.forEach((node) => {
      this.addNode(node.id, node.name, node.position)
    })
  }

  addEdges (edges: IEdgeData[]): void {
    edges.forEach((edge) => {
      this.ensureRegulation(edge)
    })
  }
}

const dummyData: { nodes: INodeData[], edges: IEdgeData[] } = {
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

}

interface IRegulation {
  source: string
  target: string
  observable: boolean
  monotonicity: Monotonicity
}

interface INodeData {
  id: string
  name: string
  position: Position
}

interface IEdgeData {
  id: string
  source: string
  target: string
  observable: boolean
  monotonicity: Monotonicity
}
