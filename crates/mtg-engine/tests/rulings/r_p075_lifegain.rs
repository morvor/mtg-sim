//! Rulings batch P075 — "whenever you gain life, put +1/+1 counters on ..." (Nykthos
//! Paragon, Archangel of Thune, Minwu, Lisette, Blech) and the Food-granting Gourmand's
//! Talent: life-gain events (CR 119.9, 119.10), "do this only once each turn", creatures
//! dealt lethal damage as the life is gained, Two-Headed Giant, and "for the first time
//! each turn".

use crate::r_p075_common::*;
use crate::r_s01_common::custom_card;
use crate::r_s02_common::create_token;
use crate::r_s19_common::gain_level;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Two lifelink attackers: P0 attacks P1 with `attackers`; P1 blocks with `blocks`; combat
/// damage is dealt and the game advances to the end of combat (triggers resolved).
fn fight(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    to_beginning_of_combat(t, P0);
    attack_with(t, &at_p1(attackers));
    t.resolve_all();
    block_and_finish(t, P1, blocks);
}

#[test]
fn nykthos_paragon_once_each_turn_per_paragon_and_only_once_used() {
    cr!("603.5", "119.9");
    ruling!(
        "Nykthos Paragon",
        "if you control multiple Nykthos Paragons, you will be able to do this once for each of them"
    );
    ruling!(
        "Nykthos Paragon",
        "As long as you haven't yet chosen to put +1/+1 counters on your creatures with it, all instances of gaining life will cause Nykthos Paragon's ability to trigger."
    );
    ruling!(
        "Nykthos Paragon",
        "Once you have chosen to put +1/+1 counters on your creatures, further instances of gaining life will not cause the ability to trigger."
    );
    supported("Nykthos Paragon");
    // Declining keeps it available; once used, it no longer triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nykthos Paragon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    gain(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(plus(&t, bears), 0);
    t.answer_yes(P0, true);
    gain(&mut t, P0, 2);
    assert_eq!(
        triggers_on_stack(&t, "+1/+1"),
        1,
        "triggers again after declining"
    );
    t.resolve_all();
    assert_eq!(plus(&t, bears), 2);
    gain(&mut t, P0, 3);
    assert_eq!(triggers_on_stack(&t, "+1/+1"), 0, "used this turn");
    // Two Paragons: once for each.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nykthos Paragon");
    t.battlefield(P0, "Nykthos Paragon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    gain(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(plus(&t, bears), 4);
    gain(&mut t, P0, 2);
    assert_eq!(triggers_on_stack(&t, "+1/+1"), 0);
}

#[test]
fn nykthos_paragon_one_trigger_per_lifelink_creature() {
    cr!("510.2", "702.15b", "119.9");
    ruling!(
        "Nykthos Paragon",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event."
    );
    // Two lifelink creatures: two triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nykthos Paragon");
    let a = t.battlefield(P0, "Child of Night");
    let b = t.battlefield(P0, "Child of Night");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[a, b]));
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    t.advance_to_step(Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "+1/+1"), 2);
    // One lifelink trampler dealing damage to a blocker and the player: one trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nykthos Paragon");
    let a = t.battlefield(P0, "Child of Night");
    grant(&mut t, a, KeywordKind::Trample);
    let elf = t.battlefield(P1, "Llanowar Elves");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[a]));
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(elf, a)]),
    );
    t.advance_to_step(Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 19, "trampled over");
    assert_eq!(t.life(P0), 22);
    assert_eq!(triggers_on_stack(&t, "+1/+1"), 1);
}

#[test]
fn lethal_damage_as_life_is_gained_kills_before_the_counters() {
    cr!("704.5g", "603.3", "510.2");
    ruling!(
        "Nykthos Paragon",
        "If a creature you control is dealt lethal damage at the same time that you gain life, it won't receive +1/+1 counters from Nykthos Paragon's ability in time to save it."
    );
    ruling!(
        "Archangel of Thune",
        "If a creature you control is dealt lethal damage at the same time that you gain life, it won't receive a counter from Archangel of Thune's ability in time to save it."
    );
    ruling!(
        "Valentin, Dean of the Vein // Lisette, Dean of the Root",
        "If you gain life at the same time a creature you control is dealt lethal damage, that creature dies before Lisette’s ability can put a +1/+1 counter on it."
    );
    supported("Archangel of Thune");
    supported("Valentin, Dean of the Vein // Lisette, Dean of the Root");
    for name in [
        "Nykthos Paragon",
        "Archangel of Thune",
        "Valentin, Dean of the Vein // Lisette, Dean of the Root",
    ] {
        let mut t = TestGame::new(2);
        if name.starts_with("Valentin") {
            back_face(&mut t, P0, name);
            t.lands(P0, "Forest", 1);
        } else {
            t.battlefield(P0, name);
        }
        let giant = t.battlefield(P0, "Hill Giant");
        let child = t.battlefield(P0, "Child of Night");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        fight(&mut t, &[child], &[(bears, child)]);
        assert_eq!(t.life(P0), 22, "{name}");
        assert!(!t.on_battlefield(child), "{name}: died before the counter");
        assert!(plus(&t, giant) >= 1, "{name}: the ability did resolve");
    }
}

#[test]
fn bellowing_aegisaur_counter_too_late_for_creatures_dealt_lethal_damage() {
    cr!("704.5g", "603.3");
    ruling!(
        "Bellowing Aegisaur",
        "If another creature you control is dealt lethal damage at the same time that Bellowing Aegisaur is dealt damage, the other creature won't be saved"
    );
    supported("Bellowing Aegisaur");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Bellowing Aegisaur");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    cast_card(&mut t, P1, "Pyroclasm", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(!t.on_battlefield(bears));
    assert_eq!(plus(&t, giant), 1);
}

#[test]
fn minwu_clerics_dealt_lethal_damage_die_first() {
    cr!("704.5g", "603.2", "510.2");
    ruling!(
        "Minwu, White Mage",
        "If Minwu or another Cleric you control is dealt lethal damage at the same time that you gain life, it will die before Minwu's last ability would resolve."
    );
    supported("Minwu, White Mage");
    let mut t = TestGame::new(2);
    let minwu = t.battlefield(P0, "Minwu, White Mage");
    let cleric = t.battlefield(P0, "Devoted Caretaker");
    let home = t.battlefield(P0, "Giver of Runes");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    fight(&mut t, &[minwu, cleric], &[(giant, minwu), (bears, cleric)]);
    assert_eq!(t.life(P0), 23);
    assert!(!t.on_battlefield(minwu) && !t.on_battlefield(cleric));
    // The ability still triggered (Minwu was on the battlefield as the life was gained).
    assert_eq!(plus(&t, home), 1);
}

#[test]
fn minwu_one_trigger_per_life_gain_event() {
    cr!("119.9", "119.10");
    ruling!(
        "Minwu, White Mage",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event"
    );
    ruling!(
        "Minwu, White Mage",
        "Minwu's last ability triggers just once for each life-gaining event"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Minwu, White Mage");
    let cleric = t.battlefield(P0, "Giver of Runes");
    cast_card(&mut t, P0, "Bountiful Harvest", &[]);
    t.resolve();
    assert_eq!(t.life(P0), 25, "gains 1 for each of 5 lands");
    assert_eq!(triggers_on_stack(&t, "Cleric"), 1);
    t.resolve_all();
    assert_eq!(plus(&t, cleric), 1);
    gain(&mut t, P0, 1);
    assert_eq!(triggers_on_stack(&t, "Cleric"), 1);
}

#[test]
fn minwu_two_headed_giant_teammate_gain_doesnt_trigger() {
    cr!("810.9", "119.9");
    ruling!(
        "Minwu, White Mage",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Minwu's last ability to trigger"
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Minwu, White Mage");
    let before = t.life(P0);
    gain(&mut t, P1, 2);
    assert_eq!(t.life(P0), before + 2, "the team's life total went up");
    assert_eq!(triggers_on_stack(&t, "Cleric"), 0);
    gain(&mut t, P0, 2);
    assert_eq!(triggers_on_stack(&t, "Cleric"), 1);
}

#[test]
fn lisette_pays_once_per_life_gain_event() {
    cr!("603.5", "118.12");
    ruling!(
        "Valentin, Dean of the Vein // Lisette, Dean of the Root",
        "You may only pay {1} once for each life gain event."
    );
    let mut t = TestGame::new(2);
    let l = back_face(
        &mut t,
        P0,
        "Valentin, Dean of the Vein // Lisette, Dean of the Root",
    );
    mana(&mut t, P0, mtg_engine::mana::ManaType::C, 3);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    gain(&mut t, P0, 3);
    t.resolve_all();
    assert_eq!(plus(&t, l), 1);
    assert_eq!(count_since(&t, from, is_yes_no), 1);
    assert_eq!(pool_total(&t, P0), 2);
}

#[test]
fn blech_one_counter_for_a_creature_with_several_listed_types() {
    cr!("205.3", "119.9");
    ruling!(
        "Blech, Loafing Pest",
        "A creature with more than one of the listed types will still get only one +1/+1 counter"
    );
    supported("Blech, Loafing Pest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blech, Loafing Pest");
    let def = custom_card(
        "Bat Insect",
        "Creature — Bat Insect Snake",
        "{1}",
        Some((1, 1)),
        "",
    );
    let both = t.custom(P0, def, Zone::Battlefield);
    gain(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(plus(&t, both), 1);
}

#[test]
fn gourmands_talent_artifact_is_a_food_on_your_turn() {
    cr!("205.1b", "613.1d", "716.2");
    ruling!(
        "Gourmand's Talent",
        "If Gourmand's Talent somehow becomes an artifact, it will also be a Food during your turn."
    );
    supported("Gourmand's Talent");
    let mut t = TestGame::new(2);
    let talent = t.battlefield(P0, "Gourmand's Talent");
    let coating = t.battlefield(P0, "Liquimetal Coating");
    t.activate(P0, coating, 0, &[obj(talent)])
        .expect("activates");
    t.resolve_all();
    assert!(t.obj_now(talent).is(CardType::Artifact));
    assert!(t.obj_now(talent).chars.has_subtype("Food"));
    t.set_step(P1, Step::PrecombatMain);
    t.g.recompute();
    assert!(
        !t.obj_now(talent).chars.has_subtype("Food"),
        "only during your turn"
    );
}

#[test]
fn food_and_clue_granting_keeps_the_artifacts_types_and_abilities() {
    cr!("205.1b", "613.1d", "613.1f");
    ruling!(
        "Gourmand's Talent",
        "The artifacts retain any types, subtypes, supertypes, and abilities they have."
    );
    ruling!(
        "Ragost, Deft Gastronaut",
        "The artifacts retain any types, subtypes, supertypes, and abilities they have."
    );
    ruling!(
        "Senator Peacock",
        "The artifacts retain any types, subtypes, supertypes, and abilities they have."
    );
    for (name, sub) in [
        ("Gourmand's Talent", "Food"),
        ("Ragost, Deft Gastronaut", "Food"),
        ("Senator Peacock", "Clue"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let treasure = create_token(&mut t, P0, "Treasure");
        t.g.recompute();
        let o = t.obj_now(treasure);
        assert!(o.chars.has_subtype(sub), "{name}");
        assert!(o.chars.has_subtype("Treasure"), "{name}");
        assert!(
            tap_for_mana(&mut t, P0, treasure, "Add"),
            "{name}: still a Treasure"
        );
    }
}

#[test]
fn gourmands_talent_levels_dont_trigger_after_an_earlier_gain() {
    cr!("716.2a", "603.2");
    ruling!(
        "Gourmand's Talent",
        "The level 2 and level 3 class abilities of Gourmand's Talent won't trigger if you already gained life earlier in the turn"
    );
    let mut t = TestGame::new(2);
    let talent = t.battlefield(P0, "Gourmand's Talent");
    gain(&mut t, P0, 1);
    t.lands(P0, "Forest", 3);
    gain_level(&mut t, P0, talent, 2).expect("levels up");
    t.resolve_all();
    gain(&mut t, P0, 1);
    t.resolve_all();
    assert!(t.named_on_battlefield("Raccoon Token").is_empty());
    assert_eq!(tokens(&t, P0).len(), 0);
    // On a later turn it does.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    gain(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}
