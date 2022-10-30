use mech3ax_api_types::AnimDef;
use mech3ax_common::assert::AssertionError;
use mech3ax_common::{assert_that, Result};

pub trait AnimDefLookup {
    fn node_from_index(&self, index: usize, offset: u32) -> Result<String>;
    fn node_to_index(&self, name: &str) -> Result<usize>;
    fn sound_from_index(&self, index: usize, offset: u32) -> Result<String>;
    fn sound_to_index(&self, name: &str) -> Result<usize>;
    fn light_from_index(&self, index: usize, offset: u32) -> Result<String>;
    fn light_to_index(&self, name: &str) -> Result<usize>;
    fn puffer_from_index(&self, index: usize, offset: u32) -> Result<String>;
    fn puffer_to_index(&self, name: &str) -> Result<usize>;
}

impl AnimDefLookup for AnimDef {
    fn node_from_index(&self, index: usize, offset: u32) -> Result<String> {
        if let Some(nodes) = &self.nodes {
            assert_that!("node index", 1 <= index <= nodes.len(), offset)?;
            Ok(nodes[index - 1].name.clone())
        } else {
            let msg = format!(
                "Tried to look up node {}, but anim def has no nodes (at {})",
                index, offset
            );
            Err(AssertionError(msg).into())
        }
    }

    fn node_to_index(&self, name: &str) -> Result<usize> {
        if let Some(nodes) = &self.nodes {
            nodes
                .iter()
                .position(|node| node.name == name)
                .map(|pos| pos + 1)
                .ok_or_else(|| {
                    AssertionError(format!("Expected to find node `{}`, but didn't", name)).into()
                })
        } else {
            let msg = format!(
                "Expected to find node `{}`, but anim def has no nodes",
                name
            );
            Err(AssertionError(msg).into())
        }
    }

    fn sound_from_index(&self, index: usize, offset: u32) -> Result<String> {
        if let Some(sounds) = &self.static_sounds {
            assert_that!("sound index", 1 <= index <= sounds.len(), offset)?;
            Ok(sounds[index - 1].name.clone())
        } else {
            let msg = format!(
                "Tried to look up sound {}, but anim def has no sounds (at {})",
                index, offset
            );
            Err(AssertionError(msg).into())
        }
    }

    fn sound_to_index(&self, name: &str) -> Result<usize> {
        if let Some(sounds) = &self.static_sounds {
            sounds
                .iter()
                .position(|sound| sound.name == name)
                .map(|pos| pos + 1)
                .ok_or_else(|| {
                    AssertionError(format!("Expected to find sound `{}`, but didn't", name)).into()
                })
        } else {
            let msg = format!(
                "Expected to find sound `{}`, but anim def has no sounds",
                name
            );
            Err(AssertionError(msg).into())
        }
    }

    fn light_from_index(&self, index: usize, offset: u32) -> Result<String> {
        if let Some(lights) = &self.lights {
            assert_that!("light index", 1 <= index <= lights.len(), offset)?;
            Ok(lights[index - 1].name.clone())
        } else {
            let msg = format!(
                "Tried to look up light {}, but anim def has no lights (at {})",
                index, offset
            );
            Err(AssertionError(msg).into())
        }
    }

    fn light_to_index(&self, name: &str) -> Result<usize> {
        if let Some(lights) = &self.lights {
            lights
                .iter()
                .position(|light| light.name == name)
                .map(|pos| pos + 1)
                .ok_or_else(|| {
                    AssertionError(format!("Expected to find light `{}`, but didn't", name)).into()
                })
        } else {
            let msg = format!(
                "Expected to find light `{}`, but anim def has no lights",
                name
            );
            Err(AssertionError(msg).into())
        }
    }

    fn puffer_from_index(&self, index: usize, offset: u32) -> Result<String> {
        if let Some(puffers) = &self.puffers {
            assert_that!("puffer index", 1 <= index <= puffers.len(), offset)?;
            Ok(puffers[index - 1].name.clone())
        } else {
            let msg = format!(
                "Tried to look up puffer {}, but anim def has no puffers (at {})",
                index, offset
            );
            Err(AssertionError(msg).into())
        }
    }

    fn puffer_to_index(&self, name: &str) -> Result<usize> {
        if let Some(puffers) = &self.puffers {
            puffers
                .iter()
                .position(|puffer| puffer.name == name)
                .map(|pos| pos + 1)
                .ok_or_else(|| {
                    AssertionError(format!("Expected to find puffer `{}`, but didn't", name)).into()
                })
        } else {
            let msg = format!(
                "Expected to find puffer `{}`, but anim def has no puffers",
                name
            );
            Err(AssertionError(msg).into())
        }
    }
}
