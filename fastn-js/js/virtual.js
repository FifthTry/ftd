let fastn_virtual = {}

let id_counter = 0;
let hydrating = false;
let ssr = false;

class ClassList {
    #classes = [];
    add(item) {
        this.#classes.push(item);
    }
    toString() {
        return this.#classes.join(' ');
    }
}

class Node {
    #id
    #tagName
    #children
    #attributes
    constructor(id, tagName) {
        this.#tagName = tagName;
        this.#id = id;
        this.classList = new ClassList();
        this.#children = [];
        this.#attributes = {};
        this.innerHTML = "";
        this.style = {};
        this.onclick = null;
    }
    appendChild(c) {
        this.#children.push(c);
    }
    removeLastChild() {
        this.#children.pop();
    }
    addAttribute(attribute, value) {
        this.#attributes[attribute] = value;
    }
    updateTagName(tagName) {
        this.#tagName = tagName;
    }
    toHtmlAsString() {
        const openingTag = `<${this.#tagName}${this.getDataIdString()}${this.getAttributesString()}${this.getClassString()}${this.getStyleString()}>`;
        const closingTag = `</${this.#tagName}>`;
        const innerHTML = this.innerHTML;
        const childNodes = this.#children.map(child => child.toHtmlAsString()).join('');

        return `${openingTag}${innerHTML}${childNodes}${closingTag}`;
    }
    getDataIdString() {
        return ` data-id="${this.#id}"`;
    }
    getClassString() {
        const classList = this.classList.toString();
        return classList ? ` class="${classList}"` : '';
    }
    getStyleString() {
        const styleProperties = Object.entries(this.style)
            .map(([prop, value]) => `${prop}:${value}`)
            .join(';');
        return styleProperties ? ` style="${styleProperties}"` : '';
    }
    getAttributesString() {
        const nodeAttributes = Object.entries(this.#attributes)
            .map(([attribute, value]) => {
                if (value != null) {
                    return `${attribute}=${value}`;
                }
                return `${attribute}`;
            }).join(' ');
        return nodeAttributes ? ` ${nodeAttributes}` : '';
    }
}

class Document2 {
    createElement(tagName) {
        id_counter++;
        if (ssr) {
            return new Node(id_counter, tagName);
        }

        if (tagName === "body") {
            return window.document.body;
        }

        if (hydrating) {
            return this.getElementByDataID(id_counter);
        } else {
            return window.document.createElement(tagName);
        }
    }

    getElementByDataID(id) {
        return window.document.querySelector(`[data-id=\"${id}\"]`);
    }
}

fastn_virtual.document = new Document2();



fastn_virtual.hydrate = function(main) {
    hydrating = true;
    let body = fastn_virtual.document.createElement("body");
    main(body);
    id_counter = 0;
    hydrating = false;
}

fastn_virtual.ssr = function(main) {
    ssr = true;
    let body = fastn_virtual.document.createElement("body");
    main(body)
    ssr = false;
    id_counter = 0;
    return body.toHtmlAsString() + fastn_dom.getClassesAsString();
}
