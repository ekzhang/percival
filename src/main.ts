import "./app.css";
import "@fontsource/source-serif-pro/400.css";
import "@fontsource/source-serif-pro/600.css";
import "@fontsource/source-serif-pro/400-italic.css";
import "@fontsource/source-serif-pro/600-italic.css";
import App from "./components/App.svelte";

const app = new App({
  target: document.getElementById("app"),
});

export default app;
