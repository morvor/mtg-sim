//! The event feed: what happened since a player's previous decision, described for that
//! player without revealing what they can't see.
//!
//! The engine records events only when its event feed is on
//! (`game.event_feed.enable()`, done by [`crate::prepare_game`]).

use crate::describe::{entity_name, object_name, player_name, revealed_name, visible_id};
use crate::view::{step_name, zone_name};
use crate::visibility::{can_see, Viewer};
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::object::Zone;
use mtg_engine::{Entity, Game, ObjectId};
use serde::{Deserialize, Serialize};

/// One event, described for a viewer.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EventView {
    /// The kind of event: "zone_change", "cast", "activate", "trigger", "resolve",
    /// "damage", "life", "draw", "attack", "block", "turn", "step", ...
    pub kind: String,
    /// A readable description.
    pub text: String,
    /// The players involved.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub players: Vec<u8>,
    /// The objects involved that the viewer may know about (by their current ids where
    /// they still exist).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objects: Vec<u32>,
}

fn ev(kind: &str, text: String) -> EventView {
    EventView {
        kind: kind.into(),
        text,
        ..Default::default()
    }
}

/// Whether the viewer could see the object as it was or as it is now (the zone change
/// from or to a zone they can see).
fn seen(g: &Game, viewer: Viewer, id: ObjectId) -> bool {
    (id.0 as usize) < g.objects.len() && can_see(g, viewer, id)
}

fn cause_text(c: MoveCause) -> &'static str {
    match c {
        MoveCause::Cast => "cast",
        MoveCause::PlayLand => "played",
        MoveCause::Resolve => "resolved",
        MoveCause::Effect => "by an effect",
        MoveCause::Destroy => "destroyed",
        MoveCause::Sacrifice => "sacrificed",
        MoveCause::Discard => "discarded",
        MoveCause::Mill => "milled",
        MoveCause::Draw => "drawn",
        MoveCause::Counter => "countered",
        MoveCause::StateBased => "state-based action",
        MoveCause::CeaseToExist => "ceased to exist",
        MoveCause::Exile => "exiled",
        MoveCause::Return => "returned",
        MoveCause::Search => "search",
        MoveCause::Cleanup => "cleanup",
        MoveCause::Cost => "paid as a cost",
        MoveCause::Commander => "commander",
        MoveCause::Venture => "venture",
        MoveCause::Other => "other",
    }
}

fn zone_owner_text(z: Zone) -> String {
    match z {
        Zone::Library(p) | Zone::Hand(p) | Zone::Graveyard(p) | Zone::Outside(p) => {
            format!("{}'s {}", player_name(p), zone_name(z))
        }
        z => zone_name(z).to_string(),
    }
}

/// Describes one event for `viewer`; `None` for bookkeeping events not worth reporting
/// (batch markers, mana, events already described by another one).
pub fn describe_event(g: &Game, viewer: Viewer, e: &Event) -> Option<EventView> {
    let name = |id: ObjectId| object_name(g, viewer, id);
    let ent = |e: Entity| entity_name(g, viewer, e);
    // An object's current id, only if the viewer could know the object in the event:
    // following a card the viewer couldn't see then (a card drawn, or put into a hand or
    // library) to where it is now would tell them which card it was.
    let obj_ids = |ids: &[ObjectId]| -> Vec<u32> {
        ids.iter()
            .filter(|i| visible_id(g, viewer, **i).is_some())
            .filter_map(|i| visible_id(g, viewer, g.current(*i)))
            .collect()
    };
    let mut v = match e {
        Event::ZoneChange {
            old,
            new,
            from,
            to,
            cause,
            by,
            ..
        } => {
            // Described by their own events.
            if matches!(
                cause,
                MoveCause::Draw | MoveCause::Cast | MoveCause::PlayLand
            ) || matches!(to, Zone::Stack)
            {
                return None;
            }
            let known = if seen(g, viewer, *new) {
                Some(name(*new))
            } else if seen(g, viewer, *old) {
                Some(name(*old))
            } else {
                None
            };
            let what = known.unwrap_or_else(|| match (from, to) {
                (Zone::Battlefield, _) | (_, Zone::Battlefield) => name(*new),
                _ => "a card".to_string(),
            });
            let mut x = ev(
                "zone_change",
                format!(
                    "{what} moved from {} to {} ({}{})",
                    zone_owner_text(*from),
                    zone_owner_text(*to),
                    cause_text(*cause),
                    by.map(|p| format!(", by {}", player_name(p)))
                        .unwrap_or_default()
                ),
            );
            x.objects = obj_ids(&[*new]);
            x.players = by.iter().map(|p| p.0).collect();
            x
        }
        Event::SpellCast { spell, player, .. } => {
            let mut text = format!("{} cast {}", player_name(*player), name(*spell));
            if let Some(si) = g.obj(*spell).stack.as_deref() {
                let ts: Vec<String> = si
                    .chosen
                    .iter()
                    .flat_map(|c| c.targets.iter().flatten())
                    .map(|t| ent(*t))
                    .collect();
                if !ts.is_empty() {
                    text.push_str(&format!(" targeting {}", ts.join(", ")));
                }
            }
            let mut x = ev("cast", text);
            x.players = vec![player.0];
            x.objects = obj_ids(&[*spell]);
            x
        }
        Event::AbilityActivated {
            ability,
            source,
            player,
            is_mana,
        } => {
            if *is_mana {
                return None;
            }
            let text = ability
                .and_then(|a| g.obj(a).stack.as_deref().map(|_| a))
                .and_then(|a| match &g.obj(a).stack.as_deref()?.kind {
                    mtg_engine::object::StackKind::Activated { ability, .. } => {
                        Some(ability.text.clone())
                    }
                    _ => None,
                })
                .unwrap_or_default();
            let mut x = ev(
                "activate",
                format!(
                    "{} activated an ability of {}{}",
                    player_name(*player),
                    name(*source),
                    if text.is_empty() {
                        String::new()
                    } else {
                        format!(": {text}")
                    }
                ),
            );
            x.players = vec![player.0];
            x.objects = obj_ids(&[*source]);
            x
        }
        Event::AbilityTriggeredOnStack { ability, source } => {
            let text = match g.obj(*ability).stack.as_deref().map(|s| &s.kind) {
                Some(mtg_engine::object::StackKind::Triggered { ability, .. }) => {
                    ability.text.clone()
                }
                _ => String::new(),
            };
            let mut x = ev(
                "trigger",
                format!(
                    "{} triggered{}",
                    name(*source),
                    if text.is_empty() {
                        String::new()
                    } else {
                        format!(": {text}")
                    }
                ),
            );
            x.objects = obj_ids(&[*source]);
            x
        }
        Event::AbilityResolved { source, .. } => {
            let mut x = ev(
                "resolve",
                format!("An ability of {} resolved", name(*source)),
            );
            x.objects = obj_ids(&[*source]);
            x
        }
        Event::SpellResolved { spell } => {
            let mut x = ev("resolve", format!("{} resolved", name(*spell)));
            x.objects = obj_ids(&[*spell]);
            x
        }
        Event::Countered { what } => ev("counter", format!("{} was countered", name(*what))),
        Event::Damage {
            source,
            target,
            amount,
            combat,
        } => {
            let mut x = ev(
                "damage",
                format!(
                    "{} dealt {amount} {}damage to {}",
                    name(*source),
                    if *combat { "combat " } else { "" },
                    ent(*target)
                ),
            );
            if let Entity::Player(p) = target {
                x.players = vec![p.0];
            }
            x.objects = obj_ids(&[*source]);
            x
        }
        Event::DamagePrevented {
            source,
            target,
            amount,
            ..
        } => ev(
            "damage_prevented",
            format!(
                "{amount} damage {} would deal to {} was prevented",
                name(*source),
                ent(*target)
            ),
        ),
        Event::LifeGained { player, amount } => {
            let mut x = ev(
                "life",
                format!("{} gained {amount} life", player_name(*player)),
            );
            x.players = vec![player.0];
            x
        }
        Event::LifeLost { player, amount } => {
            let mut x = ev(
                "life",
                format!("{} lost {amount} life", player_name(*player)),
            );
            x.players = vec![player.0];
            x
        }
        Event::Drew { player, card, .. } => {
            let c = g.current(*card);
            let what = if seen(g, viewer, c) || seen(g, viewer, *card) {
                name(*card)
            } else {
                "a card".into()
            };
            let mut x = ev("draw", format!("{} drew {what}", player_name(*player)));
            x.players = vec![player.0];
            x.objects = obj_ids(&[*card]);
            x
        }
        Event::Discarded { player, card, .. } => {
            let c = g.current(*card);
            let mut x = ev(
                "discard",
                format!("{} discarded {}", player_name(*player), name(c)),
            );
            x.players = vec![player.0];
            x.objects = obj_ids(&[c]);
            x
        }
        Event::Milled { player, cards } => {
            let names: Vec<String> = cards.iter().map(|c| name(g.current(*c))).collect();
            let mut x = ev(
                "mill",
                format!("{} milled {}", player_name(*player), names.join(", ")),
            );
            x.players = vec![player.0];
            x.objects = obj_ids(cards);
            x
        }
        Event::CountersAdded { target, kind, n } => ev(
            "counters",
            format!("{n} {kind} counter(s) put on {}", ent(*target)),
        ),
        Event::CountersRemoved {
            target, kind, n, ..
        } => ev(
            "counters",
            format!("{n} {kind} counter(s) removed from {}", ent(*target)),
        ),
        Event::Tapped { .. } | Event::Untapped { .. } => return None,
        Event::AttackersDeclared { player, attackers } => {
            if attackers.is_empty() {
                return None;
            }
            let list: Vec<String> = attackers
                .iter()
                .map(|(a, t)| format!("{} -> {}", name(*a), ent(*t)))
                .collect();
            let mut x = ev(
                "attack",
                format!("{} attacked with {}", player_name(*player), list.join(", ")),
            );
            x.players = vec![player.0];
            x.objects = obj_ids(&attackers.iter().map(|(a, _)| *a).collect::<Vec<_>>());
            x
        }
        Event::BlockersDeclared { blocks } => {
            if blocks.is_empty() {
                return Some(ev("block", "No blockers were declared".into()));
            }
            let list: Vec<String> = blocks
                .iter()
                .map(|(b, a)| format!("{} blocks {}", name(*b), name(*a)))
                .collect();
            ev("block", list.join(", "))
        }
        Event::BlockAdded {
            blocker, attacker, ..
        } => ev(
            "block",
            format!("{} blocks {}", name(*blocker), name(*attacker)),
        ),
        Event::BecameBlocked { .. }
        | Event::AttackerUnblocked { .. }
        | Event::BecameTarget { .. }
        | Event::ManaAdded { .. }
        | Event::TappedForMana { .. }
        | Event::BatchBoundary => return None,
        Event::StepBegan { step, active } => {
            if matches!(step, mtg_engine::turn::Step::Untap) {
                return None;
            }
            let mut x = ev(
                "step",
                format!(
                    "{} step of {}'s turn",
                    step_name(*step).replace('_', " "),
                    player_name(*active)
                ),
            );
            x.players = vec![active.0];
            x
        }
        Event::TurnBegan { active, number } => {
            let mut x = ev(
                "turn",
                format!("Turn {number} began ({}'s turn)", player_name(*active)),
            );
            x.players = vec![active.0];
            x
        }
        Event::TokenCreated { obj, controller } => {
            let mut x = ev(
                "token",
                format!("{} created {}", player_name(*controller), name(*obj)),
            );
            x.objects = obj_ids(&[*obj]);
            x
        }
        Event::LandPlayed { player, land } => {
            let mut x = ev(
                "land",
                format!("{} played {}", player_name(*player), name(*land)),
            );
            x.players = vec![player.0];
            x.objects = obj_ids(&[*land]);
            x
        }
        Event::Cycled { player, card, .. } => ev(
            "cycle",
            format!("{} cycled {}", player_name(*player), name(g.current(*card))),
        ),
        Event::TurnedFaceUp { obj } => ev(
            "face_up",
            format!("{} was turned face up", revealed_name(g, *obj)),
        ),
        Event::TurnedFaceDown { obj } => {
            ev("face_down", format!("{} was turned face down", name(*obj)))
        }
        Event::Transformed { obj } => ev("transform", format!("{} transformed", name(*obj))),
        Event::ControlChanged { obj, from, to } => ev(
            "control",
            format!(
                "{} gained control of {} from {}",
                player_name(*to),
                name(*obj),
                player_name(*from)
            ),
        ),
        Event::PlayerLost { player } => {
            ev("game", format!("{} lost the game", player_name(*player)))
        }
        Event::PlayerWon { player } => ev("game", format!("{} won the game", player_name(*player))),
        Event::Searched { player } => ev(
            "search",
            format!("{} searched their library", player_name(*player)),
        ),
        Event::Shuffled { player } => ev(
            "shuffle",
            format!("{} shuffled their library", player_name(*player)),
        ),
        Event::Sacrificed { obj, player } => ev(
            "sacrifice",
            format!("{} sacrificed {}", player_name(*player), name(*obj)),
        ),
        Event::Destroyed { obj } => ev("destroy", format!("{} was destroyed", name(*obj))),
        Event::Attached { obj, to } => ev(
            "attach",
            format!("{} became attached to {}", name(*obj), ent(*to)),
        ),
        Event::Unattached { obj, from } => ev(
            "attach",
            format!("{} became unattached from {}", name(*obj), ent(*from)),
        ),
        Event::PhasedOut { obj } => ev("phase", format!("{} phased out", name(*obj))),
        Event::PhasedIn { obj } => ev("phase", format!("{} phased in", name(*obj))),
        Event::DieRolled {
            player,
            sides,
            result,
            planar,
            ..
        } => ev(
            "die",
            if *planar {
                format!("{} rolled the planar die", player_name(*player))
            } else {
                format!("{} rolled a d{sides}: {result}", player_name(*player))
            },
        ),
        Event::CoinFlipped {
            player, won, heads, ..
        } => ev(
            "coin",
            format!(
                "{} flipped a coin: {}{}",
                player_name(*player),
                if *heads { "heads" } else { "tails" },
                if *won { " (won)" } else { "" }
            ),
        ),
        Event::DayNightChanged { is_day } => ev(
            "day_night",
            format!("It became {}", if *is_day { "day" } else { "night" }),
        ),
        Event::BecameMonarch { player } => ev(
            "designation",
            format!("{} became the monarch", player_name(*player)),
        ),
        Event::TookInitiative { player } => ev(
            "designation",
            format!("{} took the initiative", player_name(*player)),
        ),
        Event::CrimeCommitted { player } => ev(
            "crime",
            format!("{} committed a crime", player_name(*player)),
        ),
        Event::Exploited { obj, by, player } => ev(
            "exploit",
            format!(
                "{} exploited {} with {}",
                player_name(*player),
                name(*obj),
                name(*by)
            ),
        ),
        Event::SpellCopied { spell, player } => ev(
            "copy",
            format!("{} copied {}", player_name(*player), name(*spell)),
        ),
        Event::Custom {
            name: n,
            player,
            obj,
            amount,
        } => {
            let n = n.as_str();
            // Reveals show the card to every player (CR 701.20a).
            let reveal = n == mtg_engine::reveal::REVEALED
                || n == mtg_engine::facedown::REVEALED
                || n == mtg_engine::zones::TOP_REVEALED;
            let what = obj.map(|o| if reveal { revealed_name(g, o) } else { name(o) });
            let who = player.map(player_name);
            let text = match (reveal, who, what) {
                (true, Some(w), Some(c)) => format!("{w} revealed {c}"),
                (true, None, Some(c)) => format!("{c} was revealed"),
                (_, w, c) => {
                    let mut parts = vec![n.to_string()];
                    if let Some(w) = w {
                        parts.push(w);
                    }
                    if let Some(c) = c {
                        parts.push(c);
                    }
                    if *amount != 0 {
                        parts.push(amount.to_string());
                    }
                    parts.join(": ")
                }
            };
            let mut x = ev(if reveal { "reveal" } else { "other" }, text);
            x.players = player.iter().map(|p| p.0).collect();
            x
        }
        Event::ExcessDamage { .. } => return None,
        // Events added to the engine later: reported by kind only.
        #[allow(unreachable_patterns)]
        other => {
            let dbg = format!("{other:?}");
            let kind = dbg
                .split([' ', '{', '('])
                .next()
                .unwrap_or("event")
                .to_string();
            ev("other", kind)
        }
    };
    if v.text.is_empty() {
        v.text = v.kind.clone();
    }
    Some(v)
}

/// Describes a list of events for `viewer`.
pub fn describe_events(g: &Game, viewer: Viewer, events: &[Event]) -> Vec<EventView> {
    events
        .iter()
        .filter_map(|e| describe_event(g, viewer, e))
        .collect()
}
