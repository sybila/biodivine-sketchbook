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
import { ElementType } from './element-type'

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
  edgehandles: EdgeHandlesInstance | undefined
  _lastClickTimestamp
  @state() _nodes: NodeDefinition[] = []
  @state() _edges: EdgeDefinition[] = []
  @state() menuType = ElementType.NONE
  @state() menuPosition = { x: 0, y: 0 }
  @state() menuZoom = 1.0

  constructor () {
    super()
    cytoscape.use(dagre)
    cytoscape.use(edgeHandles)

    this.editorElement = document.createElement('div')
    this.editorElement.id = 'cytoscape-editor'
    this._lastClickTimestamp = 0
  }

  firstUpdated (): void {
    // this._nodes.push({ data: { id: 'test', label: 'test' } })
    this.cy = cytoscape(initOptions(this.editorElement, this._nodes, this._edges))
    this.edgehandles = this.cy.edgehandles(edgeOptions)
    this.dummyData()

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

  dummyData (): void {
    this.addNode('YOX1', 'YOX1', [297, 175])
    this.addNode('CLN3', 'CLN3', [128, 68])
    this.addNode('YHP1', 'YHP1', [286, 254])
    this.addNode('ACE2', 'ACE2', [74, 276])
    this.addNode('SWI5', 'SWI5', [47, 207])
    this.addNode('MBF', 'MBF', [219, 96])
    this.addNode('SBF', 'SBF', [281, 138])
    this.addNode('HCM1', 'HCM1', [305, 217])
    this.addNode('SFF', 'SFF', [186, 302])
    this.ensureRegulation({ source: 'MBF', target: 'YOX1', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'SBF', target: 'YOX1', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'YOX1', target: 'CLN3', observable: true, monotonicity: 'inhibition' })
    this.ensureRegulation({ source: 'YHP1', target: 'CLN3', observable: true, monotonicity: 'inhibition' })
    this.ensureRegulation({ source: 'ACE2', target: 'CLN3', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'SWI5', target: 'CLN3', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'MBF', target: 'YHP1', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'SBF', target: 'YHP1', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'SFF', target: 'ACE2', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'SFF', target: 'SWI5', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'CLN3', target: 'MBF', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'MBF', target: 'SBF', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'YOX1', target: 'SBF', observable: true, monotonicity: 'inhibition' })
    this.ensureRegulation({ source: 'YHP1', target: 'SBF', observable: true, monotonicity: 'inhibition' })
    this.ensureRegulation({ source: 'CLN3', target: 'SBF', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'MBF', target: 'HCM1', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'SBF', target: 'HCM1', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'SBF', target: 'SFF', observable: true, monotonicity: 'activation' })
    this.ensureRegulation({ source: 'HCM1', target: 'SFF', observable: true, monotonicity: 'activation' })
  }

  render (): TemplateResult {
    return html`
        ${this.editorElement}
        <node-menu .type=${this.menuType} .position=${this.menuPosition} .zoom=${this.menuZoom}></node-menu>
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
    this.toggleMenu(ElementType.NODE, position, zoom)
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

  addNode (id: string, name: string, position = [0, 0]): void {
    const node = this.cy?.add({
      data: { id, name },
      position: { x: position[0], y: position[1] }
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
    // element.classList.remove('invisible')
    this.menuPosition = position ?? { x: 0, y: 0 }
    this.menuZoom = zoom
    console.log(data)
    // if (data !== undefined) {
    //   element.observabilityButton.updateState(data)
    //   element.monotonicityButton.updateState(data)
    // }
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
}

interface IRegulation {
  source: string
  target: string
  observable: boolean
  monotonicity: string

}
