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
//! Application settings facade.
//!
//! Public types stay available from `crate::handlers::setting`, while
//! persistence, credentials, defaults, and theme concerns live in submodules.

mod credentials;
mod defaults;
mod secrets;
mod services;
mod storage;
mod theme;

pub use credentials::{PrereqCredentials, WebminCredentials};
#[allow(unused_imports)]
pub use services::{MailpitServiceSettings, OptionalServicesSettings, RedisServiceSettings};
pub use storage::AppSettings;
pub use theme::AppTheme;
