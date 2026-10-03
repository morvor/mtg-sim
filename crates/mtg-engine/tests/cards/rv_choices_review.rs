//! Review checks for the choice grammar: wordings read faithfully or not at all.

use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn random_cards_from_outside_the_game_are_not_read_as_permanents() {
    // "a creature card with mana value X chosen at random", "a copy of a Liliana
    // planeswalker chosen at random": cards outside the game, not the battlefield.
    for name in [
        "Momir Vig, Simic Visionary Avatar",
        "The Disciple of Vess",
    ] {
        assert!(
            !card(name).unsupported_text().is_empty(),
            "{name} compiles"
        );
    }
}

#[test]
fn aether_gust_a_permanent_goes_on_top_or_bottom_of_its_owners_library() {
    cr!("115.1", "608.2d");
    let def = card("Aether Gust");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Aether Gust");
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    let now = t.g.current(bears);
    assert_eq!(t.zone(now), Zone::Library(P1), "{}", t.dump_log());
    assert_eq!(t.g.player(P1).library[0], now, "the owner chose the bottom");
}

#[test]
fn dubious_challenge_the_exiled_cards_are_not_the_cards_looked_at() {
    // "exile up to two creature cards from among them, then shuffle. Target opponent may
    // choose one of the exiled cards ... Put the rest onto the battlefield under your
    // control.": read as choosing among (and putting onto the battlefield) the cards
    // looked at, which were shuffled away. Not read until "the exiled cards" is.
    assert!(!card("Dubious Challenge").unsupported_text().is_empty());
}

#[test]
fn storm_of_memories_casts_the_random_card_and_exiles_it_after() {
    cr!("608.2d", "614.1a");
    let def = card("Storm of Memories");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Hill Giant");
    let spell = t.hand(P0, "Storm of Memories");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    assert!(t.in_graveyard(P0, "Forest") && t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn alpha_brawl_each_of_those_creatures_deals_damage_equal_to_its_power() {
    cr!("120.2");
    let def = card("Alpha Brawl");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let mine = t.battlefield(P0, "Hill Giant");
    let spell = t.hand(P0, "Alpha Brawl");
    t.cast(P0, spell).target(wurm).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(elves));
    assert!(t.on_battlefield(mine));
    // The Craw Wurm (6/4) is dealt 2 + 1 damage.
    assert_eq!(t.obj_now(wurm).damage, 3, "{}", t.dump_log());
}

#[test]
fn coordinated_clobbering_they_each_deal_damage_equal_to_their_power() {
    cr!("120.2");
    let def = card("Coordinated Clobbering");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let spell = t.hand(P0, "Coordinated Clobbering");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert_eq!(t.obj_now(wurm).damage, 3, "{}", t.dump_log());
}

#[test]
fn the_abyss_the_upkeep_player_chooses_their_creature_to_destroy() {
    cr!("601.2c", "603.3d");
    let def = card("The Abyss");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Abyss");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    let thopter = t.battlefield(P1, "Ornithopter");
    let mine = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P1, &[Entity::Object(b)]);
    t.set_step(P0, mtg_engine::turn::Step::End);
    t.advance_to(P1, mtg_engine::turn::Step::Draw);
    assert!(t.on_battlefield(a), "{}", t.dump_log());
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(thopter) && t.on_battlefield(mine));
}

#[test]
fn witch_hunt_a_random_opponent_gains_control() {
    cr!("115.1");
    let def = card("Witch Hunt");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..12 {
        let mut t = TestGame::with_config(
            3,
            GameConfig {
                seed,
                ..Default::default()
            },
        );
        let hunt = t.battlefield(P0, "Witch Hunt");
        let from = t.asked().len();
        t.set_step(P0, mtg_engine::turn::Step::PostcombatMain);
        t.advance_to_step(mtg_engine::turn::Step::End);
        t.resolve_all();
        assert!(t.asked()[from..]
            .iter()
            .all(|(_, d)| !matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. })));
        let c = t.obj_now(hunt).controller;
        assert_ne!(c, P0, "seed {seed}: {}", t.dump_log());
        seen.insert(c);
    }
    assert_eq!(seen.len(), 2, "either opponent");
}

#[test]
fn by_invitation_only_each_player_sacrifices_that_many() {
    cr!("701.21a", "101.4");
    let def = card("By Invitation Only");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let mine = [
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P0, "Hill Giant"),
    ];
    let theirs = [
        t.battlefield(P1, "Grizzly Bears"),
        t.battlefield(P1, "Llanowar Elves"),
        t.battlefield(P1, "Craw Wurm"),
    ];
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let spell = t.hand(P0, "By Invitation Only");
    t.cast(P0, spell).go();
    t.resolve();
    assert!(mine.iter().all(|o| !t.on_battlefield(*o)), "{}", t.dump_log());
    assert_eq!(theirs.iter().filter(|o| t.on_battlefield(**o)).count(), 1);
}

#[test]
fn endless_whispers_the_chosen_opponent_gets_the_creature_at_the_end_step() {
    cr!("603.7", "608.2d");
    let def = card("Endless Whispers");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.battlefield(P0, "Endless Whispers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    t.advance_to_step(mtg_engine::turn::Step::End);
    t.resolve_all();
    let on = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(on.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(on[0]).controller, P1);
}
