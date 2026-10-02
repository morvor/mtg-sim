//! Trigger grammar for events (CR 603.2, 603.6, 603.8, 603.10): qualifiers on trigger
//! events, state triggers, and subjects and events the compositional parser in
//! `triggers.rs` doesn't cover.
//!
//! * Qualifiers after any trigger event: `[event] during your turn`, `during an
//!   opponent's turn`, `during their turn`, `for the first time each turn`, `for the first
//!   time during each of your/their turns`, `while [condition]` ("while ~ is in your
//!   graveyard": the ability functions from the graveyard, CR 113.6; "while ~ has a -1/-1
//!   counter on it"). The qualifier is part of the trigger event, checked as it occurs
//!   (unlike an intervening "if" clause, CR 603.4). In a "one or more" batch (CR 603.2c)
//!   it qualifies each event of the batch, and "for the first time each turn" means the
//!   first batch with a matching event this turn.
//! * State triggers (CR 603.8): `When [game state]` — "when there are four or more page
//!   counters on ~", "when ~ has no ice counters on it", "when you have 40 or more life",
//!   "when an opponent controls a creature with power 4 or greater", using the condition
//!   grammar.

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::patterns::triggers::event_condition;
use crate::oracle::phrases::*;

type Parsed = (TriggerCond, Sel, PlayerRef);

/// A complete trigger condition (the text after "when"/"whenever").
fn reparse(s: &str) -> Option<Parsed> {
    crate::oracle::triggers::parse_trigger_condition(&format!("whenever {s}"))
}

/// Qualifies a trigger's events with a condition checked as each event occurs (inside a
/// batch, each event of the batch).
pub(crate) fn where_events(c: TriggerCond, cond: Condition) -> TriggerCond {
    match c {
        TriggerCond::Batched { trigger, per } => TriggerCond::Batched {
            trigger: Box::new(where_events(*trigger, cond)),
            per,
        },
        // A condition on the whole batch stays outside.
        TriggerCond::Where {
            trigger,
            cond: batch_cond,
        } if matches!(*trigger, TriggerCond::Batched { .. }) => TriggerCond::Where {
            trigger: Box::new(where_events(*trigger, cond)),
            cond: batch_cond,
        },
        c => TriggerCond::Where {
            trigger: Box::new(c),
            cond,
        },
    }
}

/// "… for the first time each turn": the first matching event (for a "one or more"
/// batch, the first batch with a matching event) this turn.
pub(crate) fn first_time(c: TriggerCond) -> TriggerCond {
    match c {
        TriggerCond::Batched { trigger, per } => TriggerCond::Batched {
            trigger: Box::new(TriggerCond::FirstTimeEachTurn(trigger)),
            per,
        },
        TriggerCond::Where { trigger, cond } if matches!(*trigger, TriggerCond::Batched { .. }) => {
            TriggerCond::Where {
                trigger: Box::new(first_time(*trigger)),
                cond,
            }
        }
        c => TriggerCond::FirstTimeEachTurn(Box::new(c)),
    }
}

/// Whether a trigger event is qualified by "while ~ is in your graveyard": the ability
/// functions from the graveyard (CR 113.6).
pub(crate) fn requires_source_in_graveyard(c: &TriggerCond) -> bool {
    match c {
        TriggerCond::Where { trigger, cond } => {
            super::graveyard_order::requires_source_in_graveyard(cond)
                || requires_source_in_graveyard(trigger)
        }
        TriggerCond::Batched { trigger, .. } | TriggerCond::FirstTimeEachTurn(trigger) => {
            requires_source_in_graveyard(trigger)
        }
        TriggerCond::AnyOf(v) => !v.is_empty() && v.iter().all(requires_source_in_graveyard),
        _ => false,
    }
}

/// Whether the trigger is about a source's own events ("~ becomes tapped"): pronouns in
/// a qualifier then refer to it.
fn about_source(it: &Sel) -> bool {
    matches!(it, Sel::This)
}

/// "[event] [qualifier]".
fn qualified(r: &str) -> Option<Parsed> {
    let r = end(r);
    // Turn qualifiers. ("For the first time during each of your turns": the first
    // matching event among those during your turn, so the turn condition is part of what
    // "first" counts — inside a "one or more" batch too, see `first_time`.)
    let forms: [(&str, fn(TriggerCond) -> TriggerCond); 9] = [
        // "whenever one or more lands enter under an opponent's control without being
        // played" (CR 305.1).
        (" without being played", |c| {
            where_events(
                c,
                Condition::Custom(crate::kw::trigger_event_causes::NOT_PLAYED.into()),
            )
        }),
        (" during your turn", |c| {
            where_events(c, Condition::YourTurn)
        }),
        (" during an opponent's turn", |c| {
            where_events(c, Condition::NotYourTurn)
        }),
        (" during each opponent's turn", |c| {
            where_events(c, Condition::NotYourTurn)
        }),
        (" during their turn", |c| {
            where_events(
                c,
                Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::Active),
            )
        }),
        (" for the first time each turn", first_time),
        (" for the first time during each of your turns", |c| {
            first_time(where_events(c, Condition::YourTurn))
        }),
        (" for the first time during each of their turns", |c| {
            first_time(where_events(
                c,
                Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::Active),
            ))
        }),
        (" for the first time during each opponent's turn", |c| {
            first_time(where_events(c, Condition::NotYourTurn))
        }),
    ];
    for (suffix, wrap) in forms {
        let Some(head) = r.strip_suffix(suffix) else {
            continue;
        };
        let (c, it, p) = reparse(head)?;
        // "During their turn" needs the event's player.
        if suffix == " during their turn" && !matches!(p, PlayerRef::TriggerPlayer) {
            return None;
        }
        if suffix.ends_with("each of their turns") && !matches!(p, PlayerRef::TriggerPlayer) {
            return None;
        }
        // Only an object entering can have been played.
        if suffix == " without being played" && !enters(&c) {
            return None;
        }
        // A delayed trigger's "this turn" is its duration, not a qualifier.
        if matches!(
            c,
            TriggerCond::ThisTurn(_) | TriggerCond::UntilYourNextTurn(_)
        ) {
            return None;
        }
        return Some((wrap(c), it, p));
    }
    // "[event] while [condition]".
    let (head, cond_s) = r.rsplit_once(" while ")?;
    let (c, it, p) = reparse(head)?;
    if matches!(
        c,
        TriggerCond::ThisTurn(_) | TriggerCond::UntilYourNextTurn(_)
    ) {
        return None;
    }
    let mentions_it = cond_s
        .split(|ch: char| !ch.is_alphanumeric() && ch != '\'')
        .any(|w| matches!(w, "it" | "its" | "it's" | "itself"));
    // "while ~ has a -1/-1 counter on it": the qualifier is about the source, and so is
    // its "it".
    let cond_text = if mentions_it && !cond_s.starts_with("~ ") {
        // Otherwise only the source's own events give "it" a referent: "whenever ~
        // becomes tapped while it has a -1/-1 counter on it".
        if !about_source(&it) {
            return None;
        }
        self_pronouns(cond_s)
    } else {
        cond_s.to_string()
    };
    let cond = event_condition(&cond_text)?;
    Some((where_events(c, cond), it, p))
}

/// "it" → "~", "its" → "~'s", "it's" → "~ is" (the source's own pronouns).
fn self_pronouns(s: &str) -> String {
    s.split(' ')
        .map(|w| match w {
            "it" => "~".to_string(),
            "its" => "~'s".to_string(),
            "it's" => "~ is".to_string(),
            w => w.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether a trigger condition is about permanents entering the battlefield.
fn enters(c: &TriggerCond) -> bool {
    match c {
        TriggerCond::EntersBattlefield(_) => true,
        TriggerCond::Batched { trigger, .. } | TriggerCond::Where { trigger, .. } => {
            enters(trigger)
        }
        _ => false,
    }
}

inventory::submit! { TriggerPattern { name: "[event] during your turn / for the first time / while [condition]", priority: 120, parse: qualified } }

// ---------------------------------------------------------------------------
// Rewrites into the core grammar
// ---------------------------------------------------------------------------

/// "[objects] enter(s) under an opponent's control" is "[objects] an opponent controls
/// enter(s)" (who controls a permanent is seen as it enters, CR 603.6a); "[player] puts
/// [object] onto the battlefield" is "[object] enters under [player]'s control" ("that
/// player" is its controller).
fn rewritten(r: &str) -> Option<Parsed> {
    let r = end(r);
    for (p, rep) in [
        (
            " enters under an opponent's control",
            " an opponent controls enters",
        ),
        (
            " enter under an opponent's control",
            " an opponent controls enter",
        ),
    ] {
        if let Some(head) = r.strip_suffix(p) {
            return reparse(&format!("{head}{rep}"));
        }
        // "... without being played"
        if let Some(i) = r.find(p) {
            let tail = &r[i + p.len()..];
            if tail.starts_with(' ') {
                return reparse(&format!("{}{rep}{tail}", &r[..i]));
            }
        }
    }
    if let Some(x) = r.strip_prefix("a player puts ") {
        let obj = x.strip_suffix(" onto the battlefield")?;
        let (c, it, p) = reparse(&format!("{obj} enters"))?;
        if !matches!(c, TriggerCond::EntersBattlefield(_)) {
            return None;
        }
        return Some((c, it, p));
    }
    None
}

inventory::submit! { TriggerPattern { name: "enters under an opponent's control / a player puts onto the battlefield", priority: 115, parse: rewritten } }

// ---------------------------------------------------------------------------
// Amounts of damage (CR 120)
// ---------------------------------------------------------------------------

/// "[subject] deals N or more damage to [recipient]", "deals exactly N damage to ...",
/// "[player] is dealt N or more damage by a single source": one source dealing that much
/// to one recipient at once (one damage event; Dragonborn Champion's and Pain
/// Magnification's rulings).
fn damage_amount(r: &str) -> Option<Parsed> {
    let r = end(r);
    // "an opponent is dealt 3 or more damage by a single source"
    for (p, who) in [
        ("an opponent is dealt ", PlayerRel::Opponent),
        ("a player is dealt ", PlayerRel::Any),
        ("you're dealt ", PlayerRel::You),
    ] {
        if let Some(x) = r.strip_prefix(p) {
            let (cmp, n, x) = amount(x)?;
            if x != "damage by a single source" {
                return None;
            }
            let c = TriggerCond::Where {
                trigger: Box::new(TriggerCond::DealsDamage {
                    source: Filter::Any,
                    to: DamageRecipient::Player(who),
                    combat_only: false,
                }),
                cond: Condition::Compare(Value::EventAmount, cmp, Value::c(n)),
            };
            return Some((c, Sel::TriggerOtherObject, PlayerRef::TriggerPlayer));
        }
    }
    // "~ is dealt 3 or more damage": the damage dealt to it at once, in total (Innocent
    // Bystander's ruling).
    if let Some(i) = r.find(" is dealt ") {
        let (subj, x) = (&r[..i], &r[i + " is dealt ".len()..]);
        let (cmp, n, x) = amount(x)?;
        if x != "damage" {
            return None;
        }
        let (c, it, p) = reparse(&format!("{subj} is dealt damage"))?;
        return batch_total(c, cmp, n).map(|c| (c, it, p));
    }
    let i = r.find(" deals ")?;
    let (subj, x) = (&r[..i], &r[i + " deals ".len()..]);
    let (cmp, n, x) = amount(x)?;
    // "~ deals 4 or more damage": the damage it deals at once, in total.
    if x == "damage" {
        let (c, it, p) = reparse(&format!("{subj} deals damage"))?;
        return batch_total(c, cmp, n).map(|c| (c, it, p));
    }
    let rest = x.strip_prefix("damage to ")?;
    let (c, it, p) = reparse(&format!("{subj} deals damage to {rest}"))?;
    // One source dealing damage to one recipient (not a batch's total).
    if !matches!(c, TriggerCond::DealsDamage { .. }) {
        return None;
    }
    let c = TriggerCond::Where {
        trigger: Box::new(c),
        cond: Condition::Compare(Value::EventAmount, cmp, Value::c(n)),
    };
    Some((c, it, p))
}

/// A batch of damage events whose total must be at least / exactly N: the condition is on
/// the batch (`Where` around `Batched`, see `Game::check_batch_triggers`).
fn batch_total(c: TriggerCond, cmp: Cmp, n: i32) -> Option<TriggerCond> {
    if !matches!(c, TriggerCond::Batched { .. }) {
        return None;
    }
    Some(TriggerCond::Where {
        trigger: Box::new(c),
        cond: Condition::Compare(Value::EventAmount, cmp, Value::c(n)),
    })
}

/// Whether a trigger condition is a "one or more" batch (CR 603.2c), possibly with a
/// condition on the whole batch.
pub(crate) fn is_batched(c: &TriggerCond) -> bool {
    match c {
        TriggerCond::Batched { .. } => true,
        TriggerCond::Where { trigger, .. } => matches!(**trigger, TriggerCond::Batched { .. }),
        _ => false,
    }
}

/// "N or more ", "exactly N ": (comparison, N, rest).
fn amount(x: &str) -> Option<(Cmp, i32, &str)> {
    if let Some(y) = x.strip_prefix("exactly ") {
        let (n, rest) = parse_number(y)?;
        return Some((Cmp::Eq, n.as_const()?, rest.trim_start()));
    }
    let (n, rest) = parse_number(x)?;
    let rest = rest.trim_start().strip_prefix("or more ")?;
    Some((Cmp::Ge, n.as_const()?, rest))
}

inventory::submit! { TriggerPattern { name: "deals N or more damage / is dealt N or more damage by a single source", priority: 115, parse: damage_amount } }

// ---------------------------------------------------------------------------
// Players qualified by what they control
// ---------------------------------------------------------------------------

/// "an opponent who controls an artifact named ~ draws a card": the player event,
/// for a player controlling such an object as it happens.
fn player_who_controls(r: &str) -> Option<Parsed> {
    let r = end(r);
    let (subj, x) = [
        ("an opponent who controls ", "an opponent"),
        ("a player who controls ", "a player"),
    ]
    .into_iter()
    .find_map(|(p, s)| r.strip_prefix(p).map(|x| (s, x)))?;
    let x = x.strip_prefix("a ").or_else(|| x.strip_prefix("an "))?;
    // The object phrase ends where the player's verb starts.
    for (i, ch) in x.char_indices() {
        if ch != ' ' {
            continue;
        }
        let Some((f, false, tail)) = parse_object_phrase(&x[..i]) else {
            continue;
        };
        if !end(tail).is_empty() {
            continue;
        }
        let Some((c, it, p)) = reparse(&format!("{subj} {}", &x[i + 1..])) else {
            continue;
        };
        if !matches!(p, PlayerRef::TriggerPlayer) {
            return None;
        }
        let cond = Condition::PlayerMatches(
            PlayerRef::TriggerPlayer,
            PlayerFilter::Controls(Box::new(f), Cmp::Ge, Box::new(Value::c(1))),
        );
        return Some((where_events(c, cond), it, p));
    }
    None
}

inventory::submit! { TriggerPattern { name: "a player who controls [object] [event]", priority: 115, parse: player_who_controls } }

// ---------------------------------------------------------------------------
// Counters (CR 122.6)
// ---------------------------------------------------------------------------

/// "one or more counters are put on [object]" (any kind), "one or more [kind] counters are
/// put on one or more [objects]" (once per batch, CR 603.2c), "you put one or more [kind]
/// counters on one or more [objects]", "one or more [kind] counters are removed from ~".
fn counters_events(r: &str) -> Option<Parsed> {
    let r = end(r);
    // "one or more counters are put on a creature you control"
    if let Some(x) = r.strip_prefix("one or more counters are put on ") {
        let s = crate::oracle::patterns::triggers::parse_subject(x)?;
        let c = TriggerCond::CountersPut {
            filter: s.filter,
            kind: None,
            each: false,
        };
        if s.one_or_more {
            return Some((
                TriggerCond::Batched {
                    trigger: Box::new(c),
                    per: BatchPer::Batch,
                },
                Sel::TriggerObjects,
                PlayerRef::You,
            ));
        }
        let it = if s.self_only {
            Sel::This
        } else {
            Sel::TriggerObject
        };
        return Some((c, it, PlayerRef::ControllerOf(Box::new(Sel::TriggerObject))));
    }
    // "one or more +1/+1 counters are put on one or more Humans you control"
    if let Some(x) = r.strip_prefix("one or more ") {
        if let Some((kind, rest)) = crate::oracle::costs::counter_kind(x) {
            if let Some(on) = rest
                .trim_start()
                .strip_prefix("counters are put on one or more ")
            {
                let (f, _, tail) = parse_object_phrase(on)?;
                if !end(tail).is_empty() {
                    return None;
                }
                let c = TriggerCond::CountersPut {
                    filter: f,
                    kind: Some(kind),
                    each: false,
                };
                return Some((
                    TriggerCond::Batched {
                        trigger: Box::new(c),
                        per: BatchPer::Batch,
                    },
                    Sel::TriggerObjects,
                    PlayerRef::You,
                ));
            }
            // "one or more loyalty counters are removed from ~": counters removed at the
            // same time (damage from several sources at once) are one event, and "that
            // much" is how many (Chandra, Fire Artisan's rulings).
            if let Some(on) = rest.trim_start().strip_prefix("counters are removed from ") {
                if on != "~" {
                    return None;
                }
                return Some((
                    TriggerCond::Batched {
                        trigger: Box::new(TriggerCond::CountersRemoved {
                            filter: Filter::Source,
                            kind: Some(kind),
                        }),
                        per: BatchPer::Batch,
                    },
                    Sel::This,
                    PlayerRef::You,
                ));
            }
        }
    }
    // "you put one or more +1/+1 counters on one or more other Heroes you control"
    if let Some(x) = r.strip_prefix("you put one or more ") {
        let (kind, rest) = match x.strip_prefix("counters on ") {
            Some(rest) => (None, rest),
            None => {
                let (k, rest) = crate::oracle::costs::counter_kind(x)?;
                (Some(k), rest.trim_start().strip_prefix("counters on ")?)
            }
        };
        let on = rest.strip_prefix("one or more ")?;
        let (f, _, tail) = parse_object_phrase(on)?;
        if !end(tail).is_empty() {
            return None;
        }
        let c = TriggerCond::CountersPutBy {
            who: PlayerRel::You,
            on_objects: Some(f),
            on_players: None,
            kind,
            each: false,
        };
        return Some((
            TriggerCond::Batched {
                trigger: Box::new(c),
                per: BatchPer::Batch,
            },
            Sel::TriggerObjects,
            PlayerRef::You,
        ));
    }
    None
}

inventory::submit! { TriggerPattern { name: "counters put on / removed (batches, any kind)", priority: 115, parse: counters_events } }

// ---------------------------------------------------------------------------
// Sagas (CR 714)
// ---------------------------------------------------------------------------

/// "the final chapter ability of a Saga you control resolves" (CR 608.2p, 714.2e):
/// "that Saga" is the Saga.
fn final_chapter_resolves(r: &str) -> Option<Parsed> {
    let x = end(r)
        .strip_prefix("the final chapter ability of ")?
        .strip_suffix(" resolves")?;
    let x = x.strip_prefix("a ").or_else(|| x.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(x)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    Some((
        TriggerCond::AbilityResolved {
            source: f,
            final_chapter: true,
        },
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "the final chapter ability of a Saga resolves", priority: 115, parse: final_chapter_resolves } }

// ---------------------------------------------------------------------------
// State triggers (CR 603.8)
// ---------------------------------------------------------------------------

/// "When [game state]": the subjects a game state is phrased with. Event phrasing ("~
/// is dealt damage", "a player draws a card") never starts this way.
const STATE_STARTS: [&str; 16] = [
    "there are ",
    "there is ",
    "no ",
    "~ has ",
    "~ has no ",
    "you control ",
    "you don't control ",
    "an opponent controls ",
    "a player controls ",
    "the chosen player controls ",
    "you have ",
    "an opponent has ",
    "a player has ",
    "the chosen player has ",
    "enchanted creature has ",
    "equipped creature has ",
];

fn state_trigger(r: &str) -> Option<Parsed> {
    let r = end(r);
    // "When a player other than ~'s owner controls it, that player sacrifices it" (Bronze
    // Bombshell): its controller, who controls this ability, doesn't own it.
    if r == "a player other than ~'s owner controls it" {
        let cond = Condition::SelMatches(Sel::This, Filter::not(Filter::OwnedBy(PlayerRel::You)));
        // "That player" is that controller.
        return Some((
            TriggerCond::State(cond),
            Sel::This,
            PlayerRef::ControllerOf(Box::new(Sel::This)),
        ));
    }
    if !STATE_STARTS.iter().any(|p| r.starts_with(p)) {
        return None;
    }
    // Event phrasing with these subjects ("you control a creature that becomes ...")
    // isn't a state.
    if [" becomes ", " enters", " dies", " attacks", " is dealt "]
        .iter()
        .any(|w| r.contains(w))
    {
        return None;
    }
    let cond = event_condition(r)?;
    Some((TriggerCond::State(cond), Sel::This, PlayerRef::You))
}

inventory::submit! { TriggerPattern { name: "state triggers (CR 603.8)", priority: 900, parse: state_trigger } }

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Parsed {
        crate::oracle::triggers::parse_trigger_condition(s)
            .unwrap_or_else(|| panic!("failed to parse {s:?}"))
    }

    #[test]
    fn qualifiers_parse() {
        for s in [
            "whenever another creature dies during your turn",
            "whenever a creature you control becomes tapped during your turn",
            "whenever a dragon you control dies while ~ is in your graveyard",
            "whenever ~ becomes tapped while it has a -1/-1 counter on it",
        ] {
            p(s);
        }
    }

    #[test]
    fn state_triggers_parse() {
        for s in [
            "when there are four or more page counters on ~",
            "when ~ has no ice counters on it",
            "when you have 40 or more life",
        ] {
            assert!(matches!(p(s).0, TriggerCond::State(_)), "{s}");
        }
        // Event phrasing isn't a state.
        assert!(state_trigger("you control a creature that becomes tapped").is_none());
    }

    #[test]
    fn damage_amounts() {
        // One source, one recipient: a condition on the event.
        let (c, _, _) = p("whenever ~ deals 6 or more damage to an opponent");
        assert!(
            matches!(&c, TriggerCond::Where { trigger, .. } if matches!(**trigger, TriggerCond::DealsDamage { .. })),
            "{c:?}"
        );
        // Damage dealt to it at once, in total: a condition on the batch.
        for s in [
            "whenever ~ is dealt 3 or more damage",
            "whenever ~ deals 4 or more damage",
        ] {
            let (c, _, _) = p(s);
            assert!(
                is_batched(&c) && matches!(c, TriggerCond::Where { .. }),
                "{s}: {c:?}"
            );
        }
        let (c, _, pl) = p("whenever an opponent is dealt 3 or more damage by a single source");
        assert!(matches!(c, TriggerCond::Where { .. }));
        assert!(matches!(pl, PlayerRef::TriggerPlayer));
    }

    #[test]
    fn qualifiers_inside_batches() {
        let (c, _, _) = p("whenever one or more cards leave your graveyard during your turn");
        let TriggerCond::Batched { trigger, .. } = c else {
            panic!("{c:?}");
        };
        assert!(matches!(*trigger, TriggerCond::Where { .. }));
        let (c, _, _) = p("whenever you discard one or more cards for the first time each turn");
        let TriggerCond::Batched { trigger, .. } = c else {
            panic!("{c:?}");
        };
        assert!(matches!(*trigger, TriggerCond::FirstTimeEachTurn(_)));
        // The batch is still checked as a whole for "first" (`check_batch_triggers` reads a
        // `FirstTimeEachTurn` directly inside the batch), with the turn condition inside.
        let (c, _, _) = p(
            "whenever you discard one or more cards for the first time during each of your turns",
        );
        let TriggerCond::Batched { trigger, .. } = c else {
            panic!("{c:?}");
        };
        let TriggerCond::FirstTimeEachTurn(inner) = *trigger else {
            panic!("{trigger:?}");
        };
        assert!(matches!(*inner, TriggerCond::Where { .. }), "{inner:?}");
    }
}
