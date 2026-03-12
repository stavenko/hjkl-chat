use leptos::*;
use wasm_bindgen::JsCast;

/// A single line in the diff view, with its origin (server/local) and content.
#[derive(Clone, Debug, PartialEq)]
struct DiffLine {
    text: String,
    /// "same", "server", "local"
    kind: &'static str,
}

/// Compute a simple line-by-line diff between server and local content.
fn compute_diff(server: &str, local: &str) -> (Vec<DiffLine>, Vec<DiffLine>) {
    let server_lines: Vec<&str> = server.lines().collect();
    let local_lines: Vec<&str> = local.lines().collect();

    let mut server_diff = Vec::new();
    let mut local_diff = Vec::new();

    let max_len = server_lines.len().max(local_lines.len());
    for i in 0..max_len {
        let s = server_lines.get(i).copied().unwrap_or("");
        let l = local_lines.get(i).copied().unwrap_or("");

        if i < server_lines.len() && i < local_lines.len() && s == l {
            server_diff.push(DiffLine { text: s.to_string(), kind: "same" });
            local_diff.push(DiffLine { text: l.to_string(), kind: "same" });
        } else {
            if i < server_lines.len() {
                server_diff.push(DiffLine { text: s.to_string(), kind: "server" });
            }
            if i < local_lines.len() {
                local_diff.push(DiffLine { text: l.to_string(), kind: "local" });
            }
        }
    }

    (server_diff, local_diff)
}

#[component]
pub fn ConflictModal(
    open: RwSignal<bool>,
    server_content: RwSignal<String>,
    local_content: RwSignal<String>,
    on_resolve: Box<dyn Fn(String) + 'static>,
) -> impl IntoView {
    let result_text = create_rw_signal(String::new());

    let diff_data = create_memo(move |_| {
        let server = server_content.get();
        let local = local_content.get();
        compute_diff(&server, &local)
    });

    // Initialize result with local content when modal opens
    create_effect(move |prev: Option<bool>| {
        let is_open = open.get();
        if is_open && prev != Some(true) {
            result_text.set(local_content.get_untracked());
        }
        is_open
    });

    let on_resolve_click = std::rc::Rc::new(on_resolve);

    view! {
        {move || {
            if !open.get() {
                return view! { <div style="display:none"/> }.into_view();
            }

            let on_resolve_inner = on_resolve_click.clone();

            view! {
                <div
                    class="conflict-modal__backdrop"
                    on:click=move |ev| {
                        if let Some(target) = ev.target() {
                            if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                                if el.class_list().contains("conflict-modal__backdrop") {
                                    open.set(false);
                                }
                            }
                        }
                    }
                >
                    <div class="conflict-modal">
                        <div class="conflict-modal__header">
                            <h2 class="conflict-modal__title">"Resolve Conflict"</h2>
                            <button
                                class="conflict-modal__close"
                                on:click=move |_| open.set(false)
                            >
                                "×"
                            </button>
                        </div>

                        <div class="conflict-modal__body">
                            <div class="conflict-modal__diff">
                                // Server side
                                <div class="conflict-modal__panel">
                                    <div class="conflict-modal__panel-header">
                                        <span class="conflict-modal__panel-label">"Server"</span>
                                        <button
                                            class="conflict-modal__use-btn"
                                            on:click=move |_| {
                                                result_text.set(server_content.get_untracked());
                                            }
                                        >
                                            "Use all"
                                        </button>
                                    </div>
                                    <div class="conflict-modal__lines">
                                        <For
                                            each=move || {
                                                diff_data.get().0.into_iter().enumerate().collect::<Vec<_>>()
                                            }
                                            key=|(i, _)| *i
                                            children=move |(_, line)| {
                                                let text = line.text.clone();
                                                let text_for_click = text.clone();
                                                let kind = line.kind;
                                                view! {
                                                    <div
                                                        class=format!("conflict-modal__line conflict-modal__line--{}", kind)
                                                        on:click=move |_| {
                                                            let t = text_for_click.clone();
                                                            result_text.update(|r| {
                                                                if !r.is_empty() && !r.ends_with('\n') {
                                                                    r.push('\n');
                                                                }
                                                                r.push_str(&t);
                                                            });
                                                        }
                                                        title="Click to add this line to result"
                                                    >
                                                        {text}
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                </div>

                                // Local side
                                <div class="conflict-modal__panel">
                                    <div class="conflict-modal__panel-header">
                                        <span class="conflict-modal__panel-label">"Local"</span>
                                        <button
                                            class="conflict-modal__use-btn"
                                            on:click=move |_| {
                                                result_text.set(local_content.get_untracked());
                                            }
                                        >
                                            "Use all"
                                        </button>
                                    </div>
                                    <div class="conflict-modal__lines">
                                        <For
                                            each=move || {
                                                diff_data.get().1.into_iter().enumerate().collect::<Vec<_>>()
                                            }
                                            key=|(i, _)| *i
                                            children=move |(_, line)| {
                                                let text = line.text.clone();
                                                let text_for_click = text.clone();
                                                let kind = line.kind;
                                                view! {
                                                    <div
                                                        class=format!("conflict-modal__line conflict-modal__line--{}", kind)
                                                        on:click=move |_| {
                                                            let t = text_for_click.clone();
                                                            result_text.update(|r| {
                                                                if !r.is_empty() && !r.ends_with('\n') {
                                                                    r.push('\n');
                                                                }
                                                                r.push_str(&t);
                                                            });
                                                        }
                                                        title="Click to add this line to result"
                                                    >
                                                        {text}
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                </div>
                            </div>

                            // Result area
                            <div class="conflict-modal__result">
                                <div class="conflict-modal__panel-header">
                                    <span class="conflict-modal__panel-label">"Result"</span>
                                    <button
                                        class="conflict-modal__use-btn"
                                        on:click=move |_| result_text.set(String::new())
                                    >
                                        "Clear"
                                    </button>
                                </div>
                                <textarea
                                    class="conflict-modal__result-textarea"
                                    prop:value=move || result_text.get()
                                    on:input=move |ev| {
                                        result_text.set(event_target_value(&ev));
                                    }
                                />
                            </div>
                        </div>

                        <div class="conflict-modal__footer">
                            <button
                                class="conflict-modal__cancel-btn"
                                on:click=move |_| open.set(false)
                            >
                                "Cancel"
                            </button>
                            <button
                                class="conflict-modal__resolve-btn"
                                on:click=move |_| {
                                    let resolved = result_text.get_untracked();
                                    on_resolve_inner(resolved);
                                }
                            >
                                "Resolve"
                            </button>
                        </div>
                    </div>
                </div>
            }.into_view()
        }}
    }
}
