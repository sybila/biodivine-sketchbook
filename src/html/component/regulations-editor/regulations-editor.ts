import { html, css, unsafeCSS, LitElement, type TemplateResult } from 'lit'
import { customElement, state, query } from 'lit/decorators.js'
import style_less from './regulations-editor.less?inline'
import cytoscape, {
  type Core,
  type CytoscapeOptions,
  type EdgeDefinition,
  type EdgeSingular,
  type NodeDefinition,
  type NodeSingular,
  type Position
} from 'cytoscape'
import dagre from 'cytoscape-dagre'
import edgeHandles, { type EdgeHandlesInstance, type EdgeHandlesOptions } from 'cytoscape-edgehandles'
import './node-menu'

@customElement('regulations-editor')
class RegulationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  static doubleClickDelay = 100
  static edgeMonotonicity = {
    unspecified: 'unspecified',
    activation: 'activation',
    inhibition: 'inhibition'
  }

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
  @state() nodeMenuVisible = false
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
    this.cy = cytoscape(this.initOptions(this._nodes, this._edges))
    this.edgehandles = this.cy.edgehandles(this.edgeOptions())
    this.addNode('test', 'test')
    this.addNode('testt', 'testt', [100, 100])
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

  render (): TemplateResult {
    return html`
        ${this.editorElement}
        <node-menu .visible=${this.nodeMenuVisible} .position=${this.menuPosition} .zoom=${this.menuZoom}></node-menu>
        <div id="edge-menu" class="float-menu invisible">
            <div class="button-row">
                <img alt="Toggle observability (O)" id="edge-menu-observability" class="button" src="img/visibility_off-24px.svg"
                     src-on="img/visibility_off-24px.svg" src-off="img/visibility_on-24px.svg"
                     alt-on="Observability off (O)" alt-off="Observability on (O)"
                     state=""
                >
                <img alt="Toggle monotonicity (M)" id="edge-menu-monotonicity" class="button" src="img/trending_up-24px.svg"
                     src-unspecified="img/trending_up-24px.svg" alt-unspecified="Make activating (M)"
                     src-activation="img/trending_down-24px.svg" alt-activation="Make inhibiting (M)"
                     src-inhibition="img/swap_vert-24px.svg" alt-inhibition="Monotonicity off (M)"
                     state=""
                >
                <img alt="Remove (⌫)" id="edge-menu-remove" class="button" src="img/delete-24px.svg">
            </div>
            <br>
            <span class="hint invisible">Hint</span>
        </div>
    `
  }

  initOptions (nodes: NodeDefinition[], edges: EdgeDefinition[]): CytoscapeOptions {
    const addBoxSvg = '<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE svg><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="#ffffff" d="M4 4h16v16H4z"/><path fill="#6a7ea5" d="M19 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-2 10h-4v4h-2v-4H7v-2h4V7h2v4h4v2z"/><path d="M0 0h24v24H0z" fill="none"/></svg>'
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
      selectionType: 'single',
      style: [
        { // Style of the graph nodes
          selector: 'node[name]',
          style: {
            //
            label: 'data(name)',
            // put label in the middle of the node (vertically)
            'text-valign': 'center',
            // a rectangle with slightly sloped edges
            shape: 'round-rectangle',
            // when selecting, do not display any overlay
            'overlay-opacity': 0,
            // other visual styles
            padding: 12,
            'background-color': '#dddddd',
            'font-family': 'FiraMono',
            'font-size': '12pt',
            'border-width': '1px',
            'border-color': '#bbbbbb',
            'border-style': 'solid'
          }
        },
        { // When a node is highlighted by mouse, show it with a dashed blue border.
          selector: 'node.hover',
          style: {
            'border-width': '2.0px',
            'border-color': '#6a7ea5',
            'border-style': 'dashed'
          }
        },
        { // When a node is selected, show it with a thick blue border.
          selector: 'node:selected',
          style: {
            'border-width': '2.0px',
            'border-color': '#6a7ea5',
            'border-style': 'solid'
          }
        },
        { // General style of the graph edge
          selector: 'edge',
          style: {
            width: 3.0,
            'curve-style': 'bezier',
            'loop-direction': '-15deg',
            'loop-sweep': '30deg',
            'text-outline-width': 2.3,
            'text-outline-color': '#cacaca',
            'font-family': 'FiraMono'
          }
        },
        {
          selector: 'edge.hover',
          style: { 'overlay-opacity': 0.1 }
        },
        { // Show non-observable edges as dashed
          selector: 'edge[observable]',
          style: {
            'line-style': (edge) => { if (edge.data().observable as boolean) { return 'solid' } else { return 'dashed' } },
            'line-dash-pattern': [8, 3]
          }
        },
        { // When the edge is an activation, show it as green with normal arrow
          selector: 'edge[monotonicity="activation"]',
          style: {
            'line-color': '#4abd73',
            'target-arrow-color': '#4abd73',
            'target-arrow-shape': 'triangle'
          }
        },
        { // When the edge is an inhibition, show it as red with a `tee` arrow
          selector: 'edge[monotonicity="inhibition"]',
          style: {
            'line-color': '#d05d5d',
            'target-arrow-color': '#d05d5d',
            'target-arrow-shape': 'tee'
          }
        },
        { // When the edge has unspecified monotonicity, show it as grey with normal arrow
          selector: 'edge[monotonicity="unspecified"]',
          style: {
            'line-color': '#797979',
            'target-arrow-color': '#797979',
            'target-arrow-shape': 'triangle'
          }
        },
        { // A selected edge should be drawn with an overlay
          selector: 'edge:selected',
          style: {
            'overlay-opacity': 0.1
          }
        },
        { // Edge handles pseudo-node for adding
          selector: '.eh-handle',
          style: {
            width: '32px',
            height: '32px',
            shape: 'square',
            'background-opacity': 0,
            'background-image': () => 'data:image/svg+xml;utf8,' + encodeURIComponent(addBoxSvg),
            'background-width': '32px',
            'background-height': '32px',
            padding: 0,
            'overlay-opacity': 0,
            'border-width': 0,
            'border-opacity': 0
          }
        },
        { // Change ghost edge preview colors
          selector: '.eh-preview, .eh-ghost-edge',
          style: {
            'background-color': '#797979',
            'line-color': '#797979',
            'target-arrow-color': '#797979',
            'target-arrow-shape': 'triangle'
          }
        },
        { // Hide ghost edge when a snapped preview is visible
          selector: '.eh-ghost-edge.eh-preview-active',
          style: { opacity: 0 }
        }
      ]
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
      nodeLoopOffset: -50,
      // The `+` button should be drawn on top of each node
      handlePosition: () => 'middle top',
      handleInDrawMode: false,
      edgeType: () => 'flat',
      // Loops are always allowed
      loopAllowed: () => true,
      // Initialize edge with default parameters
      edgeParams: () => {
        return {
          data: {
            observable: true,
            monotonicity: RegulationsEditor.edgeMonotonicity.unspecified
          }
        }
      }
      // Add the edge to the live model
      // complete: (sourceNode: string, targetNode: string, addedEles: HTMLElement) => {
      //   if (!LiveModel.addRegulation(sourceNode.id(), targetNode.id(), true, RegulationsEditor.edgeMonotonicity.unspecified)) {
      //     addedEles.remove() // if we can't create the regulation, remove new edge
      //   } else {
      //     this._initEdge(addedEles[0])
      //   }
      // }
    }
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
      this.toggleMenu(ElementType.NODE)
      // this._modelEditor.selectVariable(id, false)
    })
    node?.on('click', () => {
      this._lastClickTimestamp = 0 // ensure that we cannot double-click inside the node
    })
    node?.on('drag', () => {
      if (node.selected()) this._renderMenuForSelectedNode(node)
      this._renderMenuForSelectedEdge()
    })
  }

  toggleMenu (type: ElementType, position: Position | undefined = undefined, zoom = 1.0, data = undefined): void {
    let element
    switch (type) {
      case ElementType.EDGE:
        element = this.edgeMenu
        break
      case ElementType.NODE:
        element = this.nodeMenu
        break
      default:
        return
    }
    if (position === undefined) {
      // element.classList.add('invisible')
      // element.style.left = '-100px' // move it somewhere out of clickable area
      // element.style.top = '-100px'
      this.nodeMenuVisible = false
    } else {
      // element.classList.remove('invisible')
      this.nodeMenuVisible = true
      this.menuPosition = position
      this.menuZoom = zoom
      console.log(position, zoom)
      // element.style.left = position.x + 'px'
      // element.style.top = (position.y + (60 * zoom)) + 'px'
      // Scale applies current zoom, translate ensures the middle point of menu is
      // actually at postion [left, top] (this makes it easier to align).
      // element.style.transform = 'scale(' + (zoom * 0.75) + ') translate(-50%, -50%)'
      // if (data !== undefined) {
      //   element.observabilityButton.updateState(data)
      //   element.monotonicityButton.updateState(data)
      // }
    }
  }
}

enum ElementType {
  EDGE,
  NODE
}
