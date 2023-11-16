import { type CytoscapeOptions, type EdgeDefinition, type NodeDefinition } from 'cytoscape'

export const edgeOptions = {
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
        monotonicity: {
          unspecified: 'unspecified',
          activation: 'activation',
          inhibition: 'inhibition'
        }
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
export const initOptions = (container: HTMLElement, nodes: NodeDefinition[], edges: EdgeDefinition[]): CytoscapeOptions => {
  const addBoxSvg = '<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE svg><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="#ffffff" d="M4 4h16v16H4z"/><path fill="#6a7ea5" d="M19 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-2 10h-4v4h-2v-4H7v-2h4V7h2v4h4v2z"/><path d="M0 0h24v24H0z" fill="none"/></svg>'
  return {
    elements: { nodes, edges },
    container,
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
          'background-color': '#dddddd',
          'font-family': 'FiraMono',
          'font-size': '12pt',
          'border-width': '1px',
          'border-color': '#bbbbbb',
          'border-style': 'solid',
          'padding-bottom': '12'
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
          'background-opacity': 0,
          'background-image': () => 'data:image/svg+xml;utf8,' + encodeURIComponent(addBoxSvg),
          'background-width': '32px',
          'background-height': '32px',
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
