use thiserror::Error;

#[derive(Debug, Error)]
pub(super) enum GraphRenderError {
    #[error("not enough points to render")]
    NotEnoughPoints,
    #[error("font unavailable: {0}")]
    FontUnavailable(String),
    #[error("render backend failure: {0}")]
    Backend(String),
    #[error("png encoding failure: {0}")]
    PngEncoding(String),
    #[error("render task join failure: {0}")]
    Join(String),
    #[error("render task panic: {0}")]
    Panic(String),
    #[error("render slot unavailable: {0}")]
    RenderSlot(String),
    #[error("render slot wait timeout after {0}s")]
    RenderSlotTimeout(u64),
    #[error("render execution timeout after {0}s")]
    RenderTimeout(u64),
}

impl GraphRenderError {
    pub(super) fn code(&self) -> &'static str {
        match self {
            Self::NotEnoughPoints => "GRAPH_NOT_ENOUGH_POINTS",
            Self::FontUnavailable(_) => "GRAPH_FONT_UNAVAILABLE",
            Self::Backend(_) => "GRAPH_BACKEND_ERROR",
            Self::PngEncoding(_) => "GRAPH_PNG_ENCODING_ERROR",
            Self::Join(_) => "GRAPH_TASK_JOIN_ERROR",
            Self::Panic(_) => "GRAPH_TASK_PANIC",
            Self::RenderSlot(_) => "GRAPH_RENDER_SLOT_ERROR",
            Self::RenderSlotTimeout(_) => "GRAPH_RENDER_SLOT_TIMEOUT",
            Self::RenderTimeout(_) => "GRAPH_RENDER_TIMEOUT",
        }
    }

    pub(super) fn user_message(&self) -> &'static str {
        match self {
            Self::FontUnavailable(_) => {
                "Graph engine font is unavailable. Install a system font package or disable graph feature in config."
            }
            Self::RenderSlotTimeout(_) => "Graph renderer is busy. Please retry shortly.",
            Self::RenderTimeout(_) => {
                "Graph render took too long and was cancelled. Please try a shorter window."
            }
            _ => "Could not render graph right now. Please try again.",
        }
    }
}
