import "@fontsource-variable/inter";
import { mount } from "svelte";

import App from "./App.svelte";
import "./design-tokens.css";
import "./controls.css";
import { hasMica } from "./shell/shell-api";

const MICA_CLASS = "has-mica";

const applicationRoot = document.getElementById("app");
if (applicationRoot === null) {
  throw new Error("The application root element is missing from index.html.");
}

void hasMica().then((isMicaAvailable) => {
  document.documentElement.classList.toggle(MICA_CLASS, isMicaAvailable);
});

mount(App, { target: applicationRoot });
