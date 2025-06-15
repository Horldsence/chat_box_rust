export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set(["favicon.png","favicon.svg","manifest.json","svelte.svg","tauri.svg","vite.svg"]),
	mimeTypes: {".png":"image/png",".svg":"image/svg+xml",".json":"application/json"},
	_: {
		client: {start:"_app/immutable/entry/start.B8BEnq8-.js",app:"_app/immutable/entry/app.Cn2hNgLE.js",imports:["_app/immutable/entry/start.B8BEnq8-.js","_app/immutable/chunks/Bvgo1xBU.js","_app/immutable/chunks/CyAqUou3.js","_app/immutable/entry/app.Cn2hNgLE.js","_app/immutable/chunks/CyAqUou3.js","_app/immutable/chunks/BbkoyQuY.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js'))
		],
		routes: [
			
		],
		prerendered_routes: new Set(["/"]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
