type Piece =
  | { kind: "text"; from: number; to: number; text: string }
  | { kind: "sup" | "sub"; from: number; to: number; body: Piece[] }
  | { kind: "scripts"; from: number; to: number; base: Piece; sup?: Piece; sub?: Piece }
  | { kind: "row"; from: number; to: number; body: Piece[] }
  | { kind: "frac"; from: number; to: number; num: Piece; den: Piece }
  | { kind: "sqrt"; from: number; to: number; openTo: number; body: Piece[]; close?: Piece };

const SYMBOLS: Record<string, string> = {
  pi: "π",
  theta: "θ",
  alpha: "α",
  beta: "β",
  delta: "δ",
  phi: "φ",
  lambda: "λ",
  inf: "∞",
  infinity: "∞",
  oo: "∞",
  sum: "∑",
  prod: "∏",
  product: "∏",
};

function escapeHtml(text: string): string {
  return text.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

function textPiece(from: number, to: number, text: string): Piece {
  return { kind: "text", from, to, text };
}

function parseSeq(source: string, index: number, stop: string): { pieces: Piece[]; index: number } {
  const pieces: Piece[] = [];
  while (index < source.length && !stop.includes(source[index])) {
    const next = parsePiece(source, index);
    if (!next || next.to <= index) {
      pieces.push(textPiece(index, index + 1, source[index]));
      index += 1;
      continue;
    }
    pieces.push(next.piece);
    index = next.to;
  }
  return { pieces, index };
}

function parsePiece(source: string, index: number): { piece: Piece; to: number } | null {
  const core = parseCore(source, index);
  if (!core) return null;
  let piece = core.piece;
  let cursor = core.to;
  if (cursor < source.length && source[cursor] === "/" && source[cursor + 1] !== "/") {
    const den = parseCore(source, cursor + 1);
    const denominator = den?.piece ?? textPiece(cursor + 1, cursor + 1, "");
    const to = den?.to ?? cursor + 1;
    piece = { kind: "frac", from: piece.from, to, num: piece, den: denominator };
    cursor = to;
  }
  return { piece, to: cursor };
}

function parseCore(source: string, index: number): { piece: Piece; to: number } | null {
  const atom = parseAtom(source, index);
  if (!atom) return null;
  let base = atom.piece;
  let cursor = atom.to;
  let sup: Piece | undefined;
  let sub: Piece | undefined;
  while (cursor < source.length && (source[cursor] === "^" || source[cursor] === "_")) {
    const script = parseScript(source, cursor);
    if (source[cursor] === "^") sup = script.piece;
    else sub = script.piece;
    cursor = script.to;
  }
  if (!sup && !sub) return { piece: base, to: cursor };
  return {
    piece: { kind: "scripts", from: base.from, to: cursor, base, sup, sub },
    to: cursor,
  };
}

function parseScript(source: string, index: number): { piece: Piece; to: number } {
  const kind = source[index] === "^" ? "sup" : "sub";
  if (source[index + 1] === "{") {
    const inner = parseSeq(source, index + 2, "}");
    const to = inner.index < source.length && source[inner.index] === "}" ? inner.index + 1 : inner.index;
    return { piece: { kind, from: index, to, body: inner.pieces }, to };
  }
  const atom = parseAtom(source, index + 1);
  if (!atom) return { piece: { kind, from: index, to: index + 1, body: [] }, to: index + 1 };
  return { piece: { kind, from: index, to: atom.to, body: [atom.piece] }, to: atom.to };
}

function parseAtom(source: string, index: number): { piece: Piece; to: number } | null {
  if (index >= source.length) return null;
  const ch = source[index];
  if (ch === "(" || ch === "{") {
    const close = ch === "(" ? ")" : "}";
    const inner = parseSeq(source, index + 1, close);
    const closed = inner.index < source.length && source[inner.index] === close;
    const to = closed ? inner.index + 1 : inner.index;
    const pieces = [textPiece(index, index + 1, ch), ...inner.pieces];
    if (closed) pieces.push(textPiece(inner.index, inner.index + 1, close));
    return { piece: { kind: "row", from: index, to, body: pieces }, to };
  }
  if (ch >= "0" && ch <= "9" || ch === ".") {
    let end = index + 1;
    while (end < source.length && ((source[end] >= "0" && source[end] <= "9") || source[end] === ".")) end += 1;
    return { piece: textPiece(index, end, source.slice(index, end)), to: end };
  }
  if (/[A-Za-z]/.test(ch)) {
    let end = index + 1;
    while (end < source.length && /[A-Za-z0-9]/.test(source[end])) end += 1;
    const word = source.slice(index, end);
    const lower = word.toLowerCase();
    if (lower === "sqrt" && source[end] === "(") {
      const inner = parseSeq(source, end + 1, ")");
      const closed = inner.index < source.length && source[inner.index] === ")";
      const to = closed ? inner.index + 1 : inner.index;
      const close = closed ? textPiece(inner.index, inner.index + 1, ")") : undefined;
      return { piece: { kind: "sqrt", from: index, to, openTo: end + 1, body: inner.pieces, close }, to };
    }
    const symbol = source[end] === "(" ? undefined : SYMBOLS[lower];
    if (lower === "int" && source[end] !== "(") return { piece: textPiece(index, end, "∫"), to: end };
    return { piece: textPiece(index, end, symbol ?? word), to: end };
  }
  const two = source.slice(index, index + 2);
  if (two === "<=" || two === ">=" || two === "->") {
    const shown = two === "<=" ? "≤" : two === ">=" ? "≥" : "→";
    return { piece: textPiece(index, index + 2, shown), to: index + 2 };
  }
  if ("=<>+-".includes(ch)) return { piece: textPiece(index, index + 1, ch), to: index + 1 };
  if (ch === "*") return { piece: textPiece(index, index + 1, "·"), to: index + 1 };
  return null;
}

function html(piece: Piece): string {
  if (piece.kind === "text") {
    const shown = piece.text.length > 0 ? escapeHtml(piece.text) : "&#8203;";
    return `<span data-from="${piece.from}" data-to="${piece.to}">${shown}</span>`;
  }
  if (piece.kind === "sup" || piece.kind === "sub") {
    const tag = piece.kind === "sup" ? "sup" : "sub";
    const slot = piece.to - piece.from > 1 ? piece.to - 1 : piece.to;
    const inner = piece.body.length > 0 ? piece.body.map(html).join("") : `<span data-from="${slot}" data-to="${slot}">&#8203;</span>`;
    return `<${tag}>${inner}</${tag}>`;
  }
  if (piece.kind === "row") return piece.body.map(html).join("");
  if (piece.kind === "scripts") {
    const scripts = `${piece.sup ? html(piece.sup) : ""}${piece.sub ? html(piece.sub) : ""}`;
    if (piece.sup && piece.sub) return `${html(piece.base)}<span class="scripts">${scripts}</span>`;
    return `${html(piece.base)}${scripts}`;
  }
  if (piece.kind === "frac") {
    const den = piece.den.kind === "text" && piece.den.text === ""
      ? `<span data-from="${piece.to}" data-to="${piece.to}">&#8203;</span>`
      : html(piece.den);
    return `<span class="frac"><span class="num">${html(piece.num)}</span><span class="den">${den}</span></span>`;
  }
  if (piece.kind === "sqrt") {
    const body = piece.body.map(html).join("");
    const close = piece.close ? `<span class="closer">${html(piece.close)}</span>` : "";
    return `<span class="sqrt"><span class="radical" data-from="${piece.from}" data-to="${piece.openTo}">√</span><span class="root">${body}</span></span>${close}`;
  }
  return "";
}

export function renderMath(source: string): string {
  return parseSeq(source, 0, "").pieces.map(html).join("");
}
