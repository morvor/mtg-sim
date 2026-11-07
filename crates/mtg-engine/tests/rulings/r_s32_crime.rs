//! Rulings batch S32 — committing a crime (CR 700.13): casting a spell, activating an
//! ability, or putting a triggered ability on the stack that targets an opponent, a
//! permanent, spell or ability an opponent controls, or a card in an opponent's graveyard.

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s25_common::{abilities_from, cast_new};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P1's Patrolling Peacemaker ("Whenever an opponent commits a crime, proliferate.").
fn peacemaker(t: &mut TestGame) -> ObjectId {
    // It enters with two +1/+1 counters (placed directly it would be a 0/0).
    let pm = t.enter(P1, "Patrolling Peacemaker");
    t.resolve_all();
    pm
}

/// Settles, returns how many times the Peacemaker triggered (crimes committed by its
/// controller's opponents), and resolves the stack.
fn crimes_now(t: &mut TestGame, pm: ObjectId) -> usize {
    t.settle();
    let n = abilities_from(t, pm).len();
    t.resolve_all();
    n
}

#[test]
fn targeting_an_opponent_their_permanents_spells_abilities_or_graveyard_is_a_crime() {
    cr!("700.13", "601.2c", "602.2b", "603.3d");
    ruling!(
        "Patrolling Peacemaker",
        "A player commits a crime as they cast a spell, activate an ability, or put a triggered ability on the stack that targets at least one opponent, at least one permanent, spell, or ability an opponent controls, and/or at least one card in an opponent’s graveyard."
    );
    supported("Patrolling Peacemaker");
    let mut t = TestGame::new(2);
    let pm = peacemaker(&mut t);
    // A spell targeting the opponent.
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert_eq!(crimes_now(&mut t, pm), 1, "targeting an opponent");
    // A spell targeting a permanent they control.
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    assert_eq!(crimes_now(&mut t, pm), 1, "targeting their permanent");
    // A spell targeting a card in their graveyard.
    let card = t.graveyard(P1, "Hill Giant");
    cast_new(&mut t, P0, "Cremate", &[Entity::Object(card)]);
    assert_eq!(crimes_now(&mut t, pm), 1, "targeting a card in their graveyard");
    // An activated ability targeting the opponent.
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).unwrap();
    assert_eq!(crimes_now(&mut t, pm), 1, "an activated ability");
    // A triggered ability targeting their creature.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Flametongue Kavu");
    assert_eq!(crimes_now(&mut t, pm), 1, "a triggered ability");
    // A spell targeting their spell (which targets their own creature: no crime).
    let theirs = cast_new(&mut t, P1, "Giant Growth", &[Entity::Object(bears)]);
    cast_new(&mut t, P0, "Counterspell", &[Entity::Object(theirs)]);
    assert_eq!(crimes_now(&mut t, pm), 1, "targeting their spell");
    // A spell targeting an ability they control: the Peacemaker's own trigger.
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    let trigger = *t.g.stack.last().unwrap();
    assert_eq!(abilities_from(&t, pm), vec![trigger]);
    cast_new(&mut t, P0, "Stifle", &[Entity::Object(trigger)]);
    t.settle();
    assert_eq!(abilities_from(&t, pm).len(), 2, "targeting their ability");
    t.resolve_all();
    // No crime: targeting your own things, or affecting opponents without targeting.
    let own = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(own)]);
    assert_eq!(crimes_now(&mut t, pm), 0, "targeting your own creature");
    let mine = t.graveyard(P0, "Hill Giant");
    cast_new(&mut t, P0, "Cremate", &[Entity::Object(mine)]);
    assert_eq!(crimes_now(&mut t, pm), 0, "targeting your own graveyard");
    cast_new(&mut t, P0, "Pyroclasm", &[]);
    assert_eq!(crimes_now(&mut t, pm), 0, "no targets");
    // The Peacemaker's controller targeting the opponent's things commits a crime too, but
    // the Peacemaker only cares about its controller's opponents.
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    assert_eq!(crimes_now(&mut t, pm), 0, "P1's own crime");
}

#[test]
fn forsaken_miner_returns_whenever_you_commit_a_crime() {
    // Its "you may pay {B}. If you do, return this card from your graveyard" trigger
    // functions from the graveyard (CR 113.6m).
    cr!("700.13", "603.3d", "113.6m");
    ruling!(
        "Forsaken Miner",
        "A player commits a crime as they cast a spell, activate an ability, or put a triggered ability on the stack that targets at least one opponent, at least one permanent, spell, or ability an opponent controls, and/or at least one card in an opponent's graveyard."
    );
    supported("Forsaken Miner");
    // "Whenever you commit a crime, you may pay {B}. If you do, return this card from your
    // graveyard to the battlefield." Targeting your own creature isn't a crime.
    let mut t = TestGame::new(2);
    let miner = t.graveyard(P0, "Forsaken Miner");
    let own = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(own)]);
    t.settle();
    assert_eq!(t.stack_len(), 1, "no trigger");
    t.resolve_all();
    // Targeting a card in the opponent's graveyard is.
    let card = t.graveyard(P1, "Hill Giant");
    cast_new(&mut t, P0, "Cremate", &[Entity::Object(card)]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "Forsaken Miner triggered");
    add_mana(&mut t, P0, ManaType::B, 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(miner)));
}
