pub mod parameter;

/// Re-export
pub use async_trait::async_trait;
use serde::{de::DeserializeOwned, ser::Serialize};
use std::fmt;

#[async_trait]
pub trait RpcParameter: fmt::Debug + DeserializeOwned + Serialize {
    type Output;

    fn method_name() -> &'static str;
    async fn handler(self) -> Self::Output;
}
