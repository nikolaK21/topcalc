export type MathKey = {
  label: string;
  insert: string;
  title: string;
};

export type MathGroup = {
  name: string;
  keys: MathKey[];
};

function key(label: string, insert: string, title = label): MathKey {
  return { label, insert, title };
}

function call(label: string, name = label): MathKey {
  return key(label, `${name}(|)`, name);
}

export const mathGroups: MathGroup[] = [
  {
    name: "Trig",
    keys: [
      call("sin"),
      call("cos"),
      call("tan"),
      call("csc"),
      call("sec"),
      call("cot"),
    ],
  },
  {
    name: "Inverse",
    keys: [
      call("arcsin"),
      call("arccos"),
      call("arctan"),
      call("arccsc"),
      call("arcsec"),
      call("arccot"),
    ],
  },
  {
    name: "Hyperbolic",
    keys: [
      call("sinh"),
      call("cosh"),
      call("tanh"),
      call("csch"),
      call("sech"),
      call("coth"),
      call("arcsinh"),
      call("arccosh"),
      call("arctanh"),
    ],
  },
  {
    name: "Calculus",
    keys: [
      key("∑", "sum_{|}^{} (|)", "Sum"),
      key("∏", "prod_{|}^{} (|)", "Product"),
      key("lim", "lim_{|} (|)", "Limit"),
      key("d/dx", "d/dx |", "Derivative"),
      key("d²/dx²", "d^2/dx^2 |", "Second derivative"),
      key("∫", "int_{|}^{} (|) dx", "Definite integral"),
    ],
  },
  {
    name: "Functions",
    keys: [
      call("√", "sqrt"),
      call("abs"),
      call("ln"),
      call("log"),
      call("log10"),
      call("log2"),
      call("exp"),
      call("floor"),
      call("ceil"),
      call("round"),
      call("sign"),
      key("n!", "(|)!", "Factorial"),
      call("gamma"),
    ],
  },
  {
    name: "Symbols",
    keys: [
      key("π", "pi", "pi"),
      key("e", "e", "e"),
      key("∞", "inf", "Infinity"),
      key("xⁿ", "^{|}", "Power"),
      key("a/b", "(|)/(|)", "Fraction"),
    ],
  },
];

const templates: Record<string, string> = {
  sum: "sum_{|}^{} (|)",
  prod: "prod_{|}^{} (|)",
  product: "prod_{|}^{} (|)",
  lim: "lim_{|} (|)",
  limit: "lim_{|} (|)",
  int: "int_{|}^{} (|) dx",
  integral: "int_{|}^{} (|) dx",
};

const PAIRS: Record<string, string> = { "(": ")", "{": "}" };

export function closePair(
  text: string,
  caret: number,
  key: string,
): { text: string; caret: number; autoCloser?: number } | null {
  const trimmed = text.trimStart();
  if (trimmed.startsWith(">") || trimmed.startsWith("$")) return null;
  if (key === ")" || key === "}") {
    if (text[caret] === key) return { text, caret: caret + 1 };
    return null;
  }
  const closer = PAIRS[key];
  if (!closer) return null;
  return {
    text: text.slice(0, caret) + key + closer + text.slice(caret),
    caret: caret + 1,
    autoCloser: caret + 1,
  };
}

export function abandonEmpty(text: string, closers: number[], caret: number): { text: string; caret: number } {
  const marked = new Set(closers);
  for (const index of [caret, caret - 1]) {
    if (!keepsSlot(text, index)) marked.add(index);
  }
  const pending = [...marked]
    .filter((index) => {
      const opener = text[index - 1];
      const closer = text[index];
      return (opener === "(" && closer === ")") || (opener === "{" && closer === "}");
    })
    .sort((left, right) => right - left);
  let next = text;
  let at = caret;
  for (const index of pending) {
    next = next.slice(0, index - 1) + next.slice(index + 1);
    if (at >= index) at -= 2;
  }
  return { text: next, caret: Math.max(0, at) };
}

export function erasePair(text: string, caret: number): { text: string; caret: number } | null {
  const prev = text[caret - 1];
  const next = text[caret];
  if ((prev === "(" && next === ")") || (prev === "{" && next === "}")) {
    return { text: text.slice(0, caret - 1) + text.slice(caret + 1), caret: caret - 1 };
  }
  return null;
}

function keepsSlot(text: string, closerIndex: number): boolean {
  const opener = text[closerIndex - 1];
  const closer = text[closerIndex];
  if (!((opener === "(" && closer === ")") || (opener === "{" && closer === "}"))) return true;
  const before = text[closerIndex - 2];
  if (opener === "{" && (before === "_" || before === "^")) return true;
  if (opener === "(") {
    let index = closerIndex - 2;
    while (index >= 0 && text[index] === " ") index -= 1;
    if (text[index] === "}") return true;
  }
  return false;
}

export function nextSlot(text: string, caret: number, direction: 1 | -1): number | null {
  const slots: number[] = [];
  for (let index = 0; index < text.length; index += 1) {
    const ch = text[index];
    if (ch === "{" || ch === "(") slots.push(index + 1);
  }
  if (direction > 0) return slots.find((slot) => slot > caret) ?? null;
  for (let index = slots.length - 1; index >= 0; index -= 1) {
    const slot = slots[index];
    if (slot != null && slot < caret) return slot;
  }
  return null;
}

function materialize(template: string): { insert: string; cursor: number } {
  const cursor = template.indexOf("|");
  const insert = template.replaceAll("|", "");
  return { insert, cursor: cursor < 0 ? insert.length : cursor };
}

export function expandTemplate(value: string, caret: number): { text: string; caret: number } | null {
  if (caret < 2 || value[caret - 1] !== " ") return null;
  const trimmed = value.trimStart();
  if (
    trimmed.startsWith(">") ||
    trimmed.startsWith("$") ||
    /^[a-zA-Z]:/.test(trimmed) ||
    trimmed.startsWith("/") ||
    trimmed.startsWith("\\") ||
    trimmed.startsWith("~")
  ) {
    return null;
  }
  let end = caret - 1;
  while (end > 0 && value[end - 1] === " ") end -= 1;
  let start = end;
  while (start > 0 && /[A-Za-z]/.test(value[start - 1] ?? "")) start -= 1;
  const word = value.slice(start, end).toLowerCase();
  const template = templates[word];
  if (!template) return null;
  if (start > 0 && /[A-Za-z0-9_]/.test(value[start - 1] ?? "")) return null;
  const { insert, cursor } = materialize(template);
  return {
    text: value.slice(0, start) + insert + value.slice(caret),
    caret: start + cursor,
  };
}

export function spliceTemplate(
  value: string,
  start: number,
  end: number,
  template: string,
): { text: string; caret: number } {
  const { insert, cursor } = materialize(template);
  return {
    text: value.slice(0, start) + insert + value.slice(end),
    caret: start + cursor,
  };
}
