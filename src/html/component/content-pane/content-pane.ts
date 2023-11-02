class ContentPane extends HTMLElement {
    shadow;
    heading;

    constructor() {
        super();
        const template = document.getElementById('content-pane')!;
        // @ts-ignore
        const content = template.content;
        this.shadow = this.attachShadow({mode: 'open'});
        this.shadow.appendChild(content.cloneNode(true));
        const linkElem = document.createElement('link');
        linkElem.setAttribute('rel', 'stylesheet');
        linkElem.setAttribute('href', 'component/content-pane/content-pane.less');
        this.shadow.appendChild(linkElem);
        this.heading = document.createElement('h1');
        this.heading.classList.add('uk-heading-large', 'uk-text-success');
        this.shadow.appendChild(this.heading)
    }

    connectedCallback() {
        this.addEventListener('switch-tab', (e) => {
            const message = (e as CustomEvent).detail.content;
            console.log('message recieved', message);
            this.heading.innerText = message;
        })
    }

}

customElements.define('content-pane', ContentPane);