mod api;
mod app;
mod components;

fn main() {
  console_error_panic_hook::set_once();
  wasm_tracing::set_as_global_default();

  tracing::info!(
    "starting Rivet GUI frontend"
  );

  let mount = web_sys::window()
    .and_then(|window| {
      window.document()
    })
    .and_then(|document| {
      document.get_element_by_id("app")
    })
    .expect(
      "missing #app mount element"
    );

  yew::Renderer::<app::App>::with_root(
    mount
  )
  .render();
}
