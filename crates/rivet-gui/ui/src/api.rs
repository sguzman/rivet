use serde::{Serialize, de::DeserializeOwned};
use tauri_wasm::{args, invoke};

pub async fn invoke_tauri<R, A>(cmd: &str, args_payload: &A) -> Result<R, String>
where
    R: DeserializeOwned,
    A: Serialize + ?Sized,
{
    let payload = args(args_payload).map_err(|e| format!("failed to encode args: {e}"))?;
    let value = invoke(cmd)
        .with_args(payload)
        .await
        .map_err(|e| format!("invoke error: {e:?}"))?;

    serde_wasm_bindgen::from_value(value).map_err(|e| format!("decode error: {e}"))
}
