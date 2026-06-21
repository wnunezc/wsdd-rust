// WebStack Deployer for Docker
// Copyright (c) 2026 Walter Nunez / Icaros Net S.A
// All Rights Reserved.
//
// This software is provided for development use only.
// Unauthorized commercial use is prohibited.
//
// Redistribution and modification allowed only through
// the official GitHub repository.
//
// This software is provided AS IS, without warranty of any kind.
// The author shall not be liable for any damages.
//
// Contact: wnunez@lh-2.net
//! Project domain model facade.

mod domain;
mod entry_point;
mod php_version;
mod status;
mod storage;
mod types;

pub use domain::normalize_domain;
pub use entry_point::EntryPoint;
pub use php_version::PhpVersion;
pub use status::ProjectStatus;
pub use storage::{is_native_linux_path, StorageMode, CURRENT_PROJECT_METADATA_VERSION};
pub use types::{ManagedNativeStorage, Project};
