import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, query, state } from 'lit/decorators.js'
import style_less from './regulations-editor.less?inline'
import cytoscape, {
  type Core,
  type EdgeDefinition,
  type EdgeSingular,
  type NodeDefinition,
  type NodeSingular,
  type Position
} from 'cytoscape'
import dagre from 'cytoscape-dagre'
import edgeHandles, { type EdgeHandlesInstance } from 'cytoscape-edgehandles'
import './node-menu'
import { edgeOptions, initOptions } from './regulations-editor.config'
import { ElementType, Monotonicity } from './element-type'

const SAVE_NODES = 'nodes'
const SAVE_EDGES = 'edges'

@customElement('regulations-editor')
class RegulationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  static doubleClickDelay = 100

  @query('#node-menu')
    nodeMenu!: HTMLElement

  @query('#edge-menu')
    edgeMenu!: HTMLElement

  editorElement
  cy: Core | undefined
  edgeHandles: EdgeHandlesInstance | undefined
  _lastClickTimestamp
  @state() _nodes: NodeDefinition[] = []
  @state() _edges: EdgeDefinition[] = []
  @state() menuType = ElementType.NONE
  @state() menuPosition = { x: 0, y: 0 }
  @state() menuZoom = 1.0
  @state() menuData = undefined

  constructor () {
    super()
    cytoscape.use(dagre)
    cytoscape.use(edgeHandles)
    this.addEventListener('update-edge', this.updateEdge)
    this.addEventListener('remove-element', this.removeElement)

    this.editorElement = document.createElement('div')
    this.editorElement.id = 'cytoscape-editor'
    this._lastClickTimestamp = 0
  }

  firstUpdated (): void {
    // this._nodes.push({ data: { id: 'test', label: 'test' } })
    this.cy = cytoscape(initOptions(this.editorElement, this._nodes, this._edges))
    this.edgeHandles = this.cy.edgehandles(edgeOptions)
    if (!this.loadCachedNodes() || !this.loadCachedEdges()) this.loadDummyData()

    // this.cy.on('add remove position', this.saveState)

    this.cy.on('zoom', () => {
      this._renderMenuForSelectedNode()
      this._renderMenuForSelectedEdge()
    })
    this.cy.on('pan', () => {
      this._renderMenuForSelectedNode()
      this._renderMenuForSelectedEdge()
    })
    this.cy.on('click', () => {
      const now = (new Date()).getTime()
      if ((this._lastClickTimestamp !== 0) && this._lastClickTimestamp !== undefined && now - this._lastClickTimestamp < RegulationsEditor.doubleClickDelay) {
        // LiveModel.addVariable([e.position.x, e.position.y])
      }
      this._lastClickTimestamp = now
    })

    this.cy.ready(() => {
      this.cy?.center()
      this.cy?.fit()
      this.cy?.resize()
    })
  }

  loadDummyData (): void {
    console.log('loading dummy data')
    this.cy?.nodes().remove()
    this.cy?.edges().remove()
    this.addNodes(dummyData.nodes)
    this.addEdges(dummyData.edges)
    this.saveState()
  }

  render (): TemplateResult {
    return html`
        <button @click=${this.loadDummyData} class="uk-button uk-button-danger uk-button-small uk-margin-large-left uk-position-absolute uk-position-z-index-high">reset</button>

        ${this.editorElement}
        <node-menu .type=${this.menuType} .position=${this.menuPosition} .zoom=${this.menuZoom} .data=${this.menuData}></node-menu>
    `
  }

  _findRegulationEdge (regulatorId: string, targetId: string): EdgeSingular | undefined {
    const edge = this.cy?.edges('[source = "' + regulatorId + '"][target = "' + targetId + '"]')
    if (edge?.length === 1) {
      return edge[0]
    } else {
      return undefined
    }
  }

  _renderMenuForSelectedNode (node: NodeSingular | undefined = undefined): void {
    if (node === undefined) {
      node = this.cy?.nodes(':selected').first()
      if (node === undefined || node.length === 0) return // nothing selected
    }
    const zoom = this.cy?.zoom()
    const position = node.renderedPosition()
    this.toggleMenu(ElementType.NODE, position, zoom, node.data())
  }

  _renderMenuForSelectedEdge (edge: EdgeSingular | undefined = undefined): void {
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
    const node = this.cy?.add({
      data: { id, name },
      position
    })
    node?.on('mouseover', () => {
      node.addClass('hover')
      // this._modelEditor.hoverVariable(id, true)
    })
    node?.on('mouseout', () => {
      node.removeClass('hover')
      // this._modelEditor.hoverVariable(id, false)
    })
    node?.on('select', () => {
      // deselect any previous selection - we don't support multiselection yet
      this.cy?.$(':selected').forEach((selected) => {
        if (selected.data().id !== id) {
          selected.unselect()
        }
      })
      this._renderMenuForSelectedNode(node)
      // this._modelEditor.selectVariable(id, true)
    })
    node?.on('unselect', () => {
      this.toggleMenu(ElementType.NONE)
      // this._modelEditor.selectVariable(id, false)
    })
    node?.on('click', () => {
      this._lastClickTimestamp = 0 // ensure that we cannot double-click inside the node
    })
    node?.on('drag', () => {
      if (node.selected()) this._renderMenuForSelectedNode(node)
      this._renderMenuForSelectedEdge()
    })
    console.log(node)
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
    const edge = this.cy?.add({
      group: 'edges',
      data: {
        source: regulation.source,
        target: regulation.target,
        observable: regulation.observable,
        monotonicity: regulation.monotonicity
      }
    })
    edge?.on('select', () => {
      this._renderMenuForSelectedEdge(edge)
    })
    edge?.on('unselect', () => {
      this.toggleMenu(ElementType.NONE) // hide menu
    })
    edge?.on('mouseover', () => {
      edge.addClass('hover')
      // ModelEditor.hoverRegulation(edge.data().source, edge.data().target, true);
    })
    edge?.on('mouseout', () => {
      edge.removeClass('hover')
      // ModelEditor.hoverRegulation(edge.data().source, edge.data().target, false);
    })
    // }
  }

  // _initEdge (edge): void {
  //   edge.on('select', () => {
  //     this._renderMenuForSelectedEdge(edge)
  //   })
  //   edge.on('unselect', () => {
  //    UI.toggleEdgeMenu(); // hide menu
  //   })
  //   edge.on('mouseover', () => {
  //     edge.addClass('hover')
  //     // ModelEditor.hoverRegulation(edge.data().source, edge.data().target, true);
  //   })
  //   edge.on('mouseout', () => {
  //     edge.removeClass('hover')
  //     // ModelEditor.hoverRegulation(edge.data().source, edge.data().target, false);
  //   })
  // }

  updateEdge (event: Event): void {
    const e = (event as CustomEvent)
    this.cy?.$id(e.detail.edgeId)
      .data('observable', e.detail.observable)
      .data('monotonicity', e.detail.monotonicity)
    this.menuData = this.cy?.$id(e.detail.edgeId).data()
  }

  removeElement (event: Event): void {
    const e = (event as CustomEvent)
    console.log(e)
    this.cy?.$id(e.detail.id).remove()
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
    console.log(nodes)
    if (nodes.length > 0) localStorage.setItem(SAVE_NODES, JSON.stringify(nodes))
    if (edges.length > 0) localStorage.setItem(SAVE_EDGES, JSON.stringify(edges))
    // console.log(JSON.stringify(nodes))
    console.log(localStorage.getItem(SAVE_EDGES))
    // console.log(edges)
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
