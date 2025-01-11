const RefreshRuntime = require('react-refresh/runtime');

RefreshRuntime.injectIntoGlobalHook(window);

// To avoid 'direct-eval' warning in esbuild
const _eval = eval;
let performReactRefreshTimeout = null;

class HMRClient {
  connect() {
    const socket = new WebSocket('ws://localhost:3000/hot');

    socket.addEventListener('open', () => {
      console.log('[HMR] Socket connected');
    });

    socket.addEventListener('close', () => {
      console.log('[HMR] Socket disconnected');
    });

    socket.addEventListener('message', async (event) => {
      this.handleMessage(event);
    });
  }

  handleMessage(event) {
    const payload = JSON.parse(event.data);
    console.log('[HMR] onMessage ::', payload);

    switch (payload.type) {
      /**
       * {
       *   type: "reload",
       * }
       */
      case 'reload':
        this.handleReload();
        break;

      /**
       * {
       *   type: "update",
       *   id: "module_id",
       *   body: "<transformed source>"
       * }
       */
      case 'update':
        try {
          this.handleModuleUpdate(payload.id, payload.body);
        } catch (error) {
          console.error(
            '[HMR] Unexpected error on module update. fully reload instead ::',
            error,
          );
          this.handleReload();
        }
        break;
    }
  }

  handleReload() {
    window.location.reload();
  }

  handleModuleUpdate(id, body) {
    const targetModule = global.__modules.getContext(id);
    targetModule.hot.disposeCallbacks.forEach((callback) => callback());
    _eval(body);
    targetModule.hot.acceptCallbacks.forEach((callback) => callback({ body }));
  }
}

const reactRefresh = {
  register: RefreshRuntime.register,
  getSignature:
    () =>
    (...args) => {
      RefreshRuntime.createSignatureFunctionForTransform.call(this, ...args);
      return args[0];
    },
  performReactRefresh: () => {
    if (performReactRefreshTimeout !== null) {
      return;
    }

    performReactRefreshTimeout = setTimeout(() => {
      performReactRefreshTimeout = null;
    }, 30);

    if (RefreshRuntime.hasUnrecoverableErrors()) {
      console.error('[HMR] has unrecoverable errors on react-refresh');
      return;
    }

    RefreshRuntime.performReactRefresh();
  },
};

window.$$reactRefresh$$ = reactRefresh;
// Import the jsx runtime code after the `window.$$reactRefresh$$` is initialized.
window.$$jsxDevRuntime$$ = require('react/jsx-dev-runtime');

new HMRClient().connect();
