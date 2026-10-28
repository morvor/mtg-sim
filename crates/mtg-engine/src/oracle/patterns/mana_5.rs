//! Mana grammar V (cards4-costs-mana-replacements): where the mana spent on a spell came
//! from, as a quality of the spells a static ability affects (CR 106.3, 601.2h).
//!
//! - "Each spell you cast has split second if mana from an artifact was spent to cast it."
//!   (Shadow the Hedgehog): the static ability's affected spells are and'ed with the
//!   quality "some of the mana spent to cast it came from an artifact"
//!   (`kw/cast_history.rs`, [`ch::SPENT_SOME`]). The spell has the keyword once that mana
//!   has been spent, i.e. from the end of its casting on (CR 601.2h).
//! - "The first spell you cast each turn that mana from a Treasure was spent to cast has
//!   cascade." (Rain of Riches): the first spell with that quality (earlier spells without
//!   it don't count, `kw/first_spell_each_turn.rs`).

use super::StaticPattern;
use crate::ability::*;
use crate::kw::cast_history as ch;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "mana from [kind]" → the `cast history:spent some:` filter.
fn spent_from(kind_phrase: &str) -> Option<Filter> {
    let kind = kind_phrase.strip_prefix("mana from ")?;
    let what = super::mana_4::mana_from_kind(kind)?;
    Some(Filter::Custom(format!("{}{what}", ch::SPENT_SOME).into()))
}

/// Adds `quality` to the spells a compiled static ability affects; refuses abilities
/// that don't affect spells.
fn add_spell_quality(abilities: &mut [Ability], quality: &Filter) -> Option<()> {
    if abilities.is_empty() {
        return None;
    }
    for a in abilities.iter_mut() {
        let AbilityKind::Static(s) = &mut std::sync::Arc::make_mut(a).kind else {
            return None;
        };
        let StaticEffect::Continuous {
            affected: Filter::And(parts),
            ..
        } = &mut s.effect
        else {
            return None;
        };
        if !parts.iter().any(|f| matches!(f, Filter::Spell)) {
            return None;
        }
        parts.push(quality.clone());
    }
    Some(())
}

/// "[spells] have [keyword] if mana from [kind] was spent to cast it" and "the first spell
/// you cast each turn that mana from [kind] was spent to cast has [keyword]".
fn spells_with_mana_from(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (head, quality) = if let Some((head, cond)) = l.rsplit_once(" if ") {
        let kind = cond
            .strip_suffix(" was spent to cast it")
            .or_else(|| cond.strip_suffix(" was spent to cast them"))?;
        (head.to_string(), spent_from(kind)?)
    } else {
        let r = l.strip_prefix("the first spell you cast each turn that ")?;
        let (kind, kw) = r.split_once(" was spent to cast has ")?;
        (
            format!("the first spell you cast each turn has {kw}"),
            spent_from(kind)?,
        )
    };
    let mut abilities = crate::oracle::statics::parse_static(&head, ctx)?;
    add_spell_quality(&mut abilities, &quality)?;
    for a in abilities.iter_mut() {
        std::sync::Arc::make_mut(a).text = text.into();
    }
    Some(abilities)
}

inventory::submit! { StaticPattern { name: "mana 5: spells have [keyword] if mana from [kind] was spent to cast it", priority: 100, parse: spells_with_mana_from } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Layout;
    use crate::types::TypeLine;

    fn with_ctx<T>(f: impl FnOnce(&CompileContext) -> T) -> T {
        let tl = TypeLine::parse("Enchantment");
        let ctx = CompileContext {
            card_name: "X",
            full_name: "X",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        f(&ctx)
    }

    #[test]
    fn spell_quality_wordings() {
        with_ctx(|ctx| spell_quality_wordings_in(ctx));
    }

    fn spell_quality_wordings_in(ctx: &CompileContext) {
        for l in [
            "each spell you cast has split second if mana from an artifact was spent to cast it",
            "the first spell you cast each turn that mana from a treasure was spent to cast has cascade",
            "each creature spell you cast has flying if mana from a creature was spent to cast it",
        ] {
            let a = spells_with_mana_from(l, l, ctx).unwrap_or_else(|| panic!("{l}"));
            let s = format!("{a:?}");
            assert!(s.contains("spent some:source"), "{l}: {s}");
        }
        // Not a spell group: refused.
        assert!(spells_with_mana_from(
            "creatures you control have flying if mana from a treasure was spent to cast it",
            "",
            ctx
        )
        .is_none());
    }
}
