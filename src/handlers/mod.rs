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
// Handlers — logica de negocio. Equivalente a Handlers/ en C#.

pub mod chocolatey;
pub mod deploy;
pub mod docker;
pub mod docker_deploy;
pub mod external_app;
pub mod hosts;
pub mod log_types;
pub mod log_viewer;
pub mod mkcert;
pub mod output;
pub mod project;
pub mod ps_script;
pub mod requirements;
pub mod setting;
pub mod wsl;
pub mod xml;
pub mod yml;
