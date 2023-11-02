const SAVE_KEY = 'tabs';
class TabSwitcher extends HTMLElement {
    tabs: HTMLButtonElement[] = [];
    tabIndex = 0;
    shadow;


    constructor() {
        super();
        const template = document.getElementById('tab-switcher')!;
        // @ts-ignore
        const content = template.content;
        this.shadow = this.attachShadow({mode: 'open'});
        this.shadow.appendChild(content.cloneNode(true));
        const linkElem = document.createElement('link');
        linkElem.setAttribute('rel', 'stylesheet');
        linkElem.setAttribute('href', 'component/tab-bar/tab-bar.less');
        this.shadow.appendChild(linkElem);
        this.loadTabs();
    }

    connectedCallback() {
        this.addTab();
        this.resetButton();
    }

    private addTab() {
        const addTabButton = this.shadow.querySelector('.new-tab-button');
        addTabButton?.addEventListener('click', () => {
            this.tabIndex++;
            this.createTab(this.tabIndex, `Tab ${this.tabIndex}`, ['tab', 'uk-button',  'uk-button-secondary']);
        });
    }

    private resetButton() {
        const resetButton = document.createElement('button');
        resetButton.classList.add('tab', 'uk-button',  'uk-button-secondary');
        resetButton.textContent = 'reset'
        resetButton.onclick = (_) => {
            this.tabIndex = 0;
            this.tabs.forEach(tab => tab.remove());
            this.tabs = [];
            localStorage.removeItem(SAVE_KEY);
        }
        this.shadow.querySelector('.tabs')?.appendChild(resetButton);
    }

    private saveTabs() {
        const tabData = this.tabs.map((tab) => ({
            classList: Array.from(tab.classList),
            textContent: tab.textContent,
            index: tab.dataset.index
        }));
        localStorage.setItem(SAVE_KEY, JSON.stringify(tabData));
    }

    private loadTabs() {
        const tabData = JSON.parse(localStorage.getItem(SAVE_KEY) ?? '[]');
        tabData.forEach((data: {classList: string[], textContent: string, index: string}) => {
            this.createTab(+data.index, data.textContent, data.classList);
            this.tabIndex = Math.max(this.tabIndex, +data.index)
        })
        console.log('tabs loaded')
        console.log('index: ', this.tabIndex)
    }

    private createTab(index: number, title: string, classList: string[]) {
        const newTabButton = document.createElement('button');
        newTabButton.classList.add(...classList);
        newTabButton.onauxclick = (_) => {
            this.tabs = this.tabs.filter((tab) => tab !== newTabButton);
            newTabButton.remove();
            this.saveTabs();
            console.log(`closed tab ${index}`);
        };
        newTabButton.onclick = (_) => {
            const tabIndex =  +newTabButton.dataset.index!;
            console.log(`clicked tab ${tabIndex}`)
        }
        newTabButton.dataset.index = index.toString();
        newTabButton.textContent = title;
        this.tabs.push(newTabButton);

        this.saveTabs();

        const addTabButton = this.shadow.querySelector('.new-tab-button');
        this.shadow.querySelector('.tabs')?.insertBefore(newTabButton, addTabButton);

    }
}

customElements.define('tab-switcher', TabSwitcher);