//! Oracle text of the keywords of CR 702.67–702.83 that the generic keyword parser doesn't
//! handle, and phrases that go with them:
//!
//! * "Devour artifact 1", "Devour Food 3", "Devour X, where X is the number of creatures
//!   devoured this way" (CR 702.82a, 702.82c);
//! * "for each creature it devoured", "the number of Goblins it devoured", "if it devoured
//!   a creature" (CR 702.82b);
//! * "When a Faerie is championed with ~" (CR 702.72c);
//! * "if its prowl cost was paid" (CR 702.76a), "if its evoke cost was paid";
//! * "you may play the exiled card without paying its mana cost [if ...]", "put the exiled
//!   card into its owner's hand" (hideaway, CR 702.75a, 607.2a);
//! * "Each [quality] spell you cast has conspire." (CR 702.78a), "[Quality] cards in your
//!   graveyard have retrace." (CR 702.81a).

use super::{AbilityPattern, ConditionPattern, EffectPattern, StaticPattern, TriggerPattern};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::kw::devour::{DEVOURED_COUNT, DEVOURED_OF_TYPE};
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::keywords::compile_keyword;
use crate::oracle::phrases::{end, parse_object_phrase, subtype_word};
use crate::oracle::CompileContext;

// ---------------------------------------------------------------------------
// Devour (CR 702.82)
// ---------------------------------------------------------------------------

/// "Devour [quality] N" (CR 702.82c) and "Devour X, where X is the number of creatures
/// devoured this way" (N = -1, see `kw/devour.rs`).
fn devour_line(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().trim_end_matches('.');
    let lower = t.to_lowercase();
    let r = lower.strip_prefix("devour ")?;
    if r == "x, where x is the number of creatures devoured this way" {
        let kw = Keyword::with_n(KeywordKind::Devour, -1).text(t);
        return Some(compile_keyword(kw, t));
    }
    let (quality, n) = r.rsplit_once(' ')?;
    let n: i32 = n.parse().ok()?;
    let (f, _, tail) = parse_object_phrase(quality)?;
    if !end(tail).is_empty() || quality.contains(' ') {
        return None;
    }
    let kw = Keyword {
        filter: Some(f),
        ..Keyword::with_n(KeywordKind::Devour, n)
    }
    .text(t);
    Some(compile_keyword(kw, t))
}

inventory::submit! { AbilityPattern { name: "devour [quality] N", priority: 100, parse: devour_line } }

/// The number of permanents of a quality the source devoured: "creature(s)", "artifact(s)",
/// "permanent(s)" count them all (everything devour sacrifices has its quality); a
/// subtype ("Goblins") counts those of that subtype.
fn devoured_value(quality: &str) -> Option<Value> {
    let q = quality.trim();
    if matches!(
        q,
        "creature" | "creatures" | "permanent" | "permanents" | "artifact" | "artifacts"
            | "land" | "lands" | "food" | "foods"
    ) {
        return Some(Value::Custom(DEVOURED_COUNT.into()));
    }
    let st = subtype_word(q)?;
    Some(Value::Custom(format!("{DEVOURED_OF_TYPE}{st}").into()))
}

/// Multiplies the amount of a simple effect ("draw a card", "you gain 2 life", "target
/// player discards a card") by `by`.
fn scale_amount(e: &mut Effect, by: Value) -> bool {
    let n = match e {
        Effect::Draw { n, .. }
        | Effect::GainLife { n, .. }
        | Effect::LoseLife { n, .. }
        | Effect::Discard { n, .. }
        | Effect::Mill { n, .. }
        | Effect::AddCounters { n, .. } => n,
        Effect::DealDamage { amount, .. } => amount,
        Effect::CreateToken { count, .. } => count,
        _ => return false,
    };
    let old = std::mem::replace(n, Value::c(0));
    *n = match old {
        Value::Const(1) => by,
        other => Value::Mul(Box::new(other), Box::new(by)),
    };
    true
}

/// "[effect] for each creature it devoured" (Skullmulcher, Marrow Chomper, Tar Fiend) and
/// "~ deals damage to any target equal to twice the number of Goblins it devoured"
/// (Voracious Dragon).
fn devoured_effects(l: &str, b: &mut Builder) -> Option<Effect> {
    if let Some((inner, quality)) = l.split_once(" for each ") {
        let quality = quality
            .strip_suffix(" it devoured")
            .or_else(|| quality.strip_suffix(" ~ devoured"))?;
        let by = devoured_value(quality)?;
        let mut e = parse_clause(inner, b)?;
        return scale_amount(&mut e, by).then_some(e);
    }
    let (inner, value) = l.split_once(" equal to ")?;
    let (factor, rest) = match value.strip_prefix("twice ") {
        Some(r) => (2, r),
        None => (1, value),
    };
    let quality = rest
        .strip_prefix("the number of ")?
        .strip_suffix(" it devoured")
        .or_else(|| rest.strip_prefix("the number of ")?.strip_suffix(" ~ devoured"))?;
    let count = devoured_value(quality)?;
    let by = if factor == 1 {
        count
    } else {
        Value::Mul(Box::new(Value::c(factor)), Box::new(count))
    };
    // "it deals damage to any target" → "it deals 1 damage to any target", scaled.
    let (subject, target) = inner.split_once(" deals damage ")?;
    let mut e = parse_clause(&format!("{subject} deals 1 damage {target}"), b)?;
    scale_amount(&mut e, by).then_some(e)
}

inventory::submit! { EffectPattern { name: "for each creature it devoured", priority: 100, parse: devoured_effects } }

/// "it devoured a creature" (Hellkite Hatchling).
fn devoured_condition(c: &str) -> Option<Condition> {
    let r = end(c)
        .strip_prefix("it devoured ")
        .or_else(|| end(c).strip_prefix("~ devoured "))?;
    let q = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let v = devoured_value(q)?;
    Some(Condition::Compare(v, Cmp::Gt, Value::c(0)))
}

inventory::submit! { ConditionPattern { name: "it devoured a creature", priority: 100, parse: devoured_condition } }

// ---------------------------------------------------------------------------
// Champion (CR 702.72c)
// ---------------------------------------------------------------------------

/// "a Faerie is championed with ~" (Mistbind Clique): the permanent exiled by this
/// permanent's champion ability had the quality as it was championed.
fn championed_with(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = r
        .strip_prefix("a ")
        .or_else(|| r.strip_prefix("an "))?
        .strip_suffix(" is championed with ~")?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::Custom(crate::kw::champion::CHAMPIONED.into())),
            cond: Condition::SelMatches(Sel::TriggerLki, f),
        },
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "a [quality] is championed with ~", priority: 100, parse: championed_with } }

// ---------------------------------------------------------------------------
// Prowl, evoke (CR 702.76a, 702.74a)
// ---------------------------------------------------------------------------

/// "its prowl cost was paid" / "~'s prowl cost was paid", and the same for evoke.
fn alt_cost_paid(c: &str) -> Option<Condition> {
    let r = end(c)
        .strip_prefix("its ")
        .or_else(|| end(c).strip_prefix("~'s "))?;
    match r {
        "prowl cost was paid" => Some(Condition::CostPaid(crate::kw::prowl::PROWL.into())),
        "evoke cost was paid" => Some(Condition::CostPaid(crate::kw::evoke::EVOKE.into())),
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "its prowl/evoke cost was paid", priority: 100, parse: alt_cost_paid } }

// ---------------------------------------------------------------------------
// Hideaway (CR 702.75a): "the exiled card"
// ---------------------------------------------------------------------------

/// The cards exiled with this permanent that are still in exile: one that left exile is a
/// new object, no longer "the exiled card" (CR 400.7).
fn still_exiled() -> Sel {
    Sel::All(Filter::and(vec![
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ]))
}

/// "you may play the exiled card without paying its mana cost [if <condition>]" and "put
/// the exiled card into its owner's hand": the card exiled with this permanent (by its
/// hideaway ability, CR 607.2a).
fn exiled_card_effects(l: &str, b: &mut Builder) -> Option<Effect> {
    if l == "put the exiled card into its owner's hand" {
        const V: Var = vars::USER + 73;
        return Some(Effect::ForEach {
            sel: still_exiled(),
            var: V,
            effect: Box::new(Effect::Move {
                what: Sel::Var(V),
                to: Destination::zone(ZoneKind::Hand),
            }),
        });
    }
    // "you may play ..." inside a sentence; a sentence's leading "you may" is already an
    // optional effect around the rest.
    let (optional, r) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let (play, r) = if let Some(r) = r.strip_prefix("cast the exiled card without paying its mana cost")
    {
        // "you may cast the exiled card without paying its mana cost" (Smuggler's
        // Buggy): the spells cast are "it" for "If you do, ...".
        let cast = Effect::CastCard {
            who: PlayerRef::You,
            what: still_exiled(),
            free: true,
            optional,
        };
        (cast, r)
    } else {
        let r = r.strip_prefix("play the exiled card without paying its mana cost")?;
        let play = Effect::PlayCard {
            who: PlayerRef::You,
            what: still_exiled(),
            free: true,
            optional,
        };
        (play, r)
    };
    if r.is_empty() {
        return Some(play);
    }
    let c = r.strip_prefix(" if ")?;
    let cond = crate::oracle::statics::parse_condition(c, b.ctx)?;
    Some(Effect::If {
        cond,
        then: Box::new(play),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "play / put the exiled card", priority: 100, parse: exiled_card_effects } }

// ---------------------------------------------------------------------------
// Conspire and retrace granted (CR 702.78a, 702.81a)
// ---------------------------------------------------------------------------

/// "Each [quality] spell you cast has conspire." (Wort, the Raidmother; Raiding
/// Schemes): the spells have conspire as they're cast.
fn spells_have_conspire(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("each ")?.strip_suffix(" has conspire")?;
    // "Each noncreature spell you cast from exile has conspire." (Rassilon)
    let (subject, from) = match r.strip_suffix(" spell you cast from exile") {
        Some(s) => (s, Some(Filter::CastFrom(ZoneKind::Exile))),
        None => (r.strip_suffix(" spell you cast")?, None),
    };
    let phrase = format!("{subject} card");
    let (f, _, tail) = parse_object_phrase(&phrase)?;
    if !end(tail).is_empty() {
        return None;
    }
    let mut parts = vec![f, Filter::Spell, Filter::ControlledBy(PlayerRel::You)];
    parts.extend(from);
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::and(parts),
        mods: vec![Modification::AddKeyword(
            Keyword::new(KeywordKind::Conspire).text("conspire"),
        )],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "each [quality] spell you cast has conspire", priority: 100, parse: spells_have_conspire } }

/// "[Quality] cards in your graveyard have retrace." (Deeproot Historian), optionally
/// "During your turn, ..." (the effect applies only during your turn).
fn graveyard_cards_have_retrace(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cond, r) = match l.strip_prefix("during your turn, ") {
        Some(r) => (Some(Condition::YourTurn), r),
        None => (None, l),
    };
    let subject = r.strip_suffix(" cards in your graveyard have retrace")?;
    // "Merfolk and Druid" → Merfolk or Druid cards.
    let mut fs = Vec::new();
    for part in subject.split(" and ") {
        let phrase = format!("{part} card");
        let (f, _, tail) = parse_object_phrase(&phrase)?;
        if !end(tail).is_empty() {
            return None;
        }
        fs.push(f);
    }
    let quality = if fs.len() == 1 {
        fs.pop()?
    } else {
        Filter::Or(fs)
    };
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::and(vec![
            quality,
            Filter::Card,
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
        ]),
        mods: vec![Modification::AddKeyword(
            Keyword::new(KeywordKind::Retrace).text("retrace"),
        )],
    });
    s.condition = cond;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "cards in your graveyard have retrace", priority: 100, parse: graveyard_cards_have_retrace } }

/// "~ has flying and trample if it devoured a creature." (Hellkite Hatchling): the same as
/// "As long as it devoured a creature, ~ has flying and trample."
fn has_if_devoured(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (a, c) = l.split_once(" if ")?;
    if !a.starts_with("~ has ") || devoured_condition(c).is_none() {
        return None;
    }
    let rewritten = format!("as long as {c}, {a}");
    let mut out = crate::oracle::statics::parse_static(&rewritten, ctx)?;
    for x in out.iter_mut() {
        *x = AbilityDef::with_link(x.kind.clone(), text, x.link);
    }
    Some(out)
}

inventory::submit! { StaticPattern { name: "~ has [keywords] if it devoured", priority: 100, parse: has_if_devoured } }

/// Conditions of the hideaway lands' "play the exiled card" abilities: "creatures you
/// control have total power 10 or greater", "you attacked with three or more creatures
/// this turn", "an opponent was dealt 7 or more damage this turn", "each player has no
/// cards in hand", "a library has twenty or fewer cards in it".
fn hideaway_conditions(c: &str) -> Option<Condition> {
    use crate::kw::hideaway::{ATTACKERS_THIS_TURN, MOST_DAMAGE_TO_AN_OPPONENT, SMALLEST_LIBRARY};
    use crate::oracle::phrases::parse_number;
    let c = end(c);
    if c == "each player has no cards in hand" {
        return Some(Condition::Not(Box::new(Condition::PlayerMatches(
            PlayerRef::EachPlayer,
            PlayerFilter::HandSize(Cmp::Ge, Box::new(Value::c(1))),
        ))));
    }
    let at_least = |r: &str, sfx: &str| -> Option<Value> {
        let (n, rest) = parse_number(r.strip_suffix(sfx)?)?;
        end(rest).is_empty().then_some(n)
    };
    if let Some(r) = c.strip_prefix("creatures you control have total power ") {
        let n = at_least(r, " or greater")?;
        return Some(Condition::Compare(
            Value::PowerOf(Box::new(Sel::All(Filter::creature().you_control()))),
            Cmp::Ge,
            n,
        ));
    }
    if let Some(r) = c.strip_prefix("you attacked with ") {
        let n = at_least(r, " or more creatures this turn")?;
        return Some(Condition::Compare(
            Value::Custom(ATTACKERS_THIS_TURN.into()),
            Cmp::Ge,
            n,
        ));
    }
    if let Some(r) = c.strip_prefix("an opponent was dealt ") {
        let n = at_least(r, " or more damage this turn")?;
        return Some(Condition::Compare(
            Value::Custom(MOST_DAMAGE_TO_AN_OPPONENT.into()),
            Cmp::Ge,
            n,
        ));
    }
    if let Some(r) = c.strip_prefix("a library has ") {
        let (n, rest) = parse_number(r)?;
        if end(rest) != "or fewer cards in it" {
            return None;
        }
        return Some(Condition::Compare(
            Value::Custom(SMALLEST_LIBRARY.into()),
            Cmp::Le,
            n,
        ));
    }
    None
}

inventory::submit! { ConditionPattern { name: "hideaway land conditions", priority: 100, parse: hideaway_conditions } }

/// "Whenever ~ deals combat damage to a player, if there are cards exiled with it, you may
/// play one of them without paying its mana cost." (Evercoat Ursine, whose two hideaway
/// abilities exile cards with it).
fn play_one_of_the_exiled_cards(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    if t.to_lowercase()
        != "whenever ~ deals combat damage to a player, if there are cards exiled with it, you may play one of them without paying its mana cost."
    {
        return None;
    }
    let exiled_with_it = Filter::and(vec![
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ]);
    let mut trig = TriggeredAbility::new(
        TriggerCond::DealsDamage {
            source: Filter::Source,
            to: DamageRecipient::Player(PlayerRel::Any),
            combat_only: true,
        },
        Body::effect(Effect::PlayCard {
            who: PlayerRef::You,
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: exiled_with_it.clone(),
                count: Value::c(1),
                up_to: true,
                store: None,
            },
            free: true,
            optional: false,
        }),
    );
    trig.intervening_if = Some(Condition::Exists(exiled_with_it));
    Some(vec![AbilityDef::new(AbilityKind::Triggered(trig), t)])
}

inventory::submit! { AbilityPattern { name: "play one of the cards exiled with it", priority: 100, parse: play_one_of_the_exiled_cards } }

// ---------------------------------------------------------------------------
// Persist and other keywords granted to any number of targets
// ---------------------------------------------------------------------------

/// "Choose any number of target creatures." — the targets are "those creatures" for the
/// next sentence (Cauldron Haze, Cauldron of Souls).
fn choose_any_number_of_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("choose ")?;
    if !r.starts_with("any number of target ") {
        return None;
    }
    let (spec, tail) = crate::oracle::phrases::parse_target(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    let slot = b.add_target(spec, r);
    b.it = Sel::Target(slot);
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "choose any number of target creatures", priority: 100, parse: choose_any_number_of_targets } }

/// "Each of those creatures gains persist until end of turn." after "Choose any number of
/// target creatures.".
fn each_of_those_gains(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("each of those creatures gains ")?;
    let Sel::Target(slot) = b.it else {
        return None;
    };
    let (duration, kws) = crate::oracle::effects::duration_suffix(r);
    let mods = crate::oracle::effects::keyword_mods(kws)?;
    Some(Effect::Modify {
        what: Sel::Target(slot),
        mods,
        duration,
    })
}

inventory::submit! { EffectPattern { name: "each of those creatures gains [keyword]", priority: 100, parse: each_of_those_gains } }

// ---------------------------------------------------------------------------
// Wither (CR 702.80)
// ---------------------------------------------------------------------------

/// "All damage is dealt as though its source had wither." (Everlasting Torment)
fn all_damage_as_though_wither(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "all damage is dealt as though its source had wither" {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::Custom(
        crate::kw::wither::ALL_DAMAGE_AS_THOUGH_WITHER.into(),
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "all damage is dealt as though its source had wither", priority: 100, parse: all_damage_as_though_wither } }
