import watcher, { type AsyncSubscription, type Event } from '@parcel/watcher';

let subscription: AsyncSubscription | null = null;

async function watch(basePath: string, handler: (events: Event[]) => void) {
  if (subscription) {
    return;
  }

  subscription = await watcher.subscribe(basePath, (error, events) => {
    if (error) {
      console.error(error);
    } else {
      handler(events);
    }
  });
}

async function cleanup() {
  await subscription?.unsubscribe();
}

export { watch, cleanup };
