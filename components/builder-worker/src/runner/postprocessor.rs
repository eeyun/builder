// Copyright (c) 2016-2017 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::bldr_core::logger::Logger;
use crate::config::Config;
use crate::error::Result;
use crate::hab_core::package::archive::PackageArchive;

use super::publisher::Publisher;
use super::workspace::Workspace;

pub fn post_process(
    archive: &mut PackageArchive,
    workspace: &Workspace,
    config: &Config,
    auth_token: &str,
    logger: &mut Logger,
) -> Result<()> {
    let channel_opt = if workspace.job.has_channel() {
        Some(workspace.job.get_channel().to_string())
    } else {
        None
    };

    let url = config.bldr_url.clone();

    let mut publisher = Publisher {
        enabled: config.auto_publish,
        url,
        channel_opt,
    };

    debug!("Starting post processing");
    publisher.run(archive, auth_token, logger)
}
