
// this file is generated — do not edit it


/// <reference types="@sveltejs/kit" />

/**
 * Environment variables [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env`. Like [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), this module cannot be imported into client-side code. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * _Unlike_ [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), the values exported from this module are statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * ```ts
 * import { API_KEY } from '$env/static/private';
 * ```
 * 
 * Note that all environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * 
 * ```
 * MY_FEATURE_FLAG=""
 * ```
 * 
 * You can override `.env` values from the command line like so:
 * 
 * ```sh
 * MY_FEATURE_FLAG="enabled" npm run dev
 * ```
 */
declare module '$env/static/private' {
	export const SHELL: string;
	export const LSCOLORS: string;
	export const npm_command: string;
	export const COREPACK_ENABLE_AUTO_PIN: string;
	export const npm_config_userconfig: string;
	export const npm_config_cache: string;
	export const LESS: string;
	export const HISTCONTROL: string;
	export const TERM_PROGRAM_VERSION: string;
	export const TMUX: string;
	export const HISTSIZE: string;
	export const HOSTNAME: string;
	export const NODE: string;
	export const COLOR: string;
	export const npm_config_local_prefix: string;
	export const npm_config_globalconfig: string;
	export const EDITOR: string;
	export const MACOSX_DEPLOYMENT_TARGET: string;
	export const PWD: string;
	export const LOGNAME: string;
	export const XDG_SESSION_TYPE: string;
	export const MODULESHOME: string;
	export const TAURI_PLATFORM: string;
	export const npm_config_init_module: string;
	export const _: string;
	export const CLAUDECODE: string;
	export const MOTD_SHOWN: string;
	export const HOME: string;
	export const LANG: string;
	export const LS_COLORS: string;
	export const npm_package_version: string;
	export const TAURI_PLATFORM_VERSION: string;
	export const TAURI_TARGET_TRIPLE: string;
	export const SSH_CONNECTION: string;
	export const TAURI_ARCH: string;
	export const INIT_CWD: string;
	export const npm_lifecycle_script: string;
	export const npm_config_npm_version: string;
	export const XDG_SESSION_CLASS: string;
	export const SELINUX_ROLE_REQUESTED: string;
	export const TERM: string;
	export const npm_package_name: string;
	export const ZSH: string;
	export const npm_config_prefix: string;
	export const LESSOPEN: string;
	export const USER: string;
	export const TMUX_PANE: string;
	export const MODULES_RUN_QUARANTINE: string;
	export const LOADEDMODULES: string;
	export const SELINUX_USE_CURRENT_RANGE: string;
	export const npm_lifecycle_event: string;
	export const SHLVL: string;
	export const GIT_EDITOR: string;
	export const PAGER: string;
	export const XDG_SESSION_ID: string;
	export const EMSDK_NODE: string;
	export const npm_config_user_agent: string;
	export const OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE: string;
	export const npm_execpath: string;
	export const XDG_RUNTIME_DIR: string;
	export const SSH_CLIENT: string;
	export const CLAUDE_CODE_ENTRYPOINT: string;
	export const __MODULES_LMINIT: string;
	export const DEBUGINFOD_URLS: string;
	export const TAURI_FAMILY: string;
	export const npm_package_json: string;
	export const DEBUGINFOD_IMA_CERT_PATH: string;
	export const which_declare: string;
	export const XDG_DATA_DIRS: string;
	export const npm_config_noproxy: string;
	export const PATH: string;
	export const SELINUX_LEVEL_REQUESTED: string;
	export const npm_config_node_gyp: string;
	export const MODULEPATH: string;
	export const TAURI_PLATFORM_TYPE: string;
	export const DBUS_SESSION_BUS_ADDRESS: string;
	export const npm_config_global_prefix: string;
	export const MAIL: string;
	export const SSH_TTY: string;
	export const EMSDK: string;
	export const npm_node_execpath: string;
	export const OLDPWD: string;
	export const MODULES_CMD: string;
	export const TERM_PROGRAM: string;
	export const NODE_ENV: string;
}

/**
 * Similar to [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private), except that it only includes environment variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Values are replaced statically at build time.
 * 
 * ```ts
 * import { PUBLIC_BASE_URL } from '$env/static/public';
 * ```
 */
declare module '$env/static/public' {
	
}

/**
 * This module provides access to runtime environment variables, as defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * This module cannot be imported into client-side code.
 * 
 * ```ts
 * import { env } from '$env/dynamic/private';
 * console.log(env.DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` always includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 */
declare module '$env/dynamic/private' {
	export const env: {
		SHELL: string;
		LSCOLORS: string;
		npm_command: string;
		COREPACK_ENABLE_AUTO_PIN: string;
		npm_config_userconfig: string;
		npm_config_cache: string;
		LESS: string;
		HISTCONTROL: string;
		TERM_PROGRAM_VERSION: string;
		TMUX: string;
		HISTSIZE: string;
		HOSTNAME: string;
		NODE: string;
		COLOR: string;
		npm_config_local_prefix: string;
		npm_config_globalconfig: string;
		EDITOR: string;
		MACOSX_DEPLOYMENT_TARGET: string;
		PWD: string;
		LOGNAME: string;
		XDG_SESSION_TYPE: string;
		MODULESHOME: string;
		TAURI_PLATFORM: string;
		npm_config_init_module: string;
		_: string;
		CLAUDECODE: string;
		MOTD_SHOWN: string;
		HOME: string;
		LANG: string;
		LS_COLORS: string;
		npm_package_version: string;
		TAURI_PLATFORM_VERSION: string;
		TAURI_TARGET_TRIPLE: string;
		SSH_CONNECTION: string;
		TAURI_ARCH: string;
		INIT_CWD: string;
		npm_lifecycle_script: string;
		npm_config_npm_version: string;
		XDG_SESSION_CLASS: string;
		SELINUX_ROLE_REQUESTED: string;
		TERM: string;
		npm_package_name: string;
		ZSH: string;
		npm_config_prefix: string;
		LESSOPEN: string;
		USER: string;
		TMUX_PANE: string;
		MODULES_RUN_QUARANTINE: string;
		LOADEDMODULES: string;
		SELINUX_USE_CURRENT_RANGE: string;
		npm_lifecycle_event: string;
		SHLVL: string;
		GIT_EDITOR: string;
		PAGER: string;
		XDG_SESSION_ID: string;
		EMSDK_NODE: string;
		npm_config_user_agent: string;
		OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE: string;
		npm_execpath: string;
		XDG_RUNTIME_DIR: string;
		SSH_CLIENT: string;
		CLAUDE_CODE_ENTRYPOINT: string;
		__MODULES_LMINIT: string;
		DEBUGINFOD_URLS: string;
		TAURI_FAMILY: string;
		npm_package_json: string;
		DEBUGINFOD_IMA_CERT_PATH: string;
		which_declare: string;
		XDG_DATA_DIRS: string;
		npm_config_noproxy: string;
		PATH: string;
		SELINUX_LEVEL_REQUESTED: string;
		npm_config_node_gyp: string;
		MODULEPATH: string;
		TAURI_PLATFORM_TYPE: string;
		DBUS_SESSION_BUS_ADDRESS: string;
		npm_config_global_prefix: string;
		MAIL: string;
		SSH_TTY: string;
		EMSDK: string;
		npm_node_execpath: string;
		OLDPWD: string;
		MODULES_CMD: string;
		TERM_PROGRAM: string;
		NODE_ENV: string;
		[key: `PUBLIC_${string}`]: undefined;
		[key: `${string}`]: string | undefined;
	}
}

/**
 * Similar to [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), but only includes variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Note that public dynamic environment variables must all be sent from the server to the client, causing larger network requests — when possible, use `$env/static/public` instead.
 * 
 * ```ts
 * import { env } from '$env/dynamic/public';
 * console.log(env.PUBLIC_DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 */
declare module '$env/dynamic/public' {
	export const env: {
		[key: `PUBLIC_${string}`]: string | undefined;
	}
}
