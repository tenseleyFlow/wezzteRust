import { c as create_ssr_component } from "../../chunks/ssr.js";
const css = {
  code: "main.svelte-1stzb1k{height:100vh;overflow:hidden}",
  map: `{"version":3,"file":"+layout.svelte","sources":["+layout.svelte"],"sourcesContent":["<script>\\n  import '../app.css';\\n<\/script>\\n\\n<main>\\n  <slot />\\n</main>\\n\\n<style>\\n  main {\\n    height: 100vh;\\n    overflow: hidden;\\n  }\\n</style>"],"names":[],"mappings":"AASE,mBAAK,CACH,MAAM,CAAE,KAAK,CACb,QAAQ,CAAE,MACZ"}`
};
const Layout = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  $$result.css.add(css);
  return `<main class="svelte-1stzb1k">${slots.default ? slots.default({}) : ``} </main>`;
});
export {
  Layout as default
};
