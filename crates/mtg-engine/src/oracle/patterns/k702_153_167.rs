//! Oracle text of the keywords of CR 702.153–702.167 that the generic keyword parser
//! doesn't handle, and phrases that go with them (casualty granted to each spell of a
//! quality, backup and bargain have their own files, `k702_153_casualty.rs`,
//! `k702_165_backup.rs` and `k702_166_bargain.rs`):
//!
//! * "The first [quality] spell you cast each turn has [keyword]" (e.g. casualty, see
//!   `kw/first_spell_each_turn.rs`);
//! * "Whenever ~ enlists a creature", "if it enlisted a creature this combat" (and
//!   "the creature that attacked") (CR 702.154c);
//! * "[action on] each creature in the sector of your choice" (CR 702.158d) and
//!   "Creatures in each sector can be blocked this turn only by creatures in the same
//!   sector" (CR 702.158e);
//! * an Attraction's "Prize — [effect]" paragraph, "claim the prize", and "Whenever you
//!   claim the prize of an Attraction" (CR 702.159b);
//! * "Craft with [materials] [cost]" (CR 702.167a), and "the exiled cards used to craft
//!   it" (CR 702.167c).

use super::{
    AbilityPattern, BlockGroupPattern, ConditionPattern, EffectPattern, StaticPattern,
    TriggerPattern,
};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// "Whenever ~ enlists a creature" (Guardian of New Benalia; CR 702.154c).
fn enlists(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(r) == "~ enlists a creature").then(|| {
        (
            TriggerCond::Custom(crate::kw::enlist::ENLISTS.into()),
            Sel::TriggerObject,
            PlayerRef::You,
        )
    })
}

inventory::submit! { TriggerPattern { name: "k702.154 ~ enlists a creature", priority: 100, parse: enlists } }

/// "if it enlisted a creature this combat" (Aradesh, the Founder; CR 702.154c): the
/// creature of the triggering event enlisted a creature this combat.
fn enlisted_this_combat(c: &str) -> Option<Condition> {
    (end(c) == "it enlisted a creature this combat").then(|| {
        Condition::Custom(crate::kw::enlist::ENLISTED_THIS_COMBAT.into())
    })
}

inventory::submit! { ConditionPattern { name: "k702.154 it enlisted a creature this combat", priority: 100, parse: enlisted_this_combat } }

/// "Whenever a creature you control attacks, if it enlisted a creature this combat, the
/// creature that attacked [effect]. If that creature's power is N or greater, [effect]."
/// (Aradesh, the Founder): the intervening "if" is about the attacking creature of the
/// trigger, as are "the creature that attacked" and "that creature".
fn enlisted_attacker_trigger(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    const COND: &str = ", if it enlisted a creature this combat, the creature that attacked ";
    let (head, tail) = block.trim().split_once(COND)?;
    let (first, second) = tail.split_once(". If that creature's power is ")?;
    let (n, rest) = second.split_once(" or greater, ")?;
    let n: i32 = n.trim().parse().ok()?;
    let parse_one = |text: String| -> Option<TriggeredAbility> {
        let v = crate::oracle::parse_ability(&text, ctx)?;
        match v.as_slice() {
            [a] => match &a.kind {
                AbilityKind::Triggered(t) => Some(t.clone()),
                _ => None,
            },
            _ => None,
        }
    };
    let mut t = parse_one(format!("{head}, that creature {first}."))?;
    let then = parse_one(format!("{head}, {rest}"))?;
    if !then.body.targets.is_empty() || then.intervening_if.is_some() {
        return None;
    }
    t.intervening_if = Some(Condition::Custom(
        crate::kw::enlist::ENLISTED_THIS_COMBAT.into(),
    ));
    let first_effect = std::mem::replace(&mut t.body.effect, Effect::Noop);
    t.body.effect = Effect::Seq(vec![
        first_effect,
        Effect::If {
            cond: Condition::Compare(
                Value::PowerOf(Box::new(Sel::TriggerObject)),
                Cmp::Ge,
                Value::c(n),
            ),
            then: Box::new(then.body.effect),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    Some(vec![AbilityDef::new(AbilityKind::Triggered(t), block.trim())])
}

inventory::submit! { AbilityPattern { name: "k702.154 the creature that attacked", priority: 100, parse: enlisted_attacker_trigger } }

/// "[action on] each creature in the sector of your choice", "destroy all creatures in the
/// sector of your choice" (Space Beleren; CR 702.158d): choose one of the three sector
/// designations, then perform the action on each creature with it.
fn in_sector_of_your_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    const PHRASE: &str = " in the sector of your choice";
    if !l.contains(PHRASE) {
        return None;
    }
    let rewritten = l.replacen(PHRASE, " in the chosen sector", 1);
    let e = crate::oracle::effects::parse_clause(&rewritten, b)?;
    Some(Effect::Seq(vec![
        Effect::Custom(crate::kw::space_sculptor::CHOOSE_SECTOR.into()),
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "k702.158 in the sector of your choice", priority: 100, parse: in_sector_of_your_choice } }

/// "Creatures in each sector can be blocked this turn only by creatures in the same
/// sector." (Space Beleren; CR 702.158e).
fn same_sector_blocking(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l)
        == "creatures in each sector can be blocked this turn only by creatures in the same sector")
        .then(|| Effect::Custom(crate::kw::space_sculptor::SAME_SECTOR_BLOCKING.into()))
}

inventory::submit! { EffectPattern { name: "k702.158 blocked only by creatures in the same sector", priority: 100, parse: same_sector_blocking } }

/// Marks an Attraction's prize paragraph during grouping ("Prize" looks like an ability
/// word).
const PRIZE_MARK: &str = "{PRIZE} ";

fn mark_prizes(blocks: Vec<String>, ctx: &CompileContext) -> Vec<String> {
    let attraction = ctx
        .type_line
        .subtypes
        .iter()
        .any(|s| s.as_str() == "Attraction");
    blocks
        .into_iter()
        .map(|b| {
            if attraction && b.starts_with("Prize — ") {
                format!("{PRIZE_MARK}{b}")
            } else {
                b
            }
        })
        .collect()
}

inventory::submit! { BlockGroupPattern { name: "k702.159 attraction prizes", priority: 10, group: mark_prizes } }

/// "Prize — [effect]" (CR 702.159b): part of the Attraction's visit ability, performed when
/// its prize is claimed (see `kw/visit.rs`).
fn prize(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let block = block.strip_prefix(PRIZE_MARK)?;
    let eff = block.strip_prefix("Prize — ")?;
    let body = crate::oracle::effects::parse_body(eff, ctx)?;
    let tr = TriggeredAbility::new(TriggerCond::Custom(crate::kw::visit::PRIZE.into()), body);
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), block)])
}

inventory::submit! { AbilityPattern { name: "k702.159 attraction prize", priority: 70, parse: prize } }

/// "claim the prize" (CR 702.159b).
fn claim_the_prize(l: &str, _b: &mut Builder) -> Option<Effect> {
    matches!(end(l).trim_end_matches('!'), "claim the prize")
        .then(|| Effect::Custom(crate::kw::visit::CLAIM_THE_PRIZE.into()))
}

inventory::submit! { EffectPattern { name: "k702.159 claim the prize", priority: 100, parse: claim_the_prize } }

/// "Whenever you claim the prize of an Attraction" (The Most Dangerous Gamer).
fn you_claim_the_prize(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(r) == "you claim the prize of an attraction").then(|| {
        (
            TriggerCond::Custom(crate::kw::visit::YOU_CLAIM.into()),
            Sel::TriggerObject,
            PlayerRef::You,
        )
    })
}

inventory::submit! { TriggerPattern { name: "k702.159 you claim the prize", priority: 100, parse: you_claim_the_prize } }

/// "Craft with [materials] [cost]" (CR 702.167a): the keyword, with the materials'
/// description kept in its text (see `kw/craft.rs`).
fn craft(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().trim_end_matches('.');
    let rest = t.strip_prefix("Craft with ")?;
    let i = rest.find(" {")?;
    let (materials, cost) = (rest[..i].trim(), rest[i..].trim());
    crate::kw::craft::parse_materials(materials)?;
    let cost = crate::oracle::keywords::parse_keyword_cost(cost)?;
    let mut kw = Keyword::with_cost(KeywordKind::Craft, cost);
    kw.text = Some(format!("with {}", materials.to_lowercase()).into());
    Some(crate::oracle::keywords::compile_keyword(kw, t))
}

inventory::submit! { AbilityPattern { name: "k702.167 craft", priority: 100, parse: craft } }

/// Abilities that refer to "the exiled cards used to craft it" (CR 702.167c): "exiled" is
/// part of what "used to craft it" means (cards in exile that were exiled to pay the craft
/// cost, see `kw/craft.rs`), so the phrase is read as "cards used to craft it".
fn craft_references(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !block.contains(" used to craft ") || !block.contains("exiled ") {
        return None;
    }
    let mut text = block.to_string();
    for (from, to) in [
        ("the exiled cards used to craft", "cards used to craft"),
        ("the exiled card used to craft", "the card used to craft"),
        ("an exiled card used to craft", "a card used to craft"),
        ("exiled creature card used to craft", "creature card used to craft"),
        ("exiled cards used to craft", "cards used to craft"),
    ] {
        text = text.replace(from, to);
    }
    if text == block || text.contains("exiled card") {
        return None;
    }
    let abilities = crate::oracle::parse_ability(&text, ctx)?;
    Some(
        abilities
            .into_iter()
            .map(|a| AbilityDef::with_link(a.kind.clone(), block.trim(), a.link))
            .collect(),
    )
}

inventory::submit! { AbilityPattern { name: "k702.167 exiled cards used to craft it", priority: 100, parse: craft_references } }

/// "The first [quality] spell you cast each turn has [keyword]" (Anhelo, the Painter:
/// "... has casualty 2"; Maelstrom Nexus: "The first spell you cast each turn has
/// cascade."): the spell has the keyword as it's cast.
fn first_spell_each_turn_has(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("the first ")?;
    let (subject, kw_text) = r.split_once("spell you cast each turn has ")?;
    let subject = subject.trim();
    let mut parts = if subject.is_empty() {
        vec![]
    } else {
        let phrase = format!("{subject} spell");
        let (f, _, tail) = parse_object_phrase(&phrase)?;
        if !end(tail).is_empty() {
            return None;
        }
        vec![f]
    };
    let mut kws = Vec::new();
    for a in crate::oracle::keywords::parse_keyword_line(kw_text, ctx)? {
        if let AbilityKind::Keyword(k) = &a.kind {
            kws.push(k.clone());
        }
    }
    if kws.is_empty() {
        return None;
    }
    parts.push(Filter::Spell);
    parts.push(Filter::ControlledBy(PlayerRel::You));
    parts.push(Filter::Custom(
        crate::kw::first_spell_each_turn::FIRST_THIS_TURN.into(),
    ));
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::And(parts),
        mods: kws.into_iter().map(Modification::AddKeyword).collect(),
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "the first [quality] spell you cast each turn has [keyword]", priority: 100, parse: first_spell_each_turn_has } }
