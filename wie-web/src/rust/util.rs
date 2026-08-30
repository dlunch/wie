use core::future::Future;

use futures::channel::oneshot;
use wasm_bindgen_futures::spawn_local;

struct SendWrapper<T>(T);

// Safe on single-threaded wasm; lets us carry !Send JS values through a
// Send oneshot receiver.
unsafe impl<T> Send for SendWrapper<T> {}

// wrapper to run non-send js future from a send future
pub fn run_js_future<F, R>(f: F) -> impl Future<Output = R> + Send
where
    F: Future<Output = R> + 'static,
    R: 'static,
{
    let (tx, rx) = oneshot::channel::<SendWrapper<R>>();
    spawn_local(async move {
        let _ = tx.send(SendWrapper(f.await));
    });
    async move { rx.await.unwrap().0 }
}
