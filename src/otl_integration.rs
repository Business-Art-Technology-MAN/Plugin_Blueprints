//! **FR-06** — Optional OTL smoke path after blueprint compile (feature **`marketlab`**).
//!
//! Uses [`engine_backend::marketlab::otl_eval_dispatcher`] so work stays off the GPUI thread; completion is logged only.

const SAMPLE_OTL: &str = r#"
signal smoke(
    input float x = 2.0,
    output float y = 0.0,
    aov float z = 0.0
) {
    y = x * 3.0;
    z = x + 1.0;
}
"#;

/// After a successful blueprint compile, enqueue a tiny OTL compile+eval on the Market Lab worker and log the result.
/// Does not block the caller.
pub fn schedule_otl_smoke_after_blueprint_compile() {
    use engine_backend::marketlab::{otl_eval_dispatcher, RunArgs};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    if otl_eval_dispatcher()
        .submit_compile_and_run(&tx, SAMPLE_OTL.to_string(), RunArgs::default())
        .is_err()
    {
        tracing::warn!(target: "blueprint_otl", "failed to enqueue OTL smoke job");
        return;
    }

    std::thread::spawn(move || match rx.recv() {
        Ok(Ok(res)) => {
            if let Some(p) = res.primary() {
                tracing::info!(
                    target: "blueprint_otl",
                    primary = %p.name,
                    "Market Lab OTL smoke OK after blueprint compile"
                );
            } else {
                tracing::info!(target: "blueprint_otl", "Market Lab OTL smoke OK (no primary output)");
            }
        }
        Ok(Err(e)) => tracing::warn!(target: "blueprint_otl", ?e, "OTL smoke failed"),
        Err(_) => tracing::warn!(target: "blueprint_otl", "OTL smoke channel closed"),
    });
}
