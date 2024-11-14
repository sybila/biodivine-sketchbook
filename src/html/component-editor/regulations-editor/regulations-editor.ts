import { css, html, LitElement, type PropertyValues, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import style_less from './regulations-editor.less?inline'
import cytoscape, { type Core, type EdgeSingular, type NodeSingular, type Position } from 'cytoscape'
import dagre from 'cytoscape-dagre'
import edgeHandles, { type EdgeHandlesInstance } from 'cytoscape-edgehandles'
import dblclick from 'cytoscape-dblclick'
import './float-menu/float-menu'
import { edgeOptions, initOptions } from './regulations-editor.config'
import { appWindow, WebviewWindow } from '@tauri-apps/api/window'
import { type Event as TauriEvent } from '@tauri-apps/api/event'
import { ContentData, ElementType, type IRegulationData, type IVariableData } from '../../util/data-interfaces'
import _ from 'lodash'

@customElement('regulations-editor')
export class RegulationsEditor extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  dialogs: Record<string, WebviewWindow | undefined> = {}
  editorElement
  cy: Core | undefined
  edgeHandles: EdgeHandlesInstance | undefined
  lastTabCount = 1
  highlighted = ''
  @property() contentData = ContentData.create()
  @state() showHelp = false
  @state() menuType = ElementType.NONE
  @state() menuPosition = { x: 0, y: 0 }
  @state() menuZoom = 1.0
  @state() menuData: IRegulationData | IVariableData | undefined = undefined
  @state() drawMode = false
  // The last visible width of the editor. Used to compensate
  // the camera position if the width changes.
  @state() renderedWidth = 0

  constructor () {
    super()
    cytoscape.use(dagre)
    cytoscape.use(edgeHandles)
    cytoscape.use(dblclick)
    this.addEventListener('add-edge', this.addEdge)
    this.addEventListener('edit-variable', (e) => {
      void this.editVariableDialog(e)
    })
    window.addEventListener('focus-function-field', () => {
      this.toggleMenu(ElementType.NONE)
    })
    this.editorElement = document.createElement('div')
    this.editorElement.id = 'cytoscape-editor'

    new ResizeObserver(() => {
      const currentWidth = this.editorElement.offsetWidth
      if (currentWidth !== 0 && currentWidth !== this.renderedWidth) {
        if (this.renderedWidth === 0) {
          // First render... we just save the value for later and let the user
          // position the view however they want.
          this.renderedWidth = currentWidth
        } else {
          // Re-rendering with a new width. We need to correct for
          // the shift in perspective.
          const correctionFactor = (currentWidth - this.renderedWidth) / 2
          this.renderedWidth = currentWidth
          this.cy?.panBy({ x: correctionFactor, y: 0 })
        }
      }
    }).observe(this.editorElement)
  }

  connectedCallback (): void {
    super.connectedCallback()
    window.addEventListener('focus-variable', this.focusVariable.bind(this))
    window.addEventListener('highlight-regulation', this.highlightRegulation.bind(this))
    window.addEventListener('reset-highlight', this.resetHighlights.bind(this))
    window.addEventListener('adjust-graph', this.adjustPan.bind(this))
  }

  disconnectedCallback (): void {
    super.disconnectedCallback()
    window.removeEventListener('focus-variable', this.focusVariable.bind(this))
    window.removeEventListener('highlight-regulation', this.highlightRegulation.bind(this))
    window.removeEventListener('reset-highlight', this.resetHighlights.bind(this))
    window.removeEventListener('adjust-graph', this.adjustPan.bind(this))
  }

  protected updated (_changedProperties: PropertyValues): void {
    // triggered when data are updated
    // all elements are updated and menu is reopened if it was opened

    /*
    super.updated(_changedProperties)
    if (_changedProperties.get('contentData') === undefined) return
    */

    super.updated(_changedProperties)

    // only re-render if variables or regulations were updated
    if (_changedProperties.get('contentData') === undefined) return
    const newContentData = _changedProperties.get('contentData')
    if (_.isEqual(this.contentData.variables, newContentData.variables) &&
        _.isEqual(this.contentData.regulations, newContentData.regulations) &&
        _.isEqual(this.contentData.layout, newContentData.layout)) {
      return
    }

    this.cy?.remove('node')
    this.cy?.edges().remove()
    this.addNodes()
    this.addEdges()
    this.cy?.$id(this.highlighted).addClass('highlight')
    const elementID = this.menuData?.id ?? ''
    const type = this.menuType
    if (type === ElementType.NONE) return
    this.toggleMenu(ElementType.NONE)
    const elem = this.cy?.$id(elementID)
    if (elem === undefined || elem.length === 0) {
      void this.dialogs[elementID]?.close()
      return
    }
    elem.select()
    if (type === ElementType.EDGE) {
      this.renderMenuForSelectedEdge(this.cy?.$id(elementID))
    }
    if (type === ElementType.NODE) {
      this.renderMenuForSelectedNode(this.cy?.$id(elementID))
    }
  }

  private addEdge (event: Event): void {
    if (this.cy === undefined) return
    this.cy.nodes().deselect()
    this.toggleMenu(ElementType.NONE)
    const variableId = (event as CustomEvent).detail.id
    this.edgeHandles?.start(this.cy?.$id(variableId))
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-expect-error renderer exists but its missing from the *.d.ts file
    this.cy.renderer().hoverData.capture = true
  }

  private focusVariable (event: Event): void {
    const node = this.cy?.$id((event as CustomEvent).detail.id)
    // wait for the node to be rendered
    setTimeout(() => {
      this.cy?.center(node)
    }, 150)
  }

  private highlightRegulation (event: Event): void {
    this.highlighted = (event as CustomEvent).detail.id
    this.cy?.$id(this.highlighted).addClass('highlight')
  }

  private resetHighlights (): void {
    this.highlighted = ''
    this.cy?.edges().removeClass('highlight')
    this.cy?.nodes().removeClass('highlight')
  }

  firstUpdated (): void {
    this.init()
    this.addNodes()
    this.addEdges()
  }

  private init (): void {
    this.cy = cytoscape(initOptions(this.editorElement))
    this.edgeHandles = this.cy.edgehandles(edgeOptions)

    this.cy.on('dragfree', 'node', (a) => {
      this.moveVariable(a.target.data().id, a.target.position())
    })

    this.cy.on('ehcomplete', (_event, _source, _target, edge) => {
      this.dispatchEvent(new CustomEvent('add-regulation', {
        detail: {
          ...edge.data()
        },
        composed: true,
        bubbles: true
      }))
      edge.remove()
    })

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
      this.createVariable(e.position)
    })
    this.cy.on('mouseover', 'node', function (e) {
      e.target.addClass('hover')
    })

    this.cy.on('mouseover', 'node', (e) => {
      e.target.addClass('hover')
    })
    this.cy.on('mouseout', 'node', (e) => {
      e.target.removeClass('hover')
    })
    this.cy.on('select', 'node', (e) => {
      this.renderMenuForSelectedNode(e.target)
    })
    this.cy.on('unselect', 'node', () => {
      this.toggleMenu(ElementType.NONE)
    })
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
    })
    this.cy.on('mouseout', 'edge', (e) => {
      e.target.removeClass('hover')
    })

    this.cy.ready(() => {
      this.cy?.center()
      this.cy?.fit(undefined, 50)
      this.cy?.resize()
    })
  }

  private async editVariableDialog (event: Event): Promise<void> {
    this.toggleMenu(ElementType.NONE)
    const variableId = (event as CustomEvent).detail.id
    const variableName = (event as CustomEvent).detail.name
    const variableAnnotation = (event as CustomEvent).detail.annotation
    const pos = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    if (this.dialogs[variableId] !== undefined) {
      await this.dialogs[variableId]?.setFocus()
      return
    }
    const renameDialog = new WebviewWindow(`renameDialog${Math.floor(Math.random() * 1000000)}`, {
      url: 'src/html/component-editor/regulations-editor/rename-dialog/rename-dialog.html',
      title: `Edit node (${variableId} / ${variableName})`,
      alwaysOnTop: true,
      maximizable: false,
      minimizable: false,
      skipTaskbar: true,
      resizable: false,
      height: 300,
      width: 500,
      x: pos.x + (size.width / 2) - 200,
      y: pos.y + size.height / 4
    })
    this.dialogs[variableId] = renameDialog
    void renameDialog.once('loaded', () => {
      void renameDialog.emit('edit_node_update', {
        id: variableId,
        name: variableName,
        annotation: variableAnnotation
      })
    })
    void renameDialog.once('edit_node_dialog', (event: TauriEvent<{ id: string, name: string, annotation: string }>) => {
      this.dialogs[variableId] = undefined
      const index = this.contentData.variables.findIndex(v => v.id === variableId)
      if (index === -1) return
      this.changeVariable(variableId, event.payload.id, event.payload.name, event.payload.annotation)
    })
    void renameDialog.onCloseRequested(() => {
      this.dialogs[variableId] = undefined
    })
  }

  private changeVariable (id: string, newId: string, newName: string, newAnnot: string): void {
    // first send the event to change the name & annot (with the old ID)
    this.dispatchEvent(new CustomEvent('set-variable-data', {
      detail: {
        id,
        name: newName,
        annotation: newAnnot
      },
      bubbles: true,
      composed: true
    }))

    // after quick timeout send the event to change the ID
    setTimeout(() => {
      this.dispatchEvent(new CustomEvent('set-variable-id', {
        detail: {
          oldId: id,
          newId
        },
        bubbles: true,
        composed: true
      }))
    }, 50)
  }

  private adjustPan (event: Event): void {
    const tabCount = (event as CustomEvent).detail.tabCount
    if (tabCount === this.lastTabCount) return
    const toLeft = this.lastTabCount < tabCount
    this.lastTabCount = tabCount
    this.cy?.panBy({ x: (toLeft ? -1 : 1) * (this.cy?.width() / (tabCount * 2)), y: 0 })
  }

  private renderMenuForSelectedNode (node: NodeSingular | undefined = undefined): void {
    if (node === undefined) {
      node = this.cy?.nodes(':selected').first()
      if (node === undefined || node.length === 0) return // nothing selected
    }
    const zoom = this.cy?.zoom()
    const position = node.renderedPosition()
    this.toggleMenu(ElementType.NODE, position, zoom, node.data())
  }

  private renderMenuForSelectedEdge (edge: EdgeSingular | undefined = undefined): void {
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

  private addNode (id: string, name: string, annotation: string, position = { x: 0, y: 0 }): void {
    this.cy?.add({
      data: { id, name, annotation },
      position: { ...position }
    })
  }

  private createVariable (position = { x: 0, y: 0 }): void {
    this.dispatchEvent(new CustomEvent('add-variable', {
      detail: {
        position
      },
      composed: true,
      bubbles: true
    }))
  }

  private toggleMenu (type: ElementType, position: Position | undefined = undefined, zoom = 1.0, data = undefined): void {
    this.menuType = type
    if (this.menuType === ElementType.NONE) {
      this.cy?.nodes().deselect()
      return
    }
    this.menuPosition = position ?? { x: 0, y: 0 }
    this.menuZoom = zoom
    this.menuData = data
  }

  private ensureRegulation (regulation: IRegulationData): void {
    this.cy?.add({
      group: 'edges',
      data: {
        ...regulation
      }
    })
  }

  private moveVariable (varId: string, position: Position): void {
    this.dispatchEvent(new CustomEvent('change-node-position', {
      detail: {
        id: varId,
        position
      },
      bubbles: true,
      composed: true
    }))
  }

  private addNodes (): void {
    this.contentData.variables.forEach((node) => {
      this.addNode(node.id, node.name, node.annotation, this.contentData.layout.get(node.id))
    })
  }

  private addEdges (): void {
    this.contentData.regulations.forEach((edge) => {
      this.ensureRegulation(edge)
    })
  }

  render (): TemplateResult {
    return html`
      <div class="header uk-background-primary">
        <h3 class="uk-heading-bullet uk-margin-remove-bottom ">Network</h3>
      </div>
      ${this.contentData.variables.length === 0 || this.showHelp
? html`
        <div id="quick-help" class="help-message-block">
          <h3 style="display: block; margin: 0 auto; float: right;">HELP</h3>
          <b>Network editor actions:</b>
          <ul>
            <li>Double click to create a new variable.</li>
            <li>Select element (click) to show the context menu.</li>
            <li>Add and edit regulations using the context menu.</li>
          </ul>
          <b>Keyboard shortcuts:</b>
          <ul>
            <li>(⌫) Remove selected element.</li>
            <li>(E) Edit selected variable.</li>
            <li>(A) Add edge from the selected variable.</li>
            <li>(S) Add self-loop to the selected variable.</li>
            <li>(F) Edit update function of the selected variable.</li>
            <li>(O) Change observability of the selected regulation.</li>
            <li>(M) Change monotonicity of the selected regulation.</li>
          </ul>
          <div style="width: 100%; text-align: center; font-size: 14px;">(this message disappears when you create the first variable)</div>            
        </div>`
        : ''}
      <!-- Prepares a clean environment for the cytoscape element with a floating menu. -->
      <div style="width: 100%; height: 100%; position: relative;">
        ${this.editorElement}
        <float-menu .type=${this.menuType} .position=${this.menuPosition} .zoom=${this.menuZoom}
                    .data=${this.menuData}></float-menu>
      </div>
      <button class="uk-button uk-button-small uk-button-secondary help-button" @mouseenter="${() => { this.showHelp = true }}" @mouseleave="${() => { this.showHelp = false }}">
        ?
      </button>
    `
  }
}
