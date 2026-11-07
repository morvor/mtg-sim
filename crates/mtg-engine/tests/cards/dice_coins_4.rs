//! Dice and coins, round 3 (`oracle/patterns/dice_coins_4.rs`): the results of the dice
//! that triggered an ability, each different result, subtracting from a roll, a results
//! table after a sentence about other results, and cards from among the cards exiled
//! with a permanent.

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn tokens_named(t: &TestGame, subtype: &str) -> usize {
    t.g.zone_objects(Zone::Battlefield)
        .into_iter()
        .filter(|o| t.obj_now(*o).chars.subtypes.iter().any(|s| s == subtype))
        .count()
}

#[test]
fn farideh_draws_if_any_of_the_dice_rolled_was_ten_or_higher() {
    cr!("706.1", "603.2");
    assert_supported(&["Farideh, Devil's Chosen"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let farideh = t.battlefield(P0, "Farideh, Devil's Chosen");
    t.lands(P0, "Mountain", 3);
    // Three six-sided dice: none is 10 or higher.
    let act = t.hand(P0, "Circuits Act");
    t.g.dice.loaded.extend([2, 5, 5]);
    let hand = t.hand_size(P0);
    t.cast(P0, act).go();
    t.resolve_all();
    assert!(t
        .obj_now(farideh)
        .chars
        .has_keyword(keywords::KeywordKind::Flying));
    assert_eq!(t.hand_size(P0), hand - 1);
    // A d20 showing 12 (an effect tells P0 to roll it): draw a card.
    let mut ctx = mtg_engine::eval::Ctx::new(Some(farideh), P0);
    let roll = mtg_engine::dice::DieRoll::new(20);
    t.g.dice.loaded.push_back(12);
    mtg_engine::dice::roll(&mut t.g, &roll, &mut ctx);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn circuits_act_creates_a_token_for_each_different_result() {
    cr!("706.1");
    ruling!(
        "Circuits Act",
        "For example, if you rolled 1, 4, and 4, you'd create two Clown Robots."
    );
    assert_supported(&["Circuits Act"]);
    for (dice, expected) in [([1, 4, 4], 2), ([3, 3, 3], 1), ([1, 2, 6], 3)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.lands(P0, "Mountain", 3);
        let act = t.hand(P0, "Circuits Act");
        t.g.dice.loaded.extend(dice);
        t.cast(P0, act).go();
        t.resolve_all();
        assert_eq!(tokens_named(&t, "Robot"), expected, "{dice:?}");
    }
}

#[test]
fn the_deck_of_many_things_subtracts_cards_in_hand() {
    cr!("706.2", "706.3", "706.3a");
    ruling!(
        "The Deck of Many Things",
        "cares about the result of the die roll after any adjustments"
    );
    assert_supported(&["The Deck of Many Things"]);
    // 12 minus three cards in hand: 9, return a card at random.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let deck = t.battlefield(P0, "The Deck of Many Things");
    t.lands(P0, "Plains", 2);
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    t.graveyard(P0, "Lightning Bolt");
    t.g.dice.loaded.push_back(12);
    t.activate(P0, deck, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(t.hand_size(P0), 4);
    // 2 minus four cards in hand: -2, discard your hand; no row applies.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let deck = t.battlefield(P0, "The Deck of Many Things");
    t.lands(P0, "Plains", 2);
    for _ in 0..4 {
        t.hand(P0, "Grizzly Bears");
    }
    t.graveyard(P0, "Lightning Bolt");
    t.g.dice.loaded.push_back(2);
    t.activate(P0, deck, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    // A natural 20 with an empty hand: a creature card from any graveyard.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let deck = t.battlefield(P0, "The Deck of Many Things");
    t.lands(P0, "Plains", 2);
    let wurm = t.graveyard(P1, "Craw Wurm");
    t.g.dice.loaded.push_back(20);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.activate(P0, deck, 0, &[]).unwrap();
    t.resolve_all();
    let wurms = t.named_on_battlefield("Craw Wurm");
    assert_eq!(wurms.len(), 1);
    assert_eq!(t.obj_now(wurms[0]).controller, P0);
}

#[test]
fn bag_of_devouring_returns_up_to_the_result_of_the_exiled_cards() {
    cr!("706.4", "607.1a");
    ruling!(
        "Bag of Devouring",
        "It won't be a card exiled with Bag of Devouring for the third ability."
    );
    assert_supported(&["Bag of Devouring"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bag = t.battlefield(P0, "Bag of Devouring");
    t.lands(P0, "Plains", 9);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Ornithopter");
    let c = t.battlefield(P0, "Memnite");
    for x in [a, b, c] {
        t.answer_choose(P0, &[Entity::Object(x)]);
        t.activate(P0, bag, 0, &[]).unwrap();
        t.resolve_all();
        t.g.objects[bag.0 as usize].tapped = false;
    }
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Ornithopter"));
    assert!(t.in_exile("Memnite"));
    // A 2: up to two of them return.
    let exiled: Vec<Entity> =
        t.g.zone_objects(Zone::Exile)
            .into_iter()
            .map(Entity::Object)
            .collect();
    assert_eq!(exiled.len(), 3);
    t.answer_choose(P0, &exiled[..2]);
    t.g.dice.loaded.push_back(2);
    t.activate(P0, bag, 1, &[]).unwrap();
    t.resolve_all();
    let in_hand = ["Grizzly Bears", "Ornithopter", "Memnite"]
        .iter()
        .filter(|n| t.in_hand(P0, n))
        .count();
    assert_eq!(in_hand, 2);
}

#[test]
fn druid_of_the_emerald_grove_splits_the_lands_on_a_middle_result() {
    cr!("706.3", "706.3a");
    assert_supported(&["Druid of the Emerald Grove"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let f1 = t.library_top(P0, "Forest");
    let f2 = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(f1), Entity::Object(f2)]);
    t.g.dice.loaded.push_back(12);
    t.enter(P0, "Druid of the Emerald Grove");
    t.resolve_all();
    let forests = t.named_on_battlefield("Forest");
    assert_eq!(forests.len(), 1);
    assert!(t.obj_now(forests[0]).tapped);
    assert!(t.in_hand(P0, "Forest"));
}

#[test]
fn delina_creates_one_of_those_tokens_and_may_roll_again() {
    cr!("706.3", "706.3c");
    ruling!(
        "Delina, Wild Mage",
        "Specifically, rolling again on a result of 15–20 is optional."
    );
    assert_supported(&["Delina, Wild Mage"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let delina = t.battlefield(P0, "Delina, Wild Mage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    // 17: one token, and roll again; 3: one more token.
    t.g.dice.loaded.extend([17, 3]);
    t.answer_yes(P0, true);
    t.attack(&[(delina, Entity::Player(P1))], &[]);
    t.resolve_all();
    // The tokens attacked and were exiled at end of combat; Grizzly Bears didn't attack.
    let dmg = 20 - t.life(P1);
    // Delina (3) and two 2/2 tokens.
    assert_eq!(dmg, 3 + 2 + 2);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn gales_redirection_lets_you_cast_the_exiled_spell() {
    cr!("706.2", "706.3");
    ruling!(
        "Gale's Redirection",
        "Gale's Redirection exiles the spell directly from the stack without countering it."
    );
    assert_supported(&["Gale's Redirection"]);
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Island", 6);
    let gale = t.hand(P0, "Gale's Redirection");
    // 5 + 1 (the spell's mana value) = 6: cast it for its mana cost, spending any mana.
    t.g.dice.loaded.push_back(5);
    t.cast(P0, gale).target(bolt).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_exile("Lightning Bolt"));
    let exiled =
        t.g.zone_objects(Zone::Exile)
            .into_iter()
            .find(|o| t.obj_now(*o).chars.name == "Lightning Bolt")
            .unwrap();
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, exiled).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn wizards_spellbook_copies_the_exiled_card() {
    cr!("706.3", "707.12");
    ruling!(
        "Wizard's Spellbook",
        "If your result is 1–9, you must still pay all costs to cast the copy."
    );
    assert_supported(&["Wizard's Spellbook"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let book = t.battlefield(P0, "Wizard's Spellbook");
    t.lands(P0, "Mountain", 1);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    t.g.dice.loaded.push_back(4);
    t.activate(P0, book, 0, &[Entity::Object(bolt)]).unwrap();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn wylls_reversal_redirects_and_copies_on_a_high_result() {
    cr!("706.2", "115.7", "707.10");
    ruling!(
        "Wyll's Reversal",
        "Even though you do not gain control of the original spell or ability, you control the copy."
    );
    assert_supported(&["Wyll's Reversal"]);
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Mountain", 3);
    t.battlefield(P0, "Craw Wurm");
    let wyll = t.hand(P0, "Wyll's Reversal");
    // 10 + 6 (Craw Wurm's power) = 16: new targets for it, then a copy.
    t.g.dice.loaded.push_back(10);
    t.cast(P0, wyll).target(bolt).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn mirror_march_makes_a_hasty_copy_for_each_flip_won() {
    cr!("705.2", "707.2");
    ruling!(
        "Mirror March",
        "All of the tokens enter the battlefield at the same time."
    );
    assert_supported(&["Mirror March"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Mirror March");
    // Heads, heads, tails (calling heads): two flips won.
    t.g.dice.loaded_coins.extend([true, true, false]);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 3);
    let tokens: Vec<_> = bears.iter().filter(|b| t.obj_now(**b).is_token()).collect();
    assert_eq!(tokens.len(), 2);
    assert!(t
        .obj_now(*tokens[0])
        .chars
        .has_keyword(keywords::KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn goblin_lyre_on_a_lost_flip_counts_that_players_creatures() {
    cr!("705.2");
    assert_supported(&["Goblin Lyre"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let lyre = t.battlefield(P0, "Goblin Lyre");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Craw Wurm");
    // Calling heads, tails comes up: P0 is dealt 3 (P1's creatures).
    t.g.dice.loaded_coins.push_back(false);
    t.activate(P0, lyre, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn crooked_scales_repeats_the_flip_while_you_pay() {
    cr!("705.2", "608.2c");
    ruling!(
        "Crooked Scales",
        "If you pay the {3}, you do not choose a new pair of targets when it repeats."
    );
    assert_supported(&["Crooked Scales"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let scales = t.battlefield(P0, "Crooked Scales");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 7);
    // Lose (pay {3}), then win.
    t.g.dice.loaded_coins.extend([false, true]);
    t.activate(
        P0,
        scales,
        0,
        &[Entity::Object(theirs), Entity::Object(mine)],
    )
    .unwrap();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
    // Lose and don't pay: your creature is destroyed.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let scales = t.battlefield(P0, "Crooked Scales");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 7);
    t.g.dice.loaded_coins.push_back(false);
    t.activate(
        P0,
        scales,
        0,
        &[Entity::Object(theirs), Entity::Object(mine)],
    )
    .unwrap();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert!(t.on_battlefield(theirs));
}

#[test]
fn mana_clash_repeats_until_both_coins_come_up_heads() {
    cr!("705.2", "608.2c");
    ruling!(
        "Mana Clash",
        "This card’s coin flip has no winner or loser."
    );
    ruling!("Mana Clash", "Each point of damage is dealt separately");
    assert_supported(&["Mana Clash"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let clash = t.hand(P0, "Mana Clash");
    // Pass 1: P0 tails, P1 tails. Pass 2: heads, tails. Pass 3: heads, heads.
    t.g.dice
        .loaded_coins
        .extend([false, false, true, false, true, true]);
    t.cast(P0, clash).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 18);
    // No calls were asked for.
    assert!(t
        .asked()
        .iter()
        .all(|(_, d)| !format!("{d:?}").contains("Call the coin flip")));
}

#[test]
fn spikeshell_harrier_reduces_the_fastest_opponents_speed() {
    cr!("702.179b", "702.179f");
    assert_supported(&["Spikeshell Harrier"]);
    for (theirs, mine, after) in [(3, 1, 2), (1, 1, 1), (2, 3, 2)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.players[1].speed = Some(theirs);
        t.g.players[0].speed = Some(mine);
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.enter(P0, "Spikeshell Harrier");
        t.resolve_all();
        assert!(t.in_hand(P1, "Grizzly Bears"));
        assert_eq!(t.g.players[1].speed, Some(after), "{theirs} vs {mine}");
    }
    // Speed 1 is the floor.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.players[1].speed = Some(1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Spikeshell Harrier");
    t.resolve_all();
    assert_eq!(t.g.players[1].speed, Some(1));
}

#[test]
fn ichor_elixir_rolls_an_extra_planar_die_and_ignores_one() {
    cr!("706.6", "901.9");
    assert_supported(&["Ichor Elixir"]);
    use mtg_engine::planechase::{roll_planar_die, PlanarFace};
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Ichor Elixir");
    // The Planeswalker symbol and a blank: P0 keeps the blank.
    t.g.dice.loaded.extend([1, 4]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    assert_eq!(roll_planar_die(&mut t.g, P0), PlanarFace::Blank);
    assert!(t.g.dice.loaded.is_empty());
    // Another player's rolls aren't affected.
    t.g.dice.loaded.extend([4]);
    assert_eq!(roll_planar_die(&mut t.g, P1), PlanarFace::Blank);
    assert!(t.g.dice.loaded.is_empty());
}

#[test]
fn revivify_returns_the_counted_cards_on_a_high_result() {
    cr!("706.2", "706.3");
    assert_supported(&["Revivify"]);
    for (natural, to_battlefield) in [(3, false), (14, true)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.destroy(bears, None);
        t.settle();
        // A creature card that was already in the graveyard isn't counted or returned.
        t.graveyard(P0, "Hill Giant");
        t.lands(P0, "Plains", 4);
        let r = t.hand(P0, "Revivify");
        t.g.dice.loaded.push_back(natural);
        t.cast(P0, r).go();
        t.resolve_all();
        assert_eq!(
            !t.named_on_battlefield("Grizzly Bears").is_empty(),
            to_battlefield
        );
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), !to_battlefield);
        assert!(t.in_graveyard(P0, "Hill Giant"));
    }
}

#[test]
fn journey_to_the_lost_city_counts_creature_cards_among_the_exiled_cards() {
    cr!("706.3", "706.3a");
    assert_supported(&["Journey to the Lost City"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Journey to the Lost City");
    for n in [
        "Grizzly Bears",
        "Forest",
        "Hill Giant",
        "Island",
        "Craw Wurm",
    ] {
        t.library_top(P0, n);
    }
    t.g.dice.loaded.push_back(12);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    // The top four: Craw Wurm, Island, Hill Giant, Forest — two creature cards.
    let wolves: Vec<ObjectId> =
        t.g.zone_objects(Zone::Battlefield)
            .into_iter()
            .filter(|o| t.obj_now(*o).chars.subtypes.iter().any(|s| s == "Wolf"))
            .collect();
    assert_eq!(wolves.len(), 1);
    assert_eq!(t.pt(wolves[0]), (4, 4));
}

#[test]
fn tunnel_of_love_exiles_both_chosen_creatures_or_they_fight() {
    cr!("702.159a", "701.14a");
    ruling!(
        "Tunnel of Love",
        "If either you or the chosen opponent controls no creatures, the other player still chooses a creature they control."
    );
    assert_supported(&["Tunnel of Love"]);
    for exile in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Tunnel of Love");
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_choose(P1, &[Entity::Object(bears)]);
        t.answer_choose(P0, &[Entity::Object(giant)]);
        t.answer_yes(P0, exile);
        t.g.dice.loaded.push_back(6);
        mtg_engine::variants::roll_to_visit(&mut t.g, P0);
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        if exile {
            assert!(t.in_exile("Hill Giant") && t.in_exile("Grizzly Bears"));
            t.advance_to(P0, Step::End);
            t.resolve_all();
            assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
            let b = t.named_on_battlefield("Grizzly Bears");
            assert_eq!(b.len(), 1);
            assert_eq!(t.obj_now(b[0]).controller, P1);
        } else {
            assert!(!t.on_battlefield(bears));
            assert_eq!(t.obj_now(giant).damage, 2);
        }
    }
}
