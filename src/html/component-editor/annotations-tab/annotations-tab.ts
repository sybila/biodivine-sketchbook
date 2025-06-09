import { css, html, LitElement, type TemplateResult, unsafeCSS } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import style_less from './annotations-tab.less?inline'
import { ContentData, type IObservationSet } from '../../util/data-interfaces'
import './annotation-tile/annotation-tile'
import './annotation-tile/dataset-tile'
import { aeonState } from '../../../aeon_state'

/** Component responsible for the annotation tab of the editor session. */
@customElement('annotations-tab')
export class AnnotationsTab extends LitElement {
  static styles = css`${unsafeCSS(style_less)}`
  @property() contentData = ContentData.create()

  constructor () {
    super()
    aeonState.sketch.annotationChanged.addEventListener(this.#annotationChanged.bind(this))
  }

  /** Reaction to backend sending changed annotation data. */
  #annotationChanged (annotation: string): void {
    // propagate the current version of annotation via event that will be captured by root component
    this.dispatchEvent(new CustomEvent('save-annotation', {
      bubbles: true,
      composed: true,
      detail: { annotation }
    }))
  }

  /** Inform backend user is changing annotation. */
  private changeAnnotation (event: Event): void {
    const target = event.target as HTMLTextAreaElement
    aeonState.sketch.setAnnotation(target.value)
  }

  /** Helper function to format the sketch annotation section. */
  formatSketchAnnotation (): TemplateResult<1> {
    return html`
      <textarea class="uk-textarea sketch-annotation"
        .value="${this.contentData.annotation}"
        @focusout="${this.changeAnnotation}"
        placeholder="Click to annotate the sketch..."
        rows="2"
      ></textarea>
    `
  }

  /** Helper function to format the variables annotation section. */
  formatVarAnnotations (): TemplateResult<1> {
    const annotatedVars = this.contentData.variables
      .filter(variable => variable.annotation.trim() !== '')
    if (annotatedVars.length === 0) {
      return html`<div class="placeholder"><p>No annotations available for variables.</p></div>`
    }
    return html`<div>${annotatedVars.map(variable => this.renderAnnotationTile(variable.id, variable.annotation))}</div>`
  }

  /** Helper function to format the functions annotation section. */
  formatFnAnnotations (): TemplateResult<1> {
    const annotatedFns = this.contentData.functions
      .filter(func => func.annotation.trim() !== '')
    if (annotatedFns.length === 0) {
      return html`<div class="placeholder"><p>No annotations available for functions.</p></div>`
    }
    return html`<div>${annotatedFns.map(func => this.renderAnnotationTile(func.id, func.annotation))}</div>`
  }

  /** Helper function to format the datasets annotation section. */
  formatDatasetAnnotations (): TemplateResult<1> {
    const annotatedDatasets = this.contentData.observations
      .filter(d => d.annotation.trim() !== '' || d.observations.some(obs => obs.annotation.trim() !== ''))

    if (annotatedDatasets.length === 0) {
      return html`<div class="placeholder"><p>No annotations available for datasets or observations.</p></div>`
    }
    return html`<div>${annotatedDatasets.map(dataset => this.renderDatasetTile(dataset))}</div>`
  }

  /** Helper function to format the dynamic properties annotation section. */
  formatDynPropAnnotations (): TemplateResult<1> {
    const annotatedProps = this.contentData.dynamicProperties
      .filter(dynProp => dynProp.annotation.trim() !== '')
    if (annotatedProps.length === 0) {
      return html`<div class="placeholder"><p>No annotations available for dynamic properties.</p></div>`
    }
    return html`<div>${annotatedProps.map(dynProp => this.renderAnnotationTile(dynProp.id, dynProp.annotation))}</div>`
  }

  /** Helper function to format the static properties annotation section. */
  formatStatPropAnnotations (): TemplateResult<1> {
    const annotatedProps = this.contentData.staticProperties
      .filter(dynProp => dynProp.annotation.trim() !== '')
    if (annotatedProps.length === 0) {
      return html`<div class="placeholder"><p>No annotations available for static properties.</p></div>`
    }
    return html`<div>${annotatedProps.map(statProp => this.renderAnnotationTile(statProp.id, statProp.annotation))}</div>`
  }

  /** Render a HTML tile for a single annotation. */
  private renderAnnotationTile (id: string, content: string): TemplateResult<1> {
    return html`<annotation-tile .id="${id}" .content="${content}"></annotation-tile>`
  }

  /** Render a HTML tile for a more complex dataset annotation structure. */
  private renderDatasetTile (data: IObservationSet): TemplateResult<1> {
    return html`<dataset-tile .data="${data}"></dataset-tile>`
  }

  /** Wrapper to render a full HTML section with annotations for a certain type of entity. */
  private renderAnnotationsSection (sectionId: string, sectionTitle: string, formatAnnotationsFn: () => TemplateResult<1>): TemplateResult<1> {
    return html`          
      <div class="section" id=${sectionId}>
        <div class="header uk-background-primary uk-margin-bottom">
          <h3 class="uk-heading-bullet uk-margin-remove-bottom">${sectionTitle}</h3>
        </div>
        <div class="annotation">
          ${formatAnnotationsFn()}
        </div>
      </div>
    `
  }

  protected render (): TemplateResult {
    return html`
      <div class="container">
        <div class="components-list">
          ${this.renderAnnotationsSection('whole-sketch', 'Sketch annotation', this.formatSketchAnnotation.bind(this))}
          ${this.renderAnnotationsSection('variables', 'Variables', this.formatVarAnnotations.bind(this))}
          ${this.renderAnnotationsSection('functions', 'Supplementary functions', this.formatFnAnnotations.bind(this))}
          ${this.renderAnnotationsSection('datasets', 'Datasets', this.formatDatasetAnnotations.bind(this))}
          ${this.renderAnnotationsSection('static', 'Static properties', this.formatStatPropAnnotations.bind(this))}
          ${this.renderAnnotationsSection('dynamic', 'Dynamic properties', this.formatDynPropAnnotations.bind(this))}
        </div> 
      </div>
    `
  }
}
