use crate::{LoadItem, LoadItemName, SaveItem};
use log::debug;
use mech3ax_api_types::anim::SiScript;
use mech3ax_common::Error;
use std::convert::From;

pub(crate) fn save_anim_scripts<F, E>(
    scripts: Vec<SiScript>,
    mut save_item: F,
) -> std::result::Result<Vec<String>, E>
where
    F: FnMut(SaveItem<'_>) -> std::result::Result<(), E>,
    E: From<std::io::Error> + From<Error>,
{
    scripts
        .into_iter()
        .enumerate()
        .map(|(index, si_script)| {
            let file_name = format!("si-script-{:03}.zan", index);

            debug!("Saving anim script {}: `{}`", index, file_name);
            let item = SaveItem::SiScript {
                name: &file_name,
                si_script: &si_script,
            };
            save_item(item)?;

            Ok(file_name)
        })
        .collect()
}

pub(crate) fn load_anim_scripts<F, E>(
    script_names: &[String],
    mut load_item: F,
) -> std::result::Result<Vec<SiScript>, E>
where
    F: FnMut(LoadItemName<'_>) -> std::result::Result<LoadItem, E>,
    E: From<std::io::Error> + From<Error>,
{
    script_names
        .iter()
        .enumerate()
        .map(|(index, file_name)| {
            debug!("Loading anim def {}: `{}`", index, file_name);
            let item_name = LoadItemName::SiScript(file_name);
            Ok(load_item(item_name)?.si_script(file_name)?)
        })
        .collect()
}
