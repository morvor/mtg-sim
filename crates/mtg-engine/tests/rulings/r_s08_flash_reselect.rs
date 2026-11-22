//! Rulings batch S08 — flash: the flash creatures and artifact that reselect what an
//! attacking creature is attacking (CR 508.7) when they enter during the declare attackers
//! step (Portal Mage, Misleading Signpost, Windshaper Planetar).

use crate::r_s01_common::*;
use crate::r_s08_common::*;
use mtg_engine::combat::{player_has_attacked, player_is_attacking};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn attack_target(t: &TestGame, a: ObjectId) -> Option<Entity> {
    t.g.combat
        .as_ref()
        .and_then(|c| c.attackers.iter().find(|x| x.id == a))
        .and_then(|x| x.target)
}

/// `p` flashes in the real card `name` (with lands for its mana cost); its enters trigger
/// targets `attacker` and reselects it to attack `new`. Resolves the card and its trigger.
fn flash_in_and_reselect(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    attacker: ObjectId,
    new: Entity,
) {
    give_mana_for(t, p, name);
    let card = t.hand(p, name);
    t.answer_targets(p, &[Entity::Object(attacker)]);
    t.answer_choose(p, &[new]);
    t.cast(p, card).go();
    t.resolve();
    t.resolve();
}

#[test]
fn reselecting_ignores_the_costs_of_attacking() {
    cr!("508.7", "508.7b", "508.1h");
    ruling!(
        "Portal Mage",
        "Reselecting which player or planeswalker a creature is attacking ignores all requirements, restrictions, and costs associated with attacking."
    );
    supported("Portal Mage");
    supported("Ghostly Prison");
    // Portal Mage: "When this creature enters during the declare attackers step, you may
    // reselect which player or permanent target attacking creature is attacking." P2's
    // Ghostly Prison: "Creatures can't attack you unless their controller pays {2} for
    // each creature they control that's attacking you."
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P2, "Ghostly Prison");
    t.lands(P0, "Wastes", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    // P1 flashes in Portal Mage and sends the Bears at P2: P0 pays nothing.
    flash_in_and_reselect(&mut t, P1, "Portal Mage", bears, Entity::Player(P2));
    assert_eq!(attack_target(&t, bears), Some(Entity::Player(P2)));
    assert_eq!(untapped_lands(&t, P0), 2);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!((t.life(P1), t.life(P2)), (20, 18));
}

fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s04_common::untapped_lands(t, p)
}

#[test]
fn a_reselected_creature_still_attacked_the_player_it_was_declared_to_attack() {
    cr!("508.7a", "506.2", "508.7c");
    ruling!(
        "Portal Mage",
        "If you reselect which player or planeswalker an attacking creature is attacking, that creature is still considered to have attacked the player or planeswalker as declared, but it is now attacking the new player or planeswalker."
    );
    ruling!(
        "Windshaper Planetar",
        "If you reselect which player or planeswalker an attacking creature is attacking, that creature is still considered to have attacked the player or planeswalker as declared, but it is now attacking the new player or planeswalker."
    );
    supported("Windshaper Planetar");
    // Windshaper Planetar: "When this creature enters during the declare attackers step,
    // for each attacking creature, you may reselect which player or permanent that
    // creature is attacking."
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    give_mana_for(&mut t, P1, "Windshaper Planetar");
    let planetar = t.hand(P1, "Windshaper Planetar");
    // The Bears go to P2; the Giant stays on P1 (nothing chosen).
    t.answer_choose(P1, &[Entity::Player(P2)]);
    t.answer_choose(P1, &[]);
    t.cast(P1, planetar).go();
    t.resolve();
    t.resolve();
    assert_eq!(attack_target(&t, bears), Some(Entity::Player(P2)));
    assert_eq!(attack_target(&t, giant), Some(Entity::Player(P1)));
    // P0 attacked P1 (as declared), and is now attacking both.
    assert!(player_has_attacked(&t.g, P0, P1));
    assert!(!player_has_attacked(&t.g, P0, P2));
    assert!(player_is_attacking(&t.g, P0, P2));
    assert!(t.g.is_attacking(bears));
    // A creature can't be made to attack its own controller.
    let options = mtg_engine::kw::reselect_attack::reselect_options(&t.g, bears);
    assert!(!options.contains(&Entity::Player(P0)));
    assert!(options.contains(&Entity::Player(P1)));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!((t.life(P1), t.life(P2)), (17, 18));
}

#[test]
fn a_trigger_targeting_the_old_defending_players_creature_loses_its_target() {
    cr!("508.7", "508.5", "608.2b");
    ruling!(
        "Misleading Signpost",
        "If an ability targets something controlled by the \"defending player\" of an attacking creature and the defending player for that creature changes before that ability resolves, the ability won't resolve because its target has become illegal."
    );
    supported("Misleading Signpost");
    supported("Master of Diversion");
    // Master of Diversion: "Whenever this creature attacks, tap target creature defending
    // player controls." It attacks P1, targeting P1's Grizzly Bears; P0 then flashes in
    // Misleading Signpost and sends it at P2.
    for reselect in [false, true] {
        let mut t = TestGame::new(3);
        let master = t.battlefield(P0, "Master of Diversion");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P0, Step::BeginningOfCombat);
        t.answer_targets(P0, &[Entity::Object(bears)]);
        attack_with(&mut t, &[(master, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 1);
        if reselect {
            flash_in_and_reselect(
                &mut t,
                P0,
                "Misleading Signpost",
                master,
                Entity::Player(P2),
            );
            assert_eq!(attack_target(&t, master), Some(Entity::Player(P2)));
        }
        t.resolve_all();
        assert_eq!(is_tapped(&t, bears), !reselect, "reselect: {reselect}");
    }
}

#[test]
fn the_trigger_needs_the_declare_attackers_step() {
    cr!("603.2");
    // Entering at another time, Portal Mage's ability doesn't trigger.
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P1, "Portal Mage");
    let mage = t.hand(P1, "Portal Mage");
    t.set_step(P0, Step::BeginningOfCombat);
    t.cast(P1, mage).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.named_on_battlefield("Portal Mage").len(), 1);
}
