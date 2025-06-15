

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "prerender": true,
  "ssr": false
};
export const universal_id = "src/routes/+layout.ts";
export const imports = ["_app/immutable/nodes/0.CHrcuUB8.js","_app/immutable/chunks/BbkoyQuY.js","_app/immutable/chunks/Bi4GrdmC.js","_app/immutable/chunks/CyAqUou3.js","_app/immutable/chunks/Bvgo1xBU.js","_app/immutable/chunks/CgopN5UC.js"];
export const stylesheets = ["_app/immutable/assets/0.CJlebRd3.css"];
export const fonts = [];
