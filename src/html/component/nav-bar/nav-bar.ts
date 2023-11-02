class NavBar extends HTMLElement {
    shadow;

    constructor() {
        super();
        const template = document.getElementById('nav-bar')! as HTMLTemplateElement;
        const content = template.content;
        this.shadow = this.attachShadow({mode: 'open'});
        this.shadow.appendChild(content.cloneNode(true));

        const linkElem = document.createElement('link');
        linkElem.setAttribute('rel', 'stylesheet');
        linkElem.setAttribute('href', 'component/nav-bar/nav-bar.less');
        this.shadow.appendChild(linkElem);
    }

    connectedCallback() {
    }

}

customElements.define('nav-bar', NavBar);