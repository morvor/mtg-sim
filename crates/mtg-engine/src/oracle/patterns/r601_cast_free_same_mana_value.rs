//! "... If you do, you may cast that card without paying its mana cost if the two spells
//! have the same mana value." (Powerbalance, after revealing the top card of your library
//! in a "whenever an opponent casts a spell" trigger). See
//! `kw/cast_free_same_mana_value.rs`.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::cast_free_same_mana_value::CAST_IT_FREE_IF_SAME_MANA_VALUE;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn cast_free_same_mana_value(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l.trim())
        != "you may cast that card without paying its mana cost if the two spells have the same mana value"
    {
        return None;
    }
    if !matches!(b.it, Sel::Var(vars::IT)) {
        return None;
    }
    Some(Effect::Custom(CAST_IT_FREE_IF_SAME_MANA_VALUE.into()))
}

inventory::submit! { EffectPattern { name: "r601 cast that card free if the two spells have the same mana value", priority: 100, parse: cast_free_same_mana_value } }
