// SPDX-License-Identifier: GPL-2.0-or-later

use std::fs;
use std::path::Path;

use anyhow::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeEnv {
    Docker,
    Kubernetes,
    Other,
}

impl RuntimeEnv {
    pub fn current(
        spec: &oci_spec::runtime::Spec,
        original_root_path: impl AsRef<Path>,
    ) -> Result<RuntimeEnv> {
        let has_kubernetes_secrets_dir = spec.mounts().iter().flatten().any(|m| {
            m.destination()
                .starts_with("/var/run/secrets/kubernetes.io")
        });

        let has_kubernetes_managed_etc_hosts = spec
            .mounts()
            .iter()
            .flatten()
            .filter(|m| m.destination() == Path::new("/etc/hosts"))
            .flat_map(|m| m.source())
            .next()
            .map(fs::read_to_string)
            .transpose()?
            .and_then(|hosts| hosts.lines().next().map(|line| line.to_string()))
            .map(|line| line.contains("Kubernetes-managed hosts file"))
            .unwrap_or(false);

        let has_dockerenv_dot_file = original_root_path
            .as_ref()
            .join(".dockerenv")
            .try_exists()?;

        if has_kubernetes_secrets_dir || has_kubernetes_managed_etc_hosts {
            Ok(RuntimeEnv::Kubernetes)
        } else if has_dockerenv_dot_file {
            Ok(RuntimeEnv::Docker)
        } else {
            Ok(RuntimeEnv::Other)
        }
    }
}
