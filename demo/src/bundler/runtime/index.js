const RefreshRuntime = require('react-refresh/runtime');

RefreshRuntime.injectIntoGlobalHook(window);

if (!window.__hot) {
  // To avoid 'direct-eval' warning in esbuild
  const _eval = eval;
  let performReactRefreshTimeout = null;

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

  const __r = (window.__r = {});

  window.global = window;
  window.__hot = {
    register(moduleId) {
      console.log('[HMR] Register module ::', moduleId);
      const existContext = __r[moduleId];
      if (existContext) {
        existContext.lock();
        return existContext;
      }
      return (__r[moduleId] = new HotContext(moduleId));
    },
    get(moduleId) {
      return __r[moduleId];
    },
    reactRefresh: {
      register: RefreshRuntime.register,
      getSignature:
        () =>
        (...args) => {
          RefreshRuntime.createSignatureFunctionForTransform.call(
            this,
            ...args,
          );
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
    },
  };

  const hmrSocket = new WebSocket('ws://localhost:3000/hot');
  hmrSocket.addEventListener('message', async (event) => {
    const payload = JSON.parse(event.data);
    console.log('[HMR] onMessage ::', payload);

    switch (payload.type) {
      /**
       * {
       *   type: "reload",
       * }
       */
      case 'reload':
        window.location.reload();
        break;

      /**
       * {
       *   type: "update",
       *   id: "module_id",
       *   body: "<transformed source>"
       * }
       */
      case 'update':
        const targetModule = __r[payload.id] || {};
        const acceptCallbacks = targetModule.acceptCallbacks || [];
        const disposeCallbacks = targetModule.disposeCallbacks || [];
        disposeCallbacks.forEach((callback) => callback());
        _eval(payload.body);
        acceptCallbacks.forEach((callback) => {
          callback({ body: payload.body });
        });
        break;
    }
  });
}

window.React = require('react');
