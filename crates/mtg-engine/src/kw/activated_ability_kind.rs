//! Conditions on the activated ability an "activates an ability" trigger event is about
//! (CR 602.2), for triggers qualified by its cost or kind: "Whenever a player activates an
//! ability of enchanted creature with {T} in its activation cost" (Imprison), "… an
//! artifact's ability without {T} in its activation cost" (Haunting Wind, CR 602.1a),
//! "Whenever you activate a ninjutsu ability" (Satoru Umezawa, CR 702.49).
//!
//! The ability is the one on the stack (`EventInfo::spell`), or, for a mana ability, which
//! never goes on the stack (CR 605.3a), the one its source just recorded as activated.
//!
//! Also: `Filter::Custom` [`ACTIVATED_ABILITY`] (an activated ability on the stack: "becomes
//! the target of an activated ability"), and the `TriggerCond::Custom` triggers on a player
//! becoming the target of a spell or ability ([`player_targeted`]: "Whenever you become
//! the target of a spell or ability an opponent controls", CR 115.10, 603.2).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::events::Event;
use crate::object::{EventInfo, StackKind};
use crate::types::{Entity, ObjectId, PlayerId};

/// `Condition::Custom`: the activated ability has {T} in its activation cost.
pub const TAP_IN_COST: &str = "activated ability: {T} in its activation cost";
/// `Condition::Custom`: the activated ability is a ninjutsu ability (CR 702.49a).
pub const NINJUTSU: &str = "activated ability: ninjutsu";
/// `Condition::Custom`: it's the turn's first combat phase (CR 505.1a, 506.1): "at the end of
/// the first combat phase on your turn" (Zariel, Archduke of Avernus).
pub const FIRST_COMBAT_PHASE: &str = "turn: first combat phase";
/// `Condition::Custom`: the trigger event's player has the initiative (CR 725): "whenever you
/// attack the player who has the initiative" (Loot Dispute).
pub const EVENT_PLAYER_HAS_INITIATIVE: &str = "event player: has the initiative";
/// `Condition::Custom`: the activated ability is a power-up ability (CR 702.191a).
pub const POWER_UP: &str = "activated ability: power-up";

/// The activated ability of the trigger event being evaluated.
fn event_ability(g: &Game, ctx: &Ctx) -> Option<ActivatedAbility> {
    let info = ctx.event.as_ref()?;
    if let Some(ab) = info.spell {
        return match g.obj(ab).stack.as_deref().map(|s| &s.kind) {
            Some(StackKind::Activated { ability, .. }) => match &ability.kind {
                AbilityKind::Activated(a) => Some(a.clone()),
                _ => None,
            },
            _ => None,
        };
    }
    // A mana ability: the last one recorded for its source.
    let src = info.object?;
    let (_, _, uid) = g.history.activated.iter().rev().find(|(_, s, _)| *s == src)?;
    g.obj(src)
        .chars
        .abilities
        .iter()
        .find(|a| a.uid == *uid)
        .and_then(|a| match &a.kind {
            AbilityKind::Activated(act) => Some(act.clone()),
            _ => None,
        })
}

/// Whether the event's activated ability is the expansion of keyword `k`.
fn is_keyword_ability(g: &Game, ctx: &Ctx, k: KeywordKind) -> bool {
    let Some(info) = ctx.event.as_ref() else {
        return false;
    };
    let Some(ab) = info.spell else {
        return false;
    };
    match g.obj(ab).stack.as_deref().map(|s| &s.kind) {
        Some(StackKind::Activated { ability, .. }) => {
            ability.text == k.name()
                || crate::keyword_impls::ability_from_keyword(ability) == Some(k)
        }
        _ => false,
    }
}

/// `Filter::Custom`: a permanent with a mana ability (CR 605.1a): "each creature you control
/// with a mana ability".
pub const HAS_MANA_ABILITY: &str = "object: has a mana ability";

/// `Filter::Custom`: a backup triggered ability on the stack (CR 702.165a): "becomes the
/// target of a backup ability".
pub const BACKUP_ABILITY: &str = "stack object: backup ability";

/// `Filter::Custom`: an activated ability on the stack (CR 602.2a).
pub const ACTIVATED_ABILITY: &str = "stack object: activated ability";

const PLAYER_TARGETED: &str = "player becomes the target:";

fn rel_word(r: PlayerRel) -> &'static str {
    match r {
        PlayerRel::You => "you",
        PlayerRel::Opponent => "opponent",
        PlayerRel::NotYou => "another player",
        _ => "any",
    }
}

fn word_rel(w: &str) -> Option<PlayerRel> {
    Some(match w {
        "you" => PlayerRel::You,
        "opponent" => PlayerRel::Opponent,
        "another player" => PlayerRel::NotYou,
        "any" => PlayerRel::Any,
        _ => return None,
    })
}

/// The `TriggerCond::Custom` name for "[who] becomes the target of a spell or ability
/// [by] controls": each time a player matching `who` becomes a target of a spell or
/// ability whose controller matches `by` (once per spell or ability, however many times it
/// targets them). Event player = the controller of the spell or ability, spell = it.
pub fn player_targeted(who: PlayerRel, by: PlayerRel) -> String {
    format!("{PLAYER_TARGETED}{}:{}", rel_word(who), rel_word(by))
}

/// Whether a `TriggerCond::Custom` name is one of [`player_targeted`]'s.
pub fn is_player_targeted(name: &str) -> bool {
    name.starts_with(PLAYER_TARGETED)
}

pub struct ActivatedAbilityKind;

impl KeywordRules for ActivatedAbilityKind {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        match name {
            TAP_IN_COST => Some(event_ability(g, ctx).is_some_and(|a| a.cost.has_tap())),
            NINJUTSU => Some(is_keyword_ability(g, ctx, KeywordKind::Ninjutsu)),
            POWER_UP => Some(is_keyword_ability(g, ctx, KeywordKind::PowerUp)),
            FIRST_COMBAT_PHASE => Some(g.turn.combat_phases <= 1),
            EVENT_PLAYER_HAS_INITIATIVE => Some(
                ctx.event
                    .as_ref()
                    .and_then(|e| e.player)
                    .is_some_and(|p| g.initiative == Some(p)),
            ),
            _ => None,
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        if name == HAS_MANA_ABILITY {
            return Some(
                g.obj(id)
                    .chars
                    .abilities
                    .iter()
                    .any(|a| matches!(&a.kind, AbilityKind::Activated(act) if act.is_mana_ability)),
            );
        }
        if name == BACKUP_ABILITY {
            return Some(matches!(
                g.obj(id).stack.as_deref().map(|s| &s.kind),
                Some(StackKind::Triggered { ability, .. })
                    if crate::keyword_impls::ability_from_keyword(ability) == Some(KeywordKind::Backup)
            ));
        }
        if name != ACTIVATED_ABILITY {
            return None;
        }
        Some(matches!(
            g.obj(id).stack.as_deref().map(|s| &s.kind),
            Some(StackKind::Activated { .. })
        ))
    }

    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        let rest = name.strip_prefix(PLAYER_TARGETED)?;
        let (who, by) = rest.split_once(':')?;
        let (who, by) = (word_rel(who)?, word_rel(by)?);
        let Event::BecameTarget {
            target: Entity::Player(p),
            by: s,
            controller,
        } = ev
        else {
            return Some(vec![]);
        };
        let ctx = Ctx::new(Some(src), ctl);
        Some(
            if g.player_rel_matches(who, *p, &ctx) && g.player_rel_matches(by, *controller, &ctx) {
                vec![EventInfo {
                    spell: Some(*s),
                    player: Some(*controller),
                    ..Default::default()
                }]
            } else {
                vec![]
            },
        )
    }
}

inventory::submit! { KeywordRegistration(&ActivatedAbilityKind) }
