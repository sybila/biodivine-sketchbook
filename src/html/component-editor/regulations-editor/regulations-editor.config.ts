import { type CytoscapeOptions, type EdgeSingular } from 'cytoscape'
import { Essentiality, Monotonicity } from '../../util/data-interfaces'
import PropertyValueEdge = cytoscape.Css.PropertyValueEdge

export const edgeOptions = {
  preview: true, // whether to show added edges preview before releasing selection
  // hoverDelay: 150, // time spent hovering over a target node before it is considered selected
  handleNodes: 'node', // selector/filter function for whether edges can be made from a given node
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
        essential: Essentiality.TRUE,
        monotonicity: Monotonicity.UNSPECIFIED
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
export const initOptions = (container: HTMLElement): CytoscapeOptions => {
  return {
    wheelSensitivity: 0.5,
    container,
    // Some sensible default auto-layout algorithm
    layout: {
      animate: true,
      animationDuration: 300,
      animationThreshold: 250,
      refresh: 20,
      fit: true,
      name: 'cose',
      padding: 100,
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
          label: 'data(displayName)',
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
          'padding-bottom': '12',
          width: 'label',
          height: 'label'
        }
      },
      { // When a node is highlighted by mouse, show it with a dashed blue border.
        selector: 'node.highlight',
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
      {
        selector: 'node.input-node',
        style: {
          // TODO
          'background-color': '#add8e6'
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
        selector: 'edge.highlight',
        style: { 'overlay-opacity': 0.1 }
      },
      { // Show non-observable edges as dashed
        selector: 'edge[essential]',
        style: {
          'line-style': regulationStyle,
          'line-dash-pattern': [8, 3]
        }
      },
      { // When the edge has unspecified monotonicity, show it as grey with normal arrow
        selector: 'edge',
        style: {
          'line-color': '#797979',
          'target-arrow-color': '#797979',
          'target-arrow-shape': 'triangle'
        }
      },
      { // When the edge is an activation, show it as green with normal arrow
        selector: 'edge[monotonicity="Activation"]',
        style: {
          'line-color': '#4abd73',
          'target-arrow-color': '#4abd73',
          'target-arrow-shape': 'triangle'
        }
      },
      { // When the edge is an inhibition, show it as red with a `tee` arrow
        selector: 'edge[monotonicity="Inhibition"]',
        style: {
          'line-color': '#d05d5d',
          'target-arrow-color': '#d05d5d',
          'target-arrow-shape': 'tee'
        }
      },
      { // When the edge is an inhibition, show it as red with a `tee` arrow
        selector: 'edge[monotonicity="Dual"]',
        style: {
          'line-color': '#1e87f0',
          'target-arrow-color': '#1e87f0',
          'target-arrow-shape': 'diamond'
        }
      },
      { // A selected edge should be drawn with an overlay
        selector: 'edge:selected',
        style: {
          'overlay-opacity': 0.1
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

const regulationStyle = (edge: EdgeSingular): PropertyValueEdge<any> => {
  const essential = edge.data().essential as Essentiality
  switch (essential) {
    case Essentiality.FALSE:
      return 'dotted'
    case Essentiality.TRUE:
      return 'solid'
    default:
      return 'dashed'
  }
}
