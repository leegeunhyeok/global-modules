const RefreshRuntime = require('react-refresh/runtime');

RefreshRuntime.injectIntoGlobalHook(window);

// To avoid 'direct-eval' warning in esbuild
const _eval = eval;
let performReactRefreshTimeout = null;

// TODO
class HotContext {
  id = '';
  locked = false;
  acceptCallbacks = [];
  disposeCallbacks = [];

  constructor(id) {
    this.id = id;
  }

  accept(callback) {
    if (this.locked) return;
    this.acceptCallbacks.push(callback);
  }

  dispose(callback) {
    if (this.locked) return;
    this.disposeCallbacks.push(callback);
  }

  lock() {
    this.locked = true;
  }
}

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
          // this.handleReload();
        }
        break;
    }
  }

  handleReload() {
    window.location.reload();
  }

  handleModuleUpdate(id, body) {
    const targetModule = global.__modules.getContext(id);
    // TODO
    // const acceptCallbacks = targetModule.acceptCallbacks || [];
    // const disposeCallbacks = targetModule.disposeCallbacks || [];
    const acceptCallbacks = [targetModule.accept];
    const disposeCallbacks = [targetModule.dispose];

    disposeCallbacks.forEach((callback) => callback());
    _eval(body);
    acceptCallbacks.forEach((callback) => callback({ body }));
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

window.global = window;
window.$$reactRefresh$$ = reactRefresh;
// Import the jsx runtime code after the `window.$$reactRefresh$$` is initialized.
window.$$jsxDevRuntime$$ = require('react/jsx-dev-runtime');

new HMRClient().connect();
