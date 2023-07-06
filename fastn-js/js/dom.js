let fastn_dom = {};

fastn_dom.common = {
    ".ft_row": {
        "value": {
            "display": "flex",
            "align-items": "start",
            "justify-content": "start",
            "flex-direction": "row",
        }
    },
    ".ft_column": {
        "value": {
            "display": "flex",
            "align-items": "start",
            "justify-content": "start",
            "flex-direction": "column",
        }
    }
};


fastn_dom.classes = { ...fastn_dom.common }
fastn_dom.unsanitised_classes = {}
fastn_dom.class_count = 0;
fastn_dom.property_map = {
    "color": "c",
    "width": "w",
    "padding": "p",
    "padding-horizontal": "ph",
    "padding-vertical": "pv",
    "padding-left": "pl",
    "padding-right": "pr",
    "padding-top": "pt",
    "padding-bottom": "pb",
    "margin": "m",
    "margin-horizontal": "mh",
    "margin-vertical": "mv",
    "margin-left": "ml",
    "margin-right": "mr",
    "margin-top": "mt",
    "margin-bottom": "mb",
    "height": "h",
    "border-width": "bw",
    "border-left-width": "blw",
    "border-right-width": "brw",
    "border-top-width": "btw",
    "border-bottom-width": "bbw",
    "border-radius": "br",
    "border-top-left-radius": "btlr",
    "border-top-right-radius": "btrr",
    "border-bottom-left-radius": "bblr",
    "border-bottom-right-radius": "bbrr",
    "border-style": "bs",
    "border-top-style": "bts",
    "border-bottom-style": "bbs",
    "border-left-style": "bls",
    "border-right-style": "brs",
    "border-color": "bc",
    "border-top-color": "btc",
    "border-bottom-color": "bbc",
    "border-left-color": "blc",
    "border-right-color": "brc",
    "background-color": "bgc",
    "z-index": "z",
    "sticky": "s",
    "top": "t",
    "bottom": "b",
    "left": "l",
    "right": "r",
    "overflow": "o",
    "overflow-x": "ox",
    "overflow-y": "oy",
    "gap": "g",
    "justify-content": "jc",
    "position": "pos",
    "flex-wrap": "fw",
    "text-transform": "tt",
    "text-align": "ta",
    "-webkit-box-orient": "wbo",
    "-webkit-line-clamp": "wlc",
    "display": "d",
    "opacity": "op",
    "cursor": "cur",
    "resize": "r",
    "max-height": "mxh",
    "min-height": "mnh",
    "max-width": "mxw",
    "min-width": "mnw",
};

// dynamic-class-css.md
fastn_dom.getClassesAsString = function() {
    let classes = Object.entries(fastn_dom.classes).map(entry => {
        return getClassAsString(entry[0], entry[1]);
    });

    /*.ft_text {
        padding: 0;
    }*/
    return `<style id="styles">
    ${classes.join("\n\t")}
    </style>`;
}

function getClassAsString(className, obj) {
    if (typeof obj.value === 'object' && obj.value !== null) {
        let value = "";
        for (let key in obj.value) {
            if (obj.value[key] === undefined || obj.value[key] === null) {
                continue
            }
            value = `${value} ${key}: ${obj.value[key]};`
        }
        return `${className} { ${value} }`
    } else {
        return `${className} { ${obj.property}: ${obj.value}; }`;
    }
}

fastn_dom.ElementKind = {
    Row: 0,
    Column: 1,
    Integer: 2,
    Decimal: 3,
    Boolean: 4,
    Text: 5,
    Image: 6,
    IFrame: 7,
    // To create parent for dynamic DOM
    Div: 8,
};

fastn_dom.PropertyKind = {
    Color: 0,
    IntegerValue: 1,
    StringValue: 2,
    Width: 3,
    Padding: 4,
    Height: 5,
    Id: 6,
    BorderWidth: 7,
    BorderStyle: 8,
    Margin: 9,
    Background: 10,
    PaddingHorizontal: 11,
    PaddingVertical: 12,
    PaddingLeft: 13,
    PaddingRight: 14,
    PaddingTop: 15,
    PaddingBottom: 16,
    MarginHorizontal: 17,
    MarginVertical: 18,
    MarginLeft: 19,
    MarginRight: 20,
    MarginTop: 21,
    MarginBottom: 22,
    Role: 23,
    ZIndex: 24,
    Sticky: 25,
    Top: 26,
    Bottom: 27,
    Left: 28,
    Right: 29,
    Overflow: 30,
    OverflowX: 31,
    OverflowY: 32,
    Spacing: 33,
    Wrap: 34,
    TextTransform: 35,
    TextIndent: 36,
    TextAlign: 37,
    LineClamp: 38,
    Opacity: 39,
    Cursor: 40,
    Resize: 41,
    MinHeight: 42,
    MaxHeight: 43,
    MinWidth: 44,
    MaxWidth: 45,
    WhiteSpace: 46,
    BorderTopWidth: 47,
    BorderBottomWidth: 48,
    BorderLeftWidth: 49,
    BorderRightWidth: 50,
    BorderRadius: 51,
    BorderTopLeftRadius: 52,
    BorderTopRightRadius: 53,
    BorderBottomLeftRadius: 54,
    BorderBottomRightRadius: 55,
    BorderStyleVertical: 56,
    BorderStyleHorizontal: 57,
    BorderLeftStyle: 58,
    BorderRightStyle: 59,
    BorderTopStyle: 60,
    BorderBottomStyle: 61,
    BorderColor: 62,
    BorderLeftColor: 63,
    BorderRightColor: 64,
    BorderTopColor: 65,
    BorderBottomColor: 66,
}



fastn_dom.Resizing = {
    FillContainer: "100%",
    HugContent: "fit-content",
    Fixed: (value) => { return value; }
}

fastn_dom.Spacing = {
    SpaceEvenly: "space-evenly",
    SpaceBetween: "space-between",
    SpaceAround: "space-around",
    Fixed: (value) => { return value; }
}


fastn_dom.BorderStyle = {
    Solid: "solid",
    Dashed: "dashed",
    Dotted: "dotted",
    Double: "double",
    Ridge: "ridge",
    Groove: "groove",
    Inset: "inset",
    Outset: "outset",
}

fastn_dom.Overflow = {
    Scroll: "scroll",
    Visible: "visible",
    Hidden: "hidden",
    Auto: "auto",
}

fastn_dom.Display = {
    Block: "block",
    Inline: "inline",
    InlineBlock: "inline-block",
}

fastn_dom.TextTransform = {
    None: "none",
    Capitalize: "capitalize",
    Uppercase: "uppercase",
    Lowercase: "lowercase",
    Inherit: "inherit",
    Initial: "initial",
}

fastn_dom.TextAlign = {
    Start: "start",
    Center: "center",
    End: "end",
    Justify: "justify",
}

fastn_dom.Cursor = {
    None: "none",
    Default: "default",
    ContextMenu: "context-menu",
    Help: "help",
    Pointer: "pointer",
    Progress: "progress",
    Wait: "wait",
    Cell: "cell",
    CrossHair: "crosshair",
    Text: "text",
    VerticalText: "vertical-text",
    Alias: "alias",
    Copy: "copy",
    Move: "move",
    NoDrop: "no-drop",
    NotAllowed: "not-allowed",
    Grab: "grab",
    Grabbing: "grabbing",
    EResize: "e-resize",
    NResize: "n-resize",
    NeResize: "ne-resize",
    SResize: "s-resize",
    SeResize: "se-resize",
    SwResize: "sw-resize",
    Wresize: "w-resize",
    Ewresize: "ew-resize",
    NsResize: "ns-resize",
    NeswResize: "nesw-resize",
    NwseResize: "nwse-resize",
    ColResize: "col-resize",
    RowResize: "row-resize",
    AllScroll: "all-scroll",
    ZoomIn: "zoom-in",
    ZoomOut: "zoom-out"
}

fastn_dom.Resize = {
    Vertical: "vertical",
    Horizontal: "horizontal",
    Both: "both",
}

fastn_dom.WhiteSpace = {
    Normal: "normal",
    NoWrap: "nowrap",
    Pre: "pre",
    PreLine: "pre-line",
    PreWrap: "pre-wrap",
    BreakSpaces: "break-spaces",
}



fastn_dom.BackgroundStyle = {
    Solid: (value) => { return value; }
}

fastn_dom.FontSize = {
    Px: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}px`})
        }
        return `${value}px`;
    },
    Em: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}em`})
        }
        return `${value}em`;
    },
    Rem: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}rem`})
        }
        return `${value}rem`;
    },
}

fastn_dom.Length = {
    Px: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}px`})
        }
        return `${value}px`;
    },
    Em: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}em`})
        }
        return `${value}em`;
    },
    Rem: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}rem`})
        }
        return `${value}rem`;
    },
    Percent: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}%`})
        }
        return `${value}%`;
    },
    Calc: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `calc(${value.get()})`})
        }
        return `calc(${value})`;
    },
    Vh: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vh`})
        }
        return `${value}vh`;
    },
    Vw: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vw`})
        }
        return `${value}vw`;
    },
    Vmin: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vmin`})
        }
        return `${value}vmin`;
    },
    Vmax: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () { return `${value.get()}vmax`})
        }
        return `${value}vmax`;
    },
    Responsive: (desktop, mobile) => {
        if (ftd.device.get() === "desktop") {
            return desktop;
        } else {
            return mobile ? mobile: desktop;
        }
    }
}



fastn_dom.Event = {
    Click: 0
}

class Node2 {
    #node;
    #parent;
    #mutables;
    constructor(parent, kind) {
        let [node, classes] = fastn_utils.htmlNode(kind);
        this.#node = fastn_virtual.document.createElement(node);
        for (let c in classes) {
            this.#node.classList.add(classes[c]);
        }
        this.#parent = parent;
        // this is where we store all the attached closures, so we can free them when we are done
        this.#mutables = [];
        /*if (!!parent.parent) {
            parent = parent.parent();
        }*/
        if (this.#parent.getNode) {
            this.#parent = this.#parent.getNode();
        }
        this.#parent.appendChild(this.#node);
    }
    parent() {
        return this.#parent;
    }
    // dynamic-class-css
    attachCss(property, value, createClass, className) {
        const propertyShort = fastn_dom.property_map[property] || property;
        let cls = `${propertyShort}-${JSON.stringify(value)}`;
        if (!!className) {
           cls = className;
        } else {
            if (!fastn_dom.unsanitised_classes[cls]) {
                fastn_dom.unsanitised_classes[cls] = ++fastn_dom.class_count;
            }
            cls = `${propertyShort}-${fastn_dom.unsanitised_classes[cls]}`;
        }
        let cssClass = className ? cls : `.${cls}`;

        const obj = { property, value };

        if (value === undefined) {
            if (!ssr && !hydrating) {
                for (const className of this.#node.classList.values()) {
                    if (className.startsWith(`${propertyShort}-`)) {
                        this.#node.classList.remove(className);
                    }
                }
            }
            return cls;
        }

        if (!ssr && !hydrating) {
            if (!!className) {
                if (!fastn_dom.classes[cssClass]) {
                    fastn_dom.classes[cssClass] = fastn_dom.classes[cssClass] || obj;
                    let styles = document.getElementById('styles');
                    styles.innerHTML = `${styles.innerHTML}${getClassAsString(cssClass, obj)}\n`;
                }
                return cls;
            }

            for (const className of this.#node.classList.values()) {
                if (className.startsWith(`${propertyShort}-`)) {
                    this.#node.classList.remove(className);
                }
            }

            if (createClass) {
                if (!fastn_dom.classes[cssClass]) {
                    fastn_dom.classes[cssClass] = fastn_dom.classes[cssClass] || obj;
                    let styles = document.getElementById('styles');
                    styles.innerHTML = `${styles.innerHTML}${getClassAsString(cssClass, obj)}\n`;
                }
                this.#node.style.removeProperty(property);
                this.#node.classList.add(cls);
            } else if (!fastn_dom.classes[cssClass]) {
                if (typeof value === 'object' && value !== null) {
                    for (let key in value) {
                        this.#node.style[key] = value[key];
                    }
                } else {
                    this.#node.style[property] = value;
                }
            } else {
                this.#node.style.removeProperty(property);
                this.#node.classList.add(cls);
            }

            return cls;
        }

        fastn_dom.classes[cssClass] = fastn_dom.classes[cssClass] || obj;

        if (!!className) {
            return cls;
        }

        this.#node.classList.add(cls);
        return cls;
    }

    attachColorCss(property, value) {
        let lightValue = fastn_utils.getStaticValue(value.get("light"));
        let darkValue = fastn_utils.getStaticValue(value.get("dark"));
        if (lightValue === darkValue) {
            this.attachCss(property, lightValue, false);
        } else {
            let lightClass = this.attachCss(property, lightValue, true);
            this.attachCss(property, darkValue, true, `body.dark .${lightClass}`);
        }
    }
    attachRoleCss(value) {
        let desktopValue = fastn_utils.getStaticValue(value.get("desktop"));
        let mobileValue = fastn_utils.getStaticValue(value.get("mobile"));
        if (fastn_utils.sameResponsiveRole(desktopValue, mobileValue)) {
            this.attachCss("role", fastn_utils.getRoleValues(desktopValue), true);
        } else {
            let desktopClass = this.attachCss("role", fastn_utils.getRoleValues(desktopValue), true);
            this.attachCss("role", fastn_utils.getRoleValues(mobileValue), true, `body.mobile .${desktopClass}`);
        }
    }

    setStaticProperty(kind, value) {
        // value can be either static or mutable
        let staticValue = fastn_utils.getStaticValue(value);
        if (kind === fastn_dom.PropertyKind.Id) {
            this.#node.id = staticValue;
        } else if (kind === fastn_dom.PropertyKind.Width) {
            this.attachCss("width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Height) {
            this.attachCss("height", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Padding) {
            this.attachCss("padding", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingHorizontal) {
            this.attachCss("padding-left", staticValue);
            this.attachCss("padding-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingVertical) {
            this.attachCss("padding-top", staticValue);
            this.attachCss("padding-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingLeft) {
            this.attachCss("padding-left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingRight) {
            this.attachCss("padding-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingTop) {
            this.attachCss("padding-top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingBottom) {
            this.attachCss("padding-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Margin) {
            this.attachCss("margin", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginHorizontal) {
            this.attachCss("margin-left", staticValue);
            this.attachCss("margin-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginVertical) {
            this.attachCss("margin-top", staticValue);
            this.attachCss("margin-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginLeft) {
            this.attachCss("margin-left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginRight) {
            this.attachCss("margin-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginTop) {
            this.attachCss("margin-top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginBottom) {
            this.attachCss("margin-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderWidth) {
            this.attachCss("border-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopWidth) {
            this.attachCss("border-top-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomWidth) {
            this.attachCss("border-bottom-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftWidth) {
            this.attachCss("border-left-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightWidth) {
            this.attachCss("border-right-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRadius) {
            this.attachCss("border-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopLeftRadius) {
            this.attachCss("border-top-left-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopRightRadius) {
            this.attachCss("border-top-right-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomLeftRadius) {
            this.attachCss("border-bottom-left-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomRightRadius) {
            this.attachCss("border-bottom-right-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyle) {
            this.attachCss("border-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyleVertical) {
            this.attachCss("border-top-style", staticValue);
            this.attachCss("border-bottom-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyleHorizontal) {
            this.attachCss("border-left-style", staticValue);
            this.attachCss("border-right-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftStyle) {
            this.attachCss("border-left-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightStyle) {
            this.attachCss("border-right-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopStyle) {
            this.attachCss("border-top-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomStyle) {
            this.attachCss("border-bottom-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.ZIndex) {
            this.attachCss("z-index", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Sticky) {
            // sticky is boolean type
            switch (staticValue) {
              case 'true':
              case true:
                this.attachCss("position", "sticky");
                break;
              case 'false':
              case false:
                this.attachCss("position", "static");
                break;
              default:
                this.attachCss("position", "static");
            }
        } else if (kind === fastn_dom.PropertyKind.Top) {
            this.attachCss("top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Bottom) {
            this.attachCss("bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Left) {
            this.attachCss("left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Right) {
            this.attachCss("right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Overflow) {
            this.attachCss("overflow", staticValue);
        } else if (kind === fastn_dom.PropertyKind.OverflowX) {
            this.attachCss("overflow-x", staticValue);
        } else if (kind === fastn_dom.PropertyKind.OverflowY) {
            this.attachCss("overflow-y", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Spacing) {
            switch (staticValue) {
              case 'space-evenly':
              case 'space-between':
              case 'space-around':
                this.attachCss("justify-content", staticValue);
                break;
              default:
                this.attachCss("gap", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.Wrap) {
            // sticky is boolean type
            switch (staticValue) {
              case 'true':
              case true:
                this.attachCss("flex-wrap", "wrap");
                break;
              case 'false':
              case false:
                this.attachCss("flex-wrap", "no-wrap");
                break;
              default:
                this.attachCss("flex-wrap", "no-wrap");
            }
        } else if (kind === fastn_dom.PropertyKind.TextTransform) {
            this.attachCss("text-transform", staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextIndent) {
            this.attachCss("text-indent", staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextAlign) {
            this.attachCss("text-align", staticValue);
        } else if (kind === fastn_dom.PropertyKind.LineClamp) {
            // -webkit-line-clamp: staticValue
            // display: -webkit-box, overflow: hidden
            // -webkit-box-orient: vertical
            this.attachCss("-webkit-line-clamp", staticValue);
            this.attachCss("display", "-webkit-box");
            this.attachCss("overflow", "hidden");
            this.attachCss("-webkit-box-orient", "vertical");
        } else if (kind === fastn_dom.PropertyKind.Opacity) {
            this.attachCss("opacity", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Cursor) {
            this.attachCss("cursor", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Resize) {
            // overflow: auto, resize: staticValue
            this.attachCss("resize", staticValue);
            this.attachCss("overflow", "auto");
        } else if (kind === fastn_dom.PropertyKind.MinHeight) {
            this.attachCss("min-height", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MaxHeight) {
            this.attachCss("max-height", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MinWidth) {
            this.attachCss("min-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MaxWidth) {
            this.attachCss("max-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.WhiteSpace) {
            this.attachCss("white-space", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderColor) {
            this.attachColorCss("border-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftColor) {
            this.attachColorCss("border-left-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightColor) {
            this.attachColorCss("border-right-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopColor) {
            this.attachColorCss("border-top-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomColor) {
            this.attachColorCss("border-bottom-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Color) {
            this.attachColorCss("color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Background) {
            this.attachColorCss("background-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Role) {
            this.attachRoleCss(staticValue);
        } else if (kind === fastn_dom.PropertyKind.IntegerValue ||
            kind === fastn_dom.PropertyKind.StringValue
        ) {
            this.#node.innerHTML = staticValue;
        } else {
            throw ("invalid fastn_dom.PropertyKind: " + kind);
        }
    }
    setProperty(kind, value) {
        if (value instanceof fastn.mutableClass) {
            this.setDynamicProperty(kind, [value], () => { return value.get(); });
        } else {
            this.setStaticProperty(kind, value);
        }
    }
    setDynamicProperty(kind, deps, func) {
        let closure = fastn.closure(func).addNodeProperty(this, kind);
        for (let dep in deps) {
            if (!deps[dep].addClosure) {
                continue;
            }
            deps[dep].addClosure(closure);
            this.#mutables.push(deps[dep]);
        }
    }
    getNode() {
        return this.#node;
    }
    addEventHandler(event, func) {
        if (event === fastn_dom.Event.Click) {
            this.#node.onclick = func;
        }
    }
    destroy() {
        for (let i = 0; i < this.#mutables.length; i++) {
            this.#mutables[i].unlinkNode(this);
        }
        this.#node.remove();
        this.#mutables = null;
        this.#parent = null;
        this.#node = null;
    }
}

class ConditionalDom {
    #parent;
    #node_constructor;
    #condition;
    #mutables;
    #conditionUI;

    constructor(parent, deps, condition, node_constructor) {
        let domNode = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Div);

        this.#conditionUI = null;
        let closure = fastn.closure(() => {
            if (condition()) {
                if (this.#conditionUI) {
                    this.#conditionUI.destroy();
                }
                this.#conditionUI = node_constructor(domNode);
            } else if (this.#conditionUI) {
                this.#conditionUI.destroy();
                this.#conditionUI = null;
            }
        })
        deps.forEach(dep => dep.addClosure(closure));


        this.#parent = domNode;
        this.#node_constructor = node_constructor;
        this.#condition = condition;
        this.#mutables = [];
    }

    getParent() {
        return this.#parent;
    }
}

fastn_dom.createKernel = function (parent, kind) {
    return new Node2(parent, kind);
}

fastn_dom.conditionalDom = function (parent, deps, condition, node_constructor) {
    return new ConditionalDom(parent, deps, condition, node_constructor);
}

class ForLoop {
    #node_constructor;
    #list;
    #wrapper;
    constructor(parent, node_constructor, list) {
        this.#wrapper = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Div);
        this.#node_constructor = node_constructor;
        this.#list = list;
        for (let idx in list.getList()) {
            // let v = list.get(idx);
            // node_constructor(this.#wrapper, v.item, v.index).done();
            this.createNode(idx);
        }
    }
    createNode(index) {
        let v = this.#list.get(index);
        this.#node_constructor(this.#wrapper, v.item, v.index);
    }

    getParent() {
        return this.#wrapper;
    }
}

fastn_dom.forLoop = function (parent, node_constructor, list) {
    return new ForLoop(parent, node_constructor, list);
}
