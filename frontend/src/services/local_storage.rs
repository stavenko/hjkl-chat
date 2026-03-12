use js_sys::Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    IdbDatabase, IdbIndexParameters, IdbObjectStoreParameters, IdbRequest, IdbTransaction,
    IdbTransactionMode, IdbVersionChangeEvent,
};

const DB_NAME: &str = "hjkl_chat";
const DB_VERSION: u32 = 1;

const STORE_CHATS: &str = "chats";
const STORE_MESSAGES: &str = "messages";
const STORE_DRAFTS: &str = "drafts";
const STORE_SYNC_META: &str = "sync_meta";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalChatIndex {
    pub id: String,
    pub model: String,
    pub message_ids: Vec<String>,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalChatMessage {
    pub id: String,
    pub chat_id: String,
    pub role: String,
    pub content: String,
    pub reasoning: Option<String>,
    pub created_at: String,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDraftEntry {
    pub id: String,
    pub chat_id: String,
    pub content: String,
    pub model: String,
    pub version: u64,
}

pub struct LocalDb {
    db: IdbDatabase,
}

impl LocalDb {
    pub async fn open() -> Result<Self, JsValue> {
        let window = web_sys::window().expect("no window");
        let idb_factory = window
            .indexed_db()?
            .ok_or_else(|| JsValue::from_str("IndexedDB not available"))?;

        let open_request = idb_factory.open_with_u32(DB_NAME, DB_VERSION)?;

        let on_upgrade = Closure::once(move |event: IdbVersionChangeEvent| {
            let db: IdbDatabase = event
                .target()
                .unwrap()
                .unchecked_into::<IdbRequest>()
                .result()
                .unwrap()
                .unchecked_into();

            let store_names = db.object_store_names();

            if !store_names.contains(STORE_CHATS) {
                let params = IdbObjectStoreParameters::new();
                params.set_key_path(&JsValue::from_str("id"));
                db.create_object_store_with_optional_parameters(STORE_CHATS, &params)
                    .unwrap();
            }

            if !store_names.contains(STORE_MESSAGES) {
                let params = IdbObjectStoreParameters::new();
                params.set_key_path(&JsValue::from_str("id"));
                let store = db
                    .create_object_store_with_optional_parameters(STORE_MESSAGES, &params)
                    .unwrap();
                let idx_params = IdbIndexParameters::new();
                idx_params.set_unique(false);
                store
                    .create_index_with_str_and_optional_parameters(
                        "chat_id",
                        "chat_id",
                        &idx_params,
                    )
                    .unwrap();
            }

            if !store_names.contains(STORE_DRAFTS) {
                let params = IdbObjectStoreParameters::new();
                params.set_key_path(&JsValue::from_str("id"));
                let store = db
                    .create_object_store_with_optional_parameters(STORE_DRAFTS, &params)
                    .unwrap();
                let idx_params = IdbIndexParameters::new();
                idx_params.set_unique(false);
                store
                    .create_index_with_str_and_optional_parameters(
                        "chat_id",
                        "chat_id",
                        &idx_params,
                    )
                    .unwrap();
            }

            if !store_names.contains(STORE_SYNC_META) {
                let params = IdbObjectStoreParameters::new();
                params.set_key_path(&JsValue::from_str("key"));
                db.create_object_store_with_optional_parameters(STORE_SYNC_META, &params)
                    .unwrap();
            }
        });

        open_request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
        on_upgrade.forget();

        let db = idb_request_to_future(&open_request).await?;
        let db: IdbDatabase = db.unchecked_into();

        Ok(LocalDb { db })
    }

    // --- Chats ---

    pub async fn put_chat(&self, chat: &LocalChatIndex) -> Result<(), JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_CHATS, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(STORE_CHATS)?;
        let js_val = serde_wasm_bindgen::to_value(chat)?;
        store.put(&js_val)?;
        idb_tx_to_future(&tx).await
    }

    pub async fn get_chat(&self, id: &str) -> Result<Option<LocalChatIndex>, JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_CHATS, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(STORE_CHATS)?;
        let req = store.get(&JsValue::from_str(id))?;
        let result = idb_request_to_future(&req).await?;
        if result.is_undefined() || result.is_null() {
            Ok(None)
        } else {
            let chat: LocalChatIndex = serde_wasm_bindgen::from_value(result)?;
            Ok(Some(chat))
        }
    }

    pub async fn list_chats(&self) -> Result<Vec<LocalChatIndex>, JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_CHATS, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(STORE_CHATS)?;
        let req = store.get_all()?;
        let result = idb_request_to_future(&req).await?;
        let array: Array = result.unchecked_into();
        let mut chats = Vec::new();
        for i in 0..array.length() {
            let val = array.get(i);
            let chat: LocalChatIndex = serde_wasm_bindgen::from_value(val)?;
            chats.push(chat);
        }
        Ok(chats)
    }

    // --- Messages ---

    pub async fn put_message(&self, msg: &LocalChatMessage) -> Result<(), JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_MESSAGES, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(STORE_MESSAGES)?;
        let js_val = serde_wasm_bindgen::to_value(msg)?;
        store.put(&js_val)?;
        idb_tx_to_future(&tx).await
    }

    pub async fn get_messages_for_chat(
        &self,
        chat_id: &str,
    ) -> Result<Vec<LocalChatMessage>, JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_MESSAGES, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(STORE_MESSAGES)?;
        let index = store.index("chat_id")?;
        let req = index.get_all_with_key(&JsValue::from_str(chat_id))?;
        let result = idb_request_to_future(&req).await?;
        let array: Array = result.unchecked_into();
        let mut messages = Vec::new();
        for i in 0..array.length() {
            let val = array.get(i);
            let msg: LocalChatMessage = serde_wasm_bindgen::from_value(val)?;
            messages.push(msg);
        }
        messages.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(messages)
    }

    pub async fn put_messages_bulk(&self, msgs: &[LocalChatMessage]) -> Result<(), JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_MESSAGES, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(STORE_MESSAGES)?;
        for msg in msgs {
            let js_val = serde_wasm_bindgen::to_value(msg)?;
            store.put(&js_val)?;
        }
        idb_tx_to_future(&tx).await
    }

    // --- Drafts ---

    pub async fn put_draft(&self, draft: &LocalDraftEntry) -> Result<(), JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_DRAFTS, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(STORE_DRAFTS)?;
        let js_val = serde_wasm_bindgen::to_value(draft)?;
        store.put(&js_val)?;
        idb_tx_to_future(&tx).await
    }

    pub async fn get_draft(&self, id: &str) -> Result<Option<LocalDraftEntry>, JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_DRAFTS, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(STORE_DRAFTS)?;
        let req = store.get(&JsValue::from_str(id))?;
        let result = idb_request_to_future(&req).await?;
        if result.is_undefined() || result.is_null() {
            Ok(None)
        } else {
            let draft: LocalDraftEntry = serde_wasm_bindgen::from_value(result)?;
            Ok(Some(draft))
        }
    }

    pub async fn delete_draft(&self, id: &str) -> Result<(), JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_DRAFTS, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(STORE_DRAFTS)?;
        store.delete(&JsValue::from_str(id))?;
        idb_tx_to_future(&tx).await
    }

    pub async fn get_drafts_for_chat(
        &self,
        chat_id: &str,
    ) -> Result<Vec<LocalDraftEntry>, JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_DRAFTS, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(STORE_DRAFTS)?;
        let index = store.index("chat_id")?;
        let req = index.get_all_with_key(&JsValue::from_str(chat_id))?;
        let result = idb_request_to_future(&req).await?;
        let array: Array = result.unchecked_into();
        let mut drafts = Vec::new();
        for i in 0..array.length() {
            let val = array.get(i);
            let draft: LocalDraftEntry = serde_wasm_bindgen::from_value(val)?;
            drafts.push(draft);
        }
        Ok(drafts)
    }

    // --- Sync metadata ---

    pub async fn get_last_synced_version(&self) -> Result<u64, JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_SYNC_META, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(STORE_SYNC_META)?;
        let req = store.get(&JsValue::from_str("last_synced_version"))?;
        let result = idb_request_to_future(&req).await?;
        if result.is_undefined() || result.is_null() {
            Ok(0)
        } else {
            let value = js_sys::Reflect::get(&result, &JsValue::from_str("value"))?;
            Ok(value.as_f64().unwrap_or(0.0) as u64)
        }
    }

    pub async fn set_last_synced_version(&self, version: u64) -> Result<(), JsValue> {
        let tx = self
            .db
            .transaction_with_str_and_mode(STORE_SYNC_META, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(STORE_SYNC_META)?;
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("key"),
            &JsValue::from_str("last_synced_version"),
        )?;
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("value"),
            &JsValue::from_f64(version as f64),
        )?;
        store.put(&obj)?;
        idb_tx_to_future(&tx).await
    }
}

/// Convert an IdbRequest into a Future that resolves with the request's result.
async fn idb_request_to_future(request: &IdbRequest) -> Result<JsValue, JsValue> {
    let request_clone = request.clone();
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let resolve2 = resolve.clone();
        let reject2 = reject.clone();

        let onsuccess = Closure::once(Box::new(move |_event: web_sys::Event| {
            resolve2.call0(&JsValue::NULL).unwrap();
        }) as Box<dyn FnOnce(web_sys::Event)>);

        let onerror = Closure::once(Box::new(move |_event: web_sys::Event| {
            reject2
                .call1(&JsValue::NULL, &JsValue::from_str("IDB request failed"))
                .unwrap();
        }) as Box<dyn FnOnce(web_sys::Event)>);

        request_clone.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        request_clone.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onsuccess.forget();
        onerror.forget();
    });

    JsFuture::from(promise).await?;
    request.result()
}

/// Convert an IdbTransaction complete event into a Future.
async fn idb_tx_to_future(tx: &IdbTransaction) -> Result<(), JsValue> {
    let tx_clone = tx.clone();
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let resolve2 = resolve.clone();
        let reject2 = reject.clone();

        let oncomplete = Closure::once(Box::new(move |_event: web_sys::Event| {
            resolve2.call0(&JsValue::NULL).unwrap();
        }) as Box<dyn FnOnce(web_sys::Event)>);

        let onerror = Closure::once(Box::new(move |_event: web_sys::Event| {
            reject2
                .call1(&JsValue::NULL, &JsValue::from_str("IDB transaction failed"))
                .unwrap();
        }) as Box<dyn FnOnce(web_sys::Event)>);

        tx_clone.set_oncomplete(Some(oncomplete.as_ref().unchecked_ref()));
        tx_clone.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        oncomplete.forget();
        onerror.forget();
    });

    JsFuture::from(promise).await?;
    Ok(())
}
