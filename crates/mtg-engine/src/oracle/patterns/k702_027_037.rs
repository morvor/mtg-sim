//! Oracle patterns for CR 702.27–702.37 (buyback, shadow, cycling, echo, horsemanship,
//! fading, kicker, flashback, madness, fear, morph):
//!
//! * "Buyback costs cost {2} less." / "Cycling abilities you activate cost {2} less to
//!   activate." — modifications of a keyword's costs (CR 601.2f, 602.2b; typecycling costs
//!   are cycling costs, CR 702.29f);
//! * "if it was kicked with its {1}{B} kicker" — a condition linked to one specific kicker
//!   cost (CR 702.33f, 607.2i).

use super::{ConditionPattern, EffectPattern, FollowupPattern, StaticPattern};
use crate::oracle::effects::Builder;
use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::mana::ManaCost;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "{2}" → 2.
fn generic_amount(s: &str) -> Option<i32> {
    s.trim().strip_prefix('{')?.strip_suffix('}')?.parse().ok()
}

/// "[Keyword] costs cost {N} less/more", "[Keyword] costs you pay cost {N} less", and
/// "[Keyword] abilities you activate cost {N} less/more to activate".
fn keyword_cost_change(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (kind, who, rest) = if let Some(r) = l.strip_prefix("buyback costs cost ") {
        (KeywordKind::Buyback, PlayerRel::Any, r)
    } else if let Some(r) = l.strip_prefix("cycling abilities you activate cost ") {
        let r = r.strip_suffix(" to activate").unwrap_or(r);
        (KeywordKind::Cycling, PlayerRel::You, r)
    } else if let Some(r) = l.strip_prefix("all morph costs cost ") {
        (KeywordKind::Morph, PlayerRel::Any, r)
    } else if let Some(r) = l.strip_prefix("flashback costs you pay cost ") {
        (KeywordKind::Flashback, PlayerRel::You, r)
    } else if let Some(r) = l.strip_prefix("flashback costs your opponents pay cost ") {
        (KeywordKind::Flashback, PlayerRel::Opponent, r)
    } else {
        return None;
    };
    let (amount, dir) = end(rest).rsplit_once(' ')?;
    let n = generic_amount(amount)?;
    let change = match dir {
        "less" => CostChange::ReduceGeneric(Value::c(n)),
        "more" => CostChange::IncreaseGeneric(Value::c(n)),
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::Keyword(kind),
                who,
                change,
            },
        ))),
        text,
    )])
}

/// The name recorded in `CastInfo::paid` for a paid kicker cost with this mana cost
/// (see `Game::cast_inner`, CR 607.2i).
pub fn kicker_cost_name(cost: &ManaCost) -> String {
    format!("kicker {cost}")
}

/// "it was kicked with its {1}{b} kicker" (CR 702.33f); "it was kicked twice" (a spell with
/// two kicker costs whose controller paid both, CR 702.33d).
fn kicked_with(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = ["it ", "~ ", "this spell ", "this creature ", "this permanent "]
        .iter()
        .find_map(|p| c.strip_prefix(p))?;
    match r {
        "was kicked twice" => {
            return Some(Condition::Compare(Value::TimesKicked, Cmp::Ge, Value::c(2)))
        }
        "wasn't kicked" | "was not kicked" => {
            return Some(Condition::Not(Box::new(Condition::CostPaid(
                crate::kw::kicker::KICKER.into(),
            ))))
        }
        _ => {}
    }
    let sym = r
        .strip_prefix("was kicked with its ")?
        .strip_suffix(" kicker")?;
    let cost = ManaCost::parse(&sym.to_uppercase())?;
    Some(Condition::CostPaid(kicker_cost_name(&cost).into()))
}

/// "if its madness cost was paid" / "if ~'s madness cost was paid" (CR 702.35a: the spell
/// was cast with its madness ability; a permanent's abilities see how its spell was cast,
/// CR 400.7d).
fn madness_cost_paid(c: &str) -> Option<Condition> {
    matches!(
        end(c),
        "its madness cost was paid" | "~'s madness cost was paid"
    )
    .then(|| Condition::CostPaid(crate::kw::madness::MADNESS.into()))
}

/// "~ can block creatures with shadow as though it had shadow" / "... as though they
/// didn't have shadow" (see `kw/shadow.rs`, CR 702.28b).
fn blocks_shadow(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::kw::shadow::*;
    let name = match end(l) {
        "~ can block creatures with shadow as though it had shadow" => {
            BLOCKS_SHADOW_AS_THOUGH_IT_HAD_SHADOW
        }
        "~ can block creatures with shadow as though they didn't have shadow" => {
            BLOCKS_SHADOW_AS_THOUGH_THEY_DIDNT
        }
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(name.into()))),
        text,
    )])
}

/// "Players can't cycle cards." (stops typecycling too, CR 702.29f).
fn cant_cycle(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "players can't cycle cards" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(
            Restriction::Custom(crate::kw::cycling::PLAYERS_CANT_CYCLE.into()),
        ))),
        text,
    )])
}

/// "Creature spells you cast have sticker kicker {1}." (CR 702.33h): the keyword is granted
/// to those spells while they're on the stack, where it functions.
fn spells_you_cast_have_sticker_kicker(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let (subject, granted) = end(l).split_once(" spells you cast have ")?;
    if !granted.starts_with("sticker kicker ") {
        return None;
    }
    let kws: Vec<crate::keywords::Keyword> =
        crate::oracle::keywords::parse_keyword_line(granted, ctx)?
            .into_iter()
            .filter_map(|a| match &a.kind {
                AbilityKind::Keyword(k) => Some(k.clone()),
                _ => None,
            })
            .collect();
    if kws.len() != 1 {
        return None;
    }
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(subject)?;
    if !end(tail).is_empty() {
        return None;
    }
    let affected = Filter::and(vec![f, Filter::Spell, Filter::ControlledBy(PlayerRel::You)]);
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods: kws.into_iter().map(Modification::AddKeyword).collect(),
        })),
        text,
    )])
}

/// Whether the effect grants flashback without a cost of its own.
fn grants_costless_flashback(e: &Effect) -> bool {
    match e {
        Effect::Modify { mods, .. } => mods.iter().any(|m| {
            matches!(m, Modification::AddKeyword(k)
                if k.kind == KeywordKind::Flashback && k.cost.is_none())
        }),
        Effect::Seq(v) => v.iter().any(grants_costless_flashback),
        Effect::If { then, .. } => grants_costless_flashback(then),
        Effect::May { effect, .. } => grants_costless_flashback(effect),
        Effect::ForEach { effect, .. } => grants_costless_flashback(effect),
        _ => false,
    }
}

/// "[card] gains flashback until end of turn. The flashback cost is equal to its mana
/// cost.": a granted flashback ability without a cost of its own is cast for the card's
/// mana cost (see `kw/flashback.rs`, CR 702.34a).
fn flashback_cost_is_mana_cost(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    matches!(
        end(s),
        "the flashback cost is equal to its mana cost"
            | "the flashback cost is equal to that card's mana cost"
            | "its flashback cost is equal to its mana cost"
    ) && grants_costless_flashback(prev)
}

/// In an ability that triggers on a discard: "return the discarded card from your
/// graveyard to your hand", "exile that card from your graveyard" — only if the card is in
/// the graveyard (a card with madness is exiled instead; after its madness ability put it
/// into the graveyard, the ability can find it there, CR 702.35c, 400.7k).
fn discarded_card_from_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger {
        return None;
    }
    let effect = match end(l) {
        "return the discarded card from your graveyard to your hand" => Effect::Move {
            what: Sel::TriggerObject,
            to: Destination::zone(ZoneKind::Hand),
        },
        "exile that card from your graveyard" | "exile the discarded card from your graveyard" => {
            Effect::Exile {
                what: Sel::TriggerObject,
                face_down: false,
                link: false,
            }
        }
        _ => return None,
    };
    Some(Effect::If {
        cond: Condition::SelMatches(Sel::TriggerObject, Filter::InZone(ZoneKind::Graveyard)),
        then: Box::new(effect),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! {
    FollowupPattern { name: "k702.34: flashback cost equal to mana cost", priority: 50, apply: flashback_cost_is_mana_cost }
}
inventory::submit! {
    EffectPattern { name: "k702.35c: the discarded card from your graveyard", priority: 50, parse: discarded_card_from_graveyard }
}
/// "Each land card in your hand has cycling {R}.", "Each Sliver card in each player's hand
/// has slivercycling {3}.": cycling granted to cards in hands, where it functions
/// (CR 702.29a).
fn cards_in_hand_have_cycling(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("each ")?;
    let (quality, rest) = r.split_once(" card in ")?;
    let (owner, granted) = if let Some(g) = rest.strip_prefix("your hand has ") {
        (Some(Filter::OwnedBy(PlayerRel::You)), g)
    } else if let Some(g) = rest.strip_prefix("each player's hand has ") {
        (None, g)
    } else {
        return None;
    };
    let kws: Vec<crate::keywords::Keyword> =
        crate::oracle::keywords::parse_keyword_line(granted, ctx)?
            .into_iter()
            .filter_map(|a| match &a.kind {
                AbilityKind::Keyword(k) => Some(k.clone()),
                _ => None,
            })
            .collect();
    if kws.len() != 1 || kws[0].kind != KeywordKind::Cycling {
        return None;
    }
    let phrase = format!("{quality} card");
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(&phrase)?;
    // "creature card": the head noun may leave "card" unparsed.
    if !matches!(end(tail), "" | "card") {
        return None;
    }
    let mut parts = vec![f, Filter::InZone(ZoneKind::Hand)];
    parts.extend(owner);
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::and(parts),
            mods: kws.into_iter().map(Modification::AddKeyword).collect(),
        })),
        text,
    )])
}

/// "Each Vampire creature card you own that isn't on the battlefield has madness. The
/// madness cost is equal to its mana cost." (see `kw/madness.rs`).
fn cards_you_own_have_madness(
    l: &str,
    text: &str,
    _ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("each ")?;
    let (quality, rest) = r.split_once(" card you own that isn't on the battlefield has madness")?;
    if !matches!(end(rest), "" | ". the madness cost is equal to its mana cost") {
        return None;
    }
    let phrase = format!("{quality} card");
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(&phrase)?;
    // "creature card": the head noun may leave "card" unparsed.
    if !matches!(end(tail), "" | "card") {
        return None;
    }
    // One effect per zone where the ability matters: the hand (where the discard
    // replacement functions), exile (the trigger and casting), and the graveyard.
    let out = [ZoneKind::Hand, ZoneKind::Exile, ZoneKind::Graveyard]
        .into_iter()
        .map(|z| {
            let affected = Filter::and(vec![
                f.clone(),
                Filter::OwnedBy(PlayerRel::You),
                Filter::InZone(z),
            ]);
            let s = StaticAbility::new(StaticEffect::Continuous {
                affected,
                mods: vec![Modification::AddKeyword(
                    crate::keywords::Keyword::new(KeywordKind::Madness).text("Madness"),
                )],
            });
            AbilityDef::new(AbilityKind::Static(s), text)
        })
        .collect();
    Some(out)
}

inventory::submit! {
    StaticPattern { name: "k702.35: cards you own have madness", priority: 50, parse: cards_you_own_have_madness }
}
inventory::submit! {
    StaticPattern { name: "k702.27-37: keyword cost changes", priority: 50, parse: keyword_cost_change }
}
inventory::submit! {
    StaticPattern { name: "k702.29: cards in hand have cycling", priority: 50, parse: cards_in_hand_have_cycling }
}
inventory::submit! {
    StaticPattern { name: "k702.33h: spells you cast have sticker kicker", priority: 50, parse: spells_you_cast_have_sticker_kicker }
}
inventory::submit! {
    StaticPattern { name: "k702.29: players can't cycle", priority: 50, parse: cant_cycle }
}
inventory::submit! {
    StaticPattern { name: "k702.28: blocks creatures with shadow", priority: 50, parse: blocks_shadow }
}
inventory::submit! {
    ConditionPattern { name: "k702.33f: kicked with a specific kicker", priority: 50, parse: kicked_with }
}
inventory::submit! {
    ConditionPattern { name: "k702.35: madness cost was paid", priority: 50, parse: madness_cost_paid }
}
