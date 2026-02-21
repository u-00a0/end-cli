use end_model::OutpostId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("catalog load failed: {0}")]
    Catalog(#[source] end_io::Error),

    #[error("default aic generation failed: {0}")]
    DefaultAic(#[source] end_io::Error),

    #[error("aic parse failed: {0}")]
    Aic(#[source] end_io::Error),

    #[error("optimization failed: {0}")]
    Optimize(#[source] end_opt::Error),

    #[error("missing outpost id {0:?}")]
    MissingOutpost(OutpostId),

    #[error("missing logistics node {node:?} for item {item:?}")]
    MissingLogisticsNode {
        item: u32,
        node: end_opt::LogisticsNodeId,
    },

    #[error("unknown lang `{value}` (expected `zh` or `en`)")]
    UnknownLang { value: String },

    #[error("null pointer for argument `{name}`")]
    NullPointer { name: &'static str },

    #[error("argument `{name}` is not valid UTF-8")]
    InvalidUtf8 { name: &'static str },
}

pub type Result<T> = std::result::Result<T, Error>;
