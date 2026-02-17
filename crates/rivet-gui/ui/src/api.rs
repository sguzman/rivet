use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::{
  JsCast,
  JsValue
};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys;

pub async fn invoke_tauri<R, A>(
  cmd: &str,
  args_payload: &A
) -> Result<R, String>
where
  R: DeserializeOwned,
  A: Serialize + ?Sized
{
  #[derive(Serialize)]
  struct CommandArgs<'a, T: ?Sized> {
    args: &'a T
  }

  let command_args = CommandArgs {
    args: args_payload
  };
  let payload =
    serde_wasm_bindgen::to_value(
      &command_args
    )
    .map_err(|e| {
      format!("encode error: {e}")
    })?;

  let window = web_sys::window()
    .ok_or_else(|| {
      "window unavailable".to_string()
    })?;
  let tauri_internals = js_get(
    window.as_ref(),
    "__TAURI_INTERNALS__"
  )
  .map_err(|e| {
    format!("bridge error: {e}")
  })?;
  if tauri_internals.is_undefined()
    || tauri_internals.is_null()
  {
    return Err(
      "bridge error: \
       __TAURI_INTERNALS__ unavailable"
        .to_string()
    );
  }

  let invoke_value =
    js_get(&tauri_internals, "invoke")
      .map_err(|e| {
        format!("bridge error: {e}")
      })?;
  let invoke_fn: js_sys::Function =
    invoke_value.dyn_into().map_err(
      |_| {
        "bridge error: \
         __TAURI_INTERNALS__.invoke is \
         not a function"
          .to_string()
      }
    )?;

  let promise_value = invoke_fn
    .call2(
      &tauri_internals,
      &JsValue::from_str(cmd),
      &payload
    )
    .map_err(|e| {
      format!(
        "invoke call error ({cmd}): {}",
        js_error_to_string(&e)
      )
    })?;
  let promise: js_sys::Promise =
    promise_value.dyn_into().map_err(
      |_| {
        format!(
          "invoke call error ({cmd}): \
           return value is not a \
           Promise"
        )
      }
    )?;
  let value = JsFuture::from(promise)
    .await
    .map_err(|e| {
      format!(
        "invoke error ({cmd}): {}",
        js_error_to_string(&e)
      )
    })?;

  serde_wasm_bindgen::from_value(value)
    .map_err(|e| {
      format!("decode error: {e}")
    })
}

fn js_get(
  target: &JsValue,
  key: &str
) -> Result<JsValue, String> {
  js_sys::Reflect::get(
    target,
    &JsValue::from_str(key)
  )
  .map_err(|e| {
    format!(
      "failed to read {key}: {}",
      js_error_to_string(&e)
    )
  })
}

fn js_error_to_string(
  value: &JsValue
) -> String {
  if let Some(text) = value.as_string()
  {
    return text;
  }

  if let Ok(message) =
    js_sys::Reflect::get(
      value,
      &JsValue::from_str("message")
    )
    && let Some(text) =
      message.as_string()
  {
    return text;
  }

  format!("{value:?}")
}
