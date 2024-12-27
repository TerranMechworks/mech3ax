#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod common;
pub mod mw;
pub mod pm;
pub mod rc;

use mech3ax_types::Hex;

const SIGNATURE: Hex<u32> = Hex(0x08170616);

const VERSION_RC: u32 = 28;
const VERSION_MW: u32 = 39;
const VERSION_PM: u32 = 50;

#[derive(Debug, Clone, Copy)]
pub enum SaveItem<'a> {
    AnimDef {
        name: &'a str,
        anim_def: &'a mech3ax_api_types::anim::AnimDef,
    },
    SiScript {
        name: &'a str,
        si_script: &'a mech3ax_api_types::anim::SiScript,
    },
}

impl<'a> SaveItem<'a> {
    pub fn name(self) -> &'a str {
        match self {
            Self::AnimDef { name, .. } => name,
            Self::SiScript { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LoadItemName<'a> {
    AnimDef(&'a str),
    SiScript(&'a str),
}

impl<'a> LoadItemName<'a> {
    pub fn name(self) -> &'a str {
        match self {
            Self::AnimDef(name) => name,
            Self::SiScript(name) => name,
        }
    }
}

#[derive(Debug)]
pub enum LoadItem {
    AnimDef(Box<mech3ax_api_types::anim::AnimDef>),
    SiScript(mech3ax_api_types::anim::SiScript),
}

impl LoadItem {
    pub fn anim_def(
        self,
        anim_def_name: &str,
    ) -> Result<mech3ax_api_types::anim::AnimDef, mech3ax_common::Error> {
        match self {
            LoadItem::AnimDef(anim_def) => Ok(*anim_def),
            LoadItem::SiScript(_) => Err(mech3ax_common::assert_with_msg!(
                "expected `{}` to be an anim def, but an si script was returned",
                anim_def_name
            )),
        }
    }

    pub fn si_script(
        self,
        file_name: &str,
    ) -> Result<mech3ax_api_types::anim::SiScript, mech3ax_common::Error> {
        match self {
            LoadItem::SiScript(si_script) => Ok(si_script),
            LoadItem::AnimDef(_) => Err(mech3ax_common::assert_with_msg!(
                "expected `{}` to be an si script, but an anim def was returned",
                file_name
            )),
        }
    }
}
