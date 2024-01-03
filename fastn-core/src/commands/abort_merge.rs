pub async fn abort_merge(config: &fastn_core::Config, path: &str) -> fastn_core::Result<()> {
    use itertools::Itertools;

    let mut workspaces = fastn_core::snapshot::get_workspace(config).await?;
    if let Some(workspace) = workspaces.get_mut(path) {
        if workspace
            .workspace
            .eq(&fastn_core::snapshot::WorkspaceType::CloneDeletedRemoteEdited)
        {
            if config.ds.root().join(path).exists() {
                config.ds.remove(&config.ds.root().join(path)).await?;
            }
        } else {
            config
                .ds
                .copy(
                    &config.conflicted_dir().join(path),
                    &config.ds.root().join(path),
                )
                .await?;
        }
        workspace.set_abort();
    }
    fastn_core::snapshot::create_workspace(
        config,
        workspaces.into_values().collect_vec().as_slice(),
    )
    .await?;

    Ok(())
}
