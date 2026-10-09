//! Re-exports of external dependencies shared across the crate.

// The generated code lists use these items only when their own features are enabled.
#[cfg(feature = "serde")]
#[allow(unused_imports)]
pub use serde::de::Error as DeError;
#[cfg(feature = "serde")]
#[allow(unused_imports)]
pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
