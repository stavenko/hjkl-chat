use leptos::*;

#[component]
pub fn CountdownButton(
    #[prop(into)] label: String,
    countdown_secs: u32,
    on_click: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let (remaining, set_remaining) = create_signal(0u32);
    let is_counting = move || remaining.get() > 0;
    let label_clone = label.clone();

    let on_btn_click = move |_| {
        if is_counting() {
            return;
        }
        let cb = on_click.clone();
        cb();
        set_remaining.set(countdown_secs);

        let secs = countdown_secs;
        spawn_local(async move {
            for i in (0..secs).rev() {
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    let win = web_sys::window().expect("no window");
                    win.set_timeout_with_callback_and_timeout_and_arguments_0(
                        &resolve, 1000,
                    )
                    .expect("setTimeout failed");
                });
                wasm_bindgen_futures::JsFuture::from(promise).await.expect("timer failed");
                set_remaining.set(i);
            }
        });
    };

    let btn_class = move || {
        if is_counting() {
            "btn btn--disabled"
        } else {
            "btn"
        }
    };

    view! {
        <button
            class=btn_class
            disabled=is_counting
            on:click=on_btn_click
        >
            {move || {
                let r = remaining.get();
                if r > 0 {
                    format!("{} ({}s)", label_clone, r)
                } else {
                    label_clone.clone()
                }
            }}
        </button>
    }
}
