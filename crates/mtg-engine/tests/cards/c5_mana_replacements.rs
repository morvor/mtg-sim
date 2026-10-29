//! Mana, replacement and prevention grammar, round 5 (cards5-mana-replacements:
//! `oracle/patterns/mana_6.rs`, `replacements_6.rs`, `prevention_6.rs`): mana of given
//! colors spent on one payment, an optional extra mana chosen by the tapped land's
//! controller, mana in any combination of a card's colors, draw replacements by cause,
//! with a revealed card and an offer to other players, a targeted next-draw replacement,
//! a land-entering replacement keyed on the player who puts it there, a prevention
//! trigger for a group shield, and damage-history effects about a target.

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

fn tap(t: &mut TestGame, p: PlayerId, land: ObjectId) {
    t.activate(p, land, 0, &[]).unwrap();
}

fn treasure(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let spec = mtg_engine::tokens::predefined("Treasure").expect("Treasure token");
    let tc = mtg_engine::replacement::TokenCreate {
        chars: mtg_engine::tokens::token_characteristics(&spec),
        card: None,
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    t.g.create_tokens(p, tc, 1, None)[0]
}

// ---------------------------------------------------------------------------
// Mana spent on one payment
// ---------------------------------------------------------------------------

/// Balduvian Fallen with `age` age counters, at the start of P0's upkeep, paying with
/// `lands`; returns its power after the payment's trigger resolved.
fn balduvian_after_upkeep(age: u32, lands: &[&str]) -> (i32, bool) {
    let mut t = TestGame::new(2);
    let fallen = t.battlefield(P0, "Balduvian Fallen");
    if age > 0 {
        t.g.add_counters(Entity::Object(fallen), "age", age, None);
    }
    for l in lands {
        t.battlefield(P0, l);
    }
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let alive = t.on_battlefield(fallen);
    (if alive { t.pt(fallen).0 } else { 0 }, alive)
}

#[test]
fn balduvian_fallen_gets_power_for_each_black_or_red_mana_spent_on_its_upkeep() {
    cr!("702.24a", "601.2h", "106.1");
    assert_supported("Balduvian Fallen");
    // Two age counters: {1}{1} paid with a Swamp and a Mountain: +2/+0.
    assert_eq!(balduvian_after_upkeep(1, &["Swamp", "Mountain"]), (5, true));
    // Paid with a Swamp and a Plains: +1/+0.
    assert_eq!(balduvian_after_upkeep(1, &["Swamp", "Plains"]), (4, true));
    // Paid with white and green mana: no bonus.
    assert_eq!(balduvian_after_upkeep(1, &["Plains", "Forest"]), (3, true));
}

#[test]
fn mana_of_given_colors_spent_to_activate_or_cast_is_counted() {
    cr!("602.2b", "601.2h");
    // "{X}: ... for each {R} spent to activate this ability": only the red mana counts.
    let def = oracle_card(
        "Red Counter",
        "Artifact",
        "{2}: Red Counter deals 1 damage to target player for each {B} or {R} spent to activate this ability.",
    );
    let mut t = TestGame::new(2);
    let src = t.custom(P0, def, Zone::Battlefield);
    t.battlefield(P0, "Mountain");
    t.battlefield(P0, "Plains");
    t.activate(P0, src, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // "for each mana from Treasures spent to activate this ability".
    let def = oracle_card(
        "Treasure Counter",
        "Artifact",
        "{2}: Draw a card for each mana from Treasures spent to activate this ability.",
    );
    let mut t = TestGame::new(2);
    let src = t.custom(P0, def, Zone::Battlefield);
    t.battlefield(P0, "Plains");
    treasure(&mut t, P0);
    let hand = t.hand_size(P0);
    t.activate(P0, src, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // "for each {B} spent to cast it" in an enters trigger.
    let def = oracle_card(
        "Black Counter",
        "Creature — Horror",
        "When this creature enters, draw a card for each {B} spent to cast it.",
    );
    let mut def = def;
    def.faces[0].chars.mana_cost = mtg_engine::mana::ManaCost::parse("{1}{B}{B}");
    let mut t = TestGame::new(2);
    let c = t.custom(P0, def, Zone::Hand(P0));
    t.battlefield(P0, "Swamp");
    t.battlefield(P0, "Swamp");
    t.battlefield(P0, "Swamp");
    let hand = t.hand_size(P0);
    t.cast(P0, c).go();
    t.resolve_all();
    // Three black mana paid {1}{B}{B}: three cards (the creature left the hand).
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
}

// ---------------------------------------------------------------------------
// Snowfall: the tapped land's controller may add mana
// ---------------------------------------------------------------------------

#[test]
fn snowfall_lets_the_islands_controller_add_more_blue_for_cumulative_upkeep() {
    cr!("106.12a", "605.1b", "106.6");
    ruling!("Snowfall", "is blue regardless of what type of mana the Island produces");
    assert_supported("Snowfall");
    // P1 taps an Island: P1 (not Snowfall's controller) chooses to add {U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Snowfall");
    let island = t.battlefield(P1, "Island");
    t.answer_yes(P1, true);
    tap(&mut t, P1, island);
    assert_eq!(pool(&t, P1), vec![ManaType::U, ManaType::U]);
    assert!(pool(&t, P0).is_empty());
    // The extra mana can pay only cumulative upkeep costs.
    let extra = t.g.player(P1).mana_pool.mana.iter().filter(|m| m.restriction.is_some()).count();
    assert_eq!(extra, 1);
    // Declining adds nothing more.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Snowfall");
    let island = t.battlefield(P1, "Island");
    t.answer_yes(P1, false);
    tap(&mut t, P1, island);
    assert_eq!(pool(&t, P1), vec![ManaType::U]);
    // A snow Island: {U}{U} instead.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Snowfall");
    let island = t.battlefield(P0, "Snow-Covered Island");
    t.answer_yes(P0, true);
    tap(&mut t, P0, island);
    assert_eq!(pool(&t, P0), vec![ManaType::U, ManaType::U, ManaType::U]);
}

// ---------------------------------------------------------------------------
// Omnath, Locus of All
// ---------------------------------------------------------------------------

#[test]
fn omnath_reveals_a_card_with_three_colored_symbols_for_mana_of_its_colors() {
    cr!("106.1", "105.2", "202.1", "107.4e");
    ruling!(
        "Omnath, Locus of All",
        "The mana symbols in the card you reveal may be in any combination of colors"
    );
    assert_supported("Omnath, Locus of All");
    // A card with three colored mana symbols: revealed, three mana of its colors.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omnath, Locus of All");
    t.library_top(P0, "Lightning Helix");
    t.library_top(P0, "Island");
    // Lightning Helix ({R}{W}) has two: not revealed, put into hand.
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert!(t.in_hand(P0, "Island"));
    assert!(t.in_hand(P0, "Lightning Helix"));
    assert!(pool(&t, P0).is_empty());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omnath, Locus of All");
    t.library_top(P0, "Cryptic Command");
    t.library_top(P0, "Island");
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    // Draw step drew the Island; the trigger looks at Cryptic Command ({1}{U}{U}{U}).
    t.resolve_all();
    assert!(t.in_hand(P0, "Cryptic Command"));
    assert_eq!(pool(&t, P0), vec![ManaType::U, ManaType::U, ManaType::U]);
    // Choosing not to reveal it: just put into hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omnath, Locus of All");
    t.library_top(P0, "Cryptic Command");
    t.library_top(P0, "Island");
    t.answer_yes(P0, false);
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert!(t.in_hand(P0, "Cryptic Command"));
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn mana_in_any_combination_of_a_cards_colors() {
    cr!("106.1", "106.5");
    let def = oracle_card(
        "Prism Peek",
        "Sorcery",
        "Reveal the top card of your library. Add one mana of any of that card's colors.",
    );
    let mut def = def;
    def.faces[0].chars.mana_cost = mtg_engine::mana::ManaCost::parse("{0}");
    let mut t = TestGame::new(2);
    let s = t.custom(P0, def, Zone::Hand(P0));
    t.library_top(P0, "Lightning Bolt");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    // A colorless card: no mana.
    let def = oracle_card(
        "Prism Peek",
        "Sorcery",
        "Reveal the top card of your library. Add one mana of any of that card's colors.",
    );
    let mut def = def;
    def.faces[0].chars.mana_cost = mtg_engine::mana::ManaCost::parse("{0}");
    let mut t = TestGame::new(2);
    let s = t.custom(P0, def, Zone::Hand(P0));
    t.library_top(P0, "Ornithopter");
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(pool(&t, P0).is_empty());
}

// ---------------------------------------------------------------------------
// Words of War
// ---------------------------------------------------------------------------

#[test]
fn words_of_war_replaces_the_next_draw_with_damage_to_the_target_chosen_on_activation() {
    cr!("614.1a", "614.11", "115.1c", "602.2b");
    interp!("words-target-on-activation");
    assert_supported("Words of War");
    let mut t = TestGame::new(2);
    let words = t.battlefield(P0, "Words of War");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, words, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P1), 18);
    // Used up: the next draw is a draw.
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn words_of_war_target_gone_still_replaces_the_draw_without_damage() {
    cr!("614.1a", "400.7", "608.2b");
    interp!("words-target-on-activation");
    let mut t = TestGame::new(2);
    let words = t.battlefield(P0, "Words of War");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, words, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    // The Bears leaves the battlefield before the draw.
    t.g.destroy(bears, None);
    t.settle();
    let hand = t.hand_size(P0);
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    // The draw was replaced (no card), and no damage was dealt to anything.
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.g.obj(giant).damage, 0);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn several_words_shields_each_replace_one_draw() {
    cr!("616.1", "614.11");
    ruling!("Words of War", "you can choose which one to apply");
    let mut t = TestGame::new(2);
    let words = t.battlefield(P0, "Words of War");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, words, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    t.activate(P0, words, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    let hand = t.hand_size(P0);
    // The drawing player chooses which applies: the one aimed at the Bears first.
    t.answer(P0, DecisionKind::Replacement, Answer::Index(1));
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand);
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------
// Zur's Weirding
// ---------------------------------------------------------------------------

#[test]
fn zurs_weirding_reveals_the_draw_and_another_player_may_pay_to_mill_it() {
    cr!("614.6", "614.5", "118.12", "101.4");
    ruling!("Zur's Weirding", "then the card was never drawn or discarded");
    assert_supported("Zur's Weirding");
    // P1 pays 2 life: P0's card goes to the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zur's Weirding");
    t.library_top(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.answer_yes(P1, true);
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P1), 18);
    // The drawing player isn't offered the payment.
    assert!(!t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
    // Nobody pays: the card is drawn (and not replaced again, CR 614.5).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zur's Weirding");
    t.library_top(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.answer_yes(P1, false);
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn zurs_weirding_replaces_each_draw_one_at_a_time() {
    cr!("614.6", "121.2");
    ruling!(
        "Zur's Weirding",
        "draw the cards one at a time and deal with this replacement before drawing the next card"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zur's Weirding");
    t.library_top(P1, "Island");
    t.library_top(P1, "Grizzly Bears");
    let hand = t.hand_size(P1);
    // P0 pays for the first card (the Bears), not for the second (the Island).
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.g.draw_cards(P1, 2);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Island"));
    assert_eq!(t.hand_size(P1), hand + 1);
    assert_eq!(t.life(P0), 18);
}

// ---------------------------------------------------------------------------
// Unpredictable Cyclone
// ---------------------------------------------------------------------------

#[test]
fn unpredictable_cyclone_replaces_a_cycling_draw_with_a_free_spell_of_a_shared_type() {
    cr!("702.29a", "614.1a", "121.1");
    ruling!(
        "Unpredictable Cyclone",
        "you stop when you exile a card that shares any of those types"
    );
    assert_supported("Unpredictable Cyclone");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unpredictable Cyclone");
    t.set_step(P0, Step::PrecombatMain);
    // Cycling a creature card: exile until a creature card, cast it free.
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Lightning Bolt");
    t.library_top(P0, "Island");
    // Hundroog: a creature card with cycling {3}.
    let hand_card = t.hand(P0, "Hundroog");
    t.lands(P0, "Forest", 3);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.activate(P0, hand_card, 0, &[]).unwrap();
    t.resolve_all();
    // No card drawn: the Bears was cast (it's on the battlefield), the others went to
    // the bottom.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.hand_size(P0), hand - 1);
    assert!(!t.in_exile("Lightning Bolt"));
    assert!(!t.in_exile("Island"));
    let mut bottom: Vec<String> = t.g.player(P0).library[..2]
        .iter()
        .map(|o| t.g.obj(*o).name().to_string())
        .collect();
    bottom.sort();
    assert_eq!(bottom, vec!["Island", "Lightning Bolt"]);
}

#[test]
fn unpredictable_cyclone_ignores_its_own_cycling_lands_and_other_draws() {
    cr!("702.29a", "614.1a");
    // Cycling a land card: the draw happens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unpredictable Cyclone");
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Grizzly Bears");
    let land = t.hand(P0, "Secluded Steppe");
    t.lands(P0, "Plains", 1);
    // Its second activated ability: cycling {W}.
    t.activate(P0, land, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // An ordinary draw: not replaced.
    t.library_top(P0, "Hill Giant");
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    // Cycling another Unpredictable Cyclone is "another nonland card": replaced.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unpredictable Cyclone");
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Seal of Fire");
    t.library_top(P0, "Grizzly Bears");
    let other = t.hand(P0, "Unpredictable Cyclone");
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, false);
    t.activate(P0, other, 0, &[]).unwrap();
    t.resolve_all();
    // Exiled the Bears (no shared type), stopped at Seal of Fire (an enchantment), not
    // cast: both on the bottom, nothing drawn.
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Seal of Fire"));
}

// ---------------------------------------------------------------------------
// Land Equilibrium
// ---------------------------------------------------------------------------

#[test]
fn land_equilibrium_makes_an_opponent_with_as_many_lands_sacrifice_one() {
    cr!("614.1a", "305.1", "305.4");
    ruling!("Land Equilibrium", "they may sacrifice that same land");
    assert_supported("Land Equilibrium");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Land Equilibrium");
    t.lands(P0, "Island", 2);
    t.lands(P1, "Forest", 2);
    let land = t.hand(P1, "Mountain");
    t.set_step(P1, Step::PrecombatMain);
    t.play_land(P1, land).unwrap();
    t.resolve_all();
    // P1 had 2 lands like P0: it enters, then P1 sacrifices a land.
    assert_eq!(t.g.permanents().filter(|o| o.controller == P1).count(), 2);
    // Fewer lands than P0: no sacrifice.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Land Equilibrium");
    t.lands(P0, "Island", 3);
    t.lands(P1, "Forest", 2);
    let land = t.hand(P1, "Mountain");
    t.set_step(P1, Step::PrecombatMain);
    t.play_land(P1, land).unwrap();
    t.resolve_all();
    assert_eq!(t.g.permanents().filter(|o| o.controller == P1).count(), 3);
    // Its controller isn't affected.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Land Equilibrium");
    t.lands(P0, "Island", 2);
    t.lands(P1, "Forest", 2);
    let land = t.hand(P0, "Island");
    t.set_step(P0, Step::PrecombatMain);
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is(types::CardType::Land))
            .count(),
        3
    );
}

#[test]
fn land_equilibrium_applies_to_lands_put_onto_the_battlefield_by_effects() {
    cr!("614.1a", "305.4");
    ruling!("Land Equilibrium", "This effect applies no matter how the land would enter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Land Equilibrium");
    t.lands(P0, "Island", 1);
    t.lands(P1, "Forest", 2);
    t.library_top(P1, "Mountain");
    let explore = t.hand(P1, "Rampant Growth");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, explore).go();
    t.resolve_all();
    // Rampant Growth puts a basic land onto the battlefield: then one is sacrificed (two
    // tapped to cast it, two left plus the new one, less one).
    let lands = t
        .g
        .permanents()
        .filter(|o| o.controller == P1 && o.is(types::CardType::Land))
        .count();
    assert_eq!(lands, 2);
}

// ---------------------------------------------------------------------------
// Judgment of Alexander
// ---------------------------------------------------------------------------

#[test]
fn judgment_of_alexander_prevents_and_commanders_fight_back() {
    cr!("615.5", "603.7", "903.3");
    assert_supported("Judgment of Alexander");
    let mut t = TestGame::new(2);
    let cmd = t.battlefield(P0, "Hill Giant");
    t.g.objects[cmd.0 as usize].is_commander = true;
    t.g.dirty = true;
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.lands(P0, "Plains", 3);
    let j = t.hand(P0, "Judgment of Alexander");
    t.cast(P0, j).go();
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    t.attack(
        &[(bears, Entity::Player(P0)), (lions, Entity::Player(P0))],
        &[],
    );
    t.resolve_all();
    // The damage was prevented, and the commander dealt 3 to each attacking creature.
    assert_eq!(t.life(P0), 20);
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(lions));
    assert!(t.on_battlefield(cmd));
}

#[test]
fn judgment_of_alexander_only_triggers_for_creatures() {
    cr!("615.5");
    let mut t = TestGame::new(2);
    let cmd = t.battlefield(P0, "Hill Giant");
    t.g.objects[cmd.0 as usize].is_commander = true;
    t.g.dirty = true;
    t.lands(P0, "Plains", 3);
    let j = t.hand(P0, "Judgment of Alexander");
    t.cast(P0, j).go();
    t.resolve_all();
    // A burn spell's damage is prevented, with no trigger.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.g.obj(cmd).damage, 0);
}

// ---------------------------------------------------------------------------
// Runesword
// ---------------------------------------------------------------------------

#[test]
fn runesword_creatures_dealt_damage_by_the_target_cant_regenerate_and_are_exiled() {
    cr!("701.19c", "614.1a", "704.5g");
    interp!("runesword-damage-this-turn");
    assert_supported("Runesword");
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Runesword");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let skel = t.battlefield(P1, "Drudge Skeletons");
    t.lands(P1, "Swamp", 1);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(giant, Entity::Player(P1))]),
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(skel, giant)]));
    t.advance_to(P0, Step::DeclareBlockers);
    t.activate(P0, sword, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 3));
    // A regeneration shield for the Skeletons.
    t.activate(P1, skel, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    // Dealt lethal damage by the targeted creature: not regenerated, exiled.
    assert!(t.in_exile("Drudge Skeletons"));
    assert!(!t.in_graveyard(P1, "Drudge Skeletons"));
}

#[test]
fn runesword_doesnt_affect_creatures_damaged_by_other_creatures() {
    cr!("701.19c", "614.1a");
    interp!("runesword-damage-this-turn");
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Runesword");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let skel = t.battlefield(P1, "Drudge Skeletons");
    let blocker = t.battlefield(P1, "Wall of Wood");
    t.lands(P1, "Swamp", 1);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (giant, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(skel, bears), (blocker, giant)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.activate(P0, sword, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    t.activate(P1, skel, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    // The Bears (not the target) killed the Skeletons: it regenerated.
    assert!(t.on_battlefield(skel));
    assert!(t.g.obj(skel).tapped);
}

// ---------------------------------------------------------------------------
// The grammar on other wordings
// ---------------------------------------------------------------------------

#[test]
fn the_grammar_reads_other_wordings_of_these_constructs() {
    cr!("601.2h", "614.1a", "615.5", "702.29a");
    for (ty, text) in [
        (
            "Artifact",
            "{X}: Target player loses 1 life for each {R} spent to activate this ability.",
        ),
        (
            "Artifact",
            "{4}, {T}: Draw a card for each mana from a Treasure spent to activate this ability.",
        ),
        (
            "Enchantment",
            "If a player who controls more lands than you would put a land onto the battlefield, that player instead puts that land onto the battlefield, then sacrifices a land.",
        ),
        (
            "Enchantment",
            "If a cycling ability would cause you to draw a card, instead you gain 2 life.",
        ),
        (
            "Instant",
            "Prevent all damage that would be dealt to you this turn by sources your opponents control. Whenever damage is prevented this way, you gain 1 life.",
        ),
        (
            "Enchantment",
            "Whenever a land is tapped for mana, that player may add an additional {G}.",
        ),
        (
            "Sorcery",
            "Reveal the top card of your library. If it has two or more red mana symbols in its mana cost, draw a card.",
        ),
    ] {
        oracle_card("Probe", ty, text);
    }
}

#[test]
fn a_cycling_draw_replacement_applies_to_any_cycled_card_when_unqualified() {
    cr!("702.29a", "614.1a");
    let def = oracle_card(
        "Cycle Bloom",
        "Enchantment",
        "If a cycling ability would cause you to draw a card, instead you gain 2 life.",
    );
    let mut t = TestGame::new(2);
    t.custom(P0, def, Zone::Battlefield);
    t.set_step(P0, Step::PrecombatMain);
    let land = t.hand(P0, "Secluded Steppe");
    t.lands(P0, "Plains", 1);
    let hand = t.hand_size(P0);
    t.activate(P0, land, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.hand_size(P0), hand - 1);
}
