import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import style_less from './regulations-editor.less?inline'
import cytoscape, {
  type Core,
  type CytoscapeOptions,
  type EdgeDefinition,
  type EdgeSingular,
  type NodeDefinition,
  type NodeSingular
} from 'cytoscape'
import dagre from 'cytoscape-dagre'
import edgeHandles, { type EdgeHandlesInstance, type EdgeHandlesOptions } from 'cytoscape-edgehandles'

@customElement('regulations-editor')
class RegulationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  editorElement
  cy: Core | undefined
  _edgehandles: EdgeHandlesInstance | undefined
  _lastClickTimestamp
  @state() _nodes: NodeDefinition[] = []
  @state() _edges: EdgeDefinition[] = []

  constructor () {
    super()
    cytoscape.use(dagre)
    cytoscape.use(edgeHandles)

    this.editorElement = document.createElement('div')
    this.editorElement.id = 'cytoscape-editor'
    // this.cy = cytoscape(this.initOptions(this._nodes, this._edges))
    // this._edgehandles = this.cy.edgehandles(this.edgeOptions())
    this._lastClickTimestamp = 0

    // this.cy.on('zoom', () => {
    //   this._renderMenuForSelectedNode()
    //   this._renderMenuForSelectedEdge()
    // })
    // this.cy.on('pan', () => {
    //   this._renderMenuForSelectedNode()
    //   this._renderMenuForSelectedEdge()
    // })
    // this.cy.on('click', () => {
    //   const now = (new Date()).getTime()
    //   if ((this._lastClickTimestamp !== 0) && this._lastClickTimestamp !== undefined && now - this._lastClickTimestamp < DOUBLE_CLICK_DELAY) {
    //     // LiveModel.addVariable([e.position.x, e.position.y])
    //   }
    //   this._lastClickTimestamp = now
    // })
  }

  firstUpdated (): void {
    // this._nodes.push({ data: { id: 'test', label: 'test' } })
    this.cy = cytoscape(this.initOptions(this._nodes, this._edges))
    this._edgehandles = this.cy.edgehandles(this.edgeOptions())
    this.addNode('test', 'test')
    this.addNode('testt', 'testt', [100, 100])

    this.cy.ready(() => {
      this.cy?.center()
      this.cy?.fit()
      this.cy?.resize()
    })
  }

  render (): TemplateResult {
    return html`
      ${this.editorElement}
    `
  }

  initOptions (nodes: NodeDefinition[], edges: EdgeDefinition[]): CytoscapeOptions {
    return {
      elements: { nodes, edges },
      container: this.editorElement,
      // Some sensible default auto-layout algorithm
      layout: {
        animate: true,
        animationDuration: 300,
        animationThreshold: 250,
        refresh: 20,
        fit: true,
        name: 'cose',
        padding: 250,
        nodeRepulsion: () => 100000,
        nodeDimensionsIncludeLabels: true
      },
      boxSelectionEnabled: false,
      selectionType: 'single'
      // style: [
      //   { // Style of the graph nodes
      //     selector: 'node[name]',
      //     style: {
      //       //
      //       label: 'data(name)',
      //       // put label in the middle of the node (vertically)
      //       'text-valign': 'center',
      //       // a rectangle with slightly sloped edges
      //       shape: 'round-rectangle',
      //       // when selecting, do not display any overlay
      //       'overlay-opacity': 0,
      //       // other visual styles
      //       // padding: 12,
      //       'background-color': '#dddddd',
      //       'font-family': 'FiraMono',
      //       'font-size': '12pt',
      //       'border-width': '1px',
      //       'border-color': '#bbbbbb',
      //       'border-style': 'solid'
      //     }
      //   },
      //   { // When a node is highlighted by mouse, show it with a dashed blue border.
      //     selector: 'node.hover',
      //     style: {
      //       'border-width': '2.0px',
      //       'border-color': '#6a7ea5',
      //       'border-style': 'dashed'
      //     }
      //   },
      //   { // When a node is selected, show it with a thick blue border.
      //     selector: 'node:selected',
      //     style: {
      //       'border-width': '2.0px',
      //       'border-color': '#6a7ea5',
      //       'border-style': 'solid'
      //     }
      //   },
      //   { // General style of the graph edge
      //     selector: 'edge',
      //     style: {
      //       width: 3.0,
      //       'curve-style': 'bezier',
      //       'loop-direction': '-15deg',
      //       'loop-sweep': '30deg',
      //       'text-outline-width': 2.3,
      //       'text-outline-color': '#cacaca',
      //       'font-family': 'FiraMono'
      //     }
      //   },
      //   {
      //     selector: 'edge.hover',
      //     style: { 'overlay-opacity': 0.1 }
      //   },
      //   { // Show non-observable edges as dashed
      //     selector: 'edge[observable]',
      //     style: {
      //       'line-style': (edge) => { if (edge.data().observable as boolean) { return 'solid' } else { return 'dashed' } },
      //       'line-dash-pattern': [8, 3]
      //     }
      //   },
      //   { // When the edge is an activation, show it as green with normal arrow
      //     selector: 'edge[monotonicity="activation"]',
      //     style: {
      //       'line-color': '#4abd73',
      //       'target-arrow-color': '#4abd73',
      //       'target-arrow-shape': 'triangle'
      //     }
      //   },
      //   { // When the edge is an inhibition, show it as red with a `tee` arrow
      //     selector: 'edge[monotonicity="inhibition"]',
      //     style: {
      //       'line-color': '#d05d5d',
      //       'target-arrow-color': '#d05d5d',
      //       'target-arrow-shape': 'tee'
      //     }
      //   },
      //   { // When the edge has unspecified monotonicity, show it as grey with normal arrow
      //     selector: 'edge[monotonicity="unspecified"]',
      //     style: {
      //       'line-color': '#797979',
      //       'target-arrow-color': '#797979',
      //       'target-arrow-shape': 'triangle'
      //     }
      //   },
      //   { // A selected edge should be drawn with an overlay
      //     selector: 'edge:selected',
      //     style: {
      //       'overlay-opacity': 0.1
      //     }
      //   },
      //   { // Edge handles pseudo-node for adding
      //     selector: '.eh-handle',
      //     style: {
      //       width: '32px',
      //       height: '32px',
      //       // shape: 'square',
      //       'background-opacity': 0,
      //       'background-image': () => 'data:image/svg+xml;utf8,' + encodeURIComponent(this._addBoxSvg),
      //       'background-width': '32px',
      //       'background-height': '32px',
      //       // padding: 0,
      //       'overlay-opacity': 0,
      //       'border-width': 0,
      //       'border-opacity': 0
      //     }
      //   },
      //   { // Change ghost edge preview colors
      //     selector: '.eh-preview, .eh-ghost-edge',
      //     style: {
      //       'background-color': '#797979',
      //       'line-color': '#797979',
      //       'target-arrow-color': '#797979',
      //       'target-arrow-shape': 'triangle'
      //     }
      //   },
      //   { // Hide ghost edge when a snapped preview is visible
      //     selector: '.eh-ghost-edge.eh-preview-active',
      //     style: { opacity: 0 }
      //   }
      // ]
    }
  }

  edgeOptions (): EdgeHandlesOptions {
    return {
      // preview: true, // whether to show added edges preview before releasing selection
      hoverDelay: 150, // time spent hovering over a target node before it is considered selected
      // handleNodes: 'node', // selector/filter function for whether edges can be made from a given node
      snap: false,
      snapThreshold: 50,
      snapFrequency: 15,
      noEdgeEventsInDraw: false,
      disableBrowserGestures: true,
      // nodeLoopOffset: -50,
      // The `+` button should be drawn on top of each node
      // handlePosition: () => 'middle top',
      // handleInDrawMode: false,
      // edgeType: () => 'flat',
      // Loops are always allowed
      // loopAllowed: () => true,
      // Initialize edge with default parameters
      edgeParams: () => {
        return {
          data: {
            observable: true,
            monotonicity: EdgeMonotonicity.unspecified
          }
        }
      }
      // Add the edge to the live model
      // complete: (sourceNode: string, targetNode: string, addedEles: HTMLElement) => {
      //   if (!LiveModel.addRegulation(sourceNode.id(), targetNode.id(), true, EdgeMonotonicity.unspecified)) {
      //     addedEles.remove() // if we can't create the regulation, remove new edge
      //   } else {
      //     this._initEdge(addedEles[0])
      //   }
      // }
    }
  }

  _findRegulationEdge (regulatorId: string, targetId: string): EdgeSingular | undefined {
    const edge = this.cy.edges('[source = "' + regulatorId + '"][target = "' + targetId + '"]')
    if (edge.length === 1) {
      return edge[0]
    } else {
      return undefined
    }
  }

  _renderMenuForSelectedNode (node: NodeSingular | undefined = undefined): void {
    // if (node === undefined) {
    //   node = this.cy.nodes(':selected').first()
    //   if (node.length === 0) return // nothing selected
    // }
    // const zoom = this.cy.zoom()
    // const position = node.renderedPosition()
    // const height = node.height() * zoom
    // UI.toggleNodeMenu([position.x, position.y], zoom)
  }

  _renderMenuForSelectedEdge (edge: EdgeSingular | undefined = undefined): void {
    // if (edge === undefined) {
    //   edge = this.cy.edges(':selected').first()
    //   if (edge.length === 0) return // nothing selected
    // }
    // const zoom = this.cy.zoom()
    // const boundingBox = edge.renderedBoundingBox()
    // const position = [(boundingBox.x1 + boundingBox.x2) / 2, (boundingBox.y1 + boundingBox.y2) / 2]
    // UI.toggleEdgeMenu(edge.data(), position, zoom)
  }

  addNode (id: string, name: string, position = [0, 0]): void {
    const node = this.cy.add({
      data: { id, name },
      position: { x: position[0], y: position[1] }
    })
    node.on('mouseover', () => {
      node.addClass('hover')
      // this._modelEditor.hoverVariable(id, true)
    })
    node.on('mouseout', () => {
      node.removeClass('hover')
      // this._modelEditor.hoverVariable(id, false)
    })
    node.on('select', () => {
      // deselect any previous selection - we don't support multiselection yet
      for (const selected of this.cy.$(':selected')) {
        if (selected.data().id !== id) {
          selected.unselect()
        }
      }
      this._renderMenuForSelectedNode(node)
      // this._modelEditor.selectVariable(id, true)
    })
    node.on('unselect', () => {
      // UI.toggleNodeMenu()
      // this._modelEditor.selectVariable(id, false)
    })
    node.on('click', () => {
      this._lastClickTimestamp = 0 // ensure that we cannot double-click inside the node
    })
    node.on('drag', () => {
      if (node.selected()) this._renderMenuForSelectedNode(node)
      this._renderMenuForSelectedEdge()
    })
  }
}

const EdgeMonotonicity = {
  unspecified: 'unspecified',
  activation: 'activation',
  inhibition: 'inhibition'
}
