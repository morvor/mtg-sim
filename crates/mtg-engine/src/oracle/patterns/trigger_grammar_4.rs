//! Trigger events, grammar IV (cards3-triggers-durations).
//!
//! Events qualified by the spell or ability that caused them (CR 603.2; see
//! `kw/trigger_events.rs`): "[cause] causes [subject] to [verb phrase]", read as the
//! trigger "[subject] [verb phrase]" with a condition on its cause:
//! - "a spell or ability causes a player to shuffle their library" (Psychogenic Probe);
//! - "a spell or ability causes its controller to shuffle their library" (Widespread
//!   Panic): the player who shuffles controls the spell or ability;
//! - "a spell or ability an opponent controls causes a land to be put into your graveyard
//!   from the battlefield" (Sacred Ground);
//! - "a white instant or sorcery spell causes you to gain life" (Firesong and
//!   Sunspeaker): a spell described by an object phrase.
//!
//! Exact amounts: "Whenever another source you control deals exactly 1 damage to a
//! permanent or player, ~ deals 2 damage to that permanent or player." (Ghyrson Starn,
//! Kelermorph: one source dealing exactly that much to one recipient at once, each such
//! event triggering it, CR 603.2c), "Whenever one or more opponents each lose exactly 1
//! life" (Ob Nixilis, Captive Kingpin: each opponent's total loss in one batch of
//! simultaneous events, triggering once, [`BatchPer::AnyPlayer`]).
//!
//! Untapping during the untap step (CR 502.3): "Whenever you untap one or more permanents
//! during your untap step" (The Millennium Calendar: the permanents you control that
//! become untapped then, as one batch), "Whenever ~ becomes untapped during your untap
//! step" (The Ninth Doctor).
//!
//! The creature put onto the battlefield with ~ (CR 607.2a, linked abilities): "When ~
//! enters, return target creature card from your graveyard to the battlefield." / "When
//! the creature put onto the battlefield with ~ dies, exile it and return ~ to its owner's
//! hand." / "When ~ leaves the battlefield, exile the creature put onto the battlefield
//! with ~." (Diabolic Servitude; Havengul Mystery): the returning ability notes the
//! permanent it put onto the battlefield ([`Effect::NoteLinked`]); the linked abilities
//! refer to that object, which the source loses track of once it changes zones
//! (CR 400.7).
//!
//! Playing a card exiled with ~ (CR 607.2a, 305.1, 601.2): "Whenever a player plays a
//! card exiled with ~" (Prowl, Stoic Strategist), "When you play a card exiled with ~"
//! (Voltaic Visionary): a land played or a spell cast that was a card exiled with the
//! source.

use super::{AbilityPattern, EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::kw::trigger_events::{
    CAUSED_BY_EVENT_PLAYERS, CAUSED_BY_OPPONENTS, CAUSED_BY_SPELL_OR_ABILITY,
    DURING_YOUR_UNTAP_STEP, THE_CAUSE,
};
use crate::oracle::effects::Builder;
use crate::oracle::CompileContext;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};
use crate::oracle::triggers::parse_trigger_condition;

/// "[subject] to [verb phrase]" as a clause: "a player to shuffle their library" → "a
/// player shuffles their library", "a land to be put into ..." → "a land is put into
/// ...", "you to gain life" → "you gain life".
fn clause(s: &str) -> Option<String> {
    let (subj, vp) = s.split_once(" to ")?;
    if subj == "you" {
        return Some(format!("you {vp}"));
    }
    if let Some(r) = vp.strip_prefix("be ") {
        return Some(format!("{subj} is {r}"));
    }
    let (verb, rest) = vp.split_once(' ').unwrap_or((vp, ""));
    let verb = if verb.ends_with('s') || verb.ends_with("sh") || verb.ends_with("ch") {
        format!("{verb}es")
    } else {
        format!("{verb}s")
    };
    Some(format!("{subj} {verb} {rest}").trim_end().to_string())
}

fn caused_event(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let (cause, rest) = t.split_once(" causes ")?;
    // "its controller": the spell or ability's controller is the event's player.
    let (rest, own) = match rest.strip_prefix("its controller to ") {
        Some(r) => (format!("a player to {r}"), true),
        None => (rest.to_string(), false),
    };
    let cond = match cause {
        "a spell or ability" => Condition::Custom(CAUSED_BY_SPELL_OR_ABILITY.into()),
        "a spell or ability an opponent controls" => Condition::Custom(CAUSED_BY_OPPONENTS.into()),
        _ => {
            // A spell described by an object phrase: "a white instant or sorcery spell".
            let np = cause.strip_prefix("a ").or_else(|| cause.strip_prefix("an "))?;
            let (f, plural, tail) = parse_object_phrase(np)?;
            if plural || !end(tail).is_empty() || !np.ends_with(" spell") {
                return None;
            }
            Condition::Compare(
                Value::CountSel(Box::new(Sel::All(Filter::and(vec![
                    Filter::Custom(THE_CAUSE.into()),
                    f,
                ])))),
                Cmp::Ge,
                Value::Const(1),
            )
        }
    };
    let cond = if own {
        Condition::And(vec![cond, Condition::Custom(CAUSED_BY_EVENT_PLAYERS.into())])
    } else {
        cond
    };
    let (trigger, it, player) = parse_trigger_condition(&format!("whenever {}", clause(&rest)?))?;
    // An event of the kinds read here, not a batch or a qualified trigger.
    if matches!(trigger, TriggerCond::Where { .. } | TriggerCond::Batched { .. }) {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(trigger),
            cond,
        },
        it,
        player,
    ))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: [a spell or ability] causes [an event]", priority: 100, parse: caused_event } }

/// "exactly N " / "N or more ": (comparison, N, rest).
fn amount(x: &str) -> Option<(Cmp, i32, &str)> {
    if let Some(y) = x.strip_prefix("exactly ") {
        let (n, rest) = parse_number(y)?;
        return Some((Cmp::Eq, n.as_const()?, rest.trim_start()));
    }
    let (n, rest) = parse_number(x)?;
    let rest = rest.trim_start().strip_prefix("or more ")?;
    Some((Cmp::Ge, n.as_const()?, rest))
}

/// Each damage event of the trigger (alternatives included) with a condition on its
/// amount, and its sources other than the ability's source ("another source").
fn qualify_damage(c: TriggerCond, amount: Option<(Cmp, i32)>, other: bool) -> Option<TriggerCond> {
    match c {
        TriggerCond::DealsDamage {
            source,
            to,
            combat_only,
        } => {
            let source = if other {
                Filter::and(vec![Filter::Other, source])
            } else {
                source
            };
            let d = TriggerCond::DealsDamage {
                source,
                to,
                combat_only,
            };
            Some(match amount {
                Some((cmp, n)) => TriggerCond::Where {
                    trigger: Box::new(d),
                    cond: Condition::Compare(Value::EventAmount, cmp, Value::c(n)),
                },
                None => d,
            })
        }
        TriggerCond::AnyOf(v) => Some(TriggerCond::AnyOf(
            v.into_iter()
                .map(|c| qualify_damage(c, amount, other))
                .collect::<Option<_>>()?,
        )),
        _ => None,
    }
}

/// "another source you control deals exactly 1 damage to a permanent or player", "a
/// source you control deals exactly 1 damage to a permanent or player": one event per
/// source and recipient (the simpler forms are read in `trigger_grammar_events.rs`).
fn damage_amounts(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let (other, t) = match t.strip_prefix("another ") {
        Some(r) => (true, format!("a {r}")),
        None => (false, t.to_string()),
    };
    let (subj, x) = t.split_once(" deals ")?;
    let (amt, rest) = match amount(x) {
        Some((cmp, n, r)) => (Some((cmp, n)), r),
        None => (None, x),
    };
    let rest = rest.strip_prefix("damage to ")?;
    // (Without "another" or an amount, it's what the other patterns read.)
    if !other && amt.is_none() {
        return None;
    }
    let (c, it, p) = parse_trigger_condition(&format!("whenever {subj} deals damage to {rest}"))?;
    // Only what the other patterns don't read: "another", or alternatives with an amount.
    if !other && !matches!(c, TriggerCond::AnyOf(_)) {
        return None;
    }
    // "that permanent or player": the recipient of the damage, whichever it is (the
    // events' object, else their player).
    let (it, p) = if matches!(c, TriggerCond::AnyOf(_)) && rest == "a permanent or player" {
        (Sel::TriggerObject, PlayerRef::TriggerPlayer)
    } else {
        (it, p)
    };
    Some((qualify_damage(c, amt, other)?, it, p))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: another source deals exactly N damage", priority: 110, parse: damage_amounts } }

/// "one or more opponents each lose exactly N life", "... each lose N or more life".
fn each_lose_life(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let (who, r) = if let Some(r) = t.strip_prefix("one or more opponents each lose ") {
        (PlayerRel::Opponent, r)
    } else {
        (PlayerRel::Any, t.strip_prefix("one or more players each lose ")?)
    };
    let (cmp, n, rest) = amount(r)?;
    if rest != "life" {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::Batched {
                trigger: Box::new(TriggerCond::LosesLife { who }),
                per: BatchPer::AnyPlayer,
            }),
            cond: Condition::Compare(Value::EventAmount, cmp, Value::c(n)),
        },
        Sel::None,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: one or more opponents each lose exactly N life", priority: 100, parse: each_lose_life } }

/// "~ deals N damage to that permanent or player" in an ability that triggers on damage
/// dealt to a permanent or player: the permanent, or else the player, the trigger
/// event's damage was dealt to.
fn damage_to_that_permanent_or_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("~ deals ")?;
    let (n, r) = parse_number(r)?;
    if r.trim_start() != "damage to that permanent or player"
        || !b.in_trigger
        || !matches!(b.it, Sel::TriggerObject)
        || !matches!(b.it_player, PlayerRef::TriggerPlayer)
    {
        return None;
    }
    let deal = |to: Sel| Effect::DealDamage {
        source: Sel::This,
        amount: n.clone(),
        to,
    };
    Some(Effect::If {
        cond: Condition::Compare(
            Value::CountSel(Box::new(Sel::TriggerObject)),
            Cmp::Ge,
            Value::c(1),
        ),
        then: Box::new(deal(Sel::TriggerObject)),
        otherwise: Box::new(deal(Sel::Players(PlayerRef::TriggerPlayer))),
    })
}

inventory::submit! { EffectPattern { name: "~ deals N damage to that permanent or player (the damaged one)", priority: 100, parse: damage_to_that_permanent_or_player } }

/// "[untap event] during your untap step".
fn during_untap_step(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(t).strip_suffix(" during your untap step")?;
    let ev = match r.strip_prefix("you untap one or more ") {
        Some(objs) => format!("one or more {objs} you control become untapped"),
        None => r.to_string(),
    };
    let (c, it, p) = parse_trigger_condition(&format!("whenever {ev}"))?;
    let untap = |c: &TriggerCond| matches!(c, TriggerCond::BecomesUntapped(_));
    let ok = match &c {
        TriggerCond::Batched { trigger, .. } => untap(trigger),
        c => untap(c),
    };
    if !ok {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(c),
            cond: Condition::Custom(DURING_YOUR_UNTAP_STEP.into()),
        },
        it,
        p,
    ))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: [untap event] during your untap step", priority: 100, parse: during_untap_step } }

/// "the creature put onto the battlefield with ~".
const PUT_WITH: &str = "the creature put onto the battlefield with ~";

/// Whether the card refers to the creature an ability of it put onto the battlefield.
fn card_refers_to_put_creature() -> bool {
    let raw = crate::oracle::raw_text().to_lowercase();
    raw.contains("the creature put onto the battlefield with ")
}

std::thread_local! {
    static NOTING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// An ability that returns a creature card to the battlefield ("When ~ enters, return
/// target creature card from your graveyard to the battlefield.") on such a card: the
/// permanent it returns is noted for the linked abilities.
fn note_put_onto_battlefield(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase();
    if NOTING.with(|n| n.get())
        || !lower.contains("return target creature card ")
        || !lower.contains(" to the battlefield")
        || lower.contains("the creature put onto the battlefield with ")
        || !card_refers_to_put_creature()
    {
        return None;
    }
    NOTING.with(|n| n.set(true));
    let parsed = crate::oracle::parse_ability(block, ctx);
    NOTING.with(|n| n.set(false));
    let mut out = Vec::new();
    for a in parsed? {
        let mut kind = a.kind.clone();
        let body = match &mut kind {
            AbilityKind::Triggered(t) => &mut t.body,
            AbilityKind::Activated(x) => &mut x.body,
            _ => return None,
        };
        if !matches!(&body.effect, Effect::Move { what: Sel::Target(0), to } if to.zone == ZoneKind::Battlefield)
            || body.modal.is_some()
        {
            return None;
        }
        let e = std::mem::take(&mut body.effect);
        body.effect = Effect::seq(vec![
            e,
            Effect::NoteLinked {
                what: Sel::Var(vars::IT),
                replace: true,
            },
        ]);
        out.push(AbilityDef::with_link(kind, a.text.clone(), a.link));
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "trigger grammar 4: return a creature card to the battlefield (noted for linked abilities)", priority: 45, parse: note_put_onto_battlefield } }

/// "exile the creature put onto the battlefield with ~": the noted permanent.
fn the_put_creature(l: &str, b: &mut Builder) -> Option<Effect> {
    if !l.contains(PUT_WITH) || b.named.iter().any(|(n, _)| n == PUT_WITH) {
        return None;
    }
    b.named.push((PUT_WITH.into(), Sel::LinkedNoted));
    let e = crate::oracle::effects::parse_sentence(l, b);
    b.named.retain(|(n, _)| n != PUT_WITH);
    e
}

inventory::submit! { EffectPattern { name: "trigger grammar 4: the creature put onto the battlefield with ~", priority: 30, parse: the_put_creature } }

/// "the creature put onto the battlefield with ~ dies / leaves the battlefield".
fn put_creature_event(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(t).strip_prefix(PUT_WITH)?;
    let (c, it, p) = parse_trigger_condition(&format!("whenever a creature{r}"))?;
    // The noted permanent as it last existed on the battlefield (it has left).
    let noted = |f: Filter| {
        Filter::and(vec![f, Filter::Custom(crate::kw::trigger_events::NOTED_AS_IT_LAST_EXISTED.into())])
    };
    let c = match c {
        TriggerCond::Dies(f) => TriggerCond::Dies(noted(f)),
        TriggerCond::LeavesBattlefield(f) => TriggerCond::LeavesBattlefield(noted(f)),
        _ => return None,
    };
    Some((c, it, p))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: the creature put onto the battlefield with ~ dies", priority: 100, parse: put_creature_event } }

/// "[a player / you] play(s) a card exiled with ~".
fn plays_card_exiled_with(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let who = if t == "a player plays a card exiled with ~" {
        PlayerRel::Any
    } else if t == "you play a card exiled with ~" {
        PlayerRel::You
    } else {
        return None;
    };
    let exiled = || Filter::Custom(crate::kw::trigger_events::WAS_EXILED_WITH_SOURCE.into());
    Some((
        TriggerCond::AnyOf(vec![
            TriggerCond::LandPlayed {
                who,
                filter: Filter::and(vec![Filter::Type(crate::types::CardType::Land), exiled()]),
            },
            TriggerCond::CastSpell {
                who,
                filter: Filter::and(vec![Filter::Spell, exiled()]),
            },
        ]),
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: a player plays a card exiled with ~", priority: 100, parse: plays_card_exiled_with } }

/// "you play a card with two or more card types" (Rendmaw, Creaking Nest): a land played
/// or a spell cast, described by the card's qualities.
fn plays_qualified_card(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let (who, q) = if let Some(q) = t.strip_prefix("you play a card ") {
        ("you play a card", q)
    } else {
        ("a player plays a card", t.strip_prefix("a player plays a card ")?)
    };
    if q.contains("exiled with") {
        return None;
    }
    let phrase = format!("card {q}");
    let (f, plural, tail) = parse_object_phrase(&phrase)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    let (c, it, p) = parse_trigger_condition(&format!("whenever {who}"))?;
    let TriggerCond::AnyOf(v) = c else {
        return None;
    };
    let v = v
        .into_iter()
        .map(|c| match c {
            TriggerCond::CastSpell { who, filter } => Some(TriggerCond::CastSpell {
                who,
                filter: Filter::and(vec![filter, f.clone()]),
            }),
            TriggerCond::LandPlayed { who, filter } => Some(TriggerCond::LandPlayed {
                who,
                filter: Filter::and(vec![filter, f.clone()]),
            }),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some((TriggerCond::AnyOf(v), it, p))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: you play a card [with qualities]", priority: 100, parse: plays_qualified_card } }

/// "one or more [other] permanents [you control] phase out", "[another] permanent phases
/// out" (CR 702.26b; such abilities look back in time, CR 603.10b).
fn phase_out(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let (batch, np) = if let Some(r) = t.strip_prefix("one or more ") {
        (true, r.strip_suffix(" phase out")?)
    } else {
        (false, t.strip_suffix(" phases out")?)
    };
    if np == "~" {
        return None;
    }
    let np = np
        .strip_prefix("a ")
        .or_else(|| np.strip_prefix("an "))
        .unwrap_or(np);
    let (f, plural, tail) = parse_object_phrase(np)?;
    if plural != batch
        || !end(tail).is_empty()
        || f.zone().is_some_and(|z| z != ZoneKind::Battlefield)
    {
        return None;
    }
    let c = TriggerCond::PhasesOut(Filter::and(vec![f, Filter::Permanent]));
    let c = if batch {
        TriggerCond::Batched {
            trigger: Box::new(c),
            per: BatchPer::Batch,
        }
    } else {
        c
    };
    Some((c, Sel::TriggerObject, PlayerRef::TriggerPlayer))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: [permanents] phase out", priority: 100, parse: phase_out } }

/// "one or more other permanents phase out and whenever one or more other cards are put
/// into exile from anywhere" (The War Doctor): two "one or more" trigger conditions, each
/// of a batch of simultaneous events (the batched alternative a batch's events match; see
/// the interpretation `batched-alternatives-one-batch`).
fn batched_conjunction(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let (a, b) = t.split_once(" and whenever ")?;
    if !a.starts_with("one or more ") || !b.starts_with("one or more ") {
        return None;
    }
    let (ca, _, _) = parse_trigger_condition(&format!("whenever {a}"))?;
    let (cb, _, _) = parse_trigger_condition(&format!("whenever {b}"))?;
    let batched = |c: &TriggerCond| matches!(c, TriggerCond::Batched { .. });
    if !batched(&ca) || !batched(&cb) {
        return None;
    }
    Some((TriggerCond::AnyOf(vec![ca, cb]), Sel::None, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: one or more [A] and whenever one or more [B]", priority: 100, parse: batched_conjunction } }

/// "the final chapter ability of a Saga you control triggers" (Historian's Boon).
fn final_chapter_triggers(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(t) == "the final chapter ability of a saga you control triggers").then(|| {
        (
            TriggerCond::Custom(crate::kw::trigger_events::FINAL_CHAPTER_TRIGGERS.into()),
            Sel::TriggerObject,
            PlayerRef::You,
        )
    })
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: the final chapter ability of a Saga you control triggers", priority: 100, parse: final_chapter_triggers } }

/// The condition "while it's exiled" on the source.
fn source_exiled() -> Condition {
    Condition::SelMatches(Sel::This, Filter::InZone(ZoneKind::Exile))
}

/// Whether the trigger event requires its source to be in exile ("while it's exiled"): it
/// functions there (CR 113.6b).
pub(crate) fn requires_source_exiled(c: &TriggerCond) -> bool {
    match c {
        TriggerCond::Where { cond, .. } => {
            let s = |c: &Condition| matches!(c, Condition::SelMatches(Sel::This, Filter::InZone(ZoneKind::Exile)));
            s(cond) || matches!(cond, Condition::And(v) if v.iter().any(s))
        }
        _ => false,
    }
}

/// "you remove a time counter from ~ while it's exiled" (Watcher of Hours): the counter is
/// removed by you (the effect's controller or the player paying a cost, CR 122.8) from the
/// card in exile (each counter: "a time counter").
fn you_remove_counter_while_exiled(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(t).strip_prefix("you remove a ")?;
    let (kind, r) = r.split_once(" counter from ~")?;
    if r != " while it's exiled" || kind.contains(' ') {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::CountersRemoved {
                filter: Filter::Source,
                kind: Some(kind.into()),
                each: true,
            }),
            cond: Condition::And(vec![
                Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::You),
                source_exiled(),
            ]),
        },
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: you remove a [kind] counter from ~ while it's exiled", priority: 100, parse: you_remove_counter_while_exiled } }

/// "When you gain control of ~ from another player" (Risky Move): its control changes to
/// you (a control change is always from another player, CR 108.4).
fn gain_control_of_self(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(t);
    if r != "you gain control of ~ from another player" && r != "you gain control of ~" {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::ControlChanged(Filter::Source)),
            cond: Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::You),
        },
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: you gain control of ~ from another player", priority: 100, parse: gain_control_of_self } }

/// "choose a creature you control and an opponent": the object, then the player (each
/// choice made as written, CR 608.2c), with their referents ("that creature", "that
/// opponent").
fn choose_object_and_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (obj, player) = ["an opponent", "a player", "target opponent", "target player"]
        .iter()
        .find_map(|p| r.strip_suffix(&format!(" and {p}")).map(|o| (o, *p)))?;
    if obj.contains(" and ") {
        return None;
    }
    let a = crate::oracle::effects::parse_sentence(&format!("choose {obj}"), b)?;
    let c = crate::oracle::effects::parse_sentence(&format!("choose {player}"), b)?;
    Some(Effect::seq(vec![a, c]))
}

inventory::submit! { EffectPattern { name: "trigger grammar 4: choose [an object] and [a player]", priority: 100, parse: choose_object_and_player } }

/// "~ enters attached to a creature" (Infinite Reflection): "that creature" is the one it
/// entered attached to.
fn enters_attached_to_creature(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(t) == "~ enters attached to a creature").then(|| {
        (
            TriggerCond::Custom(crate::kw::trigger_events::ENTERS_ATTACHED_TO_CREATURE.into()),
            Sel::TriggerOtherObject,
            PlayerRef::You,
        )
    })
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: ~ enters attached to a creature", priority: 100, parse: enters_attached_to_creature } }

/// "~ becomes tapped to pay a teamwork cost" (Agent Maria Hill), "you cast a spell using
/// teamwork" (Virtual Assistant): CR 702.194a-b.
fn teamwork_events(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    match end(t) {
        "~ becomes tapped to pay a teamwork cost" => Some((
            TriggerCond::Custom(crate::kw::teamwork::SELF_TAPPED_FOR_TEAMWORK.into()),
            Sel::This,
            PlayerRef::You,
        )),
        "you cast a spell using teamwork" => Some((
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::and(vec![
                    Filter::Spell,
                    Filter::CastWithCost(crate::kw::teamwork::TEAMWORK.into()),
                ]),
            },
            Sel::TriggerSpell,
            PlayerRef::You,
        )),
        _ => None,
    }
}

inventory::submit! { TriggerPattern { name: "trigger grammar 4: teamwork events", priority: 100, parse: teamwork_events } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_caused_events() {
        assert_eq!(clause("a player to shuffle their library").as_deref(), Some("a player shuffles their library"));
        assert_eq!(
            clause("a land to be put into your graveyard from the battlefield").as_deref(),
            Some("a land is put into your graveyard from the battlefield")
        );
        for t in [
            "a spell or ability causes a player to shuffle their library",
            "a spell or ability causes its controller to shuffle their library",
            "a spell or ability an opponent controls causes a land to be put into your graveyard from the battlefield",
            "a white instant or sorcery spell causes you to gain life",
        ] {
            assert!(
                matches!(caused_event(t), Some((TriggerCond::Where { .. }, _, _))),
                "{t}"
            );
        }
        assert!(caused_event("a creature causes you to gain life").is_none());
    }

    #[test]
    fn reads_untap_step_events() {
        for t in [
            "you untap one or more permanents during your untap step",
            "~ becomes untapped during your untap step",
        ] {
            assert!(matches!(during_untap_step(t), Some((TriggerCond::Where { .. }, _, _))), "{t}");
        }
        assert!(during_untap_step("you draw a card during your untap step").is_none());
    }

    #[test]
    fn reads_exact_amounts() {
        let (c, _, _) = damage_amounts(
            "another source you control deals exactly 1 damage to a permanent or player",
        )
        .expect("ghyrson");
        assert!(matches!(c, TriggerCond::AnyOf(ref v) if v.len() == 2));
        assert!(damage_amounts("a source you control deals damage to a creature").is_none());
        assert!(matches!(
            each_lose_life("one or more opponents each lose exactly 1 life"),
            Some((TriggerCond::Where { .. }, _, _))
        ));
    }
}
