//! Rulings batch S34 — cost increases, cost reductions, and additional costs change only
//! the total cost a player pays for a spell (CR 601.2f, 118.7, 118.8d), never its mana
//! value, which comes from its mana cost alone (CR 202.3).

use crate::r_s01_common::*;
use crate::r_s04_common::spell_targets;
use crate::r_s08_common::mana_value;
use crate::r_s34_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn monstrous_vortex_discovers_by_the_mana_value_not_the_reduced_cost() {
    cr!("202.3", "601.2f", "701.57a");
    ruling!(
        "Monstrous Vortex",
        "A spell's mana value is determined only by its mana cost. Ignore any alternative costs, additional costs, cost increases, or cost reductions."
    );
    supported("Monstrous Vortex");
    supported("Lashwhip Predator");
    // Monstrous Vortex: "Whenever you cast a creature spell with power 5 or greater,
    // discover X, where X is that spell's mana value." Lashwhip Predator ({4}{G}{G} 5/7,
    // "This spell costs {2} less to cast if your opponents control three or more
    // creatures.") is cast for four mana: X is still 6, so Craw Wurm (mana value 6) is
    // found first.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Monstrous Vortex");
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
    }
    stack_library(&mut t, P0, &["Craw Wurm", "Grizzly Bears"]);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 2);
    let predator = t.hand(P0, "Lashwhip Predator");
    // Put the discovered card into the hand rather than cast it.
    t.answer_yes(P0, false);
    let spell = t.cast(P0, predator).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    assert_eq!(mana_value(&t, spell), 6);
    t.resolve();
    assert!(t.in_hand(P0, "Craw Wurm"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn the_great_henge_costs_its_mana_cost_plus_increases_minus_reductions() {
    cr!("601.2f", "118.7", "202.3");
    ruling!(
        "The Great Henge",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("The Great Henge");
    supported("Thalia, Guardian of Thraben");
    supported("Tanufel Rimespeaker");
    // The Great Henge ({7}{G}{G}): "This spell costs {X} less to cast, where X is the
    // greatest power among creatures you control." With Ghalta (power 12) and P1's Thalia
    // ("Noncreature spells cost {1} more to cast."): {8}{G}{G} reduced by 12 is {G}{G}.
    // (Reducing first and then adding the {1} would make it {1}{G}{G}.) Its mana value is
    // still 9, so Tanufel Rimespeaker ("Whenever you cast a spell with mana value 4 or
    // greater, draw a card.") draws.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.battlefield(P0, "Ghalta, Primal Hunger");
    t.battlefield(P0, "Tanufel Rimespeaker");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 2);
    let henge = t.hand(P0, "The Great Henge");
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, henge).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(mana_value(&t, spell), 9);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.on_battlefield(t.g.current(henge)));
    assert_eq!(mana_value(&t, henge), 9);
}

#[test]
fn gigastorm_titan_cast_for_two_mana_is_still_mana_value_5() {
    cr!("601.2f", "202.3");
    ruling!(
        "Gigastorm Titan",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you’re paying, add any cost increases, then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Gigastorm Titan");
    supported("Up the Beanstalk");
    // Gigastorm Titan ({4}{U}): "This spell costs {3} less to cast if you've cast another
    // spell this turn." Up the Beanstalk: "... whenever you cast a spell with mana value 5
    // or greater, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Up the Beanstalk");
    let thopter = t.hand(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.resolve_all();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 4);
    let titan = t.hand(P0, "Gigastorm Titan");
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, titan).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(mana_value(&t, spell), 5);
    t.resolve_all();
    // Titan left the hand; Up the Beanstalk drew a card.
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.on_battlefield(t.g.current(titan)));
}

#[test]
fn deadly_alliance_cast_for_a_party_discount_keeps_mana_value_5() {
    cr!("601.2f", "700.8", "202.3");
    ruling!(
        "Deadly Alliance",
        "Several cards have a cost reduction based on the number of creatures in your party. To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions. The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported("Deadly Alliance");
    // Deadly Alliance ({4}{B}): "This spell costs {1} less to cast for each creature in
    // your party. Destroy target creature or planeswalker." With a full party and P1's
    // Thalia: {5}{B} reduced by 4 is {1}{B}. Its mana value is 5: Up the Beanstalk draws.
    let mut t = TestGame::new(2);
    full_party(&mut t, P0);
    t.battlefield(P0, "Up the Beanstalk");
    let thalia = t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 5);
    let alliance = t.hand(P0, "Deadly Alliance");
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, alliance).target(thalia).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(mana_value(&t, spell), 5);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P1, "Thalia, Guardian of Thraben"));
}

#[test]
fn rhonas_s_monument_reduces_the_cost_but_not_the_mana_value() {
    cr!("601.2f", "118.7", "202.3");
    ruling!(
        "Rhonas's Monument",
        "To determine the total cost of a creature spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions. The mana value of the creature remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Rhonas's Monument");
    supported("Tempest Hart // Scan the Clouds");
    // Rhonas's Monument: "Green creature spells you cast cost {1} less to cast." Tempest
    // Hart ({3}{G}) is cast for {2}{G}; it still has mana value 4, so Tanufel Rimespeaker
    // draws.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhonas's Monument");
    let tanufel = t.battlefield(P0, "Tanufel Rimespeaker");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    let hart = t.hand(P0, "Tempest Hart // Scan the Clouds");
    let hand = t.hand_size(P0);
    // The Monument's trigger targets Tanufel.
    t.answer_targets(P0, &[Entity::Object(tanufel)]);
    let spell = t.cast(P0, hart).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    assert_eq!(mana_value(&t, spell), 4);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.on_battlefield(t.g.current(hart)));
    assert_eq!(mana_value(&t, hart), 4);
}

#[test]
fn otepec_huntmaster_reduces_a_dinosaur_spell_but_kaervek_sees_its_mana_value() {
    cr!("601.2f", "118.7", "202.3");
    ruling!(
        "Otepec Huntmaster",
        "To determine the total cost of a Dinosaur spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions. The mana value of the creature remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Otepec Huntmaster");
    // Otepec Huntmaster: "Dinosaur spells you cast cost {1} less to cast." Ripjaw Raptor
    // ({2}{G}{G}) is cast for {1}{G}{G}; P1's Kaervek deals 4 damage.
    let mut t = TestGame::new(2);
    kaervek(&mut t);
    t.battlefield(P0, "Otepec Huntmaster");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 2);
    let raptor = t.hand(P0, "Ripjaw Raptor");
    aim_kaervek_at_p0(&mut t);
    t.cast(P0, raptor).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve();
    assert_eq!(t.life(P0), 16);
}

#[test]
fn stormcatch_mentor_kicked_spell_keeps_its_mana_value() {
    cr!("601.2f", "118.8d", "202.3");
    ruling!(
        "Stormcatch Mentor",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of this ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Stormcatch Mentor");
    supported("Burst Lightning");
    supported("Mental Misstep");
    supported("Think Twice");
    // Stormcatch Mentor: "Instant and sorcery spells you cast cost {1} less to cast."
    // Burst Lightning ({R}, kicker {4}) kicked costs {R} + {4} - {1}: four mana. It's still
    // a spell with mana value 1, so Mental Misstep ("Counter target spell with mana value
    // 1.") counters it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stormcatch Mentor");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let bolt = t.hand(P0, "Burst Lightning");
    let spell = t
        .cast(P0, bolt)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(tapped_lands(&t, P0), 4);
    assert_eq!(mana_value(&t, spell), 1);
    assert!(spell_targets(&mut t, P1, "Mental Misstep").contains(&Entity::Object(spell)));
    t.resolve_all();
    // Think Twice ({1}{U}) cast with flashback ({2}{U}) costs {1}{U} and has mana value 2:
    // Mental Misstep can't target it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stormcatch Mentor");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let think = t.graveyard(P0, "Think Twice");
    let flashback = CastMethod::Keyword(KeywordKind::Flashback);
    let spell = t.cast(P0, think).method(flashback).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(mana_value(&t, spell), 2);
    assert!(!spell_targets(&mut t, P1, "Mental Misstep").contains(&Entity::Object(spell)));
}

#[test]
fn ruby_medallion_changes_only_the_total_cost() {
    cr!("601.2f", "118.7", "202.3");
    ruling!(
        "Ruby Medallion",
        "The ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you pay."
    );
    supported("Ruby Medallion");
    // Ruby Medallion: "Red spells you cast cost {1} less to cast." Searing Spear ({1}{R})
    // costs {R}; Kaervek still deals 2.
    let mut t = TestGame::new(2);
    kaervek(&mut t);
    t.battlefield(P0, "Ruby Medallion");
    t.lands(P0, "Mountain", 2);
    let spear = t.hand(P0, "Searing Spear");
    aim_kaervek_at_p0(&mut t);
    let spell = t.cast(P0, spear).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 1);
    assert_eq!(
        t.obj_now(spell)
            .chars
            .mana_cost
            .as_ref()
            .unwrap()
            .to_string(),
        "{1}{R}"
    );
    t.resolve();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn sanctum_prelate_still_stops_a_spell_whose_cost_was_changed() {
    cr!("601.2f", "202.3");
    ruling!(
        "Sanctum Prelate",
        "Effects that increase or reduce the cost to cast a spell don't affect that spell's mana value."
    );
    supported("Sanctum Prelate");
    // Sanctum Prelate: "As this creature enters, choose a number. Noncreature spells with
    // mana value equal to the chosen number can't be cast." With 2 chosen, Searing Spear
    // ({1}{R}) can't be cast though Ruby Medallion would make it cost {R}; with 1 chosen,
    // Lightning Bolt can't be cast though Thalia would make it cost {1}{R}.
    let mut t = TestGame::new(2);
    t.answer(P1, DecisionKind::Number, Answer::Number(2));
    t.enter(P1, "Sanctum Prelate");
    t.settle();
    t.battlefield(P0, "Ruby Medallion");
    t.lands(P0, "Mountain", 3);
    let spear = t.hand(P0, "Searing Spear");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(!crate::r_s21_common::castable(&mut t, P0, spear));
    assert!(crate::r_s21_common::castable(&mut t, P0, bolt));
    let mut t = TestGame::new(2);
    t.answer(P1, DecisionKind::Number, Answer::Number(1));
    t.enter(P1, "Sanctum Prelate");
    t.settle();
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Mountain", 3);
    let spear = t.hand(P0, "Searing Spear");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(!crate::r_s21_common::castable(&mut t, P0, bolt));
    assert!(crate::r_s21_common::castable(&mut t, P0, spear));
}

#[test]
fn kaervek_sees_the_mana_value_whatever_was_paid() {
    cr!("118.8d", "118.9c", "202.3");
    ruling!(
        "Kaervek the Merciless",
        "Alternative costs, additional costs, and cost reductions don't change a spell's mana value. Its mana value is still based on its mana cost."
    );
    // A kicked Burst Lightning ({R} + kicker {4}): 1 damage.
    let mut t = TestGame::new(2);
    kaervek(&mut t);
    t.lands(P0, "Mountain", 5);
    let bolt = t.hand(P0, "Burst Lightning");
    aim_kaervek_at_p0(&mut t);
    t.cast(P0, bolt)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(tapped_lands(&t, P0), 5);
    t.resolve();
    assert_eq!(t.life(P0), 19);
    // Searing Spear ({1}{R}) for {R} with Ruby Medallion: 2 damage. Then Force of Will
    // ({3}{U}{U}) cast for its alternative cost (1 life and a blue card): 5 damage.
    let mut t = TestGame::new(2);
    kaervek(&mut t);
    t.battlefield(P0, "Ruby Medallion");
    t.lands(P0, "Mountain", 1);
    let spear = t.hand(P0, "Searing Spear");
    aim_kaervek_at_p0(&mut t);
    let spear = t.cast(P0, spear).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 1);
    t.resolve();
    assert_eq!(t.life(P0), 18);
    let fow = t.hand(P0, "Force of Will");
    t.hand(P0, "Opt");
    let alt = crate::r_s07_common::cast_methods(&mut t, P0, fow)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("Force of Will's alternative cost");
    aim_kaervek_at_p0(&mut t);
    t.cast(P0, fow).method(alt).target(spear).go();
    assert!(t.in_exile("Opt"));
    t.resolve();
    assert_eq!(t.life(P0), 12);
}
