//! Pump spells and combat tricks (patterns in `src/oracle/patterns/pump_combat_tricks.rs`):
//! leading durations ("Until end of turn, ..."), predicate lists with quoted abilities,
//! base power/toughness and ability loss, switching power and toughness, "for each
//! creature blocking it", and returning a creature that died.

use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// Declares attackers (and blockers) and advances to the declare blockers step, so
/// that spells can be cast before combat damage.
fn to_blocks(t: &mut TestGame, attackers: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(attackers.to_vec()));
    if !blocks.is_empty() {
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    }
    t.advance_to(P0, Step::DeclareBlockers);
}

/// Ends the turn: the cleanup step ends "until end of turn" effects (CR 514.2).
fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
}

// ---------------------------------------------------------------------------
// Leading durations
// ---------------------------------------------------------------------------

#[test]
fn leading_duration_cards_compile() {
    assert_supported(&[
        "Rookie Mistake",
        "Zealous Persecution",
        "Swift Justice",
        "Titanic Ultimatum",
        "Heroic Reinforcements",
        "Traitorous Instinct",
        "Mass Diminish",
        "Chorus of Might",
    ]);
}

#[test]
fn rookie_mistake_modifies_both_targets_until_end_of_turn() {
    cr!("611.2a", "514.2", "115.3");
    ruling!("Rookie Mistake", "You can't cast Rookie Mistake unless you choose two creatures");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    let spell = t.hand(P0, "Rookie Mistake");
    // One creature isn't enough: "another target creature" is a second target.
    assert!(t.cast(P0, spell).target(bears).try_go().is_err());
    t.clear_answers();
    let giant = t.battlefield(P1, "Hill Giant");
    t.cast(P0, spell).target(bears).target(giant).go();
    t.resolve();
    assert_eq!(t.pt(bears), (2, 4));
    assert_eq!(t.pt(giant), (1, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn zealous_persecution_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    let spell = t.hand(P0, "Zealous Persecution");
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(!t.on_battlefield(lions), "a 2/1 getting -1/-1 dies");
    assert_eq!(t.pt(giant), (2, 2));
    // A creature that comes under an opponent's control later isn't affected.
    let later = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(later), (2, 2));
}

#[test]
fn swift_justice_grants_first_strike_and_lifelink_until_end_of_turn() {
    cr!("611.2a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let spell = t.hand(P0, "Swift Justice");
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (3, 2));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Lifelink));
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Lifelink));
}

#[test]
fn mass_diminish_lasts_until_your_next_turn() {
    cr!("611.2a", "611.2c", "613.4b");
    ruling!("Mass Diminish", "affects only creatures the target player controls at the time it resolves");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "Mass Diminish");
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!(t.pt(giant), (1, 1));
    let later = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(later), (2, 2));
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.pt(giant), (1, 1), "still in effect during the opponent's turn");
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn chorus_of_might_counts_creatures_as_it_resolves() {
    cr!("608.2h");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 4);
    let spell = t.hand(P0, "Chorus of Might");
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Trample));
    // The bonus doesn't change when the count does.
    t.battlefield(P0, "Savannah Lions");
    assert_eq!(t.pt(bears), (4, 4));
}

// ---------------------------------------------------------------------------
// Quoted abilities
// ---------------------------------------------------------------------------

#[test]
fn quoted_grant_cards_compile() {
    assert_supported(&[
        "Supernatural Stamina",
        "Undying Malice",
        "Demonic Gifts",
        "Bail Out",
        "Retraction Helix",
        "Hunter's Prowess",
        "Warriors' Lesson",
        "Rabid Attack",
        "Lightning Volley",
        "Brawl",
        "Shoving Match",
        "Driven // Despair",
        "Dreadmaw's Ire",
        "Storm the Citadel",
        "Unnatural Moonrise",
    ]);
}

#[test]
fn supernatural_stamina_returns_the_creature_once() {
    cr!("613.1f", "603.10a", "400.7e");
    ruling!("Supernatural Stamina", "If that new creature dies, it won't come back a second time");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "Supernatural Stamina");
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 2));
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bears).go();
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1, "returned to the battlefield");
    let back = back[0];
    assert_ne!(back, bears, "a new object");
    assert!(t.obj_now(back).tapped);
    assert_eq!(t.obj_now(back).controller, P0);
    assert_eq!(t.pt(back), (2, 2), "the new object isn't affected");
    // The new creature doesn't have the ability.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(back).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn undying_malice_returns_under_its_owners_control_with_a_counter() {
    cr!("110.2a", "122.6");
    let mut t = TestGame::new(2);
    // Cast on an opponent's creature: it returns under its owner's control.
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Undying Malice");
    t.cast(P0, spell).target(giant).go();
    t.resolve();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(giant).go();
    t.resolve_all();
    let back = t.named_on_battlefield("Hill Giant");
    assert_eq!(back.len(), 1);
    let back = back[0];
    assert_eq!(t.obj_now(back).controller, P1);
    assert!(t.obj_now(back).tapped);
    assert_eq!(t.counters(back, "+1/+1"), 1);
    assert_eq!(t.pt(back), (4, 4));
}

#[test]
fn bail_out_returned_creature_deals_damage() {
    cr!("400.7e");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Bail Out");
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn cant_stay_away_grants_the_ability_to_the_returned_permanent() {
    cr!("400.7", "611.2a");
    assert_supported(&["Can't Stay Away"]);
    let mut t = TestGame::new(2);
    let dead = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Can't Stay Away");
    t.cast(P0, spell).target(dead).go();
    t.resolve();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    // "It" is the permanent the card became; the grant has no duration.
    t.set_step(P1, Step::PrecombatMain);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(back[0]).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn retraction_helix_grants_an_activated_ability_until_end_of_turn() {
    cr!("613.1f", "611.2a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 1);
    let spell = t.hand(P0, "Retraction Helix");
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    t.activate(P0, bears, 0, &[giant.into()]).unwrap();
    assert!(t.obj_now(bears).tapped, "{{T}} is part of the cost");
    t.resolve();
    assert!(t.in_hand(P1, "Hill Giant"));
    next_turn(&mut t);
    assert!(t.obj_now(bears).chars.abilities.is_empty());
}

#[test]
fn lightning_volley_creatures_deal_damage_themselves() {
    cr!("611.2c", "613.1f");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    let spell = t.hand(P0, "Lightning Volley");
    t.cast(P0, spell).go();
    t.resolve();
    let later = t.battlefield(P0, "Savannah Lions");
    t.activate(P0, bears, 0, &[P1.into()]).unwrap();
    t.resolve();
    t.activate(P0, giant, 0, &[P1.into()]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(
        t.obj_now(later).chars.abilities.is_empty(),
        "a creature that came later doesn't gain it"
    );
}

#[test]
fn hunters_prowess_draws_cards_equal_to_the_combat_damage() {
    cr!("702.19b", "510.1c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Forest", 5);
    let spell = t.hand(P0, "Hunter's Prowess");
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    let hand = t.hand_size(P0);
    t.attack(&[(bears, Entity::Player(P1))], &[(elves, bears)]);
    assert!(!t.on_battlefield(elves));
    // Trample: 1 to the blocker, 4 to the player.
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn warriors_lesson_gives_each_target_the_trigger() {
    cr!("613.1f");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 1);
    let spell = t.hand(P0, "Warriors' Lesson");
    t.cast(P0, spell)
        .targets(&[bears.into(), lions.into()])
        .go();
    t.resolve();
    let hand = t.hand_size(P0);
    let p1 = Entity::Player(P1);
    t.attack(&[(bears, p1), (lions, p1), (giant, p1)], &[]);
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.hand_size(P0), hand + 2, "only the two targets draw");
}

// ---------------------------------------------------------------------------
// Predicates in other orders, base P/T, losing abilities, switching
// ---------------------------------------------------------------------------

#[test]
fn predicate_list_cards_compile() {
    assert_supported(&[
        "Strength in Numbers",
        "Xenagos, God of Revels",
        "Square Up",
        "Ovinize",
        "Water Wings",
        "Wings of Velis Vel",
        "Sudden Spoiling",
        "Twisted Image",
        "Phantasmal Fiend",
        "Might of Alara",
        "Rabid Elephant",
        "Jungle Wurm",
    ]);
}

#[test]
fn strength_in_numbers_counts_attackers_as_it_resolves() {
    cr!("608.2h");
    ruling!("Strength in Numbers", "It won't change later in the turn if the number of attacking creatures changes");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 2);
    let p1 = Entity::Player(P1);
    to_blocks(&mut t, &[(bears, p1), (lions, p1), (giant, p1)], &[]);
    let spell = t.hand(P0, "Strength in Numbers");
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Trample));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.pt(bears), (5, 5), "X isn't recounted");
    assert_eq!(t.life(P1), 20 - 5 - 2 - 3);
}

#[test]
fn xenagos_gives_another_creature_haste_and_doubles_its_power() {
    cr!("608.2h");
    ruling!("Xenagos, God of Revels", "The value of X is calculated only once, as the ability resolves.");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Xenagos, God of Revels");
    let giant = t.battlefield_sick(P0, "Hill Giant");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(giant), (6, 6));
    assert!(t.obj_now(giant).has_keyword(KeywordKind::Haste));
}

#[test]
fn square_up_sets_base_pt_under_other_modifications() {
    cr!("613.4b", "613.4c");
    ruling!("Square Up", "will apply after its base power and toughness are set, regardless of the order");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.resolve();
    let spell = t.hand(P0, "Square Up");
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (7, 7));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn ovinize_removes_abilities_but_later_ones_stay() {
    cr!("613.1f", "613.4b");
    ruling!("Ovinize", "If the affected creature gains an ability after Ovinize resolves, it will keep that ability");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    t.lands(P0, "Island", 3);
    let spell = t.hand(P0, "Ovinize");
    t.cast(P0, spell).target(angel).go();
    t.resolve();
    assert_eq!(t.pt(angel), (0, 1));
    assert!(!t.obj_now(angel).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(angel).has_keyword(KeywordKind::Vigilance));
    let jump = t.hand(P0, "Jump");
    t.cast(P0, jump).target(angel).go();
    t.resolve();
    assert!(t.obj_now(angel).has_keyword(KeywordKind::Flying));
}

#[test]
fn water_wings_sets_base_pt_and_grants_keywords() {
    cr!("613.4b", "613.1f");
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P0, "Savannah Lions");
    t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "Water Wings");
    t.cast(P0, spell).target(lions).go();
    t.resolve();
    assert_eq!(t.pt(lions), (4, 4));
    assert!(t.obj_now(lions).has_keyword(KeywordKind::Flying));
    assert!(t.obj_now(lions).has_keyword(KeywordKind::Hexproof));
}

#[test]
fn twisted_image_switches_power_and_toughness() {
    cr!("613.4d", "704.5f");
    ruling!("Twisted Image", "nonlethal damage dealt to a creature may become lethal");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Island", 1);
    let spell = t.hand(P0, "Twisted Image");
    let hand = t.hand_size(P0);
    t.cast(P0, spell).target(wall).go();
    t.resolve();
    assert!(!t.on_battlefield(wall), "an 8/0 is put into the graveyard");
    assert_eq!(t.hand_size(P0), hand, "drew a card (the spell left the hand)");
}

#[test]
fn switching_applies_after_other_pt_changes() {
    cr!("613.4d");
    ruling!("Twisted Image", "Effects that switch a creature's power and toughness apply after all other effects");
    let mut t = TestGame::new(2);
    let fiend = t.battlefield(P0, "Phantasmal Fiend");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 1);
    // Switch first, then +1/-1: 1/5 + (+1/-1) = 2/4, switched = 4/2.
    t.activate(P0, fiend, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(fiend), (5, 1));
    t.activate(P0, fiend, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(fiend), (4, 2));
}

#[test]
fn might_of_alara_counts_basic_land_types() {
    cr!("608.2h");
    ruling!("Might of Alara", "count the number of basic land types among lands you control");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    let spell = t.hand(P0, "Might of Alara");
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn rabid_elephant_gets_bigger_for_each_blocker() {
    cr!("509.1h");
    let mut t = TestGame::new(2);
    let elephant = t.battlefield(P0, "Rabid Elephant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    t.attack(
        &[(elephant, Entity::Player(P1))],
        &[(b1, elephant), (b2, elephant)],
    );
    assert_eq!(t.pt(elephant), (7, 8));
    assert!(!t.on_battlefield(b1) && !t.on_battlefield(b2));
    assert!(t.on_battlefield(elephant));
}

#[test]
fn jungle_wurm_shrinks_for_each_blocker_beyond_the_first() {
    cr!("509.1h");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Jungle Wurm");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    let b3 = t.battlefield(P1, "Grizzly Bears");
    to_blocks(
        &mut t,
        &[(wurm, Entity::Player(P1))],
        &[(b1, wurm), (b2, wurm), (b3, wurm)],
    );
    t.resolve_all();
    assert_eq!(t.pt(wurm), (3, 3));
}
