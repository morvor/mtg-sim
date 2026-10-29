//! Rulings batch P147 — creatures put onto the battlefield attacking (Zareth San, the
//! Trickster; Shark Shredder, Killer Clone): never declared as attackers (CR 508.4),
//! activation windows once blockers are declared (CR 509.1h), combat damage steps (CR
//! 510.4), and what happens to stolen permanents when their controller leaves (CR 800.4a).

use crate::r_p147_common::*;
use crate::r_s03_common::to_blockers;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Action;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const ZARETH: &str = "Zareth San, the Trickster";
/// A Rogue: "Faerie Impostor" (2/1 flier) put directly onto the battlefield.
const ROGUE: &str = "Faerie Impostor";

/// P0 controls a Rogue and has Zareth San in hand with mana for its ability ({2}{U}{B}).
/// Returns (rogue, Zareth card).
fn zareth_setup(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported(ZARETH);
    let rogue = t.battlefield(P0, ROGUE);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    let z = t.hand(P0, ZARETH);
    (rogue, z)
}

/// P0 activates Zareth San's ability from hand, returning `rogue`.
fn zareth(t: &mut TestGame, z: ObjectId, rogue: ObjectId) {
    t.answer_choose(P0, &[obj(rogue)]);
    t.activate(P0, z, 0, &[]).expect("Zareth San's ability");
}

#[test]
fn zareth_wasnt_declared_as_an_attacker() {
    cr!("508.4", "508.3a");
    ruling!(
        "Zareth San, the Trickster",
        "Although Zareth San is attacking, it was never declared as an attacking creature"
    );
    supported("Hellrider");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hellrider");
    let (rogue, z) = zareth_setup(&mut t);
    to_blockers(&mut t, &[(rogue, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    zareth(&mut t, z, rogue);
    assert!(t.in_hand(P0, ROGUE));
    t.resolve_all();
    let zz = now(&t, z);
    assert!(attacking(&t, zz) && t.obj(zz).tapped);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn zareth_needs_an_unblocked_rogue_so_not_before_blockers() {
    cr!("509.1h", "602.2");
    ruling!(
        "Zareth San, the Trickster",
        "Before blockers are declared, attacking Rogues are neither blocked nor unblocked, so you can't pay the cost to activate Zareth San's second ability."
    );
    let mut t = TestGame::new(2);
    let (rogue, z) = zareth_setup(&mut t);
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[rogue]));
    assert!(attacking(&t, rogue));
    assert!(!can_activate(&mut t, P0, z));
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(can_activate(&mut t, P0, z));
}

#[test]
fn zareth_leaving_hand_before_resolution() {
    cr!("608.2b", "400.7");
    ruling!(
        "Zareth San, the Trickster",
        "If Zareth San leaves your hand before its second ability resolves, the ability does nothing as it resolves. Zareth San remains in its new zone and the Rogue you returned remains in its owner's hand."
    );
    let mut t = TestGame::new(2);
    let (rogue, z) = zareth_setup(&mut t);
    to_blockers(&mut t, &[(rogue, Entity::Player(P1))], &[]);
    zareth(&mut t, z, rogue);
    move_to(&mut t, z, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.in_graveyard(P0, ZARETH));
    assert!(t.named_on_battlefield(ZARETH).is_empty());
    assert!(t.in_hand(P0, ROGUE));
}

#[test]
fn zareth_in_the_first_strike_step_deals_regular_damage_even_with_first_strike() {
    cr!("510.4", "702.7c");
    ruling!(
        "Zareth San, the Trickster",
        "If a creature in combat has first strike or double strike, you can activate Zareth San's second ability during the first-strike combat damage step. Zareth San will deal combat damage during the regular combat damage step in this case, even if it somehow gains first strike."
    );
    supported("Youthful Knight");
    let mut t = TestGame::new(2);
    let (rogue, z) = zareth_setup(&mut t);
    let knight = t.battlefield(P0, "Youthful Knight");
    to_blockers(
        &mut t,
        &[(rogue, Entity::Player(P1)), (knight, Entity::Player(P1))],
        &[],
    );
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    assert!(can_activate(&mut t, P0, z));
    zareth(&mut t, z, rogue);
    t.resolve_all();
    let zz = now(&t, z);
    assert!(attacking(&t, zz));
    crate::r_s26_common::modify_until_eot(
        &mut t,
        zz,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::FirstStrike))],
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn zareth_after_combat_damage_deals_no_combat_damage() {
    cr!("510.1", "509.1h", "511.1");
    ruling!(
        "Zareth San, the Trickster",
        "Zareth San's second ability can be activated during the declare blockers step, combat damage step, or end of combat step. If you wait until after the declare blockers step, because all combat damage is dealt at once, Zareth San won't normally deal combat damage."
    );
    let mut t = TestGame::new(2);
    let (rogue, z) = zareth_setup(&mut t);
    to_blockers(&mut t, &[(rogue, Entity::Player(P1))], &[]);
    assert!(can_activate(&mut t, P0, z));
    t.advance_to(P0, Step::CombatDamage);
    assert_eq!(t.life(P1), 18);
    assert!(can_activate(&mut t, P0, z));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(can_activate(&mut t, P0, z));
    zareth(&mut t, z, rogue);
    t.resolve_all();
    assert!(attacking(&t, now(&t, z)));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(now(&t, z)));
}

#[test]
fn zareth_chooses_what_it_attacks() {
    cr!("508.4");
    ruling!(
        "Zareth San, the Trickster",
        "You choose which player or planeswalker Zareth San is attacking. It doesn't have to be attacking the same player or planeswalker that the returned Rogue was attacking."
    );
    let mut t = TestGame::new(3);
    let (rogue, z) = zareth_setup(&mut t);
    to_blockers(&mut t, &[(rogue, Entity::Player(P1))], &[]);
    zareth(&mut t, z, rogue);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.resolve_all();
    let zz = now(&t, z);
    assert_eq!(
        crate::r_s12_common::attackers_at(&t, Entity::Player(P2)),
        vec![zz]
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P2), 16);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn zareths_stolen_permanents_are_exiled_when_you_leave_the_game() {
    cr!("800.4a");
    ruling!(
        "Zareth San, the Trickster",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any permanents you control from Zareth San's last ability are exiled."
    );
    let mut t = TestGame::new(3);
    let z = t.battlefield(P0, ZARETH);
    let bears = t.graveyard(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[z]));
    yes(&mut t, P0);
    t.answer_targets(P0, &[obj(bears)]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    let b = now(&t, bears);
    assert!(t.on_battlefield(b));
    assert_eq!(t.obj(b).controller, P0);
    t.g.take_action(P0, Action::Concede);
    t.g.flush_events();
    t.settle();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn shark_shredders_creature_entering_in_the_regular_step_deals_no_combat_damage() {
    cr!("510.4", "508.4");
    ruling!(
        "Shark Shredder, Killer Clone",
        "the creature returned with Shark Shredder's last ability won't deal combat damage at all during that combat phase."
    );
    supported("Shark Shredder, Killer Clone");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shark Shredder, Killer Clone");
    let giant = t.graveyard(P1, "Hill Giant");
    let bears = t.graveyard(P1, "Grizzly Bears");
    crate::r_s26_common::modify_until_eot(
        &mut t,
        s,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::DoubleStrike))],
    );
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[s]));
    // First-strike step: the Giant is returned (and deals damage in the regular step);
    // regular step: the Bears are returned, too late to deal combat damage.
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    for c in [giant, bears] {
        let c = now(&t, c);
        assert!(t.on_battlefield(c) && attacking(&t, c));
        assert_eq!(t.obj(c).controller, P0);
    }
    assert_eq!(t.life(P1), 20 - 4 - 4 - 3);
}

#[test]
fn zareth_stays_revealed_until_its_ability_resolves() {
    cr!("602.2a", "701.20a");
    ruling!(
        "Zareth San, the Trickster",
        "To activate Zareth San's second ability, you reveal it in your hand. It remains revealed until the ability has resolved."
    );
    let mut t = TestGame::new(2);
    let (rogue, z) = zareth_setup(&mut t);
    to_blockers(&mut t, &[(rogue, Entity::Player(P1))], &[]);
    assert!(!mtg_engine::reveal::is_revealed(&t.g, z));
    zareth(&mut t, z, rogue);
    // Still revealed after the activation, while the ability waits on the stack (and
    // after other actions in the meantime).
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_hand(P0, ZARETH));
    assert!(mtg_engine::reveal::is_revealed(&t.g, z));
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.g.turn.priority = Some(P1);
    t.cast(P1, shock).target(P0).go();
    t.resolve();
    assert!(mtg_engine::reveal::is_revealed(&t.g, z));
    t.resolve_all();
    assert!(t.on_battlefield(now(&t, z)));
    // A card whose ability is countered stops being revealed once the ability is gone.
    let mut t = TestGame::new(2);
    let (rogue, z) = zareth_setup(&mut t);
    to_blockers(&mut t, &[(rogue, Entity::Player(P1))], &[]);
    zareth(&mut t, z, rogue);
    let ab = t.g.stack[0];
    t.g.counter(ab, None);
    t.settle();
    assert!(t.in_hand(P0, ZARETH));
    assert!(!mtg_engine::reveal::is_revealed(&t.g, z));
}
