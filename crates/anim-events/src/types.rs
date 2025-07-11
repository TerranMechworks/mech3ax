use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_types::maybe::{Maybe, PrimitiveRepr, SupportsMaybe};
use std::fmt;

pub(crate) const INPUT_NODE_NAME: &str = "INPUT_NODE";

macro_rules! index {
    (input) => {
        ::mech3ax_types::maybe::Maybe::new(-200)
    };
    ($v:literal) => {
        ::mech3ax_types::maybe::Maybe::new($v)
    };
}
pub(crate) use index;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[repr(transparent)]
pub(crate) struct Index(pub(crate) i16);

impl Index {
    const fn to_usize(self) -> Option<usize> {
        let Self(v) = self;
        if v < 0 {
            None
        } else {
            Some(v as _)
        }
    }

    const fn from_usize(value: usize) -> Option<Self> {
        const MAX: usize = i16::MAX as _;
        if value > MAX {
            None
        } else {
            Some(Self(value as _))
        }
    }
}

impl fmt::Display for Index {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub(crate) type Idx16 = Maybe<i16, Index>;
pub(crate) type Idx32 = Maybe<i32, Index>;

impl From<Index> for Idx16 {
    #[inline]
    fn from(value: Index) -> Self {
        Self::new(value.0)
    }
}

impl From<Index> for Idx32 {
    #[inline]
    fn from(value: Index) -> Self {
        Self::new(value.0 as _)
    }
}

impl SupportsMaybe<i16> for Index {
    #[inline]
    fn from_bits(v: i16) -> Option<Self> {
        Some(Self(v))
    }

    #[inline]
    fn fmt_value(v: i16, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <i16 as fmt::Debug>::fmt(&v, f)
    }

    #[inline]
    fn maybe(self) -> Maybe<i16, Self> {
        Maybe::new(self.0)
    }

    #[inline]
    fn check(v: i16) -> Result<Self, String> {
        Ok(Self(v))
    }
}

impl SupportsMaybe<i32> for Index {
    #[inline]
    fn from_bits(v: i32) -> Option<Self> {
        const MIN: i32 = i16::MIN as _;
        const MAX: i32 = i16::MAX as _;
        if !(MIN..=MAX).contains(&v) {
            None
        } else {
            Some(Self(v as i16))
        }
    }

    #[inline]
    fn fmt_value(v: i32, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <i32 as fmt::Debug>::fmt(&v, f)
    }

    #[inline]
    fn maybe(self) -> Maybe<i32, Self> {
        Maybe::new(self.0 as i32)
    }

    #[inline]
    fn check(v: i32) -> Result<Self, String> {
        Self::from_bits(v).ok_or_else(|| format!("expected {} in {}..={}", v, i16::MIN, i16::MAX))
    }
}

fn node_from_index(anim_def: &AnimDef, index: Index, offset: usize) -> Result<String> {
    let index = index
        .to_usize()
        .ok_or_else(|| assert_with_msg!("Node index {} is negative (at {})", index, offset))?;
    let nodes = anim_def.nodes.as_ref().ok_or_else(|| {
        assert_with_msg!(
            "Tried to look up node {}, but anim def has no nodes (at {})",
            index,
            offset
        )
    })?;
    assert_that!("node index", 1 <= index <= nodes.len(), offset)?;
    Ok(nodes[index - 1].name.clone())
}

fn node_to_index(anim_def: &AnimDef, name: &str) -> Result<Index> {
    anim_def
        .nodes
        .as_ref()
        .ok_or_else(|| {
            assert_with_msg!("Tried to find node `{}`, but anim def has no nodes", name)
        })?
        .iter()
        .position(|node| node.name == name)
        .map(|pos| pos + 1)
        .ok_or_else(|| assert_with_msg!("Expected to find node `{}`, but didn't", name))
        .and_then(|value| {
            Index::from_usize(value).ok_or_else(|| assert_with_msg!("Too many nodes in anim def"))
        })
}

fn light_from_index(anim_def: &AnimDef, index: Index, offset: usize) -> Result<String> {
    let index = index
        .to_usize()
        .ok_or_else(|| assert_with_msg!("Light index {} is negative (at {})", index, offset))?;

    let lights = anim_def.lights.as_ref().ok_or_else(|| {
        assert_with_msg!(
            "Tried to look up light {}, but anim def has no lights (at {})",
            index,
            offset
        )
    })?;
    assert_that!("light index", 1 <= index <= lights.len(), offset)?;
    Ok(lights[index - 1].name.clone())
}

fn light_to_index(anim_def: &AnimDef, name: &str) -> Result<Index> {
    anim_def
        .lights
        .as_ref()
        .ok_or_else(|| {
            assert_with_msg!("Tried to find light `{}`, but anim def has no lights", name)
        })?
        .iter()
        .position(|light| light.name == name)
        .map(|pos| pos + 1)
        .ok_or_else(|| assert_with_msg!("Expected to find light `{}`, but didn't", name))
        .and_then(|value| {
            Index::from_usize(value).ok_or_else(|| assert_with_msg!("Too many lights in anim def"))
        })
}

fn puffer_from_index(anim_def: &AnimDef, index: Index, offset: usize) -> Result<String> {
    let index = index
        .to_usize()
        .ok_or_else(|| assert_with_msg!("Puffer index {} is negative (at {})", index, offset))?;

    let puffers = anim_def.puffers.as_ref().ok_or_else(|| {
        assert_with_msg!(
            "Tried to look up puffer {}, but anim def has no puffers (at {})",
            index,
            offset
        )
    })?;
    assert_that!("puffer index", 1 <= index <= puffers.len(), offset)?;
    Ok(puffers[index - 1].name.clone())
}

fn puffer_to_index(anim_def: &AnimDef, name: &str) -> Result<Index> {
    anim_def
        .puffers
        .as_ref()
        .ok_or_else(|| {
            assert_with_msg!(
                "Tried to find puffer `{}`, but anim def has no puffers",
                name
            )
        })?
        .iter()
        .position(|puffer| puffer.name == name)
        .map(|pos| pos + 1)
        .ok_or_else(|| assert_with_msg!("Expected to find puffer `{}`, but didn't", name))
        .and_then(|value| {
            Index::from_usize(value).ok_or_else(|| assert_with_msg!("Too many puffers in anim def"))
        })
}

fn dyn_sound_from_index(anim_def: &AnimDef, index: Index, offset: usize) -> Result<String> {
    let index = index.to_usize().ok_or_else(|| {
        assert_with_msg!("Sound node index {} is negative (at {})", index, offset)
    })?;

    let sounds = anim_def.dynamic_sounds.as_ref().ok_or_else(|| {
        assert_with_msg!(
            "Tried to look up sound node {}, but anim def has no sound nodes (at {})",
            index,
            offset
        )
    })?;
    assert_that!("sound node index", 1 <= index <= sounds.len(), offset)?;
    Ok(sounds[index - 1].name.clone())
}

fn dyn_sound_to_index(anim_def: &AnimDef, name: &str) -> Result<Index> {
    anim_def
        .dynamic_sounds
        .as_ref()
        .ok_or_else(|| {
            assert_with_msg!(
                "Tried to find sound node `{}`, but anim def has no sound nodes",
                name
            )
        })?
        .iter()
        .position(|sound| sound.name == name)
        .map(|pos| pos + 1)
        .ok_or_else(|| assert_with_msg!("Expected to find sound node `{}`, but didn't", name))
        .and_then(|value| {
            Index::from_usize(value)
                .ok_or_else(|| assert_with_msg!("Too many sound nodes in anim def"))
        })
}

fn stc_sound_from_index(anim_def: &AnimDef, index: Index, offset: usize) -> Result<String> {
    let index = index.to_usize().ok_or_else(|| {
        assert_with_msg!("Static sound index {} is negative (at {})", index, offset)
    })?;

    let sounds = anim_def.static_sounds.as_ref().ok_or_else(|| {
        assert_with_msg!(
            "Tried to look up static sound {}, but anim def has no static sounds (at {})",
            index,
            offset
        )
    })?;
    assert_that!("static sound index", 1 <= index <= sounds.len(), offset)?;
    Ok(sounds[index - 1].name.clone())
}

fn std_sound_to_index(anim_def: &AnimDef, name: &str) -> Result<Index> {
    anim_def
        .static_sounds
        .as_ref()
        .ok_or_else(|| {
            assert_with_msg!(
                "Tried to find static sound `{}`, but anim def has no static sounds",
                name
            )
        })?
        .iter()
        .position(|sound| sound.name == name)
        .map(|pos| pos + 1)
        .ok_or_else(|| assert_with_msg!("Expected to find static sound `{}`, but didn't", name))
        .and_then(|value| {
            Index::from_usize(value)
                .ok_or_else(|| assert_with_msg!("Too many static sounds in anim def"))
        })
}

fn effect_from_index(anim_def: &AnimDef, index: Index, offset: usize) -> Result<String> {
    let index = index
        .to_usize()
        .ok_or_else(|| assert_with_msg!("Effect index {} is negative (at {})", index, offset))?;
    let effects = anim_def.effects.as_ref().ok_or_else(|| {
        assert_with_msg!(
            "Tried to look up effect {}, but anim def has no effects (at {})",
            index,
            offset
        )
    })?;
    assert_that!("effect index", 1 <= index <= effects.len(), offset)?;
    Ok(effects[index - 1].name.clone())
}

fn effect_to_index(anim_def: &AnimDef, name: &str) -> Result<Index> {
    anim_def
        .effects
        .as_ref()
        .ok_or_else(|| {
            assert_with_msg!(
                "Tried to find effect `{}`, but anim def has no effects",
                name
            )
        })?
        .iter()
        .position(|effect| effect.name == name)
        .map(|pos| pos + 1)
        .ok_or_else(|| assert_with_msg!("Expected to find effect `{}`, but didn't", name))
        .and_then(|value| {
            Index::from_usize(value).ok_or_else(|| assert_with_msg!("Too many effects in anim def"))
        })
}

fn anim_ref_validate_index(anim_def: &AnimDef, index: i16, offset: usize) -> Result<i16> {
    let i = Index(index)
        .to_usize()
        .ok_or_else(|| assert_with_msg!("Anim ref index {} is negative (at {})", index, offset))?;
    let anim_refs = anim_def.anim_refs.as_ref().ok_or_else(|| {
        assert_with_msg!(
            "Tried to look up anim ref {}, but anim def has no anim refs (at {})",
            index,
            offset
        )
    })?;
    assert_that!("anim ref index", i < anim_refs.len(), offset)?;
    Ok(index)
}

pub(crate) trait AnimDefLookup {
    fn node_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>;
    fn node_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>;

    fn light_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>;
    fn light_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>;

    fn puffer_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>;
    fn puffer_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>;

    fn dyn_sound_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>;
    fn dyn_sound_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>;

    fn stc_sound_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>;
    fn stc_sound_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>;

    fn effect_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>;
    fn effect_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>;

    fn anim_ref_from_index(&self, index: Idx16, offset: usize) -> Result<i16>;
    fn anim_ref_to_index(&self, index: i16) -> Result<Idx16>;
}

impl AnimDefLookup for AnimDef {
    fn node_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>,
    {
        Index::from_bits(index.value)
            .ok_or_else(|| assert_with_msg!("Node index {} is out of range (at {})", index, offset))
            .and_then(|index| node_from_index(self, index, offset))
    }

    fn node_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>,
    {
        node_to_index(self, name).map(Index::into)
    }

    fn light_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>,
    {
        Index::from_bits(index.value)
            .ok_or_else(|| {
                assert_with_msg!("Light index {} is out of range (at {})", index, offset)
            })
            .and_then(|index| light_from_index(self, index, offset))
    }

    fn light_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>,
    {
        light_to_index(self, name).map(Index::into)
    }

    fn puffer_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>,
    {
        Index::from_bits(index.value)
            .ok_or_else(|| {
                assert_with_msg!("Puffer index {} is out of range (at {})", index, offset)
            })
            .and_then(|index| puffer_from_index(self, index, offset))
    }

    fn puffer_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>,
    {
        puffer_to_index(self, name).map(Index::into)
    }

    fn dyn_sound_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>,
    {
        Index::from_bits(index.value)
            .ok_or_else(|| {
                assert_with_msg!("Sound node index {} is out of range (at {})", index, offset)
            })
            .and_then(|index| dyn_sound_from_index(self, index, offset))
    }

    fn dyn_sound_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>,
    {
        dyn_sound_to_index(self, name).map(Index::into)
    }

    fn stc_sound_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>,
    {
        Index::from_bits(index.value)
            .ok_or_else(|| {
                assert_with_msg!(
                    "Static sound index {} is out of range (at {})",
                    index,
                    offset
                )
            })
            .and_then(|index| stc_sound_from_index(self, index, offset))
    }

    fn stc_sound_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>,
    {
        std_sound_to_index(self, name).map(Index::into)
    }

    fn effect_from_index<R>(&self, index: Maybe<R, Index>, offset: usize) -> Result<String>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R>,
    {
        Index::from_bits(index.value)
            .ok_or_else(|| {
                assert_with_msg!("Effect index {} is out of range (at {})", index, offset)
            })
            .and_then(|index| effect_from_index(self, index, offset))
    }

    fn effect_to_index<R>(&self, name: &str) -> Result<Maybe<R, Index>>
    where
        R: PrimitiveRepr,
        Index: SupportsMaybe<R> + Into<Maybe<R, Index>>,
    {
        effect_to_index(self, name).map(Index::into)
    }

    fn anim_ref_from_index(&self, index: Idx16, offset: usize) -> Result<i16> {
        anim_ref_validate_index(self, index.value, offset)
    }

    fn anim_ref_to_index(&self, index: i16) -> Result<Idx16> {
        anim_ref_validate_index(self, index, 0).map(Idx16::new)
    }
}
