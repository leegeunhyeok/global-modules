class HotContext {
  id = 0;
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

window.__modules.hot = (id) => new HotContext(id);
