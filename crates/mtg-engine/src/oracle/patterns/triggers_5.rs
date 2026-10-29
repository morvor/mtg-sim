//! Triggers, grammar V (cards4-triggers-durations): delayed triggered abilities and
//! triggers created by effects (CR 603.7).
//!
//! - "create X 1/1 Sand Warrior creature tokens ... at the beginning of your next upkeep,
//!   where X is the number of lands you control at that time" (Hazezon Tamar): a value
//!   the delayed ability determines as it resolves (CR 603.7c, 608.2h).

use super::{EffectPattern, FollowupPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;

/// "[instruction] at the beginning of your next upkeep, where X is [value] at that time":
/// the delayed ability's instruction, with X determined as it resolves (the core grammar
/// reads the X of a delayed instruction inside its body).
fn where_x_at_that_time(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let head = l.strip_suffix(" at that time")?;
    let (instruction, value) = head.split_once(", where x is ")?;
    let (trigger, inner) = super::triggers_delayed::split_delay(instruction)?;
    let saved = (b.targets.len(), b.it.clone());
    let effect = parse_effect_text(&format!("{inner}, where x is {value}"), b);
    // The delayed ability's own instruction: nothing it refers to comes from the creating
    // ability (no targets, objects or event of its own).
    let json = effect
        .as_ref()
        .and_then(|e| serde_json::to_string(e).ok())
        .unwrap_or_default();
    if effect.is_none()
        || b.targets.len() != saved.0
        || ["Target", "Trigger", "EventAmount", "\"This\""]
            .iter()
            .any(|p| json.contains(p))
    {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    Some(Effect::DelayedTrigger {
        trigger,
        body: Box::new(Body::effect(effect?)),
        once: true,
    })
}

inventory::submit! { EffectPattern { name: "triggers 5: delayed instruction, where X is [value] at that time", priority: 100, parse: where_x_at_that_time } }

/// "If it's a creature, put a +1/+1 counter on it for each 1 damage prevented this way at
/// the beginning of the next end step." (Scars of the Veteran): an instruction with its
/// delay written after it, about what the previous instruction did ("it", "this way"),
/// read as the delay written first ("At the beginning of the next end step, put ...",
/// Sacred Boon), with a leading condition kept in front.
fn delay_after_referent_instruction(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !l.contains(" this way ") {
        return false;
    }
    let (lead, rest) = match l.strip_prefix("if ").and_then(|r| r.split_once(", ")) {
        Some((c, r)) => (format!("if {c}, "), r),
        None => (String::new(), l),
    };
    let Some(delay) = [
        "at the beginning of the next end step",
        "at the beginning of the next cleanup step",
        "at the beginning of your next upkeep",
    ]
    .into_iter()
    .find(|d| rest.ends_with(&format!(" {d}") as &str)) else {
        return false;
    };
    let instruction = rest[..rest.len() - delay.len()].trim_end();
    if instruction.contains(", ") {
        return false;
    }
    crate::oracle_ext::apply_followup_ext(&format!("{lead}{delay}, {instruction}"), prev, b)
}

inventory::submit! { FollowupPattern { name: "triggers 5: [instruction about this way] at the beginning of the next step", priority: 100, apply: delay_after_referent_instruction } }

/// One event of "when you next [events] this turn": "cast an instant spell", "activate a
/// loyalty ability", "activate an ability that isn't a mana ability".
fn next_event(e: &str) -> Option<TriggerCond> {
    if let Some(spell) = e.strip_prefix("cast ") {
        let (cast, _, _) = crate::oracle::triggers::parse_trigger_condition(&format!(
            "whenever you cast {spell}"
        ))?;
        return matches!(
            cast,
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                ..
            }
        )
        .then_some(cast);
    }
    let ability = e.strip_prefix("activate ")?;
    let activated = TriggerCond::AbilityActivated {
        who: PlayerRel::You,
        source: Filter::Any,
        include_mana: false,
    };
    let with = |name: &str| TriggerCond::Where {
        trigger: Box::new(activated.clone()),
        cond: Condition::SelMatches(Sel::TriggerSpell, Filter::Custom(name.into())),
    };
    Some(match ability {
        "an ability" | "an ability that isn't a mana ability" => activated,
        "a loyalty ability" => with(crate::stack_ability_filters::LOYALTY_ABILITY),
        "an ability with {x} in its activation cost" => {
            with(crate::stack_ability_filters::X_IN_ACTIVATION_COST)
        }
        _ => return None,
    })
}

/// "When you next cast an instant spell, cast a sorcery spell, or activate a loyalty
/// ability this turn, copy that spell or ability twice." (Repeated Reverberation), "When
/// you next activate an ability that isn't a mana ability this turn by spending four or
/// more mana to activate it, copy that ability." (Dynaheir, Invoker Adept): a delayed
/// triggered ability that triggers once, on the first of those events this turn
/// (CR 603.7b, 603.7c).
fn when_you_next(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("when you next ")?;
    let (events, rest) = r.split_once(" this turn")?;
    // "... by spending four or more mana to activate it, ..."
    let (spent, eff) = match rest.strip_prefix(", ") {
        Some(e) => (None, e),
        None => {
            let r = rest.strip_prefix(" by spending ")?;
            let (n, r) = crate::oracle::phrases::parse_number(r)?;
            let Value::Const(n) = n else { return None };
            let eff = r.trim_start().strip_prefix("or more mana to activate it, ")?;
            (Some(n), eff)
        }
    };
    let items: Vec<&str> = events
        .split(", ")
        .flat_map(|x| x.split(" or "))
        .map(|x| x.strip_prefix("or ").unwrap_or(x).trim())
        .filter(|x| !x.is_empty())
        .collect();
    // A list of at least one spell and one ability, or a single event, is read here; the
    // plain "cast [spell] or activate [ability]" pair also by `next_cast_or_activate`.
    let mut conds: Vec<TriggerCond> = items.iter().map(|e| next_event(e)).collect::<Option<_>>()?;
    let any_ability = conds
        .iter()
        .any(|c| !matches!(c, TriggerCond::CastSpell { .. }));
    if let Some(n) = spent {
        // Only about an activated ability.
        if conds.len() != 1 || !any_ability {
            return None;
        }
        let c = conds.pop()?;
        conds.push(TriggerCond::Where {
            trigger: Box::new(c),
            cond: Condition::Custom(
                format!("{}{n}", crate::kw::activation_mana_spent::MANA_SPENT_TO_ACTIVATE).into(),
            ),
        });
    }
    if !any_ability {
        return None;
    }
    let trigger = if conds.len() == 1 {
        conds.pop()?
    } else {
        TriggerCond::AnyOf(conds)
    };
    let body =
        crate::oracle::effects::parse_trigger_body(eff, b.ctx, Sel::TriggerSpell, PlayerRef::You)?;
    if body.modal.is_some() {
        return None;
    }
    Some(Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(trigger)),
        body: Box::new(body),
        once: true,
    })
}

inventory::submit! { EffectPattern { name: "triggers 5: when you next [cast a spell or activate an ability, ...] this turn", priority: 105, parse: when_you_next } }

/// "copy the next loyalty ability you activate this turn when you activate it" (Jaya's
/// Phoenix): "when you next activate a loyalty ability this turn, copy that ability".
fn copy_the_next_ability(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("copy the next ")?;
    let (ability, rest) = r.split_once(" you activate this turn when you activate it")?;
    let rest = rest.trim_start();
    let tail = if rest.is_empty() {
        String::new()
    } else {
        format!(" {rest}")
    };
    when_you_next(
        &format!("when you next activate a {ability} this turn, copy that ability{tail}"),
        b,
    )
}

inventory::submit! { EffectPattern { name: "triggers 5: copy the next [ability] you activate this turn when you activate it", priority: 105, parse: copy_the_next_ability } }

/// "Whenever a player casts an instant or sorcery card" (Eye of the Storm): a spell cast
/// from a card, so not a copy of a card (CR 601.2a, 707.12; a copy cast doesn't trigger
/// it).
fn casts_a_card(l: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let l = end(l);
    let head = l.strip_suffix(" card")?;
    if !(head.contains(" cast a ") || head.contains(" casts a ") || head.contains(" cast an ") || head.contains(" casts an ")) {
        return None;
    }
    let (t, it, p) = crate::oracle::triggers::parse_trigger_condition(&format!("whenever {head} spell"))?;
    let TriggerCond::CastSpell { who, filter } = t else {
        return None;
    };
    Some((
        TriggerCond::CastSpell {
            who,
            filter: Filter::and(vec![filter, Filter::Card]),
        },
        it,
        p,
    ))
}

inventory::submit! { TriggerPattern { name: "triggers 5: [player] casts [a quality] card", priority: 100, parse: casts_a_card } }

/// "Then that player copies each instant or sorcery card exiled with ~." (Eye of the
/// Storm) after "exile it": the exile is linked to the ability (CR 607.2a), and the player
/// the trigger event names makes a copy of each card of that kind exiled with it that's
/// still in exile, so they own and control the copies (CR 707.12).
fn that_player_copies_each(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let l = end(&lower);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some(r) = l
        .strip_prefix("that player copies each ")
        .and_then(|r| r.strip_suffix(" exiled with ~"))
    else {
        return false;
    };
    if !matches!(b.it_player, PlayerRef::TriggerPlayer) {
        return false;
    }
    let Some((f, false, tail)) = crate::oracle::phrases::parse_object_phrase(r) else {
        return false;
    };
    if !end(tail).is_empty() || f.zone().is_some() {
        return false;
    }
    fn link_last_exile(e: &mut Effect) -> bool {
        match e {
            Effect::Exile {
                face_down: false,
                link,
                ..
            } => {
                *link = true;
                true
            }
            Effect::Seq(v) => v.last_mut().is_some_and(link_last_exile),
            _ => false,
        }
    }
    if !link_last_exile(prev) {
        return false;
    }
    let copy = Effect::AsPlayer {
        who: PlayerRef::TriggerPlayer,
        effect: Box::new(Effect::CopyCard {
            what: Sel::All(Filter::and(vec![
                Filter::In(Box::new(Sel::Linked)),
                Filter::InZone(ZoneKind::Exile),
                f,
            ])),
            named: None,
        }),
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, copy]);
    true
}

inventory::submit! { FollowupPattern { name: "triggers 5: then that player copies each [card] exiled with ~", priority: 44, apply: that_player_copies_each } }

/// "For each copy, the player may cast the copy without paying its mana cost." (Eye of
/// the Storm): the player who made the copies casts any of them, in the order they choose.
fn the_player_may_cast_each_copy(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !matches!(b.it_player, PlayerRef::TriggerPlayer)
        || !matches!(
            l,
            "for each copy, the player may cast the copy without paying its mana cost"
                | "for each copy, that player may cast the copy without paying its mana cost"
        )
    {
        return None;
    }
    let effect = crate::oracle::effects::parse_sentence(
        "you may cast any number of the copies without paying their mana costs",
        b,
    )?;
    Some(Effect::AsPlayer {
        who: PlayerRef::TriggerPlayer,
        effect: Box::new(effect),
    })
}

inventory::submit! { EffectPattern { name: "triggers 5: for each copy, the player may cast the copy", priority: 100, parse: the_player_may_cast_each_copy } }

/// "Whenever one or more creatures you control and/or creature cards in your graveyard
/// are put into exile" (Kaya, Spirits' Justice): one trigger event for a batch of either
/// kind of zone change, or both (CR 603.2c).
fn exiled_and_or(l: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let l = end(l);
    let r = l.strip_prefix("one or more ")?;
    let (subjects, verb) = r.split_once(" are put into exile")?;
    if !verb.is_empty() {
        return None;
    }
    let (a, b) = subjects.split_once(" and/or ")?;
    let mut inner = Vec::new();
    let mut player = PlayerRef::You;
    for part in [a, b] {
        let phrase = match part.strip_suffix(" in your graveyard") {
            Some(cards) => format!("whenever one or more {cards} are put into exile from your graveyard"),
            None => format!("whenever one or more {part} are put into exile"),
        };
        let (t, _, p) = crate::oracle::triggers::parse_trigger_condition(&phrase)?;
        let TriggerCond::Batched {
            trigger,
            per: BatchPer::Batch,
        } = t
        else {
            return None;
        };
        if !matches!(*trigger, TriggerCond::ZoneChange { .. }) {
            return None;
        }
        inner.push(*trigger);
        player = p;
    }
    // "them": the objects of the batch.
    Some((
        TriggerCond::Batched {
            trigger: Box::new(TriggerCond::AnyOf(inner)),
            per: BatchPer::Batch,
        },
        Sel::TriggerObjects,
        player,
    ))
}

inventory::submit! { TriggerPattern { name: "triggers 5: one or more [objects] and/or [cards in your graveyard] are put into exile", priority: 100, parse: exiled_and_or } }

/// The card chosen by "choose a creature card from among them".
const CHOSEN_AMONG: Var = vars::USER + 43101;

/// "You may choose a creature card from among them." (Kaya, Spirits' Justice): one of the
/// objects the instruction or trigger event before named, of that kind, chosen as the
/// ability resolves; "it" is then that card.
fn choose_a_card_from_among_them(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (up_to, r) = match l.strip_prefix("you may choose ") {
        Some(r) => (true, r),
        None => (false, l.strip_prefix("choose ")?),
    };
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let noun = r.strip_suffix(" from among them")?;
    // The objects of the batch of events that triggered the ability.
    let them = match super::pronoun_groups::plural_object_ref("them", b) {
        Some(Some((sel, _))) => sel,
        _ if matches!(b.it, Sel::TriggerObjects) => Sel::TriggerObjects,
        _ => return None,
    };
    if !matches!(them, Sel::TriggerObjects) {
        return None;
    }
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() || f.zone().is_some() {
        return None;
    }
    let choose = Sel::Choose {
        chooser: PlayerRef::You,
        filter: Filter::and(vec![Filter::In(Box::new(them)), f]),
        count: Value::c(1),
        up_to,
        store: None,
    };
    b.it = Sel::Var(CHOSEN_AMONG);
    Some(Effect::Store {
        var: CHOSEN_AMONG,
        sel: choose,
    })
}

inventory::submit! { EffectPattern { name: "triggers 5: choose a [card] from among them", priority: 100, parse: choose_a_card_from_among_them } }

/// The cards exiled with the source that are still in exile (CR 607.2a).
fn exiled_with_source() -> Filter {
    Filter::and(vec![
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ])
}

/// "Whenever you play a card with the same name as one of the exiled cards" (Search the
/// City): playing a land or casting a spell from a card (its ruling: "To “play a card” is
/// to play that card as a land or cast that card as a spell") that shares a name with a
/// card exiled with the source (CR 607.2a).
fn play_a_card_named_like_exiled(l: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let head = end(l).strip_suffix(" with the same name as one of the exiled cards")?;
    let (t, it, p) = crate::oracle::triggers::parse_trigger_condition(&format!("whenever {head}"))?;
    let named = Filter::SameNameAs(Box::new(Sel::All(exiled_with_source())));
    let TriggerCond::AnyOf(v) = t else {
        return None;
    };
    let v: Option<Vec<TriggerCond>> = v
        .into_iter()
        .map(|c| match c {
            TriggerCond::CastSpell { who, filter } => Some(TriggerCond::CastSpell {
                who,
                filter: Filter::and(vec![filter, named.clone()]),
            }),
            TriggerCond::LandPlayed { who, filter } => Some(TriggerCond::LandPlayed {
                who,
                filter: Filter::and(vec![filter, named.clone()]),
            }),
            _ => None,
        })
        .collect();
    Some((TriggerCond::AnyOf(v?), it, p))
}

inventory::submit! { TriggerPattern { name: "triggers 5: you play a card with the same name as one of the exiled cards", priority: 100, parse: play_a_card_named_like_exiled } }

/// "you may put one of those cards with that name into its owner's hand" after "Whenever
/// you play a card with the same name as one of the exiled cards" (Search the City): one
/// of the cards exiled with the source with the name of the card played.
fn one_of_those_cards_with_that_name(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("put one of those cards with that name ")?;
    if !crate::oracle::raw_text()
        .to_lowercase()
        .contains("with the same name as one of the exiled cards, ")
    {
        return None;
    }
    crate::oracle::effects::parse_sentence(&format!("put a card exiled with ~ with that name {r}"), b)
}

inventory::submit! { EffectPattern { name: "triggers 5: put one of those cards with that name", priority: 100, parse: one_of_those_cards_with_that_name } }

/// "Then if there are no cards exiled with ~, sacrifice it." (Search the City): "it" is
/// the object the condition names, the source (not the card the instruction before moved).
fn if_none_exiled_with_source_sacrifice_it(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let l = end(&lower);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "if there are no cards exiled with ~, sacrifice it" {
        return false;
    }
    let Some(sacrifice) = crate::oracle::effects::parse_sentence("sacrifice ~", b) else {
        return false;
    };
    let none_left = Condition::Compare(
        Value::Count(Filter::and(vec![Filter::Card, exiled_with_source()])),
        Cmp::Eq,
        Value::c(0),
    );
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: none_left,
            then: Box::new(sacrifice),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "triggers 5: then if there are no cards exiled with ~, sacrifice it", priority: 40, apply: if_none_exiled_with_source_sacrifice_it } }

/// "that permanent phases in", "it phases in" (CR 702.26c): the permanent named before.
fn that_permanent_phases_in(l: &str, b: &mut Builder) -> Option<Effect> {
    let subject = end(l).strip_suffix(" phases in")?;
    if !matches!(subject, "that permanent" | "it" | "that creature")
        || !matches!(b.it, Sel::Target(_) | Sel::Var(_) | Sel::TriggerObject)
    {
        return None;
    }
    Some(Effect::PhaseIn { what: b.it.clone() })
}

inventory::submit! { EffectPattern { name: "triggers 5: that permanent phases in", priority: 100, parse: that_permanent_phases_in } }

/// "When ~ becomes untapped or leaves the battlefield, that permanent phases in." (The
/// Pandorica), "When ~ leaves the battlefield or becomes untapped, remove all -1/-1
/// counters from the creature." (Giant Oyster): a delayed triggered ability created as the
/// ability resolves (CR 603.7a), triggering once on the first of those events of the
/// source (CR 603.7c), about the object the ability named (captured as it's created).
fn when_source_event_or_event(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("when ~ ")?;
    let (events, eff) = r.split_once(", ")?;
    let parts: Vec<&str> = events.split(" or ").collect();
    if parts.len() < 2 || !matches!(b.it, Sel::Target(_) | Sel::Var(_)) {
        return None;
    }
    let conds: Option<Vec<TriggerCond>> = parts
        .iter()
        .map(|p| match *p {
            "becomes untapped" => Some(TriggerCond::BecomesUntapped(Filter::Source)),
            "leaves the battlefield" => Some(TriggerCond::LeavesBattlefield(Filter::Source)),
            _ => None,
        })
        .collect();
    let saved = (b.targets.len(), b.it.clone());
    let effect = parse_effect_text(eff, b);
    let effect = match effect {
        Some(e) if b.targets.len() == saved.0 => e,
        _ => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            return None;
        }
    };
    let (mut stores, effect) = super::triggers_delayed::capture(&effect)?;
    stores.push(Effect::DelayedTrigger {
        trigger: TriggerCond::AnyOf(conds?),
        body: Box::new(Body::effect(effect)),
        once: true,
    });
    Some(Effect::seq(stores))
}

inventory::submit! { EffectPattern { name: "triggers 5: when ~ [event] or [event], [effect] (delayed)", priority: 100, parse: when_source_event_or_event } }

/// "When ~ is put into exile, create ..." (granted by Elemental Expressionist): a
/// permanent's ability functions only on the battlefield (CR 113.6), so it triggers on
/// the permanent being exiled from the battlefield, looking back in time (CR 603.10a), as
/// "dies or is put into exile" does (Kaya's Ghostform).
fn source_put_into_exile(l: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    if end(l) != "~ is put into exile" {
        return None;
    }
    crate::oracle::triggers::parse_trigger_condition("when ~ is put into exile from the battlefield")
}

inventory::submit! { TriggerPattern { name: "triggers 5: ~ is put into exile", priority: 100, parse: source_put_into_exile } }

/// The permanent the delayed ability put onto the battlefield, and the source, for its own
/// "when you lose control of ~" ability.
const RETURNED: Var = vars::USER + 43102;
const RETURNED_BY: Var = vars::USER + 43103;

/// "Sacrifice the creature when you lose control of ~." after "put that card onto the
/// battlefield under your control at the beginning of the next end step" (Seraph): as
/// the delayed ability puts the card onto the battlefield, it creates another delayed
/// ability that triggers the next time its controller loses control of the source
/// (CR 603.7a, 603.10d), sacrificing that permanent if they still control it (CR 701.21a).
/// If the source has left the battlefield by then, it's a new object it never triggers
/// for (Seraph ruling).
fn sacrifice_when_you_lose_control(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    if end(&lower) != "sacrifice the creature when you lose control of ~" {
        return false;
    }
    fn returns(e: &mut Effect) -> Option<&mut Body> {
        match e {
            Effect::Seq(v) => v.last_mut().and_then(returns),
            Effect::DelayedTrigger { body, once: true, .. }
                if matches!(&body.effect, Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield) =>
            {
                Some(body)
            }
            _ => None,
        }
    }
    let Some(body) = returns(prev) else {
        return false;
    };
    let mv = std::mem::take(&mut body.effect);
    body.effect = Effect::seq(vec![
        mv,
        Effect::Store {
            var: RETURNED,
            sel: Sel::Var(vars::IT),
        },
        Effect::Store {
            var: RETURNED_BY,
            sel: Sel::This,
        },
        Effect::DelayedTrigger {
            trigger: TriggerCond::LoseControl(Filter::In(Box::new(Sel::Var(RETURNED_BY)))),
            body: Box::new(Body::effect(Effect::If {
                cond: Condition::SelMatches(
                    Sel::Var(RETURNED),
                    Filter::ControlledBy(PlayerRel::You),
                ),
                then: Box::new(Effect::SacrificeObjects {
                    what: Sel::Var(RETURNED),
                }),
                otherwise: Box::new(Effect::Noop),
            })),
            once: true,
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "triggers 5: sacrifice the creature when you lose control of ~ (after a delayed return)", priority: 40, apply: sacrifice_when_you_lose_control } }
