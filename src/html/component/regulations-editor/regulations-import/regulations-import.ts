import { LitElement, TemplateResult, html, unsafeCSS } from "lit";
import { customElement, state } from "lit/decorators.js";

// Include the content of regulations-import.less as raw string variable.
import style_less from './regulations-import.less?inline'

@customElement('regulations-import')
export default class RegulationsImport extends LitElement {

    // Informs lit that the CSS included for this component comes 
    // from the `style_less` variable (we could also write CSS
    // right here using css`...` string literal).
    static styles = unsafeCSS(style_less)

    // Internal state of the component that is not visible to the
    // "outside" world. Use @property to declare state that can be
    // modified by external components. 
    @state() counter = 0

    increment() {
        this.counter += 1
    }

    protected render(): TemplateResult {
        return html`
            <div class="uk-margin-top uk-margin-bottom uk-margin-left uk-margin-right">
                Count: <span class="uk-label">${this.counter}</span>
                <button @click="${this.increment}" class="uk-button uk-button-primary">Add</button>
            </div>
        `;
    }
    
}