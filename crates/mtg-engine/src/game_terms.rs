//! Game terms defined in CR 700 that the rest of the engine asks about: committing a
//! crime (CR 700.13), a player's party (CR 700.8), outlaws (CR 700.12), worthy creatures
//! (CR 700.16), permanents that were activated this turn (CR 700.10), and how many times a
//! player descended this turn (CR 700.11).

use crate::events::Event;
use crate::game::Game;
use crate::object::*;
use crate::types::*;

// ---------------------------------------------------------------------------
// Crimes (CR 700.13)
// ---------------------------------------------------------------------------

/// Whether the targets of the spell or ability `stack_obj` make putting it on the stack a
/// crime by `player`: it targets at least one opponent; at least one permanent, spell, or
/// ability an opponent controls; and/or at least one card in an opponent's graveyard
/// (CR 700.13).
pub fn targets_make_a_crime(g: &Game, stack_obj: ObjectId, player: PlayerId) -> bool {
    let Some(si) = g.obj(stack_obj).stack.as_deref() else {
        return false;
    };
    si.chosen
        .iter()
        .flat_map(|m| m.targets.iter().flatten())
        .any(|e| match *e {
            Entity::Player(p) => g.are_opponents(player, p),
            Entity::Object(o) => {
                let ob = g.obj(o);
                match ob.zone {
                    Zone::Battlefield | Zone::Stack => g.are_opponents(player, ob.controller),
                    Zone::Graveyard(owner) => g.are_opponents(player, owner),
                    _ => false,
                }
            }
        })
}

/// Called for the events that put a spell or ability on the stack: a player commits a
/// crime as they cast a spell, activate an ability, or put a triggered ability on the
/// stack that targets an opponent or something of theirs (CR 700.13).
pub fn check_crime(g: &mut Game, ev: &Event) {
    let (obj, player) = match ev {
        Event::SpellCast { spell, player, .. } => (*spell, *player),
        Event::AbilityActivated {
            ability: Some(a),
            player,
            ..
        } => (*a, *player),
        Event::AbilityTriggeredOnStack { ability, .. } => (*ability, g.obj(*ability).controller),
        _ => return,
    };
    if targets_make_a_crime(g, obj, player) {
        g.emit(Event::CrimeCommitted { player });
    }
}
