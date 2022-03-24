// Copyright 2022 Singularity Data
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(map_try_insert)]
#![feature(let_chains)]

#[macro_use]
pub mod catalog;
pub mod binder;
pub mod expr;
pub mod handler;
pub mod observer;
pub mod optimizer;
pub mod planner;
mod scheduler;
pub mod session;
pub mod utils;
extern crate log;
mod meta_client;
pub mod test_utils;
extern crate risingwave_common;

use std::ffi::OsString;
use std::iter;
use std::sync::Arc;

use clap::Parser;
use pgwire::pg_server::pg_serve;
use session::SessionManagerImpl;

#[derive(Parser, Clone, Debug)]
pub struct FrontendOpts {
    #[clap(long, default_value = "127.0.0.1:4566")]
    pub host: String,

    #[clap(long, default_value = "http://127.0.0.1:5690")]
    pub meta_addr: String,

    /// No given `config_path` means to use default config.
    #[clap(long, default_value = "")]
    pub config_path: String,
}

impl Default for FrontendOpts {
    fn default() -> Self {
        FrontendOpts::parse_from(iter::empty::<OsString>())
    }
}

/// Start frontend
pub async fn start(opts: FrontendOpts) {
    let session_mgr = Arc::new(SessionManagerImpl::new(&opts).await.unwrap());
    pg_serve(&opts.host, session_mgr).await.unwrap();
}
