//! # vidra-ir
//!
//! The Vidra Intermediate Representation (IR) — a queryable, composable,
//! deterministic scene graph that serves as the universal language for video.
//!
//! Every input (VidraScript, SDK, MCP) compiles down to this IR before rendering.

pub mod animation;
pub mod asset;
pub mod data;
pub mod layer;
pub mod layout;
pub mod project;
pub mod scene;
pub mod validate;

pub use animation::{Animation, Keyframe};
pub use asset::{Asset, AssetId, AssetRegistry, AssetType};
pub use layer::{Layer, LayerContent, LayerId};
pub use layout::{LayoutConstraint, LayoutSolver, ResolvedLayout};
pub use project::{Project, ProjectSettings};
pub use scene::{Scene, SceneId};
pub mod builder;
pub mod crdt;
pub mod transition;
