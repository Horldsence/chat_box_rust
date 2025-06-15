import{a as f,b as g,p as v,s as b,c as y}from"./vendor.js";let i="",s=i;const B="_app",m={base:i,assets:s};function I(n){i=n.base,s=n.assets}function L(){i=m.base,s=m.assets}function A(n){s=m.assets=n}let x={},_={};function T(n){}function j(n){x=n}function P(n){_=n}let w=null;function U(n){w=n}function R(n){}function k(n){const e=f(n),t=(r,{context:o}={})=>{const a=g(n,{props:r,context:o});return{css:{code:"",map:null},head:a.head,html:a.body}};return e.render=t,e}let S=!1;function D(){}function M(){S=!0}function C(n,e){v();let{stores:t,page:r,constructors:o,components:a=[],form:l,data_0:u=null,data_1:p=null}=e;b("__svelte__",t),t.page.set(r);const h=o[1];if(o[1]){n.out+="<!--[-->";const d=o[0];n.out+="<!---->",d(n,{data:u,form:l,children:c=>{c.out+="<!---->",h(c,{data:p,form:l}),c.out+="<!---->"},$$slots:{default:!0}}),n.out+="<!---->"}else{n.out+="<!--[!-->";const d=o[0];n.out+="<!---->",d(n,{data:u,form:l}),n.out+="<!---->"}n.out+="<!--]--> ",n.out+="<!--[!-->",n.out+="<!--]-->",y()}const E=k(C),O={app_template_contains_nonce:!1,csp:{mode:"auto",directives:{"upgrade-insecure-requests":!1,"block-all-mixed-content":!1},reportOnly:{"upgrade-insecure-requests":!1,"block-all-mixed-content":!1}},csrf_check_origin:!0,embedded:!1,env_public_prefix:"PUBLIC_",env_private_prefix:"",hash_routing:!1,hooks:null,preload_strategy:"modulepreload",root:E,service_worker:!1,templates:{app:({head:n,body:e,assets:t,nonce:r,env:o})=>`<!doctype html>
<html lang="zh-CN" data-theme="light">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta
            name="description"
            content="基于 SvelteKit 和 Rust/Tauri 构建的现代化 AI 聊天应用"
        />
        <meta name="author" content="Chat Box Team" />
        <meta name="keywords" content="AI聊天,桌面应用,Svelte,Tauri,Rust" />

        <!-- PWA Meta Tags -->
        <meta name="theme-color" content="#667eea" />
        <meta name="apple-mobile-web-app-capable" content="yes" />
        <meta name="apple-mobile-web-app-status-bar-style" content="default" />
        <meta name="apple-mobile-web-app-title" content="Chat Box" />

        <!-- Security -->
        <meta
            http-equiv="Content-Security-Policy"
            content="default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: https:; font-src 'self' data:; connect-src 'self' tauri: https:; media-src 'self' blob:;"
        />

        <!-- Icons -->
        <link
            rel="icon"
            href="`+t+`/favicon.svg"
            type="image/svg+xml"
        />
        <link rel="icon" href="`+t+`/favicon.svg" />
        <link rel="manifest" href="`+t+`/manifest.json" />

        <!-- Preloads -->
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />

        <!-- Title -->
        <title>Chat Box - AI 聊天助手</title>

        <!-- SvelteKit Head -->
        `+n+`

        <!-- App Styles -->
        <style>
            /* Loading styles to prevent FOUC */
            html {
                scroll-behavior: smooth;
                -webkit-text-size-adjust: 100%;
                -webkit-font-smoothing: antialiased;
                -moz-osx-font-smoothing: grayscale;
            }

            body {
                margin: 0;
                padding: 0;
                font-family:
                    -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
                    "Helvetica Neue", Arial, sans-serif;
                line-height: 1.6;
                color: #2d3748;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                overflow: hidden;
                user-select: none;
                -webkit-user-select: none;
                -moz-user-select: none;
                -ms-user-select: none;
            }

            /* Prevent text selection in desktop app */
            * {
                -webkit-user-select: none;
                -moz-user-select: none;
                -ms-user-select: none;
                user-select: none;
            }

            /* Allow text selection in input/textarea */
            input,
            textarea,
            [contenteditable] {
                -webkit-user-select: text;
                -moz-user-select: text;
                -ms-user-select: text;
                user-select: text;
            }

            /* Loading screen */
            .loading-screen {
                position: fixed;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 9999;
                transition: opacity 0.3s ease-in-out;
            }

            .loading-content {
                text-align: center;
                color: white;
            }

            .loading-spinner {
                width: 50px;
                height: 50px;
                border: 4px solid rgba(255, 255, 255, 0.3);
                border-top: 4px solid white;
                border-radius: 50%;
                animation: spin 1s linear infinite;
                margin: 0 auto 20px;
            }

            @keyframes spin {
                0% {
                    transform: rotate(0deg);
                }
                100% {
                    transform: rotate(360deg);
                }
            }

            .loading-text {
                font-size: 18px;
                font-weight: 500;
                margin-top: 16px;
            }

            /* Hide loading screen when app is ready */
            .app-ready .loading-screen {
                opacity: 0;
                pointer-events: none;
            }

            /* Error styles */
            .error-boundary {
                position: fixed;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                background: #1a202c;
                color: white;
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 10000;
            }

            .error-content {
                text-align: center;
                max-width: 500px;
                padding: 40px;
            }

            .error-icon {
                font-size: 64px;
                margin-bottom: 20px;
            }

            .error-title {
                font-size: 24px;
                font-weight: 600;
                margin-bottom: 16px;
            }

            .error-message {
                font-size: 16px;
                opacity: 0.8;
                line-height: 1.5;
                margin-bottom: 24px;
            }

            .error-button {
                background: #667eea;
                color: white;
                border: none;
                padding: 12px 24px;
                border-radius: 8px;
                font-size: 16px;
                font-weight: 500;
                cursor: pointer;
                transition: background-color 0.2s;
            }

            .error-button:hover {
                background: #5a67d8;
            }
        </style>
    </head>

    <body data-sveltekit-preload-data="hover" data-sveltekit-reload>
        <!-- Loading Screen -->
        <div class="loading-screen" id="loading-screen">
            <div class="loading-content">
                <div class="loading-spinner"></div>
                <div class="loading-text">正在启动 Chat Box...</div>
            </div>
        </div>

        <!-- Main App Container -->
        <div style="display: contents" class="app-container">
            `+e+`
        </div>

        <!-- Error Boundary -->
        <div class="error-boundary" id="error-boundary" style="display: none">
            <div class="error-content">
                <div class="error-icon">⚠️</div>
                <div class="error-title">应用启动失败</div>
                <div class="error-message">
                    Chat Box 遇到了一个意外错误。请尝试重新启动应用。
                </div>
                <button class="error-button" onclick="window.location.reload()">
                    重新加载
                </button>
            </div>
        </div>

        <!-- App Ready Script -->
        <script>
            // Remove loading screen when app is ready
            document.addEventListener("DOMContentLoaded", function () {
                setTimeout(() => {
                    document.body.classList.add("app-ready");
                    setTimeout(() => {
                        const loadingScreen =
                            document.getElementById("loading-screen");
                        if (loadingScreen) {
                            loadingScreen.remove();
                        }
                    }, 300);
                }, 1000);
            });

            // Global error handler
            window.addEventListener("error", function (event) {
                console.error("Global error:", event.error);
                const errorBoundary = document.getElementById("error-boundary");
                const loadingScreen = document.getElementById("loading-screen");

                if (errorBoundary) {
                    errorBoundary.style.display = "flex";
                }
                if (loadingScreen) {
                    loadingScreen.style.display = "none";
                }
            });

            // Unhandled promise rejection handler
            window.addEventListener("unhandledrejection", function (event) {
                console.error("Unhandled promise rejection:", event.reason);
                // Optionally show error boundary for critical errors
            });

            // Prevent default drag and drop behavior
            document.addEventListener("dragover", function (e) {
                e.preventDefault();
            });

            document.addEventListener("drop", function (e) {
                e.preventDefault();
            });

            // Prevent context menu (right-click) in production
            if (window.__TAURI__ && !window.__TAURI_DEBUG__) {
                document.addEventListener("contextmenu", function (e) {
                    e.preventDefault();
                });
            }

            // Handle theme changes
            function updateTheme() {
                const theme = localStorage.getItem("theme") || "auto";
                if (theme === "auto") {
                    const prefersDark = window.matchMedia(
                        "(prefers-color-scheme: dark)",
                    ).matches;
                    document.documentElement.setAttribute(
                        "data-theme",
                        prefersDark ? "dark" : "light",
                    );
                } else {
                    document.documentElement.setAttribute("data-theme", theme);
                }
            }

            // Initialize theme
            updateTheme();

            // Listen for system theme changes
            window
                .matchMedia("(prefers-color-scheme: dark)")
                .addEventListener("change", updateTheme);
        <\/script>
    </body>
</html>
`,error:({status:n,message:e})=>`<!doctype html>
<html lang="en">
	<head>
		<meta charset="utf-8" />
		<title>`+e+`</title>

		<style>
			body {
				--bg: white;
				--fg: #222;
				--divider: #ccc;
				background: var(--bg);
				color: var(--fg);
				font-family:
					system-ui,
					-apple-system,
					BlinkMacSystemFont,
					'Segoe UI',
					Roboto,
					Oxygen,
					Ubuntu,
					Cantarell,
					'Open Sans',
					'Helvetica Neue',
					sans-serif;
				display: flex;
				align-items: center;
				justify-content: center;
				height: 100vh;
				margin: 0;
			}

			.error {
				display: flex;
				align-items: center;
				max-width: 32rem;
				margin: 0 1rem;
			}

			.status {
				font-weight: 200;
				font-size: 3rem;
				line-height: 1;
				position: relative;
				top: -0.05rem;
			}

			.message {
				border-left: 1px solid var(--divider);
				padding: 0 0 0 1rem;
				margin: 0 0 0 1rem;
				min-height: 2.5rem;
				display: flex;
				align-items: center;
			}

			.message h1 {
				font-weight: 400;
				font-size: 1em;
				margin: 0;
			}

			@media (prefers-color-scheme: dark) {
				body {
					--bg: #222;
					--fg: #ddd;
					--divider: #666;
				}
			}
		</style>
	</head>
	<body>
		<div class="error">
			<span class="status">`+n+`</span>
			<div class="message">
				<h1>`+e+`</h1>
			</div>
		</div>
	</body>
</html>
`},version_hash:"1a4t8oj"};async function F(){return{handle:void 0,handleFetch:void 0,handleError:void 0,init:void 0,reroute:void 0,transport:void 0}}export{s as a,i as b,B as c,w as d,O as e,T as f,F as g,S as h,j as i,P as j,U as k,A as l,D as m,R as n,I as o,x as p,M as q,L as r,_ as s};
