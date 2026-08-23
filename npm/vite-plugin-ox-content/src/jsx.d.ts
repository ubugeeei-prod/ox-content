/**
 * JSX Type Definitions for Static HTML Generation
 *
 * These types enable TypeScript support for JSX in ox-content themes.
 */

import type { JSXNode, JSXChild } from "./jsx-html";

export namespace JSX {
  /**
   * The type returned by JSX expressions.
   */
  export type Element = JSXNode;

  /**
   * Attributes accepted on every component element, on top of its own props.
   *
   * Intrinsic elements do not go through this — they pick `key` up from
   * `HTMLAttributes` instead, the same way React's types do it.
   */
  export interface IntrinsicAttributes {
    key?: string | number;
  }

  /**
   * Props for intrinsic elements (HTML tags).
   */
  export interface IntrinsicElements {
    // Document metadata
    html: HtmlHTMLAttributes;
    head: HTMLAttributes;
    title: HTMLAttributes;
    base: BaseHTMLAttributes;
    link: LinkHTMLAttributes;
    meta: MetaHTMLAttributes;
    style: StyleHTMLAttributes;

    // Sections
    body: HTMLAttributes;
    article: HTMLAttributes;
    section: HTMLAttributes;
    nav: HTMLAttributes;
    aside: HTMLAttributes;
    header: HTMLAttributes;
    footer: HTMLAttributes;
    main: HTMLAttributes;
    address: HTMLAttributes;

    // Headings
    h1: HTMLAttributes;
    h2: HTMLAttributes;
    h3: HTMLAttributes;
    h4: HTMLAttributes;
    h5: HTMLAttributes;
    h6: HTMLAttributes;
    hgroup: HTMLAttributes;

    // Text content
    p: HTMLAttributes;
    hr: HTMLAttributes;
    pre: HTMLAttributes;
    blockquote: BlockquoteHTMLAttributes;
    ol: OlHTMLAttributes;
    ul: HTMLAttributes;
    li: LiHTMLAttributes;
    dl: HTMLAttributes;
    dt: HTMLAttributes;
    dd: HTMLAttributes;
    figure: HTMLAttributes;
    figcaption: HTMLAttributes;
    div: HTMLAttributes;

    // Inline text
    a: AnchorHTMLAttributes;
    em: HTMLAttributes;
    strong: HTMLAttributes;
    small: HTMLAttributes;
    s: HTMLAttributes;
    cite: HTMLAttributes;
    q: QuoteHTMLAttributes;
    dfn: HTMLAttributes;
    abbr: HTMLAttributes;
    ruby: HTMLAttributes;
    rt: HTMLAttributes;
    rp: HTMLAttributes;
    data: DataHTMLAttributes;
    time: TimeHTMLAttributes;
    code: HTMLAttributes;
    var: HTMLAttributes;
    samp: HTMLAttributes;
    kbd: HTMLAttributes;
    sub: HTMLAttributes;
    sup: HTMLAttributes;
    i: HTMLAttributes;
    b: HTMLAttributes;
    u: HTMLAttributes;
    mark: HTMLAttributes;
    bdi: HTMLAttributes;
    bdo: HTMLAttributes;
    span: HTMLAttributes;
    br: HTMLAttributes;
    wbr: HTMLAttributes;

    // Edits
    ins: InsHTMLAttributes;
    del: DelHTMLAttributes;

    // Embedded content
    picture: HTMLAttributes;
    source: SourceHTMLAttributes;
    img: ImgHTMLAttributes;
    iframe: IframeHTMLAttributes;
    embed: EmbedHTMLAttributes;
    object: ObjectHTMLAttributes;
    param: ParamHTMLAttributes;
    video: VideoHTMLAttributes;
    audio: AudioHTMLAttributes;
    track: TrackHTMLAttributes;
    map: MapHTMLAttributes;
    area: AreaHTMLAttributes;

    // SVG
    svg: SVGAttributes;
    path: SVGAttributes;
    circle: SVGAttributes;
    ellipse: SVGAttributes;
    line: SVGAttributes;
    polygon: SVGAttributes;
    polyline: SVGAttributes;
    rect: SVGAttributes;
    g: SVGAttributes;
    defs: SVGAttributes;
    symbol: SVGAttributes;
    use: SVGAttributes;
    text: SVGAttributes;
    tspan: SVGAttributes;
    image: SVGAttributes;
    clipPath: SVGAttributes;
    mask: SVGAttributes;
    pattern: SVGAttributes;
    linearGradient: SVGAttributes;
    radialGradient: SVGAttributes;
    stop: SVGAttributes;
    filter: SVGAttributes;
    feBlend: SVGAttributes;
    feColorMatrix: SVGAttributes;
    feComponentTransfer: SVGAttributes;
    feComposite: SVGAttributes;
    feConvolveMatrix: SVGAttributes;
    feDiffuseLighting: SVGAttributes;
    feDisplacementMap: SVGAttributes;
    feFlood: SVGAttributes;
    feGaussianBlur: SVGAttributes;
    feImage: SVGAttributes;
    feMerge: SVGAttributes;
    feMergeNode: SVGAttributes;
    feMorphology: SVGAttributes;
    feOffset: SVGAttributes;
    feSpecularLighting: SVGAttributes;
    feTile: SVGAttributes;
    feTurbulence: SVGAttributes;

    // Tables
    table: TableHTMLAttributes;
    caption: HTMLAttributes;
    colgroup: ColgroupHTMLAttributes;
    col: ColHTMLAttributes;
    tbody: HTMLAttributes;
    thead: HTMLAttributes;
    tfoot: HTMLAttributes;
    tr: HTMLAttributes;
    td: TdHTMLAttributes;
    th: ThHTMLAttributes;

    // Forms
    form: FormHTMLAttributes;
    label: LabelHTMLAttributes;
    input: InputHTMLAttributes;
    button: ButtonHTMLAttributes;
    select: SelectHTMLAttributes;
    datalist: HTMLAttributes;
    optgroup: OptgroupHTMLAttributes;
    option: OptionHTMLAttributes;
    textarea: TextareaHTMLAttributes;
    output: OutputHTMLAttributes;
    progress: ProgressHTMLAttributes;
    meter: MeterHTMLAttributes;
    fieldset: FieldsetHTMLAttributes;
    legend: HTMLAttributes;

    // Interactive
    details: DetailsHTMLAttributes;
    summary: HTMLAttributes;
    dialog: DialogHTMLAttributes;
    menu: MenuHTMLAttributes;

    // Scripting
    script: ScriptHTMLAttributes;
    noscript: HTMLAttributes;
    template: HTMLAttributes;
    slot: SlotHTMLAttributes;
    canvas: CanvasHTMLAttributes;
  }

  /**
   * Base HTML attributes shared by all elements.
   */
  export interface HTMLAttributes {
    /**
     * List key. The automatic runtime hoists it out of the props object and
     * passes it as `jsx(type, props, key)`, where static HTML generation
     * ignores it — but it still has to type-check, or every `.map()` over a
     * list of elements fails to compile.
     */
    key?: string | number;

    // Global attributes
    accessKey?: string;
    autoCapitalize?: string;
    autoFocus?: boolean;
    class?: string;
    className?: string;
    contentEditable?: boolean | "true" | "false" | "inherit";
    contextMenu?: string;
    dir?: "ltr" | "rtl" | "auto";
    draggable?: boolean | "true" | "false";
    enterKeyHint?: "enter" | "done" | "go" | "next" | "previous" | "search" | "send";
    hidden?: boolean;
    id?: string;
    inert?: boolean;
    inputMode?: "none" | "text" | "tel" | "url" | "email" | "numeric" | "decimal" | "search";
    is?: string;
    itemId?: string;
    itemProp?: string;
    itemRef?: string;
    itemScope?: boolean;
    itemType?: string;
    lang?: string;
    nonce?: string;
    part?: string;
    popover?: "auto" | "manual";
    role?: string;
    slot?: string;
    spellCheck?: boolean | "true" | "false";
    style?: string | Record<string, string | number>;
    tabIndex?: number;
    title?: string;
    translate?: "yes" | "no";

    // Data attributes
    [key: `data-${string}`]: string | number | boolean | undefined;

    // ARIA attributes
    "aria-activedescendant"?: string;
    "aria-atomic"?: boolean | "true" | "false";
    "aria-autocomplete"?: "none" | "inline" | "list" | "both";
    "aria-busy"?: boolean | "true" | "false";
    "aria-checked"?: boolean | "true" | "false" | "mixed";
    "aria-colcount"?: number;
    "aria-colindex"?: number;
    "aria-colspan"?: number;
    "aria-controls"?: string;
    "aria-current"?: boolean | "true" | "false" | "page" | "step" | "location" | "date" | "time";
    "aria-describedby"?: string;
    "aria-details"?: string;
    "aria-disabled"?: boolean | "true" | "false";
    "aria-dropeffect"?: "none" | "copy" | "execute" | "link" | "move" | "popup";
    "aria-errormessage"?: string;
    "aria-expanded"?: boolean | "true" | "false";
    "aria-flowto"?: string;
    "aria-grabbed"?: boolean | "true" | "false";
    "aria-haspopup"?: boolean | "true" | "false" | "menu" | "listbox" | "tree" | "grid" | "dialog";
    "aria-hidden"?: boolean | "true" | "false";
    "aria-invalid"?: boolean | "true" | "false" | "grammar" | "spelling";
    "aria-keyshortcuts"?: string;
    "aria-label"?: string;
    "aria-labelledby"?: string;
    "aria-level"?: number;
    "aria-live"?: "off" | "assertive" | "polite";
    "aria-modal"?: boolean | "true" | "false";
    "aria-multiline"?: boolean | "true" | "false";
    "aria-multiselectable"?: boolean | "true" | "false";
    "aria-orientation"?: "horizontal" | "vertical";
    "aria-owns"?: string;
    "aria-placeholder"?: string;
    "aria-posinset"?: number;
    "aria-pressed"?: boolean | "true" | "false" | "mixed";
    "aria-readonly"?: boolean | "true" | "false";
    "aria-relevant"?:
      | "additions"
      | "additions removals"
      | "additions text"
      | "all"
      | "removals"
      | "removals additions"
      | "removals text"
      | "text"
      | "text additions"
      | "text removals";
    "aria-required"?: boolean | "true" | "false";
    "aria-roledescription"?: string;
    "aria-rowcount"?: number;
    "aria-rowindex"?: number;
    "aria-rowspan"?: number;
    "aria-selected"?: boolean | "true" | "false";
    "aria-setsize"?: number;
    "aria-sort"?: "none" | "ascending" | "descending" | "other";
    "aria-valuemax"?: number;
    "aria-valuemin"?: number;
    "aria-valuenow"?: number;
    "aria-valuetext"?: string;

    // Children
    children?: JSXChild;
  }

  // Element-specific attributes
  export interface HtmlHTMLAttributes extends HTMLAttributes {
    manifest?: string;
  }

  export interface BaseHTMLAttributes extends HTMLAttributes {
    href?: string;
    target?: string;
  }

  export interface LinkHTMLAttributes extends HTMLAttributes {
    as?: string;
    crossOrigin?: string;
    href?: string;
    hrefLang?: string;
    integrity?: string;
    media?: string;
    referrerPolicy?: string;
    rel?: string;
    sizes?: string;
    type?: string;
  }

  export interface MetaHTMLAttributes extends HTMLAttributes {
    charset?: string;
    charSet?: string;
    content?: string;
    "http-equiv"?: string;
    httpEquiv?: string;
    name?: string;
    property?: string;
  }

  export interface StyleHTMLAttributes extends HTMLAttributes {
    media?: string;
    scoped?: boolean;
    type?: string;
  }

  export interface BlockquoteHTMLAttributes extends HTMLAttributes {
    cite?: string;
  }

  export interface OlHTMLAttributes extends HTMLAttributes {
    reversed?: boolean;
    start?: number;
    type?: "1" | "a" | "A" | "i" | "I";
  }

  export interface LiHTMLAttributes extends HTMLAttributes {
    value?: number;
  }

  export interface AnchorHTMLAttributes extends HTMLAttributes {
    download?: string | boolean;
    href?: string;
    hrefLang?: string;
    media?: string;
    ping?: string;
    referrerPolicy?: string;
    rel?: string;
    target?: string;
    type?: string;
  }

  export interface QuoteHTMLAttributes extends HTMLAttributes {
    cite?: string;
  }

  export interface DataHTMLAttributes extends HTMLAttributes {
    value?: string | number;
  }

  export interface TimeHTMLAttributes extends HTMLAttributes {
    dateTime?: string;
  }

  export interface InsHTMLAttributes extends HTMLAttributes {
    cite?: string;
    dateTime?: string;
  }

  export interface DelHTMLAttributes extends HTMLAttributes {
    cite?: string;
    dateTime?: string;
  }

  export interface SourceHTMLAttributes extends HTMLAttributes {
    media?: string;
    sizes?: string;
    src?: string;
    srcSet?: string;
    type?: string;
  }

  export interface ImgHTMLAttributes extends HTMLAttributes {
    alt?: string;
    crossOrigin?: "anonymous" | "use-credentials";
    decoding?: "async" | "auto" | "sync";
    height?: number | string;
    loading?: "eager" | "lazy";
    referrerPolicy?: string;
    sizes?: string;
    src?: string;
    srcSet?: string;
    useMap?: string;
    width?: number | string;
  }

  export interface IframeHTMLAttributes extends HTMLAttributes {
    allow?: string;
    allowFullScreen?: boolean;
    height?: number | string;
    loading?: "eager" | "lazy";
    name?: string;
    referrerPolicy?: string;
    sandbox?: string;
    src?: string;
    srcDoc?: string;
    width?: number | string;
  }

  export interface EmbedHTMLAttributes extends HTMLAttributes {
    height?: number | string;
    src?: string;
    type?: string;
    width?: number | string;
  }

  export interface ObjectHTMLAttributes extends HTMLAttributes {
    data?: string;
    form?: string;
    height?: number | string;
    name?: string;
    type?: string;
    useMap?: string;
    width?: number | string;
  }

  export interface ParamHTMLAttributes extends HTMLAttributes {
    name?: string;
    value?: string | number;
  }

  export interface VideoHTMLAttributes extends HTMLAttributes {
    autoPlay?: boolean;
    controls?: boolean;
    crossOrigin?: "anonymous" | "use-credentials";
    height?: number | string;
    loop?: boolean;
    muted?: boolean;
    playsInline?: boolean;
    poster?: string;
    preload?: "none" | "metadata" | "auto";
    src?: string;
    width?: number | string;
  }

  export interface AudioHTMLAttributes extends HTMLAttributes {
    autoPlay?: boolean;
    controls?: boolean;
    crossOrigin?: "anonymous" | "use-credentials";
    loop?: boolean;
    muted?: boolean;
    preload?: "none" | "metadata" | "auto";
    src?: string;
  }

  export interface TrackHTMLAttributes extends HTMLAttributes {
    default?: boolean;
    kind?: "subtitles" | "captions" | "descriptions" | "chapters" | "metadata";
    label?: string;
    src?: string;
    srcLang?: string;
  }

  export interface MapHTMLAttributes extends HTMLAttributes {
    name?: string;
  }

  export interface AreaHTMLAttributes extends HTMLAttributes {
    alt?: string;
    coords?: string;
    download?: string | boolean;
    href?: string;
    ping?: string;
    referrerPolicy?: string;
    rel?: string;
    shape?: "rect" | "circle" | "poly" | "default";
    target?: string;
  }

  export interface TableHTMLAttributes extends HTMLAttributes {
    cellPadding?: number | string;
    cellSpacing?: number | string;
  }

  export interface ColgroupHTMLAttributes extends HTMLAttributes {
    span?: number;
  }

  export interface ColHTMLAttributes extends HTMLAttributes {
    span?: number;
  }

  export interface TdHTMLAttributes extends HTMLAttributes {
    colSpan?: number;
    headers?: string;
    rowSpan?: number;
  }

  export interface ThHTMLAttributes extends HTMLAttributes {
    abbr?: string;
    colSpan?: number;
    headers?: string;
    rowSpan?: number;
    scope?: "row" | "col" | "rowgroup" | "colgroup";
  }

  export interface FormHTMLAttributes extends HTMLAttributes {
    acceptCharset?: string;
    action?: string;
    autoComplete?: string;
    encType?: string;
    method?: "get" | "post" | "dialog";
    name?: string;
    noValidate?: boolean;
    target?: string;
  }

  export interface LabelHTMLAttributes extends HTMLAttributes {
    for?: string;
    htmlFor?: string;
  }

  export interface InputHTMLAttributes extends HTMLAttributes {
    accept?: string;
    alt?: string;
    autoComplete?: string;
    capture?: boolean | "user" | "environment";
    checked?: boolean;
    disabled?: boolean;
    form?: string;
    formAction?: string;
    formEncType?: string;
    formMethod?: string;
    formNoValidate?: boolean;
    formTarget?: string;
    height?: number | string;
    list?: string;
    max?: number | string;
    maxLength?: number;
    min?: number | string;
    minLength?: number;
    multiple?: boolean;
    name?: string;
    pattern?: string;
    placeholder?: string;
    readOnly?: boolean;
    required?: boolean;
    size?: number;
    src?: string;
    step?: number | string;
    type?: string;
    value?: string | number;
    width?: number | string;
  }

  export interface ButtonHTMLAttributes extends HTMLAttributes {
    disabled?: boolean;
    form?: string;
    formAction?: string;
    formEncType?: string;
    formMethod?: string;
    formNoValidate?: boolean;
    formTarget?: string;
    name?: string;
    type?: "submit" | "reset" | "button";
    value?: string | number;
  }

  export interface SelectHTMLAttributes extends HTMLAttributes {
    autoComplete?: string;
    disabled?: boolean;
    form?: string;
    multiple?: boolean;
    name?: string;
    required?: boolean;
    size?: number;
    value?: string | number;
  }

  export interface OptgroupHTMLAttributes extends HTMLAttributes {
    disabled?: boolean;
    label?: string;
  }

  export interface OptionHTMLAttributes extends HTMLAttributes {
    disabled?: boolean;
    label?: string;
    selected?: boolean;
    value?: string | number;
  }

  export interface TextareaHTMLAttributes extends HTMLAttributes {
    autoComplete?: string;
    cols?: number;
    disabled?: boolean;
    form?: string;
    maxLength?: number;
    minLength?: number;
    name?: string;
    placeholder?: string;
    readOnly?: boolean;
    required?: boolean;
    rows?: number;
    value?: string;
    wrap?: "hard" | "soft";
  }

  export interface OutputHTMLAttributes extends HTMLAttributes {
    for?: string;
    form?: string;
    name?: string;
  }

  export interface ProgressHTMLAttributes extends HTMLAttributes {
    max?: number;
    value?: number | string;
  }

  export interface MeterHTMLAttributes extends HTMLAttributes {
    form?: string;
    high?: number;
    low?: number;
    max?: number;
    min?: number;
    optimum?: number;
    value?: number | string;
  }

  export interface FieldsetHTMLAttributes extends HTMLAttributes {
    disabled?: boolean;
    form?: string;
    name?: string;
  }

  export interface DetailsHTMLAttributes extends HTMLAttributes {
    open?: boolean;
  }

  export interface DialogHTMLAttributes extends HTMLAttributes {
    open?: boolean;
  }

  export interface MenuHTMLAttributes extends HTMLAttributes {
    type?: "context" | "toolbar";
  }

  export interface ScriptHTMLAttributes extends HTMLAttributes {
    async?: boolean;
    crossOrigin?: string;
    defer?: boolean;
    integrity?: string;
    noModule?: boolean;
    referrerPolicy?: string;
    src?: string;
    type?: string;
  }

  export interface SlotHTMLAttributes extends HTMLAttributes {
    name?: string;
  }

  export interface CanvasHTMLAttributes extends HTMLAttributes {
    height?: number | string;
    width?: number | string;
  }

  export interface SVGAttributes extends HTMLAttributes {
    // SVG-specific attributes
    viewBox?: string;
    xmlns?: string;
    fill?: string;
    stroke?: string;
    strokeWidth?: string | number;
    strokeLinecap?: "butt" | "round" | "square";
    strokeLinejoin?: "miter" | "round" | "bevel";
    d?: string;
    cx?: string | number;
    cy?: string | number;
    r?: string | number;
    rx?: string | number;
    ry?: string | number;
    x?: string | number;
    y?: string | number;
    x1?: string | number;
    y1?: string | number;
    x2?: string | number;
    y2?: string | number;
    width?: string | number;
    height?: string | number;
    points?: string;
    transform?: string;
    opacity?: string | number;
    clipPath?: string;
    mask?: string;
    filter?: string;
  }

  /**
   * Type for intrinsic element constructors.
   */
  export type IntrinsicElementType = keyof IntrinsicElements;

  /**
   * Props for a specific intrinsic element.
   */
  export type IntrinsicElementProps<T extends IntrinsicElementType> = IntrinsicElements[T];
}

type JsxRuntimeIntrinsicElements = import("./jsx.d.ts").JSX.IntrinsicElements;
type JsxRuntimeIntrinsicAttributes = import("./jsx.d.ts").JSX.IntrinsicAttributes;

declare global {
  namespace JSX {
    type Element = import("./jsx-html").JSXNode;
    interface IntrinsicElements extends JsxRuntimeIntrinsicElements {}
    interface IntrinsicAttributes extends JsxRuntimeIntrinsicAttributes {}
  }
}
