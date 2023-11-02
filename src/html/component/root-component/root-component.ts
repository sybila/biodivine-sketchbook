class RootComponent extends HTMLElement {
    shadow;

    constructor() {
        super();
        const template = document.getElementById('root-component')!;
        // @ts-ignore
        const content = template.content;
        this.shadow = this.attachShadow({mode: 'open'});
        this.shadow.appendChild(content.cloneNode(true));
        const linkElem = document.createElement('link');
        linkElem.setAttribute('rel', 'stylesheet');
        linkElem.setAttribute('href', 'component/root-component/root-component.less');
        this.shadow.appendChild(linkElem);
        this.switchTab();
    }

    connectedCallback() {
    }

    private switchTab() {
        this.addEventListener('switch-tab', (e) => {
            console.log(e);
        })
    }
}

customElements.define('root-component', RootComponent);
