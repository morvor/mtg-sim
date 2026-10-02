//! Instructions with any player as the subject ("target player mills half their
//! library", "each player may put a card from their hand onto the battlefield", "each
//! player chooses six lands they control, then sacrifices the rest", "you and target
//! opponent each draw three cards", "that player may pay {2}. If the player doesn't, ..."):
//! the player named performs the instruction, with their own library, hand and choices.

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// The "choose entities" decisions asked of `p` so far: (candidates, min, max).
fn choices_of(t: &TestGame, p: PlayerId) -> Vec<(Vec<Entity>, u32, u32)> {
    t.asked()
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities {
                candidates,
                min,
                max,
                ..
            } if q == p => Some((candidates, min, max)),
            _ => None,
        })
        .collect()
}

fn library(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.library_top(p, "Island");
    }
}

#[test]
fn traumatize_target_player_mills_half_their_library_rounded_down() {
    cr!("701.17a", "107.1a");
    assert_supported("Traumatize");
    let mut t = TestGame::new(2);
    library(&mut t, P1, 9);
    let before = t.library_size(P1);
    let mine = t.library_size(P0);
    t.lands(P0, "Island", 5);
    let spell = t.hand(P0, "Traumatize");
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!(t.library_size(P1), before - before / 2);
    assert_eq!(t.graveyard_size(P1), before / 2);
    // Not the caster's library.
    assert_eq!(t.library_size(P0), mine);
}

#[test]
fn kitsune_s_technique_rounds_up() {
    cr!("701.17a", "107.1a");
    assert_supported("Kitsune's Technique");
    let mut t = TestGame::new(2);
    library(&mut t, P1, 9);
    let before = t.library_size(P1);
    t.lands(P0, "Island", 6);
    let spell = t.hand(P0, "Kitsune's Technique");
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!(t.graveyard_size(P1), before.div_ceil(2));
}

#[test]
fn show_and_tell_each_player_puts_a_card_under_their_own_control() {
    cr!("101.4", "101.4b");
    ruling!(
        "Show and Tell",
        "After all choices are made, the cards are put onto the battlefield simultaneously."
    );
    assert_supported("Show and Tell");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let dragon = t.hand(P0, "Shivan Dragon");
    let bears = t.hand(P1, "Grizzly Bears");
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.hand(P0, "Show and Tell");
    t.answer_yes(P0, true).answer_yes(P1, true);
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.cast(P0, spell).go();
    t.resolve();
    let d = t.named_on_battlefield("Shivan Dragon");
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(d.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(t.obj_now(d[0]).controller, P0);
    assert_eq!(t.obj_now(b[0]).controller, P1);
    // P1 chose among their own cards of the listed types only.
    let p1 = choices_of(&t, P1);
    assert_eq!(p1.last().unwrap().0, vec![Entity::Object(bears)]);
    assert_eq!(t.zone(bolt), Zone::Hand(P1));
}

#[test]
fn show_and_tell_a_player_may_decline() {
    cr!("101.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.hand(P0, "Shivan Dragon");
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Show and Tell");
    t.answer_yes(P0, true).answer_yes(P1, false);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Shivan Dragon").len(), 1);
    assert_eq!(t.zone(bears), Zone::Hand(P1));
}

#[test]
fn blackmail_the_target_reveals_three_and_the_caster_chooses() {
    cr!("701.20a", "701.9b");
    ruling!("Blackmail", "If the player has less than 3 cards, all of them are revealed.");
    assert_supported("Blackmail");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let a = t.hand(P1, "Lightning Bolt");
    let b = t.hand(P1, "Shock");
    let c = t.hand(P1, "Grizzly Bears");
    let d = t.hand(P1, "Forest");
    let spell = t.hand(P0, "Blackmail");
    // The target player chooses which three cards to reveal...
    t.answer_choose(P1, &[Entity::Object(a), Entity::Object(b), Entity::Object(c)]);
    // ...and the caster chooses one of those.
    t.answer_choose(P0, &[Entity::Object(c)]);
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    let (cands, min, max) = choices_of(&t, P1)[0].clone();
    assert_eq!((min, max), (3, 3));
    assert_eq!(cands.len(), 4);
    let mut offered = choices_of(&t, P0).last().unwrap().0.clone();
    offered.sort();
    let mut revealed = vec![Entity::Object(a), Entity::Object(b), Entity::Object(c)];
    revealed.sort();
    assert_eq!(offered, revealed);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.zone(d), Zone::Hand(P1));
    assert_eq!(t.hand_size(P1), 3);

    // With fewer than three cards, all of them are revealed.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let a = t.hand(P1, "Lightning Bolt");
    let b = t.hand(P1, "Shock");
    let spell = t.hand(P0, "Blackmail");
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    let (_, min, _) = choices_of(&t, P1)[0].clone();
    assert_eq!(min, 2);
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.zone(a), Zone::Hand(P1));
}

#[test]
fn mire_s_toll_counts_the_caster_s_swamps() {
    cr!("701.20a");
    ruling!(
        "Mire's Toll",
        "If the number of Swamps you control exceeds the number of cards in the targeted player’s hand, that player reveals all the cards in their hand."
    );
    assert_supported("Mire's Toll");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    // The target's own Swamps don't count.
    t.lands(P1, "Swamp", 3);
    for _ in 0..4 {
        t.hand(P1, "Grizzly Bears");
    }
    let spell = t.hand(P0, "Mire's Toll");
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    let (_, min, max) = choices_of(&t, P1)[0].clone();
    assert_eq!((min, max), (2, 2));
    assert_eq!(t.hand_size(P1), 3);
}

#[test]
fn planetary_annihilation_each_player_keeps_six_lands() {
    cr!("101.4", "701.21a");
    assert_supported("Planetary Annihilation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    t.lands(P1, "Forest", 9);
    let ring = t.battlefield(P1, "Sol Ring");
    let spell = t.hand(P0, "Planetary Annihilation");
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Mountain").len(), 6);
    assert_eq!(t.named_on_battlefield("Forest").len(), 6);
    // Only lands are sacrificed.
    assert!(t.on_battlefield(ring));
    // Each player chose six different lands, one at a time.
    let p1 = choices_of(&t, P1);
    assert_eq!(p1.len(), 6);
    assert_eq!(p1[5].0.len(), 4);
}

#[test]
fn covetous_elegy_players_may_keep_fewer() {
    cr!("101.4");
    ruling!(
        "Covetous Elegy",
        "Players may choose fewer than two creatures even if they control two or more creatures."
    );
    assert_supported("Covetous Elegy");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    t.lands(P0, "Swamp", 1);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Covetous Elegy");
    // P1 keeps one creature and declines the second choice.
    t.answer_choose(P1, &[Entity::Object(a)]);
    t.answer_choose(P1, &[]);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert!(!t.on_battlefield(c));
    // One Treasure for the creature P1 kept.
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 1);
}

#[test]
fn secret_rendezvous_you_and_target_opponent_each_draw_three() {
    cr!("608.2c");
    assert_supported("Secret Rendezvous");
    let mut t = TestGame::new(2);
    library(&mut t, P0, 5);
    library(&mut t, P1, 5);
    t.lands(P0, "Plains", 3);
    let spell = t.hand(P0, "Secret Rendezvous");
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.hand_size(P1), 3);
}

#[test]
fn depopulate_each_player_who_controls_a_multicolored_creature_draws() {
    cr!("608.2c");
    assert_supported("Depopulate");
    let mut t = TestGame::new(2);
    library(&mut t, P0, 3);
    library(&mut t, P1, 3);
    t.lands(P0, "Plains", 4);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Watchwolf");
    let spell = t.hand(P0, "Depopulate");
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.hand_size(P1), 1);
    assert!(t.named_on_battlefield("Watchwolf").is_empty());
}

#[test]
fn wheel_and_deal_any_number_of_target_opponents_each_discard_and_draw() {
    cr!("701.9b");
    ruling!("Wheel and Deal", "You can choose to target zero opponents.");
    assert_supported("Wheel and Deal");
    let mut t = TestGame::new(2);
    library(&mut t, P0, 3);
    library(&mut t, P1, 10);
    t.lands(P0, "Island", 4);
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Shock");
    let spell = t.hand(P0, "Wheel and Deal");
    t.cast(P0, spell).targets(&[Entity::Player(P1)]).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.hand_size(P0), 1);

    // No targets: only the caster draws.
    let mut t = TestGame::new(2);
    library(&mut t, P0, 3);
    t.lands(P0, "Island", 4);
    t.hand(P1, "Shock");
    let spell = t.hand(P0, "Wheel and Deal");
    t.cast(P0, spell).targets(&[]).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn smothering_tithe_that_player_may_pay() {
    cr!("118.12");
    assert_supported("Smothering Tithe");
    for pays in [true, false] {
        let mut t = TestGame::new(2);
        library(&mut t, P1, 3);
        t.battlefield(P0, "Smothering Tithe");
        t.lands(P1, "Island", 3);
        let opt = t.hand(P1, "Opt");
        t.answer_yes(P1, pays);
        t.cast(P1, opt).go();
        t.resolve_all();
        let treasures = t.named_on_battlefield("Treasure Token").len();
        assert_eq!(treasures, if pays { 0 } else { 1 }, "pays: {pays}");
        let untapped = t
            .g
            .battlefield
            .iter()
            .filter(|o| t.obj_now(**o).controller == P1 && !t.obj_now(**o).tapped)
            .count();
        assert_eq!(untapped, if pays { 0 } else { 2 }, "pays: {pays}");
    }
}

#[test]
fn goblin_guide_defending_player_puts_a_revealed_land_into_their_hand() {
    cr!("701.20a");
    ruling!(
        "Goblin Guide",
        "If the defending player reveals a nonland card, it remains on top of their library."
    );
    assert_supported("Goblin Guide");
    for land in [true, false] {
        let mut t = TestGame::new(2);
        let guide = t.battlefield(P0, "Goblin Guide");
        let top = t.library_top(P1, if land { "Forest" } else { "Grizzly Bears" });
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(guide, Entity::Player(P1))], &[]);
        if land {
            assert_eq!(t.zone(top), Zone::Hand(P1));
        } else {
            assert_eq!(t.zone(top), Zone::Library(P1));
        }
    }
}

#[test]
fn monomania_target_player_keeps_one_card() {
    cr!("701.9b");
    ruling!(
        "Monomania",
        "If there are one or zero cards in the player’s hand, they will discard no cards."
    );
    assert_supported("Monomania");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let keep = t.hand(P1, "Shock");
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Forest");
    let spell = t.hand(P0, "Monomania");
    t.answer_choose(P1, &[Entity::Object(keep)]);
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.zone(keep), Zone::Hand(P1));
}

#[test]
fn new_frontiers_each_player_who_searched_shuffles() {
    cr!("101.4", "701.24a");
    assert_supported("New Frontiers");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let f = t.library_top(P0, "Forest");
    let p = t.library_top(P1, "Plains");
    let spell = t.hand(P0, "New Frontiers");
    t.answer_yes(P0, true).answer_yes(P1, true);
    t.answer_choose(P0, &[Entity::Object(f)]);
    t.answer_choose(P1, &[Entity::Object(p)]);
    t.cast(P0, spell).x(2).go();
    t.resolve();
    let forests: Vec<ObjectId> = t
        .named_on_battlefield("Forest")
        .into_iter()
        .filter(|o| t.obj_now(*o).tapped)
        .collect();
    let plains = t.named_on_battlefield("Plains");
    // The searched-for lands entered tapped under their owners' control (the three
    // Forests that paid for the spell are tapped too).
    assert_eq!(forests.len(), 4);
    assert_eq!(plains.len(), 1);
    assert_eq!(t.obj_now(plains[0]).controller, P1);
    assert!(t.obj_now(plains[0]).tapped);
}

#[test]
fn blightning_the_targeted_player_discards_two() {
    cr!("701.9b");
    assert_supported("Blightning");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 1);
    t.hand(P1, "Shock");
    t.hand(P1, "Forest");
    t.hand(P1, "Island");
    let spell = t.hand(P0, "Blightning");
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn stinging_vitriol_they_discard_the_chosen_card() {
    cr!("701.9b");
    assert_supported("Stinging Vitriol");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Stinging Vitriol");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.zone(shock), Zone::Hand(P1));
}

#[test]
fn grave_consequences_each_player_may_exile_from_their_graveyard() {
    cr!("101.4");
    assert_supported("Grave Consequences");
    let mut t = TestGame::new(2);
    library(&mut t, P0, 2);
    t.lands(P0, "Swamp", 2);
    let a = t.graveyard(P0, "Shock");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P1, "Shock");
    t.graveyard(P1, "Shock");
    t.graveyard(P1, "Shock");
    let spell = t.hand(P0, "Grave Consequences");
    t.answer_yes(P0, true).answer_yes(P1, false);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.cast(P0, spell).go();
    t.resolve();
    // P0 exiled one of their two cards; then each loses 1 life per card in their
    // graveyard (Grave Consequences itself isn't there yet as it resolves).
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn tyrannize_target_player_discards_their_hand_unless_they_pay_seven_life() {
    cr!("118.12");
    assert_supported("Tyrannize");
    for pays in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 5);
        t.hand(P1, "Shock");
        t.hand(P1, "Forest");
        let spell = t.hand(P0, "Tyrannize");
        t.answer_yes(P1, pays);
        t.cast(P0, spell).target(P1).go();
        t.resolve();
        if pays {
            assert_eq!(t.life(P1), 13);
            assert_eq!(t.hand_size(P1), 2);
        } else {
            assert_eq!(t.life(P1), 20);
            assert_eq!(t.hand_size(P1), 0);
        }
    }
}

#[test]
fn thieving_sprite_counts_faeries_as_the_ability_resolves() {
    cr!("701.20a", "608.2h");
    ruling!(
        "Thieving Sprite",
        "The number of cards that are revealed is equal to the number of Faeries you control when the ability resolves."
    );
    assert_supported("Thieving Sprite");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faerie Miscreant");
    for _ in 0..4 {
        t.hand(P1, "Grizzly Bears");
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Thieving Sprite");
    t.resolve();
    let (_, min, max) = choices_of(&t, P1)[0].clone();
    assert_eq!((min, max), (2, 2));
    assert_eq!(t.hand_size(P1), 3);
}

#[test]
fn keldon_firebombers_each_player_keeps_three_lands() {
    cr!("101.4", "701.21a");
    assert_supported("Keldon Firebombers");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    t.lands(P1, "Forest", 2);
    t.enter(P0, "Keldon Firebombers");
    t.resolve();
    assert_eq!(t.named_on_battlefield("Mountain").len(), 3);
    // A player with three or fewer lands keeps them all.
    assert_eq!(t.named_on_battlefield("Forest").len(), 2);
}

#[test]
fn dire_fleet_ravager_each_player_loses_a_third_rounded_up() {
    cr!("107.1a");
    assert_supported("Dire Fleet Ravager");
    let mut t = TestGame::new(2);
    t.enter(P0, "Dire Fleet Ravager");
    t.resolve();
    assert_eq!(t.life(P0), 13);
    assert_eq!(t.life(P1), 13);
}

#[test]
fn rites_of_flourishing_each_player_may_play_an_additional_land() {
    cr!("305.2");
    assert_supported("Rites of Flourishing");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rites of Flourishing");
    t.set_step(P0, Step::PrecombatMain);
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Forest");
    let c = t.hand(P0, "Forest");
    assert!(t.play_land(P0, a).is_ok());
    assert!(t.play_land(P0, b).is_ok());
    assert!(t.play_land(P0, c).is_err());
}
