import {LitElement, html} from 'lit-element';

export class SseqPageIndicator extends LitElement {
    static get properties() {
        return { 
            page_value : { type: String }
        };
    }

    constructor(){
        super(); 
        this.page_value = "";
    }

    firstUpdated(changedProperties) {
        let elt = this;
        while(elt !== undefined && elt.nodeName !== "SSEQ-DISPLAY"){
            elt = elt.parentElement;
        }
        if(elt === undefined){
            throw Error("sseq-class-highlighter must be a descendant of sseq-display.");
        }
        this.disp = elt;
        this.page_value = this.disp.page;
        this.disp.addEventListener("page-change", (e) => {
            this.page_value = e.detail[0];
        });
    }

    getPageDescriptor(pageRange) {
        if (!this.sseq) {
            return;  
        }

        let basePage = 2;
        if(this.sseq.page_list.includes(1)){
            basePage = 1;
        }
        if (pageRange[0] === INFINITY) {
            return "Page ∞";
        }
        if (pageRange === 0) {
            return `Page ${basePage} with all differentials`;
        }
        if (pageRange === 1 && basePage === 2) {
            return `Page ${basePage} with no differentials`;
        }
        if (pageRange.length) {
            if(pageRange[1] === INFINITY){
                return `Page ${pageRange[0]} with all differentials`;
            }
            if(pageRange[1] === -1){
                return `Page ${pageRange[0]} with no differentials`;
            }

            if(pageRange[0] === pageRange[1]){
                return `Page ${pageRange[0]}`;
            }

            return `Pages ${pageRange[0]} – ${pageRange[1]}`.replace(INFINITY, "∞");
        }
        return `Page ${pageRange}`;
    }

    render() {
        return html`<p>${this.getPageDescriptor(this.page_value)}</p>`;
    }
}
customElements.define('sseq-page-indicator', SseqPageIndicator);
