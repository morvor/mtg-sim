//! Rulings batch P052 — sacrificial artifacts that affect combat and characteristics:
//! evasion granted after blockers are declared (CR 509.1h, 702.9b), "can't attack or
//! block" on a creature already in combat (CR 506.4), effects that don't lock in their
//! set of affected objects (CR 611.2c), "as though it didn't have hexproof" (CR 702.11),
//! "doesn't untap during its controller's next untap step" (CR 502.3), type-changing
//! effects with and without a duration (CR 205.1, 305.7, 611.2a), and Dragon cards
//! (CR 205.3).

use crate::r_p006_common::choices_since;
use crate::r_p046_common::*;
use crate::r_p052_common::*;
use crate::r_s01_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 attacks P1 with `attacker`, P1 blocks with `blockers`, and the game stops in the
/// declare blockers step with P0 holding priority.
fn blocked_by(t: &mut TestGame, attacker: ObjectId, blockers: &[ObjectId]) {
    attack_with(t, &[(attacker, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(blockers.iter().map(|b| (*b, attacker)).collect()),
    );
    t.advance_to(P0, Step::DeclareBlockers);
}

fn blocked(t: &TestGame, id: ObjectId) -> bool {
    t.g.combat.as_ref().is_some_and(|c| c.is_blocked(id))
}

#[test]
fn orb_of_dragonkind_finds_dragon_cards_not_cards_named_dragon() {
    cr!("205.3m", "106.6");
    ruling!(
        "Orb of Dragonkind",
        "A Dragon card is a card with the creature type Dragon in its type line. Similarly, a creature on the battlefield is a Dragon if it has the Dragon creature type. A card that has “Dragon” in the name (such as Orb of Dragonkind) isn't a Dragon card unless it also has the Dragon creature type."
    );
    supported("Orb of Dragonkind");
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Orb of Dragonkind");
    let cards = stack_library(
        &mut t,
        P0,
        &["Dragon Fodder", "Orb of Dragonkind", "Shivan Dragon", "Dragon's Approach"],
    );
    fill_pool(&mut t, P0, 1);
    let m = mark(&t);
    t.answer_choose(P0, &[obj(cards[2])]);
    activate_containing(&mut t, P0, orb, "Look at the top seven").expect("activate");
    t.resolve_all();
    let offered = &choices_since(&t, P0, m)[0].0;
    assert_eq!(offered, &vec![obj(cards[2])]);
    assert!(t.in_hand(P0, "Shivan Dragon"));
    // On the battlefield: Orb's mana pays for a Dragon spell, not for a spell with
    // "Dragon" in its name.
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Orb of Dragonkind");
    let fodder = t.hand(P0, "Dragon Fodder");
    let hatchling = t.hand(P0, "Dragon Hatchling");
    t.g.players[0].mana_pool.add_type(ManaType::C, 1);
    // Two red mana: enough for either spell's {1}{R}.
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    assert!(tap_for_mana(&mut t, P0, orb, "Add two mana"));
    assert_eq!(pool(&t, P0), 2);
    assert!(t.cast(P0, fodder).try_go().is_err());
    assert!(t.cast(P0, hatchling).try_go().is_ok());
}

#[test]
fn wedding_invitation_doesnt_unblock_a_blocked_creature() {
    cr!("509.1h");
    ruling!(
        "Wedding Invitation",
        "Activating Wedding Invitation's last ability targeting a creature that has already been blocked won't cause that creature to become unblocked."
    );
    supported("Wedding Invitation");
    let mut t = TestGame::new(2);
    let invitation = t.battlefield(P0, "Wedding Invitation");
    let giant = t.battlefield(P0, "Hill Giant");
    old(&mut t, giant);
    let wall = t.battlefield(P1, "Wall of Wood");
    blocked_by(&mut t, giant, &[wall]);
    t.activate(P0, invitation, 0, &[obj(giant)]).expect("activate");
    t.resolve_all();
    assert!(blocked(&t, giant));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(wall));
}

#[test]
fn alchemists_vial_stops_only_later_attacks_and_blocks() {
    cr!("506.4", "508.1c", "500.8");
    ruling!(
        "Alchemist's Vial",
        "Activating the last ability of Alchemist’s Vial targeting a creature that’s already attacking or blocking won’t cause that creature to stop attacking or blocking. It will prevent that creature from attacking or blocking in any additional combat phases the turn may have (although this is unusual)."
    );
    supported("Alchemist's Vial");
    supported("Relentless Assault");
    let mut t = TestGame::new(2);
    let vial = t.battlefield(P0, "Alchemist's Vial");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    old(&mut t, giant);
    old(&mut t, bears);
    attack_with(&mut t, &[(giant, pl(P1)), (bears, pl(P1))]);
    fill_pool(&mut t, P0, 1);
    t.activate(P0, vial, 0, &[obj(giant)]).expect("activate");
    t.resolve_all();
    assert!(t.g.combat.as_ref().unwrap().attacker(giant).is_some());
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
    // An additional combat phase: the Giant can't attack in it; the Bears can.
    t.advance_to(P0, Step::PostcombatMain);
    let assault = t.hand(P0, "Relentless Assault");
    fill_pool(&mut t, P0, 4);
    t.cast(P0, assault).go();
    t.resolve_all();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.g.recompute();
    assert!(!t.g.can_attack(giant));
    assert!(t.g.can_attack(bears));
}

#[test]
fn tezzerets_gatebreaker_affects_creatures_you_control_later() {
    cr!("611.2c", "509.1b");
    ruling!(
        "Tezzeret's Gatebreaker",
        "Because the last ability of Tezzeret’s Gatebreaker doesn’t change the characteristics of any permanents, the set of creatures affected by it is constantly updated. Creatures that come under your control later in the turn can’t be blocked."
    );
    supported("Tezzeret's Gatebreaker");
    let mut t = TestGame::new(2);
    let gatebreaker = t.battlefield(P0, "Tezzeret's Gatebreaker");
    let wall = t.battlefield(P1, "Wall of Wood");
    activate_resolve(&mut t, P0, gatebreaker, 0, &[]);
    let giant = t.battlefield(P0, "Hill Giant");
    old(&mut t, giant);
    attack_with(&mut t, &[(giant, pl(P1))]);
    assert!(!t.g.can_block(wall, giant));
}

#[test]
fn glaring_spotlight_creatures_controlled_later_cant_be_blocked_but_lack_hexproof() {
    cr!("611.2c", "509.1b");
    ruling!(
        "Glaring Spotlight",
        "Creatures that come under your control after Glaring Spotlight’s last ability resolves won’t have hexproof but they can’t be blocked that turn."
    );
    supported("Glaring Spotlight");
    let mut t = TestGame::new(2);
    let spotlight = t.battlefield(P0, "Glaring Spotlight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    activate_resolve(&mut t, P0, spotlight, 0, &[]);
    assert!(has_kw(&t, bears, KeywordKind::Hexproof));
    let giant = t.battlefield(P0, "Hill Giant");
    old(&mut t, giant);
    old(&mut t, bears);
    t.g.recompute();
    assert!(!has_kw(&t, giant, KeywordKind::Hexproof));
    attack_with(&mut t, &[(giant, pl(P1)), (bears, pl(P1))]);
    assert!(!t.g.can_block(wall, giant));
    assert!(!t.g.can_block(wall, bears));
}

#[test]
fn glaring_spotlight_ignores_hexproof_only_for_its_controller() {
    cr!("702.11b", "609.4");
    ruling!(
        "Glaring Spotlight",
        "Creatures your opponents control don’t actually lose hexproof, although you will ignore hexproof for purposes of choosing targets of spells and abilities you control."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Glaring Spotlight");
    let scout = t.battlefield(P1, "Gladecover Scout");
    assert!(has_kw(&t, scout, KeywordKind::Hexproof));
    // P2 can't target it; P0 can.
    let shock2 = t.hand(P2, "Shock");
    assert!(t.g.object_untargetable(scout, P2, Some(shock2)));
    // P0's can.
    let shock = t.hand(P0, "Shock");
    assert!(!t.g.object_untargetable(scout, P0, Some(shock)));
    fill_pool(&mut t, P0, 1);
    t.cast(P0, shock).target(obj(scout)).go();
    assert!(has_kw(&t, scout, KeywordKind::Hexproof));
    t.resolve_all();
    assert!(!t.on_battlefield(scout));
}

#[test]
fn flight_spellbomb_after_blocks_doesnt_unblock() {
    cr!("509.1h", "702.9b");
    ruling!(
        "Flight Spellbomb",
        "For flying to work as an evasion ability, an attacking creature must be granted flying before the declare blockers step begins. Once an attacking creature has become blocked, giving it flying won’t change that."
    );
    supported("Flight Spellbomb");
    let mut t = TestGame::new(2);
    let bomb = t.battlefield(P0, "Flight Spellbomb");
    let giant = t.battlefield(P0, "Hill Giant");
    old(&mut t, giant);
    let wall = t.battlefield(P1, "Wall of Wood");
    blocked_by(&mut t, giant, &[wall]);
    t.answer_yes(P0, false);
    t.activate(P0, bomb, 0, &[obj(giant)]).expect("activate");
    t.resolve_all();
    assert!(has_kw(&t, giant, KeywordKind::Flying));
    assert!(blocked(&t, giant));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    // Before blockers: the Giant with flying can't be blocked by the Wall.
    let mut t = TestGame::new(2);
    let bomb = t.battlefield(P0, "Flight Spellbomb");
    let giant = t.battlefield(P0, "Hill Giant");
    old(&mut t, giant);
    let wall = t.battlefield(P1, "Wall of Wood");
    t.answer_yes(P0, false);
    t.activate(P0, bomb, 0, &[obj(giant)]).expect("activate");
    t.resolve_all();
    attack_with(&mut t, &[(giant, pl(P1))]);
    assert!(!t.g.can_block(wall, giant));
}

#[test]
fn sleeper_dart_doesnt_tap_and_is_used_up_by_the_next_untap_step() {
    cr!("502.3");
    ruling!(
        "Sleeper Dart",
        "Sleeper Dart’s second ability doesn’t tap the creature. It can target any creature, tapped or untapped. If that creature is already untapped at the beginning of its controller’s next untap step, the effect won’t do anything."
    );
    supported("Sleeper Dart");
    let mut t = TestGame::new(2);
    let dart = t.battlefield(P0, "Sleeper Dart");
    let bears = t.battlefield(P1, "Grizzly Bears");
    activate_resolve(&mut t, P0, dart, 0, &[obj(bears)]);
    assert!(!t.obj_now(bears).tapped);
    // P1's next untap step passes with the Bears untapped; tapped later, they untap as
    // usual in P1's following untap step.
    t.advance_to(P1, Step::Upkeep);
    let now = t.g.current(bears);
    t.g.objects[now.0 as usize].tapped = true;
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
    // A tapped target stays tapped through its controller's next untap step.
    let mut t = TestGame::new(2);
    let dart = t.battlefield(P0, "Sleeper Dart");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    activate_resolve(&mut t, P0, dart, 0, &[obj(bears)]);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn mishras_groundbreaker_lasts_indefinitely() {
    cr!("611.2a", "205.1b");
    ruling!(
        "Mishra's Groundbreaker",
        "The ability’s effect has no duration. The affected land will remain an artifact, creature, and land until the game ends, it leaves the battlefield, or some other effect changes its types."
    );
    supported("Mishra's Groundbreaker");
    let mut t = TestGame::new(2);
    let breaker = t.battlefield(P0, "Mishra's Groundbreaker");
    let forest = t.battlefield(P1, "Forest");
    activate_resolve(&mut t, P0, breaker, 0, &[obj(forest)]);
    for _ in 0..2 {
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::Upkeep);
        let o = t.obj_now(forest);
        assert!(o.is(CardType::Artifact) && o.is(CardType::Creature) && o.is(CardType::Land));
        assert!(o.chars.has_subtype("Forest"));
        assert_eq!(t.pt(forest), (3, 3));
    }
}

#[test]
fn ashnods_transmogrant_keeps_the_creature_an_artifact_without_the_counter() {
    cr!("611.2a", "205.1b", "122.1");
    ruling!(
        "Ashnod's Transmogrant",
        "It’s the ability, not the counter that makes the creature an artifact. The creature remains an artifact even if the +1/+1 counter is removed."
    );
    supported("Ashnod's Transmogrant");
    supported("Vampire Hexmage");
    let mut t = TestGame::new(2);
    let transmogrant = t.battlefield(P0, "Ashnod's Transmogrant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    activate_resolve(&mut t, P0, transmogrant, 0, &[obj(bears)]);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert!(t.obj_now(bears).is(CardType::Artifact));
    // Vampire Hexmage: "Sacrifice this creature: Remove all counters from target
    // permanent."
    let hexmage = t.battlefield(P0, "Vampire Hexmage");
    activate_resolve(&mut t, P0, hexmage, 0, &[obj(bears)]);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert!(t.obj_now(bears).is(CardType::Artifact));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn elsewhere_flask_sets_one_land_type_and_keeps_name_supertypes() {
    cr!("305.7", "205.1b", "205.4a");
    ruling!(
        "Elsewhere Flask",
        "All your lands will have the chosen land type and no other land type."
    );
    ruling!(
        "Elsewhere Flask",
        "Changing a land's type doesn't change its name or whether it's legendary or basic."
    );
    let mut t = TestGame::new(2);
    let flask = t.battlefield(P0, "Elsewhere Flask");
    let tundra = t.battlefield(P0, "Tundra");
    let forest = t.battlefield(P0, "Forest");
    let cradle = t.battlefield(P0, "Gaea's Cradle");
    // Swamp is the third basic land type offered.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    activate_containing(&mut t, P0, flask, "Choose a basic land type").expect("activate");
    t.resolve_all();
    for land in [tundra, forest, cradle] {
        let o = t.obj_now(land);
        let subtypes: Vec<String> = o.chars.subtypes.iter().map(|s| s.to_string()).collect();
        assert_eq!(subtypes, vec!["Swamp"], "{}", o.chars.name);
    }
    assert_eq!(t.obj_now(forest).chars.name, "Forest");
    assert!(t.obj_now(forest).chars.supertypes.contains(Supertype::Basic));
    assert!(t.obj_now(cradle).chars.supertypes.contains(Supertype::Legendary));
    assert!(!t.obj_now(tundra).chars.supertypes.contains(Supertype::Basic));
    assert!(tap_for_mana(&mut t, P0, tundra, "Add {B}"));
}
