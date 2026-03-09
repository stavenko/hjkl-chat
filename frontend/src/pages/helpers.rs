use leptos::*;

pub fn start_resend_countdown(resend_available_at: f64, remaining: RwSignal<i32>) {
    let now = js_sys::Date::now() / 1000.0;
    let secs = ((resend_available_at - now).ceil() as i32).max(0);
    remaining.set(secs);

    if secs > 0 {
        spawn_local(async move {
            for i in (0..secs).rev() {
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    let win = web_sys::window().expect("no window");
                    win.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 1000)
                        .expect("setTimeout failed");
                });
                wasm_bindgen_futures::JsFuture::from(promise)
                    .await
                    .expect("timer failed");
                remaining.set(i);
            }
        });
    }
}
