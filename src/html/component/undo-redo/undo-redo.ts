class UndoRedo extends HTMLElement {
    shadow;

    constructor() {
        super();
        const template = document.getElementById('undo-redo')! as HTMLTemplateElement;
        const content = template.content;
        this.shadow = this.attachShadow({mode: 'open'});
        this.shadow.appendChild(content.cloneNode(true));
        const linkElem = document.createElement('link');
        linkElem.setAttribute('rel', 'stylesheet');
        linkElem.setAttribute('href', 'component/undo-redo/undo-redo.less');
        this.shadow.appendChild(linkElem);
    }

    connectedCallback() {

    }

}

customElements.define('undo-redo', UndoRedo);