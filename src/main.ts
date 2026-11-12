import { mount } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App.svelte";
import Desk from "./Desk.svelte";
import { inTauri } from "./lib/api";
import "./app.css";

function tauriLabel(): string {
  const internals = (
    window as unknown as {
      __TAURI_INTERNALS__?: {
        metadata?: { currentWindow?: { label?: string }; currentWebview?: { label?: string } };
      };
    }
  ).__TAURI_INTERNALS__;
  return internals?.metadata?.currentWindow?.label || internals?.metadata?.currentWebview?.label || "";
}

function isDeskPage(): boolean {
  if (new URLSearchParams(location.search).has("desk")) return true;
  if (tauriLabel() === "desk") return true;
  if (!inTauri()) return false;
  try {
    return getCurrentWindow().label === "desk";
  } catch {
    return false;
  }
}

const desk = isDeskPage();
document.documentElement.dataset.window = desk ? "desk" : "launcher";
if (desk) {
  document.documentElement.style.background = "#1c1c20";
  document.body.style.background = "#1c1c20";
}

const target = document.getElementById("app");
if (target) {
  target.textContent = "";
  mount(desk ? Desk : App, { target });
}
