//! "That player" / "that opponent" / "they" whose antecedent is the trigger condition's "an
//! opponent" (interpretation `that-opponent-fixed-as-it-triggers`): "Whenever a creature an
//! opponent controls enters, you may have that player lose 1 life" (Blood Seeker),
//! "Whenever a land enters under an opponent's control, if that player had another land
//! enter the battlefield under their control this turn, ~ deals 3 damage to that player"
//! (Tunnel Ignus), "Whenever a source an opponent controls deals damage to you, that player
//! sacrifices a permanent of their choice" (Michiko Konda, Truth Seeker).
//!
//! The trigger patterns read "that player" of an object the condition names as that
//! object's controller (`PlayerRef::ControllerOf`). When the condition also names the
//! controller ("an opponent controls", "a player controls", "under an opponent's control",
//! "enchanted player controls": "Whenever a creature enchanted player controls enters, that
//! player loses 1 life and you gain 1 life", Trespasser's Curse), "that player" refers back
//! to the player it named as the event happened: the controller the object had as the
//! ability triggered (`PlayerRef::ControllerAsTriggered`, recorded by
//! `Game::trigger_matches_ctx`), even if the object changes control (or leaves the
//! battlefield) before the ability resolves; nothing checks the trigger condition again
//! (CR 603.4 checks only an intervening "if"). Object-based wordings ("its controller",
//! "that creature's controller", "that source's controller") stay `ControllerOf`: the
//! object's controller as the ability resolves, or as it last existed (CR 608.2h; the
//! Tainted Aether, Ankh of Mishra, Celestial Mantle and Archfiend of Spite rulings).

use crate::ability::*;

/// The parsed trigger condition with its "that player" fixed as it triggers when the
/// condition names the object's controller (see the module docs).
pub(crate) fn fix_that_player(
    parsed: (TriggerCond, Sel, PlayerRef),
) -> (TriggerCond, Sel, PlayerRef) {
    let (cond, it, who) = parsed;
    let who = match who {
        PlayerRef::ControllerOf(sel) if names_controller(&cond, &sel) => {
            PlayerRef::ControllerAsTriggered(sel)
        }
        who => who,
    };
    (cond, it, who)
}

/// Whether trigger condition `c` names the controller of the object `sel` refers to: the
/// event's object (`Sel::TriggerObject`) or its other object (`Sel::TriggerOtherObject`,
/// the source of damage).
fn names_controller(c: &TriggerCond, sel: &Sel) -> bool {
    let object = match sel {
        Sel::TriggerObject => true,
        Sel::TriggerOtherObject => false,
        _ => return false,
    };
    match c {
        TriggerCond::Batched { trigger, .. }
        | TriggerCond::Where { trigger, .. }
        | TriggerCond::FirstTimeEachTurn(trigger)
        | TriggerCond::ThisTurn(trigger)
        | TriggerCond::UntilYourNextTurn(trigger)
        | TriggerCond::ThisCombat(trigger)
        | TriggerCond::Noncombat(trigger)
        | TriggerCond::ForAsLongAs(trigger, _) => names_controller(trigger, sel),
        TriggerCond::AnyOf(v) => !v.is_empty() && v.iter().all(|t| names_controller(t, sel)),
        // The source dealing the damage is the event's other object; a permanent dealt
        // damage, its object.
        TriggerCond::DealsDamage { source, to, .. } => {
            if object {
                matches!(to, DamageRecipient::Object(f) if names_other_controller(f))
            } else {
                names_other_controller(source)
            }
        }
        // "~ blocks a creature": the attacking creature; "~ becomes blocked by a
        // creature": the blocking creature.
        TriggerCond::BlocksCreature { attacker: f, .. }
        | TriggerCond::BlockedByCreature { blocker: f, .. } => object && names_other_controller(f),
        TriggerCond::ZoneChange {
            filter,
            to: Some(ZoneKind::Battlefield),
            ..
        } => object && names_other_controller(filter),
        TriggerCond::EntersBattlefield(f)
        | TriggerCond::Attacks(f)
        | TriggerCond::AttacksUnblocked(f)
        | TriggerCond::Blocks(f)
        | TriggerCond::BecomesBlocked(f)
        | TriggerCond::BecomesTapped(f)
        | TriggerCond::BecomesUntapped(f)
        | TriggerCond::TurnedFaceUp(f)
        | TriggerCond::Transforms(f)
        | TriggerCond::IsDealtDamage { filter: f, .. }
        | TriggerCond::Phases { filter: f, .. }
        | TriggerCond::CountersPut { filter: f, .. }
        | TriggerCond::TappedForMana { filter: f, .. } => object && names_other_controller(f),
        _ => false,
    }
}

/// Whether an object phrase names its controller as a player other than you ("a creature
/// an opponent controls", "a land a player controls"; "a land enters under an opponent's
/// control" is read as "a land an opponent controls enters"; "a creature enchanted player
/// controls").
fn names_other_controller(f: &Filter) -> bool {
    match f {
        Filter::And(v) => v.iter().any(names_other_controller),
        // "an artifact or creature an opponent controls" as alternatives.
        Filter::Or(v) => !v.is_empty() && v.iter().all(names_other_controller),
        // A player the condition names by another description ("enchanted player").
        Filter::ControlledByPlayer(p) => !matches!(**p, PlayerRef::You),
        f => matches!(
            f,
            Filter::ControlledBy(PlayerRel::Opponent | PlayerRel::Any | PlayerRel::NotYou)
        ),
    }
}
