//! Damage, targeting and activation trigger conditions (CR 120, 115.10, 602.2): source
//! qualifiers ("a source you control other than ~", "a noncreature source you control"),
//! several recipients ("you or a permanent you control", "a creature or opponent"), batches
//! ("one or more opponents are dealt noncombat damage", "~ deals damage to one or more
//! creatures"), the objects the event names ("destroy that creature", "that Archer",
//! "that spell's controller"), players becoming targets, and activating abilities
//! ("that isn't a mana ability", "a loyalty ability", "without {T} in its activation cost").

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The card's ability `text` compiled (other abilities of it may belong to other work).
fn assert_compiled(name: &str, text: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        !c.unsupported_text().iter().any(|u| u.contains(text)),
        "{name}: {text:?} is unsupported"
    );
}

fn combat(t: &mut TestGame, attackers: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(attackers.to_vec()));
    if !blocks.is_empty() {
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    }
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
}

#[test]
fn toxin_sliver_destroys_the_creature_dealt_damage() {
    cr!("510.2", "603.2");
    assert_supported("Toxin Sliver");
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P0, "Toxin Sliver");
    let wurm = t.battlefield(P1, "Craw Wurm");
    combat(&mut t, &[(sliver, Entity::Player(P1))], &[(wurm, sliver)]);
    assert!(!t.on_battlefield(wurm), "the creature dealt damage by a Sliver");
}

#[test]
fn talon_of_pain_ignores_its_own_damage() {
    cr!("120.3", "603.2");
    assert_supported("Talon of Pain");
    let mut t = TestGame::new(2);
    let talon = t.battlefield(P0, "Talon of Pain");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.counters(talon, "charge"), 1);
    // A source you control damaging yourself isn't damage to an opponent.
    let shock = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, shock).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.counters(talon, "charge"), 1);
}

#[test]
fn flameblade_angel_answers_damage_to_you_or_your_permanents() {
    cr!("120.3");
    assert_supported("Flameblade Angel");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Flameblade Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 2);
    t.answer_yes(P0, true);
    let a = t.hand(P1, "Shock");
    t.cast(P1, a).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    t.answer_yes(P0, true);
    let b = t.hand(P1, "Shock");
    t.cast(P1, b).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "that source's controller");
}

#[test]
fn night_dealings_counts_damage_to_another_player() {
    cr!("120.3", "122.1");
    assert_supported("Night Dealings");
    let mut t = TestGame::new(2);
    let nd = t.battlefield(P0, "Night Dealings");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.counters(nd, "theft"), 2);
}

#[test]
fn curse_of_stalked_prey_grows_creatures_hitting_the_enchanted_player() {
    cr!("303.4", "510.2");
    assert_supported("Curse of Stalked Prey");
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P0, "Curse of Stalked Prey");
    assert!(t.g.attach(curse, Entity::Player(P1)));
    let bears = t.battlefield(P0, "Grizzly Bears");
    combat(&mut t, &[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn tephraderm_strikes_back_at_creatures_and_spells_controllers() {
    cr!("120.3", "603.2");
    assert_supported("Tephraderm");
    let mut t = TestGame::new(2);
    let teph = t.battlefield(P1, "Tephraderm");
    let baloth = t.battlefield(P0, "Enormous Baloth");
    combat(&mut t, &[(baloth, Entity::Player(P1))], &[(teph, baloth)]);
    // The 7/7 Baloth took 4 combat damage (not lethal) and 7 from the trigger.
    assert!(!t.on_battlefield(baloth), "{}", t.dump_log());
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Tephraderm");
    let teph = t.named_on_battlefield("Tephraderm")[0];
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Object(teph)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18, "that spell's controller");
}

#[test]
fn flesh_reaver_hurts_you_for_damage_to_creatures_or_opponents() {
    cr!("120.3");
    assert_supported("Flesh Reaver");
    let mut t = TestGame::new(2);
    let reaver = t.battlefield(P0, "Flesh Reaver");
    combat(&mut t, &[(reaver, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 16);
}

#[test]
fn swarmborn_giant_is_sacrificed_when_you_are_dealt_combat_damage() {
    cr!("510.2");
    assert_supported("Swarmborn Giant");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Swarmborn Giant");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant), "noncombat damage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    combat(&mut t, &[(bears, Entity::Player(P1))], &[]);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn master_of_barbs_triggers_once_for_noncombat_damage_to_opponents() {
    cr!("120.3", "603.2c");
    assert_supported("Master of Barbs");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Barbs");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.pt(master), (3, 1));
}

#[test]
fn chandras_phoenix_returns_when_a_red_instant_hits_an_opponent() {
    cr!("120.3", "113.6");
    assert_supported("Chandra's Phoenix");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Chandra's Phoenix");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Chandra's Phoenix"));
}

#[test]
fn greatbow_doyen_makes_the_archer_hit_the_creatures_controller() {
    cr!("120.3", "510.2");
    assert_supported("Greatbow Doyen");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Greatbow Doyen");
    let archer = t.battlefield(P0, "Elvish Archers");
    let bears = t.battlefield(P1, "Grizzly Bears");
    combat(&mut t, &[(archer, Entity::Player(P1))], &[(bears, archer)]);
    // Elvish Archers (2/1, +1/+1 from the Doyen) deals 3 damage to the Bears and so to
    // their controller.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn briarbridge_patrol_investigates_once_per_damage_event() {
    cr!("120.3", "603.2c");
    assert_compiled("Briarbridge Patrol", "one or more creatures");
    let mut t = TestGame::new(2);
    let patrol = t.battlefield(P0, "Briarbridge Patrol");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    combat(&mut t, &[(patrol, Entity::Player(P1))], &[(a, patrol), (b, patrol)]);
    // Both blockers were dealt damage at once (2 + 1, or destroyed).
    for x in [a, b] {
        assert!(!t.on_battlefield(x) || t.obj_now(x).damage > 0, "{}", t.dump_log());
    }
    let clues = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.g.obj(**o).chars.subtypes.iter().any(|s| s == "Clue"))
        .count();
    assert_eq!(clues, 1);
}

// ---------------------------------------------------------------------------
// Becoming the target
// ---------------------------------------------------------------------------

#[test]
fn unsettled_mariner_triggers_for_you_and_for_your_permanents() {
    cr!("115.10", "603.2");
    ruling!(
        "Unsettled Mariner",
        "each become the targets of the same spell or ability an opponent controls"
    );
    assert_supported("Unsettled Mariner");
    let mut t = TestGame::new(2);
    let mariner = t.battlefield(P0, "Unsettled Mariner");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.resolve_all();
    // P1 couldn't pay {1}: the Shock was countered.
    assert_eq!(t.life(P0), 20);
    t.lands(P1, "Mountain", 2);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Object(mariner)).go();
    t.answer_yes(P1, true);
    t.resolve_all();
    // P1 paid {1} this time.
    assert!(!t.on_battlefield(mariner), "{}", t.dump_log());
    // A spell targeting you and a permanent you control: two triggers, {1} each.
    let mut t = TestGame::new(2);
    let mariner = t.battlefield(P0, "Unsettled Mariner");
    let lands = t.lands(P1, "Mountain", 5);
    let flames = t.hand(P1, "Hungry Flames");
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.cast(P1, flames)
        .targets(&[Entity::Object(mariner), Entity::Player(P0)])
        .go();
    t.resolve_all();
    let untapped = lands.iter().filter(|l| !t.obj_now(**l).tapped).count();
    assert_eq!(untapped, 0, "paid {{1}} twice: {}", t.dump_log());
    assert_eq!(t.life(P0), 18);
    assert!(!t.on_battlefield(mariner));
}

#[test]
fn dormant_gomazoa_untaps_when_you_become_the_target_of_a_spell() {
    cr!("115.10");
    assert_supported("Dormant Gomazoa");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Dormant Gomazoa");
    t.g.objects[g.0 as usize].tapped = true;
    t.lands(P1, "Mountain", 1);
    t.answer_yes(P0, true);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert!(!t.obj_now(g).tapped);
}

#[test]
fn wild_defiance_pumps_creatures_targeted_by_instants() {
    cr!("115.10");
    assert_supported("Wild Defiance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wild Defiance");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Object(wurm)).go();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (9, 7));
}

#[test]
fn leyline_of_combustion_triggers_once_per_spell() {
    cr!("115.10", "603.2c");
    ruling!(
        "Leyline of Combustion",
        "if a spell targets you and one or more permanents you control"
    );
    assert_supported("Leyline of Combustion");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Combustion");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // A spell targeting you and a permanent you control: still one trigger.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 3);
    let flames = t.hand(P1, "Hungry Flames");
    t.cast(P1, flames)
        .targets(&[Entity::Object(bears), Entity::Player(P0)])
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    assert_eq!(t.life(P0), 18 - 2);
}

// ---------------------------------------------------------------------------
// Activating abilities
// ---------------------------------------------------------------------------

#[test]
fn burning_tree_shaman_ignores_mana_abilities() {
    cr!("602.2", "605.1a");
    assert_supported("Burning-Tree Shaman");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Burning-Tree Shaman");
    let elf = t.battlefield(P1, "Llanowar Elves");
    t.activate(P1, elf, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "a mana ability");
    let ghoul = t.battlefield(P1, "Cudgel Troll");
    t.lands(P1, "Forest", 1);
    t.activate(P1, ghoul, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn ajani_unrelenting_and_gideon_the_oathless_see_loyalty_abilities() {
    cr!("606.2");
    assert_supported("Ajani Unrelenting");
    assert_supported("Gideon the Oathless");
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Unrelenting");
    t.battlefield(P1, "Gideon the Oathless");
    t.activate(P0, ajani, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Cadet").len(), 1);
    // 1 for the loyalty ability, 1 for the Cadet entering under an opponent's control.
    assert_eq!(t.life(P0), 18);
}

#[test]
fn haunting_wind_triggers_on_tapping_or_tapless_artifact_abilities() {
    cr!("602.2", "701.26a");
    assert_supported("Haunting Wind");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Haunting Wind");
    let stone = t.battlefield(P1, "Mind Stone");
    // {T}: Add {C} taps it.
    t.activate(P1, stone, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // An ability with {T} in its cost triggers only by tapping it (once).
    let lantern = t.battlefield(P1, "Brass Gnat");
    let _ = lantern;
    let lute = t.battlefield(P1, "Jandor's Saddlebags");
    t.lands(P1, "Island", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    t.activate(P1, lute, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // An artifact's ability without {T} in its cost: one trigger, and its lands tapping
    // for mana aren't artifacts.
    let sphere = t.battlefield(P1, "Chimeric Sphere");
    t.lands(P1, "Island", 2);
    t.activate(P1, sphere, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(sphere).is(CardType::Creature));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn imprison_counters_tap_abilities_of_the_enchanted_creature() {
    cr!("602.2", "701.6b");
    assert_compiled("Imprison", "with {T} in its activation cost");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    let imprison = t.battlefield(P0, "Imprison");
    assert!(t.g.attach(imprison, Entity::Object(sorcerer)));
    t.lands(P0, "Swamp", 1);
    t.answer_yes(P0, true);
    t.activate(P1, sorcerer, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "the ability was countered");
    assert!(t.on_battlefield(imprison));
}
