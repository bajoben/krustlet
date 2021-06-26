use std::path::Path;

use k8s_openapi::api::core::v1::{KeyToPath, Secret, Volume as KubeVolume};
use k8s_openapi::ByteString;
use tracing::warn;

use super::*;

/// A type that can manage a Secret volume with mounting and unmounting support
pub struct SecretVolume {
    vol_name: String,
    sec_name: String,
    client: kube::Api<Secret>,
    items: Option<Vec<KeyToPath>>,
    mounted_path: Option<PathBuf>,
}

impl SecretVolume {
    /// Creates a new Secret volume from a Kubernetes volume object. Passing a non-Secret volume
    /// type will result in an error
    pub fn new(vol: &KubeVolume, namespace: &str, client: kube::Client) -> anyhow::Result<Self> {
        let sec_source = vol.secret.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Called a Secret volume constructor with a non-Secret volume")
        })?;
        Ok(SecretVolume {
            vol_name: vol.name.clone(),
            sec_name: sec_source
                .secret_name
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Secret volume does not have a name"))?,
            client: Api::namespaced(client, namespace),
            items: sec_source.items.clone(),
            mounted_path: None,
        })
    }

    /// Returns the path where the volume is mounted on the host. Will return `None` if the volume
    /// hasn't been mounted yet
    pub fn get_path(&self) -> Option<&Path> {
        self.mounted_path.as_deref()
    }

    /// Mounts the Secret volume in the given directory. The actual path will be
    /// $BASE_PATH/$VOLUME_NAME
    pub async fn mount(&mut self, base_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let secret = self.client.get(&self.sec_name).await?;
        let path = base_path.as_ref().join(&self.vol_name);
        tokio::fs::create_dir_all(&path).await?;
        let data = secret.data.unwrap_or_default();
        // We could probably just move the data out of the option, but I don't know what the correct
        // behavior is from k8s point of view if something tries to mount a volume again
        let data = data
            .into_iter()
            .filter_map(
                |(key, ByteString(data))| match mount_setting_for(&key, &self.items) {
                    ItemMount::MountAt(mount_path) => Some((path.join(mount_path), data)),
                    ItemMount::DoNotMount => None,
                },
            )
            .map(|(file_path, data)| async move { tokio::fs::write(file_path, &data).await });
        futures::future::join_all(data)
            .await
            .into_iter()
            .collect::<tokio::io::Result<_>>()?;
        // Set secret directory to read-only.
        let mut perms = tokio::fs::metadata(&path).await?.permissions();
        perms.set_readonly(true);
        tokio::fs::set_permissions(&path, perms).await?;

        self.mounted_path = Some(path);

        Ok(())
    }

    /// Unmounts the directory, which removes all files. Calling `unmount` on a directory that
    /// hasn't been mounted will log a warning, but otherwise not error
    pub async fn unmount(&mut self) -> anyhow::Result<()> {
        match self.mounted_path.take() {
            Some(p) => {
                //although remove_dir_all crate could default to std::fs::remove_dir_all for unix family, we still prefer std::fs implemetation for unix
                #[cfg(target_family = "windows")]
                tokio::task::spawn_blocking(|| remove_dir_all::remove_dir_all(p)).await?;

                #[cfg(target_family = "unix")]
                tokio::fs::remove_dir_all(p).await?;
            }
            None => {
                warn!("Attempted to unmount ConfigMap directory that wasn't mounted, this generally shouldn't happen");
            }
        }
        Ok(())
    }
}
