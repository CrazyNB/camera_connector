import { cssToken, readable } from "./presentation";

export function statusDot(value: string) {
  return el("span", `status-dot ${cssToken(value)}`);
}

export function intelligenceLine(label: string, value: string) {
  return append(el("div", "intelligence-line"), el("span", "", label), el("strong", "", value));
}

export function renderIntelligenceField(label: string, control: HTMLElement, note = "") {
  const field = append(el("label", "intelligence-field"), el("span", "", label), control);
  if (note) {
    append(field, el("small", "", note));
  }
  return field;
}

export function settingsSectionHead(title: string, note = "") {
  const head = append(el("div", "settings-section-head"), el("h3", "", title));
  if (note) {
    append(head, el("p", "", note));
  }
  return head;
}

export function renderToggleRow(label: string, checked: boolean, onChange: (checked: boolean) => void) {
  const input = el("input", "") as HTMLInputElement;
  input.type = "checkbox";
  input.checked = checked;
  input.addEventListener("change", () => onChange(input.checked));
  return append(el("label", "toggle-row"), append(el("span", ""), el("strong", "", label)), input);
}

export function selectControl(value: string, options: Array<[string, string]>, onChange: (value: string) => void) {
  const select = el("select", "select-control") as HTMLSelectElement;
  select.value = value;
  for (const [optionValue, label] of options) {
    const option = el("option", "", label) as HTMLOptionElement;
    option.value = optionValue;
    option.selected = optionValue === value;
    append(select, option);
  }
  select.addEventListener("change", () => onChange(select.value));
  return select;
}

export function compactMetric(label: string, value: string) {
  return append(el("div", "compact-metric"), el("span", "", label), el("strong", "", value));
}

export function statusChip(value: string, kind: string) {
  return el("span", `status-chip ${kind} ${cssToken(value)}`, readable(value));
}

export function commandButton(label: string, className: string, onClick: (event: MouseEvent) => void, disabled = false) {
  const node = el("button", className, label);
  node.type = "button";
  node.disabled = disabled;
  node.addEventListener("click", (event) => onClick(event));
  return node;
}

export function textInput(value: string, placeholder: string, onInput: (value: string) => void) {
  const node = el("input", "text-input") as HTMLInputElement;
  node.value = value;
  node.placeholder = placeholder;
  node.addEventListener("input", () => onInput(node.value));
  return node;
}

export function passwordInput(value: string, placeholder: string, onInput: (value: string) => void) {
  const node = el("input", "text-input") as HTMLInputElement;
  node.type = "password";
  node.value = value;
  node.placeholder = placeholder;
  node.addEventListener("input", () => onInput(node.value));
  return node;
}

export function numberInput(value: number, min: number, max: number, onInput: (value: number) => void) {
  const node = el("input", "text-input") as HTMLInputElement;
  node.type = "number";
  node.min = String(min);
  node.max = String(max);
  node.value = String(value);
  node.addEventListener("input", () => {
    const parsed = Number(node.value);
    if (!Number.isFinite(parsed)) return;
    onInput(Math.min(max, Math.max(min, parsed)));
  });
  return node;
}

export function textAreaInput(value: string, placeholder: string, onInput: (value: string) => void) {
  const node = el("textarea", "textarea-input") as HTMLTextAreaElement;
  node.value = value;
  node.placeholder = placeholder;
  node.addEventListener("input", () => onInput(node.value));
  return node;
}

export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

export function append<T extends HTMLElement>(parent: T, ...children: Array<Node | null | undefined>) {
  for (const child of children) {
    if (child) parent.appendChild(child);
  }
  return parent;
}
