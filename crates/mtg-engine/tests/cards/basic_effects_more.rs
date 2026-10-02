//! More basic effects: referents after an intervening "if defending player ..." and after a
//! target in an opponent's graveyard; "the other creature"; "destroy any of them that are
//! Walls"; "count the number of ..."; "any target of an opponent's choice"; power lists;
//! "that player or a planeswalker that player controls"; "each deal damage equal to their
//! power"; die results tables printed with dashes; base toughness; "as you cast ~".

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn siege_dragon_damages_creatures_without_flying_of_defending_player() {
    cr!("603.4", "508.1");
    assert_supported("Siege Dragon");
    for wall in [false, true] {
        let mut t = TestGame::new(2);
        let dragon = t.battlefield(P0, "Siege Dragon");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let angel = t.battlefield(P1, "Serra Angel");
        let mine = t.battlefield(P0, "Grizzly Bears");
        if wall {
            t.battlefield(P1, "Wall of Stone");
        }
        t.advance_to(P0, Step::BeginningOfCombat);
        t.attack(&[(dragon, Entity::Player(P1))], &[]);
        // Only the defending player's creatures without flying.
        assert_eq!(t.on_battlefield(bears), wall, "wall: {wall}");
        assert!(t.on_battlefield(angel));
        assert!(t.on_battlefield(mine));
    }
}

#[test]
fn venomous_fangs_destroys_the_creature_dealt_damage() {
    cr!("603.2", "510.2");
    assert_supported("Venomous Fangs");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fangs = t.battlefield(P0, "Venomous Fangs");
    assert!(t.g.attach(fangs, Entity::Object(bears)));
    let wall = t.battlefield(P1, "Wall of Stone");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(wall, bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(wall));
    assert!(t.on_battlefield(bears));
}

#[test]
fn blow_your_house_down_destroys_only_the_walls_among_its_targets() {
    cr!("115.1");
    assert_supported("Blow Your House Down");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other_wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Blow Your House Down");
    t.cast(P0, s)
        .targets(&[Entity::Object(wall), Entity::Object(bears)])
        .go();
    t.resolve();
    assert!(!t.on_battlefield(wall));
    assert!(t.on_battlefield(bears));
    // Not a target: not one of "them".
    assert!(t.on_battlefield(other_wall));
}

#[test]
fn carrion_locust_that_player_is_the_cards_owner() {
    cr!("608.2h");
    assert_supported("Carrion Locust");
    for creature in [true, false] {
        let mut t = TestGame::new(3);
        let card = t.graveyard(P2, if creature { "Grizzly Bears" } else { "Forest" });
        t.lands(P0, "Swamp", 3);
        let locust = t.hand(P0, "Carrion Locust");
        t.answer_targets(P0, &[Entity::Object(card)]);
        t.cast(P0, locust).go();
        t.resolve_all();
        assert!(t.in_exile(if creature { "Grizzly Bears" } else { "Forest" }));
        assert_eq!(t.life(P2), if creature { 19 } else { 20 }, "{creature}");
        assert_eq!(t.life(P1), 20);
    }
}

#[test]
fn touch_of_the_eternal_sets_life_to_the_number_of_permanents() {
    cr!("119.5");
    assert_supported("Touch of the Eternal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Touch of the Eternal");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 2);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn any_target_of_an_opponents_choice_is_chosen_by_an_opponent() {
    cr!("601.2c", "115.1");
    assert_supported("Karplusan Minotaur");
    let def = oracle_card(
        "Opponent's Choice Pinger",
        "Artifact",
        "{T}: This artifact deals 1 damage to any target of an opponent's choice.",
    );
    let mut t = TestGame::new(2);
    let p = t.custom(P0, def, mtg_engine::object::Zone::Battlefield);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.activate(P0, p, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn sarkhans_unsealing_triggers_for_power_four_five_or_six() {
    cr!("208.1", "603.2");
    assert_supported("Sarkhan's Unsealing");
    for (name, triggers) in [("Craw Wurm", true), ("Hill Giant", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Sarkhan's Unsealing");
        t.lands(P0, "Forest", 5);
        t.lands(P0, "Mountain", 1);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let c = t.hand(P0, name);
        t.cast(P0, c).go();
        t.resolve_all();
        assert_eq!(t.life(P1), if triggers { 16 } else { 20 }, "{name}");
    }
}

#[test]
fn curse_of_the_pierced_heart_damages_the_enchanted_player() {
    cr!("303.4", "120.3");
    assert_supported("Curse of the Pierced Heart");
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P0, "Curse of the Pierced Heart");
    assert!(t.g.attach(curse, Entity::Player(P1)));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn band_together_each_target_deals_damage_equal_to_its_power() {
    cr!("120.3", "115.3");
    assert_supported("Band Together");
    assert_supported("Allies at Last");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let victim = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Forest", 3);
    let s = t.hand(P0, "Band Together");
    t.cast(P0, s)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .target(victim)
        .go();
    t.resolve();
    // 2 + 3 damage to a 6/4.
    assert!(!t.on_battlefield(victim));
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
}

#[test]
fn six_sided_die_rows_printed_with_dashes() {
    cr!("706.2", "706.3b");
    assert_supported("Six-Sided Die");
    for (roll, expect) in [(5u32, "destroyed"), (6, "exiled"), (1, "toughness 1")] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, "Hill Giant");
        t.g.dice.loaded.push_back(roll);
        t.lands(P0, "Swamp", 3);
        let s = t.hand(P0, "Six-Sided Die");
        t.cast(P0, s).target(c).go();
        t.resolve();
        match expect {
            "destroyed" => assert!(t.in_graveyard(P1, "Hill Giant")),
            "exiled" => assert!(t.in_exile("Hill Giant")),
            _ => assert_eq!(t.pt(c), (3, 1)),
        }
    }
}

#[test]
fn regenerate_regenerates_target_creature() {
    cr!("701.19a");
    assert_supported("Regenerate");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    let r = t.hand(P1, "Regenerate");
    t.cast(P1, r).target(bears).go();
    t.resolve();
    t.lands(P0, "Swamp", 3);
    let m = t.hand(P0, "Murder");
    t.cast(P0, m).target(bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
}

#[test]
fn flame_discharge_and_faerie_fencing_check_what_you_controlled_as_you_cast() {
    cr!("601.2i", "700.9");
    assert_supported("Flame Discharge");
    assert_supported("Faerie Fencing");
    for modified in [false, true] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        if modified {
            t.g.add_counters(Entity::Object(mine), "+1/+1", 1, None);
        }
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.lands(P0, "Mountain", 3);
        let s = t.hand(P0, "Flame Discharge");
        t.cast(P0, s).x(2).target(wurm).go();
        t.resolve();
        // 2 damage, or 4 to the 6/4.
        assert_eq!(t.on_battlefield(wurm), !modified, "modified: {modified}");
    }
    for faerie in [false, true] {
        let mut t = TestGame::new(2);
        if faerie {
            t.battlefield(P0, "Spellstutter Sprite");
        }
        let giant = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Swamp", 2);
        let s = t.hand(P0, "Faerie Fencing");
        t.cast(P0, s).x(1).target(giant).go();
        t.resolve();
        assert_eq!(t.on_battlefield(giant), !faerie, "faerie: {faerie}");
    }
}

#[test]
fn joyful_stormsculptor_damages_each_opponent() {
    cr!("702.51a", "603.2");
    assert_supported("Joyful Stormsculptor");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Joyful Stormsculptor");
    t.lands(P0, "Mountain", 4);
    let s = t.hand(P0, "Stoke the Flames");
    t.cast(P0, s).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P2), 19);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn rampaging_raptor_deals_that_much_damage_to_that_players_planeswalker() {
    cr!("120.3", "603.2");
    assert_supported("Rampaging Raptor");
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Rampaging Raptor");
    let jace = t.battlefield(P1, "Jace Beleren");
    let mine = t.battlefield(P0, "Jace Beleren");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(jace)]);
    t.attack(&[(raptor, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(!t.on_battlefield(jace));
    assert!(t.on_battlefield(mine));
}

#[test]
fn dose_of_dawnglow_blights_only_outside_your_main_phase() {
    cr!("505.1", "701.68a");
    assert_supported("Dose of Dawnglow");
    for main in [true, false] {
        let mut t = TestGame::new(2);
        let giant = t.graveyard(P0, "Hill Giant");
        if main {
            t.advance_to(P0, Step::PrecombatMain);
        } else {
            t.advance_to(P1, Step::PrecombatMain);
        }
        t.lands(P0, "Swamp", 5);
        let s = t.hand(P0, "Dose of Dawnglow");
        t.cast(P0, s).target(giant).go();
        t.resolve_all();
        let back = t.named_on_battlefield("Hill Giant");
        assert_eq!(back.len(), 1);
        assert_eq!(t.counters(back[0], "-1/-1"), if main { 0 } else { 2 }, "main: {main}");
    }
}

#[test]
fn mausoleum_turnkey_an_opponent_chooses_a_card_from_your_graveyard() {
    cr!("601.2c", "115.1");
    assert_supported("Mausoleum Turnkey");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    let wurm = t.graveyard(P1, "Craw Wurm");
    t.answer_targets(P1, &[Entity::Object(bears)]);
    t.enter(P0, "Mausoleum Turnkey");
    t.resolve_all();
    let cands = last_target_candidates(&t, P1);
    assert!(cands.contains(&Entity::Object(giant)));
    assert!(!cands.contains(&Entity::Object(wurm)), "only your graveyard");
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Hill Giant"));
}

#[test]
fn icy_prison_returns_the_exiled_creature_when_it_leaves() {
    cr!("610.3", "118.12");
    assert_supported("Icy Prison");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let prison = t.enter(P0, "Icy Prison");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    // Nobody pays {3} at P0's upkeep: it's sacrificed and the Giant returns.
    t.answer_yes(P0, false);
    t.answer_yes(P1, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(prison));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn sorin_the_mirthless_takes_the_top_card_and_loses_its_mana_value() {
    cr!("606.3");
    assert_supported("Sorin the Mirthless");
    for take in [true, false] {
        let mut t = TestGame::new(2);
        let sorin = t.battlefield(P0, "Sorin the Mirthless");
        let giant = t.library_top(P0, "Hill Giant");
        t.answer_yes(P0, take);
        t.activate(P0, sorin, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Hill Giant"), take);
        assert_eq!(t.life(P0), if take { 16 } else { 20 });
        if !take {
            assert_eq!(t.g.player(P0).library.last(), Some(&giant));
        }
    }
}

#[test]
fn nissas_defeat_draws_only_for_a_destroyed_nissa() {
    cr!("608.2h");
    assert_supported("Nissa's Defeat");
    for (name, draws) in [("Nissa, Voice of Zendikar", true), ("Garruk Wildspeaker", false)] {
        let mut t = TestGame::new(2);
        let pw = t.battlefield(P1, name);
        t.lands(P0, "Forest", 3);
        let defeat = t.hand(P0, "Nissa's Defeat");
        let hand = t.hand_size(P0);
        t.cast(P0, defeat).target(pw).go();
        t.resolve();
        assert!(!t.on_battlefield(pw));
        assert_eq!(t.hand_size(P0), hand - 1 + draws as usize, "{name}");
    }
}
