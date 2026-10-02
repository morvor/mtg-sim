//! Rulings batch P205 — defender (CR 702.3): "this creature can't attack"; losing it,
//! regaining it, and attacking as though a creature didn't have it.

use crate::r_p205_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_attack, destroy};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::{activate_containing, attach_new, give_control, has_kw};
use crate::r_s11_common::turn_face_up;
use crate::r_s20_common::{tap_for_mana, to_beginning_of_combat};
use mtg_engine::ability::{Effect, Sel};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 attacks P0 with a fresh Grizzly Bears and P0 blocks it with `blocker`; the block
/// triggers resolve.
fn block_bears(t: &mut TestGame, blocker: ObjectId) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_beginning_of_combat(t, P1);
    to_blockers(
        t,
        &[(bears, Entity::Player(P0))],
        &[(t.g.current(blocker), bears)],
    );
    t.resolve_all();
}

/// Back to P0's turn, at the beginning of combat of a fresh combat phase.
fn p0_combat(t: &mut TestGame) {
    to_beginning_of_combat(t, P0);
}

#[test]
fn elder_land_wurm_loses_each_new_instance_of_defender_when_it_blocks() {
    cr!("702.3b", "613.1f", "509.1i");
    ruling!(
        "Elder Land Wurm",
        "Another effect can give Elder Land Wurm defender after it has lost it. Blocking again will cause it to lose that instance of defender too."
    );
    supported("Elder Land Wurm");
    supported("Guard Duty");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Elder Land Wurm");
    assert!(has_defender(&mut t, wurm));
    block_bears(&mut t, wurm);
    assert!(!has_defender(&mut t, wurm));
    // Guard Duty ("Enchanted creature has defender") gives it defender again.
    attach_new(&mut t, P0, "Guard Duty", wurm);
    assert!(has_defender(&mut t, wurm));
    p0_combat(&mut t);
    assert!(!can_attack(&mut t, wurm));
    // Blocking again: it loses that defender too.
    block_bears(&mut t, wurm);
    assert!(!has_defender(&mut t, wurm));
    p0_combat(&mut t);
    assert!(can_attack(&mut t, wurm));
}

#[test]
fn axebane_guardian_mana_ability_counts_defenders_without_using_the_stack() {
    cr!("605.1a", "605.3a", "605.3b");
    ruling!(
        "Axebane Guardian",
        "Axebane Guardian’s activated ability is a mana ability. It doesn’t use the stack and can’t be responded to."
    );
    supported("Axebane Guardian");
    supported("Wall of Wood");
    let mut t = TestGame::new(2);
    let axe = t.battlefield(P0, "Axebane Guardian");
    t.battlefield(P0, "Wall of Wood");
    t.battlefield(P0, "Wall of Wood");
    // A non-defender creature isn't counted.
    t.battlefield(P0, "Grizzly Bears");
    assert!(tap_for_mana(&mut t, P0, axe, "Add X"));
    // Three creatures with defender (Axebane Guardian counts itself); nothing went on the
    // stack.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn overgrown_battlement_mana_ability_cant_be_responded_to() {
    cr!("605.1a", "605.3a", "605.3b");
    ruling!(
        "Overgrown Battlement",
        "Overgrown Battlement's last ability is a mana ability. It doesn't use the stack and can't be responded to (such as by removing creatures with defender you control)."
    );
    supported("Overgrown Battlement");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Overgrown Battlement");
    let wood = t.battlefield(P0, "Wall of Wood");
    assert!(tap_for_mana(&mut t, P0, wall, "Add {G}"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
    // Removing a creature with defender afterward doesn't change the mana already added.
    destroy(&mut t, wood);
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

#[test]
fn colossus_of_akros_keeps_defender_when_monstrous_but_can_attack() {
    cr!("702.3b", "701.37a", "701.37b");
    ruling!(
        "Colossus of Akros",
        "Colossus of Akros doesn't lose defender when it's monstrous. It's just able to attack."
    );
    supported("Colossus of Akros");
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P0, "Colossus of Akros");
    p0_combat(&mut t);
    assert!(!can_attack(&mut t, colossus));
    t.lands(P0, "Wastes", 10);
    activate_containing(&mut t, P0, colossus, "Monstrosity").unwrap();
    t.resolve_all();
    assert!(t.obj(colossus).monstrous);
    assert!(has_defender(&mut t, colossus));
    assert!(has_kw(&t, colossus, KeywordKind::Trample));
    assert!(can_attack(&mut t, colossus));
}

#[test]
fn wingmantle_chaplain_counts_defenders_including_itself_as_it_resolves() {
    cr!("608.2h", "603.2");
    ruling!(
        "Wingmantle Chaplain",
        "Count the number of creatures with defender you control as the second ability resolves to determine how many Bird tokens you create. If Wingmantle Chaplain’s still under your control at that time, it will count itself."
    );
    supported("Wingmantle Chaplain");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wall of Wood");
    let chaplain = t.enter(P0, "Wingmantle Chaplain");
    t.settle();
    // Another defender arrives before it resolves.
    t.battlefield(P0, "Wall of Wood");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Bird").len(), 3);
    assert!(t.on_battlefield(chaplain));

    // Gone before the ability resolves: it doesn't count itself.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wall of Wood");
    let chaplain = t.enter(P0, "Wingmantle Chaplain");
    t.settle();
    destroy(&mut t, chaplain);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Bird").len(), 1);
}

#[test]
fn wingmantle_chaplain_entering_with_other_defenders() {
    cr!("603.6a", "603.10a");
    ruling!(
        "Wingmantle Chaplain",
        "If Wingmantle Chaplain enters the battlefield under your control at the same time as other creatures with defender, the second ability will trigger once and the last ability will trigger once for each of those other creatures."
    );
    let mut t = TestGame::new(2);
    let chaplain = t.hand(P0, "Wingmantle Chaplain");
    let w1 = t.hand(P0, "Wall of Wood");
    let w2 = t.hand(P0, "Wall of Wood");
    crate::r_s05_common::run_from(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::AllTargets,
            to: mtg_engine::ability::Destination::battlefield(),
        },
        &[
            Entity::Object(chaplain),
            Entity::Object(w1),
            Entity::Object(w2),
        ],
    );
    assert_eq!(triggers_on_stack(&t, "for each creature with defender"), 1);
    assert_eq!(triggers_on_stack(&t, "Whenever another creature"), 2);
    t.resolve_all();
    // Three for the enters ability (three defenders) and one for each other Wall.
    assert_eq!(with_subtype(&t, P0, "Bird").len(), 5);
}

#[test]
fn ogre_jailbreaker_keeps_attacking_after_losing_its_gate() {
    cr!("702.3b", "506.4");
    ruling!(
        "Ogre Jailbreaker",
        "Defender only matters when Ogre Jailbreaker could be declared as an attacking creature. If Ogre Jailbreaker is already attacking, losing control of your only Gate won't cause Ogre Jailbreaker to leave combat."
    );
    supported("Ogre Jailbreaker");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Ogre Jailbreaker");
    p0_combat(&mut t);
    assert!(!can_attack(&mut t, ogre));
    let gate = t.battlefield(P0, "Azorius Guildgate");
    assert!(can_attack(&mut t, ogre));
    attack_with(&mut t, &[(ogre, Entity::Player(P1))]);
    give_control(&mut t, gate, P1);
    assert!(is_attacking(&t, ogre));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
}

/// Casts `name` face down for {3} and lets it resolve; it's been under P0's control
/// since the turn began.
fn face_down(t: &mut TestGame, name: &str) -> ObjectId {
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, name);
    let spell = t
        .cast(P0, card)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let id = t.g.current(spell);
    assert!(t.obj(id).face_down);
    t.g.objects[id.0 as usize].summoning_sick = false;
    id
}

#[test]
fn an_attacking_face_down_dirgur_nemesis_turned_up_keeps_attacking() {
    cr!("702.3b", "708.8", "506.4");
    ruling!(
        "Dirgur Nemesis",
        "If an attacking face-down Dirgur Nemesis is turned face up, it will continue to be attacking even though it will have defender."
    );
    supported("Dirgur Nemesis");
    let mut t = TestGame::new(2);
    let nemesis = face_down(&mut t, "Dirgur Nemesis");
    p0_combat(&mut t);
    attack_with(&mut t, &[(nemesis, Entity::Player(P1))]);
    t.lands(P0, "Island", 7);
    assert!(turn_face_up(&mut t, P0, nemesis));
    assert!(has_defender(&mut t, nemesis));
    assert!(is_attacking(&t, nemesis));
    block_and_finish(&mut t, P1, &[]);
    // 6/5 with a megamorph +1/+1 counter.
    assert_eq!(t.life(P1), 13);
}

#[test]
fn an_attacking_monastery_flock_turned_up_keeps_attacking_and_has_flying() {
    cr!("702.3b", "708.8", "702.9b", "702.17b");
    ruling!(
        "Monastery Flock",
        "If an attacking face-down Monastery Flock is turned face up, it will continue to be attacking even though it will have defender. If it's turned face up before blockers are declared, then creatures without flying or reach won't be able to block it."
    );
    supported("Monastery Flock");
    supported("Giant Spider");
    let mut t = TestGame::new(2);
    let flock = face_down(&mut t, "Monastery Flock");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spider = t.battlefield(P1, "Giant Spider");
    p0_combat(&mut t);
    attack_with(&mut t, &[(flock, Entity::Player(P1))]);
    assert!(t.g.can_block(bears, flock));
    t.lands(P0, "Island", 1);
    assert!(turn_face_up(&mut t, P0, flock));
    t.g.recompute();
    assert!(has_defender(&mut t, flock));
    assert!(is_attacking(&t, flock));
    assert!(!t.g.can_block(bears, flock));
    assert!(t.g.can_block(spider, flock));
}

/// P1 attacks P0 with a fresh Grizzly Bears; the attack triggers resolve.
fn bears_attack_p0(t: &mut TestGame) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_beginning_of_combat(t, P1);
    attack_with(t, &[(bears, Entity::Player(P0))]);
}

#[test]
fn guardian_of_the_ages_stops_triggering_once_it_lost_defender() {
    cr!("603.4", "702.3b");
    ruling!(
        "Guardian of the Ages",
        "Once Guardian of the Ages has lost defender, its triggered ability will no longer trigger when a creature attacks you or a planeswalker you control (unless it has somehow gained defender again)."
    );
    supported("Guardian of the Ages");
    let mut t = TestGame::new(2);
    let guardian = t.battlefield(P0, "Guardian of the Ages");
    bears_attack_p0(&mut t);
    assert_eq!(triggers_on_stack(&t, "loses defender"), 1);
    t.resolve_all();
    assert!(!has_defender(&mut t, guardian));
    // Another attack: no trigger.
    bears_attack_p0(&mut t);
    assert_eq!(triggers_on_stack(&t, "loses defender"), 0);
    // With defender again (Guard Duty), it triggers again.
    attach_new(&mut t, P0, "Guard Duty", guardian);
    bears_attack_p0(&mut t);
    assert_eq!(triggers_on_stack(&t, "loses defender"), 1);
}

#[test]
fn guardian_of_the_ages_effect_lasts_indefinitely() {
    cr!("611.2a", "514.2");
    ruling!(
        "Guardian of the Ages",
        "The effect of Guardian of the Ages’s triggered ability doesn’t wear off as the turn ends. Losing defender and gaining trample last indefinitely."
    );
    let mut t = TestGame::new(2);
    let guardian = t.battlefield(P0, "Guardian of the Ages");
    bears_attack_p0(&mut t);
    t.resolve_all();
    // Through P1's cleanup step into P0's next turn.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!has_defender(&mut t, guardian));
    assert!(has_kw(&t, guardian, KeywordKind::Trample));
    p0_combat(&mut t);
    assert!(can_attack(&mut t, guardian));
}

#[test]
fn shieldmate_and_sentry_can_attack_once_an_artifact_entered_this_turn() {
    cr!("702.3b", "611.3a");
    ruling!(
        "Mechan Shieldmate",
        "Once an artifact enters the battlefield under your control, Mechan Shieldmate can attack that turn as though it didn’t have defender. It doesn’t matter if that artifact stays an artifact or stays under your control."
    );
    ruling!(
        "Shipwreck Sentry",
        "Once an artifact enters the battlefield under your control, Shipwreck Sentry can attack that turn as though it didn't have defender. It doesn't matter if that artifact stays an artifact or stays under your control."
    );
    for name in ["Mechan Shieldmate", "Shipwreck Sentry"] {
        supported(name);
        let mut t = TestGame::new(2);
        let wall = t.battlefield(P0, name);
        p0_combat(&mut t);
        assert!(!can_attack(&mut t, wall), "{name}");
        let thopter = t.enter(P0, "Ornithopter");
        t.settle();
        assert!(can_attack(&mut t, wall), "{name}");
        // The Ornithopter changes control, then leaves: it still entered this turn.
        give_control(&mut t, thopter, P1);
        assert!(can_attack(&mut t, wall), "{name}");
        destroy(&mut t, thopter);
        assert!(can_attack(&mut t, wall), "{name}");
        assert!(has_defender(&mut t, wall), "{name}");
    }
}

#[test]
fn ageless_sentinels_stops_being_a_wall_and_can_attack() {
    cr!("702.3b", "613.1c", "613.1f");
    ruling!(
        "Ageless Sentinels",
        "Once it stops being a Wall, it can attack because it also loses Defender."
    );
    supported("Ageless Sentinels");
    let mut t = TestGame::new(2);
    let sentinels = t.battlefield(P0, "Ageless Sentinels");
    block_bears(&mut t, sentinels);
    t.g.recompute();
    let o = t.obj_now(sentinels);
    assert!(!o.chars.has_subtype("Wall"));
    assert!(o.chars.has_subtype("Bird") && o.chars.has_subtype("Giant"));
    assert!(!has_defender(&mut t, sentinels));
    p0_combat(&mut t);
    assert!(can_attack(&mut t, sentinels));
}

#[test]
fn spire_serpent_keeps_defender_with_metalcraft_but_can_attack() {
    cr!("702.3b", "613.4c");
    ruling!(
        "Spire Serpent",
        "Spire Serpent still has defender if you control three or more artifacts, although it will be able to attack."
    );
    supported("Spire Serpent");
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Spire Serpent");
    t.lands(P0, "Ornithopter", 2);
    p0_combat(&mut t);
    assert!(!can_attack(&mut t, serpent));
    t.battlefield(P0, "Ornithopter");
    assert!(can_attack(&mut t, serpent));
    assert!(has_defender(&mut t, serpent));
    assert_eq!(t.pt(serpent), (5, 7));
}

#[test]
fn stalwart_shield_bearers_checks_defender_continuously() {
    cr!("702.3b", "611.3a", "613.4c");
    ruling!(
        "Stalwart Shield-Bearers",
        "Stalwart Shield-Bearers continually checks which creatures you control have defender and gives the bonus only to them."
    );
    supported("Stalwart Shield-Bearers");
    supported("Shoal Serpent");
    supported("Warmonger's Chariot");
    let mut t = TestGame::new(2);
    let bearers = t.battlefield(P0, "Stalwart Shield-Bearers");
    // Not itself.
    assert_eq!(t.pt(bearers), (0, 3));
    let serpent = t.battlefield(P0, "Shoal Serpent");
    assert_eq!(t.pt(serpent), (5, 7));
    // Landfall: Shoal Serpent loses defender until end of turn, and the bonus with it.
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    t.resolve_all();
    assert!(!has_defender(&mut t, serpent));
    assert_eq!(t.pt(serpent), (5, 5));
    // Warmonger's Chariot lets a Wall attack as though it didn't have defender: it keeps
    // defender and the bonus (0/3, +2/+2, +0/+2).
    let wall = t.battlefield(P0, "Wall of Wood");
    attach_new(&mut t, P0, "Warmonger's Chariot", wall);
    assert!(has_defender(&mut t, wall));
    assert_eq!(t.pt(wall), (2, 7));
    p0_combat(&mut t);
    assert!(can_attack(&mut t, wall));
}

#[test]
fn wakestone_gargoyle_affects_defenders_that_arrive_after_it_resolves() {
    cr!("611.2c", "302.6");
    ruling!(
        "Wakestone Gargoyle",
        "Wakestone Gargoyle’s ability will affect creatures with defender that come under your control after the ability resolves but before you declare attackers (though those creatures still can’t attack unless they have haste)."
    );
    supported("Wakestone Gargoyle");
    let mut t = TestGame::new(2);
    let gargoyle = t.battlefield(P0, "Wakestone Gargoyle");
    t.lands(P0, "Plains", 2);
    activate_containing(&mut t, P0, gargoyle, "can attack this turn").unwrap();
    t.resolve_all();
    // A Wall of Wood enters afterward: affected, but summoning sick.
    let wall = t.battlefield_sick(P0, "Wall of Wood");
    p0_combat(&mut t);
    assert!(can_attack(&mut t, gargoyle));
    assert!(!can_attack(&mut t, wall));
    // With haste (Fervor: "Creatures you control have haste."), it can attack.
    t.battlefield(P0, "Fervor");
    assert!(can_attack(&mut t, wall));
    // Only creatures P0 controls: P1's Wall isn't affected, even this turn.
    let theirs = t.battlefield(P1, "Wall of Wood");
    assert!(!can_attack(&mut t, theirs));
    // A Wall P0 gains control of later this turn is affected.
    give_control(&mut t, theirs, P0);
    let theirs = t.g.current(theirs);
    t.g.objects[theirs.0 as usize].summoning_sick = false;
    assert!(can_attack(&mut t, theirs));
    // Without the Gargoyle's effect, a hasty Wall still couldn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fervor");
    let wall = t.battlefield_sick(P0, "Wall of Wood");
    p0_combat(&mut t);
    assert!(!can_attack(&mut t, wall));
}
