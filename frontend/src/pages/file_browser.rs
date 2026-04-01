use leptos::*;
use leptos_router::*;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;
use wasm_bindgen::JsCast;

use crate::services::auth_service;
use crate::services::local_storage::{LocalDb, LocalFileKeywords};
use crate::services::search_service;

#[derive(Clone)]
struct FileEntry {
    path: String,
    title: String,
    file_type: String,
}

fn collect_unique_keywords(entries: &[LocalFileKeywords]) -> Vec<String> {
    let set: BTreeSet<String> = entries
        .iter()
        .flat_map(|e| e.keywords.iter().cloned())
        .collect();
    set.into_iter().collect()
}

fn current_partial_word(query: &str) -> &str {
    query.rsplit(|c: char| c.is_whitespace()).next().unwrap_or("")
}

fn replace_last_word(query: &str, replacement: &str) -> String {
    match query.rfind(|c: char| c.is_whitespace()) {
        Some(pos) => format!("{} {} ", &query[..=pos], replacement),
        None => format!("{} ", replacement),
    }
}

#[component]
pub fn FileBrowserPage() -> impl IntoView {
    if !auth_service::is_authenticated() {
        let navigate = use_navigate();
        navigate(
            "/login",
            NavigateOptions {
                replace: true,
                ..Default::default()
            },
        );
        return view! { <div/> }.into_view();
    }

    let search_query = create_rw_signal(String::new());
    let results: RwSignal<Vec<FileEntry>> = create_rw_signal(Vec::new());
    let all_files: RwSignal<Vec<FileEntry>> = create_rw_signal(Vec::new());
    let index_ready = create_rw_signal(false);
    let loading = create_rw_signal(true);
    let search_timeout: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

    let all_keywords: Rc<RefCell<Vec<LocalFileKeywords>>> = Rc::new(RefCell::new(Vec::new()));
    let vocabulary: RwSignal<Vec<String>> = create_rw_signal(Vec::new());
    let suggestions: RwSignal<Vec<String>> = create_rw_signal(Vec::new());
    let show_suggestions = create_rw_signal(false);
    let hide_timeout: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

    // Initialize: open DB, rebuild index if needed, load all files
    let all_keywords_init = all_keywords.clone();
    spawn_local({
        let all_files = all_files;
        let results = results;
        let index_ready = index_ready;
        let loading = loading;
        async move {
            let db = match LocalDb::open().await {
                Ok(db) => Rc::new(db),
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("IndexedDB unavailable: {:?}", e).into(),
                    );
                    loading.set(false);
                    return;
                }
            };

            if search_service::needs_rebuild(&db).await {
                if let Err(e) = search_service::rebuild_index(&db).await {
                    web_sys::console::warn_1(
                        &format!("Index rebuild failed: {}", e).into(),
                    );
                }
            }

            match db.list_all_file_keywords().await {
                Ok(entries) => {
                    let files: Vec<FileEntry> = entries
                        .iter()
                        .map(|e| FileEntry {
                            path: e.path.clone(),
                            title: e.title.clone(),
                            file_type: e.file_type.clone(),
                        })
                        .collect();
                    all_files.set(files.clone());
                    results.set(files);
                    vocabulary.set(collect_unique_keywords(&entries));
                    *all_keywords_init.borrow_mut() = entries;
                    index_ready.set(true);
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to load file keywords: {:?}", e).into(),
                    );
                }
            }

            loading.set(false);
        }
    });

    // Keyword completion suggestions
    create_effect(move |_| {
        let query = search_query.get();
        let partial = current_partial_word(&query).to_lowercase();
        if partial.len() < 2 {
            suggestions.set(Vec::new());
            return;
        }
        let vocab = vocabulary.get_untracked();
        let matches: Vec<String> = vocab
            .iter()
            .filter(|kw| kw.starts_with(&partial) && kw.as_str() != partial)
            .take(6)
            .cloned()
            .collect();
        suggestions.set(matches);
    });

    // Debounced search effect
    let search_timeout_clone = search_timeout.clone();
    let all_keywords_search = all_keywords.clone();
    create_effect(move |_| {
        let query = search_query.get();

        if let Some(timeout_id) = search_timeout_clone.borrow_mut().take() {
            let window = web_sys::window().expect("no window");
            window.clear_timeout_with_handle(timeout_id);
        }

        if query.trim().is_empty() {
            results.set(all_files.get_untracked());
            return;
        }

        let all_kw = all_keywords_search.borrow().clone();
        let is_ready = index_ready.get_untracked();
        let window = web_sys::window().expect("no window");

        let cb = wasm_bindgen::closure::Closure::once(move || {
            if is_ready {
                let search_results = search_service::search_local(&all_kw, &query);
                let files: Vec<FileEntry> = search_results
                    .into_iter()
                    .map(|r| FileEntry {
                        path: r.path,
                        title: r.title,
                        file_type: r.file_type,
                    })
                    .collect();
                results.set(files);
            } else {
                spawn_local(async move {
                    match search_service::search_remote(&query).await {
                        Ok(search_results) => {
                            let files: Vec<FileEntry> = search_results
                                .into_iter()
                                .map(|r| FileEntry {
                                    path: r.path,
                                    title: r.title,
                                    file_type: r.file_type,
                                })
                                .collect();
                            results.set(files);
                        }
                        Err(e) => {
                            web_sys::console::error_1(
                                &format!("Search failed: {}", e).into(),
                            );
                        }
                    }
                });
            }
        });

        let timeout_id = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                300,
            )
            .expect("failed to set timeout");
        cb.forget();
        *search_timeout.borrow_mut() = Some(timeout_id);
    });

    view! {
        <div class="file-browser">
            <div class="file-browser__header">
                <h1 class="file-browser__title">"Files"</h1>
                <button class="file-browser__add-btn" disabled=true title="Create new document (coming soon)">
                    "+"
                </button>
            </div>

            <div class="file-browser__search">
                <input
                    class="file-browser__search-input"
                    type="text"
                    placeholder="Search documents..."
                    prop:value=move || search_query.get()
                    on:input=move |ev| {
                        search_query.set(event_target_value(&ev));
                        show_suggestions.set(true);
                    }
                    on:focus=move |_| {
                        show_suggestions.set(true);
                    }
                    on:blur={
                        let hide_timeout = hide_timeout.clone();
                        move |_| {
                            // Delay hiding so click on suggestion registers first
                            let window = web_sys::window().expect("no window");
                            let cb = wasm_bindgen::closure::Closure::once(move || {
                                show_suggestions.set(false);
                            });
                            if let Some(prev) = hide_timeout.borrow_mut().take() {
                                window.clear_timeout_with_handle(prev);
                            }
                            let tid = window
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    cb.as_ref().unchecked_ref(),
                                    150,
                                )
                                .expect("failed to set timeout");
                            cb.forget();
                            *hide_timeout.borrow_mut() = Some(tid);
                        }
                    }
                />
                {move || {
                    let items = suggestions.get();
                    let visible = show_suggestions.get();
                    if items.is_empty() || !visible {
                        view! { <div/> }.into_view()
                    } else {
                        view! {
                            <div class="file-browser__suggestions">
                                {items.into_iter().map(|kw| {
                                    let kw_display = kw.clone();
                                    let kw_click = kw.clone();
                                    view! {
                                        <button
                                            class="file-browser__suggestion"
                                            on:mousedown=move |ev| {
                                                ev.prevent_default();
                                            }
                                            on:click=move |_| {
                                                let current = search_query.get_untracked();
                                                let new_query = replace_last_word(&current, &kw_click);
                                                search_query.set(new_query);
                                                show_suggestions.set(false);
                                            }
                                        >
                                            {kw_display}
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_view()
                    }
                }}
            </div>

            <div class="file-browser__list">
                {move || {
                    if loading.get() {
                        view! { <div class="file-browser__empty">"Loading..."</div> }.into_view()
                    } else if results.get().is_empty() {
                        if search_query.get().trim().is_empty() {
                            view! { <div class="file-browser__empty">"No documents yet"</div> }.into_view()
                        } else {
                            view! { <div class="file-browser__empty">"No results found"</div> }.into_view()
                        }
                    } else {
                        view! {
                            <For
                                each=move || results.get()
                                key=|f| f.path.clone()
                                children=move |f| {
                                    let path = f.path.clone();
                                    view! {
                                        <button
                                            class="file-browser__row"
                                            on:click=move |_| {
                                                let navigate = use_navigate();
                                                if let Some(chat_id) = path.strip_prefix("chats/") {
                                                    navigate(
                                                        &format!("/chat/{}", chat_id),
                                                        NavigateOptions::default(),
                                                    );
                                                }
                                            }
                                        >
                                            <span class="file-browser__row-type">{f.file_type.clone()}</span>
                                            <span class="file-browser__row-title">{f.title.clone()}</span>
                                        </button>
                                    }
                                }
                            />
                        }.into_view()
                    }
                }}
            </div>
        </div>
    }
    .into_view()
}
