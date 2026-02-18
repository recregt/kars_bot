use std::{panic::AssertUnwindSafe, sync::Arc, time::Duration};

use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    task::JoinError,
    time::timeout,
};

use super::{
    error::GraphRenderError, render::render_graph_png, stats::GraphPoint, types::GraphMetric,
};

pub(super) async fn acquire_render_slot(
    slots: Arc<Semaphore>,
    wait_timeout_secs: u64,
) -> Result<OwnedSemaphorePermit, GraphRenderError> {
    match timeout(
        Duration::from_secs(wait_timeout_secs),
        slots.acquire_owned(),
    )
    .await
    {
        Ok(Ok(permit)) => Ok(permit),
        Ok(Err(error)) => Err(GraphRenderError::RenderSlot(error.to_string())),
        Err(_) => Err(GraphRenderError::RenderSlotTimeout(wait_timeout_secs)),
    }
}

pub(super) async fn run_render_task(
    points: Vec<GraphPoint>,
    metric: GraphMetric,
    threshold: f32,
    render_slot: OwnedSemaphorePermit,
    render_timeout_secs: u64,
) -> Result<Vec<u8>, GraphRenderError> {
    let render_handle = tokio::task::spawn_blocking(move || {
        let _render_slot = render_slot;
        std::panic::catch_unwind(AssertUnwindSafe(|| {
            render_graph_png(points, metric, threshold)
        }))
        .map_err(|panic_payload| GraphRenderError::Panic(describe_panic_payload(panic_payload)))?
    });

    match timeout(Duration::from_secs(render_timeout_secs), render_handle).await {
        Ok(join_result) => match join_result {
            Ok(inner_result) => inner_result,
            Err(join_error) => Err(join_error_to_error(join_error)),
        },
        Err(_) => Err(GraphRenderError::RenderTimeout(render_timeout_secs)),
    }
}

fn join_error_to_error(join_error: JoinError) -> GraphRenderError {
    GraphRenderError::Join(join_error.to_string())
}

fn describe_panic_payload(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }

    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }

    "unknown panic payload".to_string()
}
