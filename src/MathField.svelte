<script lang="ts">
  import { onMount, tick } from "svelte";
  import { renderMath } from "./lib/mathtext";
  import { abandonEmpty, closePair, erasePair, expandTemplate, nextSlot } from "./lib/mathpad";

  let {
    value,
    placeholder = "",
    label = "Expression",
    lineId,
    plain = false,
    onChange,
    onKeydown,
    onFocus: propsOnFocus,
  }: {
    value: string;
    placeholder?: string;
    label?: string;
    lineId: number;
    plain?: boolean;
    onChange: (value: string) => void;
    onKeydown?: (event: KeyboardEvent) => void;
    onFocus?: () => void;
  } = $props();

  let el = $state<HTMLDivElement | null>(null);
  let caret = $state(0);
  let anchor = $state(0);
  let local = $state("");
  let sent = "";
  let autoAt: number[] = [];

  const rendered = $derived(local ? renderMath(local) : "");

  $effect(() => {
    if (value === sent || value === local) return;
    sent = value;
    local = value;
    caret = value.length;
    anchor = value.length;
    autoAt = [];
  });

  function clamp(offset: number): number {
    return Math.max(0, Math.min(local.length, offset));
  }

  function place(from: number, to: number) {
    const node = el;
    if (!node) return;
    const start = pointAt(node, local, clamp(Math.min(from, to)));
    const end = pointAt(node, local, clamp(Math.max(from, to)));
    const range = document.createRange();
    if (start) range.setStart(start.node, start.at);
    else range.setStart(node, 0);
    if (end) range.setEnd(end.node, end.at);
    else range.collapse(true);
    const selection = getSelection();
    selection?.removeAllRanges();
    selection?.addRange(range);
  }

  $effect(() => {
    rendered;
    caret;
    anchor;
    if (!el || document.activeElement !== el) return;
    place(anchor, caret);
  });

  function write(next: string, at: number) {
    const spot = Math.max(0, Math.min(next.length, at));
    local = next;
    sent = next;
    caret = spot;
    anchor = spot;
    if (next !== value) onChange(next);
    else void tick().then(() => place(spot, spot));
  }

  function revise(at: number, removed: number, added: number, extra?: number) {
    autoAt = autoAt.filter((index) => index < at || index >= at + removed);
    autoAt = autoAt.map((index) => (index >= at ? index + (added - removed) : index));
    if (extra != null) autoAt = [...autoAt, extra];
  }

  function insert(text: string) {
    const start = Math.min(anchor, caret);
    const end = Math.max(anchor, caret);
    if (start === end && text.length === 1) {
      const paired = closePair(local, start, text);
      if (paired) {
        const expanded = expandTemplate(paired.text, paired.caret);
        if (expanded) autoAt = [];
        else revise(start, 0, paired.text.length - local.length, paired.autoCloser);
        write(expanded?.text ?? paired.text, expanded?.caret ?? paired.caret);
        return;
      }
    }
    let next = local.slice(0, start) + text + local.slice(end);
    let at = start + text.length;
    const expanded = expandTemplate(next, at);
    if (expanded) {
      autoAt = [];
      write(expanded.text, expanded.caret);
      return;
    }
    revise(start, end - start, text.length);
    write(next, at);
  }

  function erase(direction: -1 | 1) {
    if (anchor !== caret) {
      insert("");
      return;
    }
    if (direction < 0 && caret === 0) return;
    if (direction > 0 && caret >= local.length) return;
    if (direction < 0) {
      const paired = erasePair(local, caret);
      if (paired) {
        revise(caret - 1, 2, 0);
        write(paired.text, paired.caret);
        return;
      }
    }
    const at = direction < 0 ? caret - 1 : caret;
    revise(at, 1, 0);
    write(local.slice(0, at) + local.slice(at + 1), at);
  }

  function move(direction: -1 | 1, extend: boolean) {
    const next = clamp(caret + direction);
    if (!extend) anchor = next;
    caret = next;
  }

  function onKey(event: KeyboardEvent) {
    onKeydown?.(event);
    if (event.defaultPrevented) return;
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      move(-1, event.shiftKey);
      return;
    }
    if (event.key === "ArrowRight") {
      event.preventDefault();
      move(1, event.shiftKey);
      return;
    }
    if (event.key === "Home") {
      event.preventDefault();
      caret = 0;
      if (!event.shiftKey) anchor = 0;
      return;
    }
    if (event.key === "End") {
      event.preventDefault();
      caret = local.length;
      if (!event.shiftKey) anchor = caret;
      return;
    }
    if (event.key === "Tab") {
      const slot = nextSlot(local, caret, event.shiftKey ? -1 : 1);
      if (slot == null) return;
      event.preventDefault();
      caret = slot;
      anchor = slot;
      return;
    }
    if (event.key === "Backspace") {
      event.preventDefault();
      erase(-1);
      return;
    }
    if (event.key === "Delete") {
      event.preventDefault();
      erase(1);
      return;
    }
    if (event.key === "a" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      anchor = 0;
      caret = local.length;
      return;
    }
    if (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey) {
      event.preventDefault();
      insert(event.key);
    }
  }

  function onPaste(event: ClipboardEvent) {
    event.preventDefault();
    insert(event.clipboardData?.getData("text/plain") ?? "");
  }

  function onBeforeInput(event: InputEvent) {
    event.preventDefault();
  }

  function onBlur() {
    const next = abandonEmpty(local, autoAt, caret);
    autoAt = [];
    if (next.text !== local) write(next.text, next.caret);
  }

  onMount(() => {
    const leave = (event: PointerEvent) => {
      if (!el || el.contains(event.target as Node)) return;
      onBlur();
    };
    document.addEventListener("pointerdown", leave, true);
    return () => document.removeEventListener("pointerdown", leave, true);
  });

  function onFocus() {
    propsOnFocus?.();
    void tick().then(() => {
      if (document.activeElement === el) place(anchor, caret);
    });
  }

  function onPointerUp() {
    const node = el;
    if (!node) return;
    const next = offsetAtSelection(node, local);
    if (next == null) return;
    caret = next;
    anchor = next;
  }
</script>

<div
  bind:this={el}
  class="field"
  class:plain
  class:blank={local.length === 0}
  contenteditable="true"
  role="textbox"
  spellcheck="false"
  autocapitalize="off"
  data-line={lineId}
  data-placeholder={placeholder}
  aria-label={label}
  tabindex="0"
  onkeydown={onKey}
  onpaste={onPaste}
  onbeforeinput={onBeforeInput}
  onpointerup={onPointerUp}
  onfocus={onFocus}
  onblur={onBlur}
>{@html rendered}</div>

<script lang="ts" module>
  function pointAt(root: HTMLElement, source: string, offset: number): { node: Text; at: number } | null {
    const hosts = [...root.querySelectorAll<HTMLElement>("[data-from]")];
    let fallback: { node: Text; at: number } | null = null;
    for (const host of hosts) {
      const from = Number(host.dataset.from);
      const to = Number(host.dataset.to);
      const text = host.firstChild;
      if (!text || text.nodeType !== Node.TEXT_NODE) continue;
      const node = text as Text;
      const same = node.textContent === source.slice(from, to);
      if (same && offset >= from && offset <= to) return { node, at: offset - from };
      if (!same && (offset === from || offset === to)) {
        return { node, at: offset <= from ? 0 : node.textContent?.length ?? 0 };
      }
      if (to <= offset) fallback = { node, at: node.textContent?.length ?? 0 };
    }
    return fallback;
  }

  function offsetAtSelection(root: HTMLElement, source: string): number | null {
    const selection = getSelection();
    if (!selection || selection.rangeCount === 0 || !root.contains(selection.anchorNode)) return null;
    const parent = selection.anchorNode?.nodeType === Node.TEXT_NODE
      ? selection.anchorNode.parentElement
      : (selection.anchorNode as HTMLElement | null);
    const host = parent?.closest<HTMLElement>("[data-from]");
    if (!host) return source.length;
    const from = Number(host.dataset.from);
    const to = Number(host.dataset.to);
    const text = host.firstChild;
    if (text && text.nodeType === Node.TEXT_NODE && text.textContent === source.slice(from, to)) {
      return from + Math.min(selection.anchorOffset, to - from);
    }
    return selection.anchorOffset === 0 ? from : to;
  }
</script>

<style>
  .field {
    width: 100%;
    min-height: 36px;
    border: 0;
    border-radius: 8px;
    background: #12141a;
    color: #f3f4f6;
    padding: 8px 10px;
    outline: 1px solid transparent;
    line-height: 1.35;
    white-space: pre-wrap;
    overflow-x: auto;
    cursor: text;
  }
  .field:focus {
    outline-color: #6ea8fe;
  }
  .field.plain {
    background: transparent;
    border-radius: 0;
    min-height: 28px;
    padding: 2px 0;
  }
  .field.plain:focus {
    outline: none;
    box-shadow: inset 0 -1px 0 #6ea8fe;
  }
  .field.blank::before {
    content: attr(data-placeholder);
    color: #8b93a7;
  }
  .field :global(sup),
  .field :global(sub) {
    font-size: 0.72em;
    line-height: 1;
  }
  .field :global(.scripts) {
    display: inline-flex;
    flex-direction: column;
    vertical-align: middle;
    margin-left: 1px;
    line-height: 1;
  }
  .field :global(.scripts sup),
  .field :global(.scripts sub) {
    display: inline-block;
    min-width: 0.7em;
    min-height: 1em;
    padding: 0 2px;
    text-align: center;
  }
  .field :global(.frac) {
    display: inline-flex;
    flex-direction: column;
    vertical-align: middle;
    text-align: center;
    font-size: 0.92em;
    margin: 0 2px;
    line-height: 1.05;
  }
  .field :global(.num) {
    border-bottom: 1px solid currentColor;
    padding: 0 3px;
  }
  .field :global(.den) {
    padding: 0 3px;
  }
  .field :global(.sqrt) {
    display: inline-flex;
    align-items: stretch;
  }
  .field :global(.radical) {
    display: inline-flex;
    align-items: flex-end;
    padding-right: 1px;
  }
  .field :global(.root) {
    border-top: 1px solid currentColor;
    padding: 0 2px;
  }
  .field :global(.closer) {
    font-size: 0;
    line-height: 0;
  }
</style>
