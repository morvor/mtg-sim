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
//!
//! Mana production (CR 106, 605):
//!
//! - "Add {W}{W}, {U}{U}, {B}{B}, {R}{R}, and {G}{G}." (Esper Terra): a list of mana.
//! - "If ~ has a +1/+1 counter on it, add three mana of that type instead." after "Add one
//!   mana of any type that a land you control could produce" (Incubation Druid): three mana
//!   of one type such a land could produce ([`ManaProduction::CouldProduceAmount`]).
//! - "~ becomes that color until end of turn." after "Add one mana of any color" (Foraging
//!   Wickermaw): the color of the mana just added (`kw/mana_ability_added.rs`).
//! - "Whenever a land's ability causes you to add one or more mana of the chosen color"
//!   (Caged Sun): a triggered mana ability on an activated mana ability of a land
//!   resolving (CR 605.1b), whose mana included the chosen color.
//! - "add an additional one mana of any type that land produced for each other land you
//!   control with the same name as it" (Sasaya's Essence): one mana for each, each of a
//!   type that land produced (chosen separately).
//! - "The next spell they cast this turn has cascade." (Bigger on the Inside's granted
//!   ability, after "Target player adds ..."): the effect waits for that player's spell.

use super::{EffectPattern, FollowupPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::kw::cast_history as ch;
use crate::oracle::effects::Builder;
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

/// "add {W}{W}, {U}{U}, {B}{B}, {R}{R}, and {G}{G}": the listed mana, all of it.
fn add_mana_list(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("add ")?;
    if !r.contains(", ") {
        return None;
    }
    let mut syms = String::new();
    let mut n = 0;
    for part in r.split(", ") {
        let part = part.strip_prefix("and ").unwrap_or(part).trim();
        let only_symbols = part.starts_with('{')
            && part.ends_with('}')
            && part
                .trim_start_matches('{')
                .trim_end_matches('}')
                .split("}{")
                .all(|x| matches!(x, "w" | "u" | "b" | "r" | "g" | "c"));
        if !only_symbols {
            return None;
        }
        syms.push_str(part);
        n += 1;
    }
    if n < 3 {
        return None;
    }
    match crate::oracle::effects::parse_clause(&format!("add {syms}"), b)? {
        e @ Effect::AddMana {
            mana: ManaProduction::Fixed(_),
            ..
        } => Some(e),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "mana 5: add {W}{W}, {U}{U}, and {B}{B}", priority: 100, parse: add_mana_list } }

/// "If [condition], add [N] mana of that type instead." after "Add one mana of any type
/// that [a permanent] could produce": N mana of one type such a permanent could produce.
fn that_type_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Effect::AddMana {
        who,
        mana: ManaProduction::CouldProduce(f),
        restriction,
    } = &*prev
    else {
        return false;
    };
    let Some(r) = end(l).strip_prefix("if ") else {
        return false;
    };
    let Some((c, act)) = r.split_once(", add ") else {
        return false;
    };
    let Some(n) = act.strip_suffix(" mana of that type instead") else {
        return false;
    };
    let Some((n, rest)) = crate::oracle::phrases::parse_number(n) else {
        return false;
    };
    if !rest.trim().is_empty() {
        return false;
    }
    let Some(cond) = super::conditions_referents::parse_condition_with(c, b) else {
        return false;
    };
    let then = Effect::AddMana {
        who: who.clone(),
        mana: ManaProduction::CouldProduceAmount(f.clone(), n),
        restriction: restriction.clone(),
    };
    *prev = Effect::If {
        cond,
        then: Box::new(then),
        otherwise: Box::new(prev.clone()),
    };
    true
}

inventory::submit! { FollowupPattern { name: "mana 5: if [condition], add N mana of that type instead", priority: 90, apply: that_type_instead } }

/// "~ becomes that color until end of turn." after "Add one mana of any color".
fn becomes_color_added(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(
        end(l),
        "~ becomes that color until end of turn"
            | "this creature becomes that color until end of turn"
    ) {
        return false;
    }
    if !matches!(
        prev,
        Effect::AddMana {
            mana: ManaProduction::AnyOneColor(Value::Const(1)),
            ..
        }
    ) {
        return false;
    }
    let modify = Effect::Modify {
        what: Sel::This,
        mods: vec![Modification::Custom {
            name: crate::kw::mana_ability_added::BECOMES_COLOR_ADDED.into(),
            layer: Layer::L5Color,
        }],
        duration: Duration::EndOfTurn,
    };
    *prev = Effect::seq(vec![prev.clone(), modify]);
    true
}

inventory::submit! { FollowupPattern { name: "mana 5: ~ becomes that color (the mana added)", priority: 90, apply: becomes_color_added } }

/// "Whenever a land's ability causes you to add one or more mana of the chosen color" /
/// "... causes a player to add one or more mana".
fn ability_causes_add(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = t.strip_prefix("a ").or_else(|| t.strip_prefix("an "))?;
    let (ty, r) = r.split_once("'s ability causes ")?;
    let ty = crate::types::CardType::from_word(ty)?;
    let (you, r) = if let Some(r) = r.strip_prefix("you to add one or more mana") {
        (true, r)
    } else {
        (false, r.strip_prefix("a player to add one or more mana")?)
    };
    let trigger = TriggerCond::Custom(crate::kw::mana_ability_added::trigger_name(ty, you).into());
    let trigger = match r {
        "" => trigger,
        " of the chosen color" => TriggerCond::Where {
            trigger: Box::new(trigger),
            cond: Condition::Custom(
                crate::kw::mana_produced_chosen_color::PRODUCED_CHOSEN_COLOR.into(),
            ),
        },
        _ => return None,
    };
    Some((trigger, Sel::TriggerObject, PlayerRef::TriggerPlayer))
}

inventory::submit! { TriggerPattern { name: "mana 5: whenever a [type]'s ability causes you to add one or more mana", priority: 100, parse: ability_causes_add } }

/// "add an additional one mana of that color" after "causes you to add one or more mana of
/// the chosen color": the chosen color (CR 607.2d), added to you.
fn add_one_of_that_color(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger
        || end(l) != "add an additional one mana of that color"
        || !crate::oracle::raw_text()
            .to_lowercase()
            .contains("causes you to add one or more mana of the chosen color, add an additional")
    {
        return None;
    }
    Some(Effect::AddMana {
        who: PlayerRef::You,
        mana: ManaProduction::ChosenColor(Value::c(1)),
        restriction: None,
    })
}

inventory::submit! { EffectPattern { name: "mana 5: add an additional one mana of that color (the chosen color)", priority: 100, parse: add_one_of_that_color } }

/// "add [an additional] one mana of any type that land produced for each [objects]": that
/// many mana, each of a type the land produced (chosen separately).
fn add_produced_for_each(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("add an additional one mana of any type that land produced for each ")
        .or_else(|| end(l).strip_prefix("add one mana of any type that land produced for each "))?;
    let saved = b.targets.len();
    let (v, tail) = crate::oracle::statics::parse_value_phrase(&format!("the number of {r}"), b)?;
    if !tail.trim().is_empty() || b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::Repeat {
        times: v,
        effect: Box::new(Effect::AddMana {
            who: PlayerRef::You,
            mana: ManaProduction::AnyTypeProduced,
            restriction: None,
        }),
    })
}

inventory::submit! { EffectPattern { name: "mana 5: add one mana of any type that land produced for each [count]", priority: 100, parse: add_produced_for_each } }

/// "the next spell they cast this turn [has ...]": the instruction for the player "they"
/// refers to.
fn next_spell_they_cast(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.starts_with("the next ") || !l.contains(" they cast this turn ") {
        return None;
    }
    // "they": the player the previous instruction named ("Target player adds ...").
    if matches!(b.it_player, PlayerRef::You)
        || super::oracle_hardening_referents::is_no_player_referent(&b.it_player)
    {
        return None;
    }
    let who = b.it_player.clone();
    let inner =
        crate::oracle::effects::parse_clause(&l.replacen(" they cast ", " you cast ", 1), b)?;
    if !matches!(inner, Effect::NextSpell { .. }) {
        return None;
    }
    Some(Effect::AsPlayer {
        who,
        effect: Box::new(inner),
    })
}

inventory::submit! { EffectPattern { name: "mana 5: the next spell they cast this turn has [ability]", priority: 100, parse: next_spell_they_cast } }

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
