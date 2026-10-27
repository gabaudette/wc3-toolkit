import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { report } from "./api";
import { errorMessage } from "./result";
import "@fontsource/cormorant-sc/600.css";
import "@fontsource/cormorant-sc/700.css";
import "@fontsource/nunito-sans/400.css";
import "@fontsource/nunito-sans/600.css";
import "@fontsource/nunito-sans/700.css";
import "./styles.css";

window.addEventListener("error", (event) => report("error", "window", event.message));
window.addEventListener("unhandledrejection", (event) =>
  report("error", "window", errorMessage(event.reason)),
);

const root = document.getElementById("root");

if (root) {
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
