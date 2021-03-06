import "@fontsource/source-serif-pro/400.css";
import "@fontsource/source-serif-pro/600.css";
import "@fontsource/source-serif-pro/400-italic.css";
import "@fontsource/source-serif-pro/600-italic.css";
import "katex/dist/katex.css";
import "./app.css";
import App from "./components/App.svelte";
import init from "percival-wasm";

await init();

const app = new App({
  target: document.getElementById("app")!,
});

export default app;
