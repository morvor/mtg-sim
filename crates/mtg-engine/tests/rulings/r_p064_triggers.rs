//! Rulings batch P064 — triggered abilities that grant flying: when their conditions are
//! checked (on resolution, CR 608.2; intervening "if", 603.4; look-back, 603.10), APNAP
//! order (101.4), "the first time each turn", multiplayer counts, and fizzling (608.2b).

use crate::r_p064_common::*;
use crate::r_s01_common::supported;
use crate::r_s14_common::triggers_from;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Majestic Myriarch
// ---------------------------------------------------------------------------------------

#[test]
fn majestic_myriarch_pt_counts_itself_and_works_in_every_zone() {
    cr!("604.3", "604.3a");
    ruling!(
        "Majestic Myriarch",
        "Majestic Myriarch's first ability applies in all zones. While it's on the battlefield, it counts itself, so it'll be at least 2/2."
    );
    supported("Majestic Myriarch");
    let mut t = TestGame::new(2);
    let h = t.hand(P0, "Majestic Myriarch");
    t.g.recompute();
    assert_eq!(t.pt(h), (0, 0));
    t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(h), (2, 2), "in hand: twice one creature");
    let b = t.battlefield(P0, "Majestic Myriarch");
    assert_eq!(t.pt(b), (4, 4), "counting itself");
}

#[test]
fn majestic_myriarch_triggers_each_combat_whatever_you_control() {
    cr!("603.2", "507.1");
    ruling!(
        "Majestic Myriarch",
        "Majestic Myriarch's second ability triggers at the beginning of each combat, not just combat on your turn"
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Majestic Myriarch");
    crate::r_p057_common::into_beginning_of_combat(&mut t, P1);
    assert_eq!(triggers_from(&t, m), 1, "on an opponent's turn, with no keywords");
    t.resolve_all();
    assert!(!flies(&mut t, m));
}

#[test]
fn majestic_myriarch_sees_abilities_gained_before_it_resolves() {
    cr!("608.2", "603.3b");
    ruling!(
        "Majestic Myriarch",
        "If a creature gains one of the listed abilities before Majestic Mryiarch's triggered ability resolves"
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Majestic Myriarch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_p057_common::into_beginning_of_combat(&mut t, P0);
    assert_eq!(triggers_from(&t, m), 1);
    cast_and_resolve(&mut t, P0, "Jump", &[obj(bears)]);
    assert!(flies(&mut t, m), "the Bears flew as the trigger resolved");
}

#[test]
fn majestic_myriarch_keeps_what_it_gained_after_the_flyer_leaves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Majestic Myriarch",
        "Which abilities Majestic Myriarch gains is determined as the ability resolves."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Majestic Myriarch");
    let angel = t.battlefield(P0, "Serra Angel");
    crate::r_p057_common::into_beginning_of_combat(&mut t, P0);
    t.resolve_all();
    assert!(flies(&mut t, m));
    assert!(has(&mut t, m, KeywordKind::Vigilance));
    crate::r_s02_common::destroy(&mut t, angel);
    assert!(flies(&mut t, m));
    assert!(has(&mut t, m, KeywordKind::Vigilance));
    t.battlefield(P0, "Raging Goblin");
    assert!(!has(&mut t, m, KeywordKind::Haste), "a later haste creature adds nothing");
}

#[test]
fn majestic_myriarch_gains_the_specific_variant() {
    cr!("702.11d", "702.11b");
    ruling!(
        "Majestic Myriarch",
        "If one of those creatures has one or more variants of the listed keywords (for example, hexproof from white), this creature gains those specific variants."
    );
    supported("Knight of Grace");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Majestic Myriarch");
    t.battlefield(P0, "Knight of Grace");
    crate::r_p057_common::into_beginning_of_combat(&mut t, P0);
    t.resolve_all();
    assert!(has(&mut t, m, KeywordKind::FirstStrike));
    // Hexproof from black: a black spell of the opponent can't target it, a red one can.
    t.set_step(P1, Step::PrecombatMain);
    let murder = crate::r_s04_common::spell_targets(&mut t, P1, "Murder");
    assert!(!murder.contains(&obj(m)), "hexproof from black");
    let shock = crate::r_s04_common::spell_targets(&mut t, P1, "Shock");
    assert!(shock.contains(&obj(m)), "not hexproof from everything");
}

// ---------------------------------------------------------------------------------------
// Attack triggers
// ---------------------------------------------------------------------------------------

#[test]
fn thunderous_orator_gains_the_ability_you_have_not_flying() {
    cr!("608.2c", "508.1m");
    ruling!(
        "Thunderous Orator",
        "If you control a creature with one of the abilities listed in the last sentence, Thunderous Orator will gain the that ability, not flying."
    );
    supported("Thunderous Orator");
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Thunderous Orator");
    t.battlefield(P0, "Knight of Grace");
    crate::r_s01_common::attack_with(&mut t, &[(o, Entity::Player(P1))]);
    t.resolve_all();
    assert!(has(&mut t, o, KeywordKind::FirstStrike));
    assert!(!flies(&mut t, o));
}

#[test]
fn skyclaw_thrash_losing_the_flip_still_attacks() {
    cr!("705.2", "508.1m");
    ruling!(
        "Skyclaw Thrash",
        "If you lose the flip, nothing happens. Skyclaw Thrash continues to attack as normal."
    );
    supported("Skyclaw Thrash");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skyclaw Thrash");
    crate::r_p017_common::load_coins(&mut t, &[false, false]);
    t.attack(&[(s, Entity::Player(P1))], &[]);
    assert!(!flies(&mut t, s));
    assert_eq!(t.life(P1), 16, "a 4/4 attacked as normal");
}

#[test]
fn kitesail_corsair_flies_only_once_attacking() {
    cr!("508.1", "508.1m", "506.4");
    ruling!(
        "Kitesail Corsair",
        "Kitesail Corsair has flying immediately after it attacks."
    );
    supported("Kitesail Corsair");
    supported("Windreader Sphinx");
    // Abilities that trigger when a creature with flying attacks do trigger.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kitesail Corsair");
    let s = t.battlefield(P1, "Windreader Sphinx");
    crate::r_s01_common::attack_with(&mut t, &[(k, Entity::Player(P1))]);
    assert_eq!(triggers_from(&t, s), 1);
    // Restrictions on attacking that look at flying don't apply: Moat stops it.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kitesail Corsair");
    t.battlefield(P1, "Moat");
    crate::r_s20_common::to_beginning_of_combat(&mut t, P0);
    assert!(!crate::r_s02_common::can_attack(&mut t, k));
}

#[test]
fn reaper_of_night_attacking_a_planeswalker_checks_its_controller() {
    cr!("506.2", "508.5", "603.4");
    ruling!(
        "Reaper of Night // Harvest Fear",
        "If Reaper of Night attacks a planeswalker, the controller of the planeswalker is the defending player."
    );
    ruling!(
        "Reaper of Night // Harvest Fear",
        "Once Reaper of Night's ability has resolved, the number of cards in defending player's hand no longer matters."
    );
    supported("Reaper of Night // Harvest Fear");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Reaper of Night // Harvest Fear");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.hand(P1, "Forest");
    t.hand(P1, "Forest");
    crate::r_s01_common::attack_with(&mut t, &[(r, obj(jace))]);
    assert_eq!(triggers_from(&t, r), 1, "P1 has two cards");
    t.resolve_all();
    assert!(flies(&mut t, r));
    for _ in 0..3 {
        t.hand(P1, "Forest");
    }
    assert!(flies(&mut t, r), "the hand size no longer matters");
}

#[test]
fn bladewings_thrall_returns_under_its_owners_control() {
    cr!("603.2", "110.2");
    ruling!(
        "Bladewing's Thrall",
        "The second ability of Bladewing's Thrall triggers even if a Dragon enters the battlefield under an opponent's control."
    );
    supported("Bladewing's Thrall");
    let mut t = TestGame::new(2);
    let th = t.graveyard(P0, "Bladewing's Thrall");
    t.answer_yes(P0, true);
    t.enter(P1, "Shivan Dragon");
    t.settle();
    assert_eq!(triggers_from(&t, th), 1);
    t.resolve_all();
    assert!(t.on_battlefield(th));
    assert_eq!(t.obj_now(th).controller, P0);
    assert!(!flies(&mut t, th), "P0 controls no Dragon");
}

#[test]
fn marketwatch_phantom_checks_power_only_as_the_creature_enters() {
    cr!("603.2", "603.10a");
    ruling!(
        "Marketwatch Phantom",
        "Marketwatch Phantom's ability checks the power of a creature only at the moment it enters the battlefield."
    );
    supported("Marketwatch Phantom");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Marketwatch Phantom");
    let b = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_from(&t, m), 1);
    cast_and_resolve(&mut t, P0, "Giant Growth", &[obj(b)]);
    assert!(flies(&mut t, m), "the Bears' power grew after it triggered");
    // A 2/2 entering with a +1/+1 counter has it counted: power 3, no trigger.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Marketwatch Phantom");
    let bears = t.g.create_card_object(
        mtg_engine::card::card("Grizzly Bears"),
        P0,
        mtg_engine::object::Zone::Nowhere,
    );
    t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: bears,
        to: mtg_engine::object::Zone::Battlefield,
        pos: mtg_engine::ability::LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P0),
        etb: mtg_engine::replacement::EtbInfo {
            controller: Some(P0),
            counters: vec![("+1/+1".into(), 1)],
            ..Default::default()
        },
        source: None,
    })
    .expect("entered");
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_from(&t, m), 0);
}

#[test]
fn mystic_skyfish_triggers_only_on_the_second_draw() {
    cr!("603.2", "121.2");
    ruling!(
        "Mystic Skyfish",
        "The triggered ability can trigger only once each turn. It doesn't matter if Mystic Skyfish was on the battlefield when the first card was drawn."
    );
    supported("Mystic Skyfish");
    let draw = |t: &mut TestGame| {
        t.g.draw_cards(P0, 1);
        t.g.flush_events();
        t.settle();
    };
    let mut t = TestGame::new(2);
    draw(&mut t);
    let s = t.battlefield(P0, "Mystic Skyfish");
    draw(&mut t);
    assert_eq!(triggers_from(&t, s), 1);
    t.resolve_all();
    assert!(flies(&mut t, s));
    draw(&mut t);
    assert_eq!(triggers_from(&t, s), 0, "not for the third card");
    // Not on the battlefield for the second card: no trigger that turn.
    let mut t = TestGame::new(2);
    draw(&mut t);
    draw(&mut t);
    let s = t.battlefield(P0, "Mystic Skyfish");
    draw(&mut t);
    assert_eq!(triggers_from(&t, s), 0);
}

#[test]
fn uthros_research_craft_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Uthros Research Craft",
        "It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    supported("Uthros Research Craft");
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Uthros Research Craft");
    counters(&mut t, u, "charge", 3);
    let orn = crate::r_s25_common::cast_new(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    assert_eq!(triggers_from(&t, u), 1);
    crate::r_s25_common::cast_new(&mut t, P1, "Counterspell", &[obj(orn)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!(t.hand_size(P0), hand + 1, "the trigger still drew a card");
    assert_eq!(t.counters(u, "charge"), 4);
}

#[test]
fn kithkeeper_counts_colors_once_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Kithkeeper",
        "The value of X is calculated only once, as Kithkeeper's first ability resolves."
    );
    supported("Kithkeeper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wind Drake");
    t.enter(P0, "Kithkeeper");
    t.resolve_all();
    // White and blue: two tokens (their green doesn't add a third).
    assert_eq!(
        crate::r_s27_common::tokens_named(&t, "Kithkin Token").len(),
        2
    );
}

// ---------------------------------------------------------------------------------------
// Embereth Skyblazer (multiplayer)
// ---------------------------------------------------------------------------------------

/// A game of `n` players where P0's Embereth Skyblazer attacks P1, with its trigger on the
/// stack and the mana to pay {2}{R} twice.
fn skyblazer_attacks(n: usize) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(n);
    let s = t.battlefield(P0, "Embereth Skyblazer");
    crate::r_s01_common::attack_with(&mut t, &[(s, Entity::Player(P1))]);
    assert_eq!(triggers_from(&t, s), 1);
    pool(&mut t, P0, ManaType::R, 2);
    pool(&mut t, P0, ManaType::C, 4);
    (t, s)
}

#[test]
fn embereth_skyblazer_counts_opponents_still_in_the_game_as_it_resolves() {
    cr!("608.2h", "800.4a");
    ruling!(
        "Embereth Skyblazer",
        "In a multiplayer game, opponents who have left the game before the ability resolves aren't counted"
    );
    ruling!(
        "Embereth Skyblazer",
        "The value of X is determined only as Embereth Skyblazer's triggered ability resolves."
    );
    supported("Embereth Skyblazer");
    let (mut t, s) = skyblazer_attacks(4);
    crate::r_p079_common::concede(&mut t, P2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(s), (6, 3), "two opponents left");
    crate::r_p079_common::concede(&mut t, P3);
    assert_eq!(t.pt(s), (6, 3), "X doesn't change later");
}

#[test]
fn embereth_skyblazer_cost_is_paid_only_once() {
    cr!("603.5", "608.2c");
    ruling!(
        "Embereth Skyblazer",
        "While resolving Embereth Skyblazer's triggered ability, you can't pay {2}{R} multiple times to increase the bonus."
    );
    let (mut t, s) = skyblazer_attacks(2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(s), (5, 3));
    assert_eq!(t.g.player(P0).mana_pool.total(), 3, "paid once");
}

// ---------------------------------------------------------------------------------------
// APNAP, look-back and timing
// ---------------------------------------------------------------------------------------

#[test]
fn warped_researcher_opponents_cycling_trigger_resolves_first_on_your_turn() {
    cr!("101.4", "603.3b", "702.29a");
    ruling!(
        "Warped Researcher",
        "On your turn, an opponent can cycle a card that has a cycling triggered ability and target this card with that ability."
    );
    supported("Warped Researcher");
    supported("Choking Tethers");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Warped Researcher");
    let ct = t.hand(P1, "Choking Tethers");
    pool(&mut t, P1, ManaType::U, 2);
    t.answer_targets(P1, &[obj(w)]);
    t.answer_yes(P1, true);
    crate::r_s04_common::cycle(&mut t, P1, ct, 0).unwrap();
    t.resolve_all();
    assert!(t.obj_now(w).tapped, "tapped before it gained shroud");
    assert!(flies(&mut t, w));
}

#[test]
fn bloodfeather_phoenix_must_already_be_in_the_graveyard() {
    cr!("603.2", "603.10");
    ruling!(
        "Bloodfeather Phoenix",
        "Bloodfeather Phoenix's last ability triggers only if Bloodfeather Phoenix is in your graveyard at the moment"
    );
    supported("Bloodfeather Phoenix");
    supported("Hurricane");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Bloodfeather Phoenix");
    t.lands(P0, "Forest", 3);
    let h = t.hand(P0, "Hurricane");
    t.cast(P0, h).x(2).go();
    t.resolve();
    assert!(!t.on_battlefield(p));
    assert_eq!(t.life(P1), 18);
    assert_eq!(triggers_from(&t, t.g.current(p)), 0, "it died at the same time");
    // Already in the graveyard: a Shock to the opponent triggers it.
    crate::r_s25_common::cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert_eq!(triggers_from(&t, t.g.current(p)), 1);
}

#[test]
fn elvish_dreadlord_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Elvish Dreadlord",
        "Elvish Dreadlord's middle ability affects only creatures on the battlefield that aren't Elves at the time it resolves."
    );
    supported("Elvish Dreadlord");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Elvish Dreadlord");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s02_common::destroy(&mut t, d);
    t.resolve_all();
    assert!(t.on_battlefield(elf));
    assert!(!t.on_battlefield(giant), "a 3/3 got -3/-3");
    let later = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(later), (3, 3), "entered later");
}

#[test]
fn ebony_fly_choice_is_made_on_resolution_only() {
    cr!("608.2d", "706.2");
    ruling!(
        "Ebony Fly",
        "You choose whether Ebony Fly becomes a creature as the second ability resolves, after you see the result of the die roll."
    );
    supported("Ebony Fly");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Ebony Fly");
    crate::r_s35_common::load_dice(&mut t, &[2, 5]);
    pool(&mut t, P0, ManaType::C, 8);
    t.answer_yes(P0, false);
    activate_and_resolve(&mut t, P0, f, "Roll a d6");
    assert!(!crate::r_s20_common::creature_now(&mut t, f), "declined");
    t.answer_yes(P0, true);
    activate_and_resolve(&mut t, P0, f, "Roll a d6");
    assert!(crate::r_s20_common::creature_now(&mut t, f));
    assert_eq!(t.pt(f), (5, 5));
    assert!(flies(&mut t, f));
}

#[test]
fn veiled_apparition_upkeep_payment_chosen_on_resolution() {
    cr!("608.2", "701.21a");
    ruling!(
        "Veiled Apparition",
        "You choose whether to pay or not on resolution. If not, then you sacrifice the creature."
    );
    supported("Veiled Apparition");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Veiled Apparition");
    t.set_step(P1, Step::PrecombatMain);
    crate::r_s25_common::cast_new(&mut t, P1, "Grizzly Bears", &[]);
    t.resolve_all();
    assert!(crate::r_s20_common::creature_now(&mut t, v));
    assert!(flies(&mut t, v));
    // P0's upkeep: P0 declines to pay; it's sacrificed.
    crate::r_s04_common::next_upkeep(&mut t, P0);
    pool(&mut t, P0, ManaType::U, 1);
    pool(&mut t, P0, ManaType::C, 1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Veiled Apparition"));
}

// ---------------------------------------------------------------------------------------
// Illegal targets
// ---------------------------------------------------------------------------------------

#[test]
fn urborg_scavengers_without_its_target_gets_no_counter() {
    cr!("608.2b");
    ruling!(
        "Urborg Scavengers",
        "If the target of the first ability is an illegal target at the time the ability tries to resolve"
    );
    supported("Urborg Scavengers");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P1, "Serra Angel");
    t.answer_targets(P0, &[obj(card)]);
    let u = t.enter(P0, "Urborg Scavengers");
    t.settle();
    assert_eq!(triggers_from(&t, u), 1);
    crate::r_s05_common::move_to(&mut t, card, mtg_engine::object::Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(t.counters(u, "+1/+1"), 0);
}

#[test]
fn extinguisher_battleship_without_its_target_deals_no_damage() {
    cr!("608.2b");
    ruling!(
        "Extinguisher Battleship",
        "If the target permanent is an illegal target as Extinguisher Battleship’s first ability tries to resolve"
    );
    supported("Extinguisher Battleship");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(anthem)]);
    let e = t.enter(P0, "Extinguisher Battleship");
    t.settle();
    assert_eq!(triggers_from(&t, e), 1);
    crate::r_s05_common::move_to(&mut t, anthem, mtg_engine::object::Zone::Hand(P1));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn bahamut_chapter_three_fizzles_and_the_saga_is_sacrificed() {
    cr!("608.2b", "714.4");
    ruling!(
        "Dion, Bahamut's Dominant // Bahamut, Warden of Light",
        "If the target permanent is an illegal target when Bahamut's third chapter ability tries to resolve"
    );
    supported("Dion, Bahamut's Dominant // Bahamut, Warden of Light");
    let mut t = TestGame::new(2);
    let b = crate::r_p079_common::back_face(&mut t, P0, "Dion, Bahamut's Dominant // Bahamut, Warden of Light");
    t.resolve_all();
    crate::r_s19_common::add_lore(&mut t, b, 1);
    t.resolve_all();
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_targets(P0, &[obj(anthem)]);
    crate::r_s19_common::add_lore(&mut t, b, 1);
    assert_eq!(crate::r_s19_common::chapters_on_stack(&t, b), vec![3]);
    crate::r_s05_common::move_to(&mut t, anthem, mtg_engine::object::Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(
        t.zone(b),
        mtg_engine::object::Zone::Graveyard(P0),
        "sacrificed for its lore counters, not exiled and returned"
    );
}
