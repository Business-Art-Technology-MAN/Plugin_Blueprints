//! Feature modules - each feature (nodes, connections, etc.) is self-contained
//!
//! Each feature module typically contains:
//! - types.rs: Feature-specific types
//! - operations.rs: Business logic and state mutations
//! - rendering.rs: GPUI rendering code
//! - panel.rs: Dockable panel (if applicable)

pub mod nodes;
pub mod connections;
pub mod comments;
pub mod variables;
pub mod macros;
pub mod viewport;
pub mod compilation;
