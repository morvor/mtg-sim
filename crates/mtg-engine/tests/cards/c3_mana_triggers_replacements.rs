//! Mana riders and replacements (cards3-costs-mana-replacements): tapped-for-mana
//! triggers on a list of land types or on mana of the chosen color, "they add", mana
//! scaled by a count, "an amount of mana of that color", "using mana produced by ~"
//! copies, "would enter and no mana was spent to cast it", a fight's target named again,
//! and "the next time you would lose the game this turn".

use mtg_engine::ability::Duration;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

fn tap(t: &mut TestGame, p: PlayerId, land: ObjectId) {
    t.activate(p, land, 0, &[]).unwrap();
}

#[test]
fn keeper_of_progenitus_doubles_mountains_forests_and_plains() {
    cr!("106.12a", "605.1b");
    assert_supported("Keeper of Progenitus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keeper of Progenitus");
    let forest = t.battlefield(P1, "Forest");
    let island = t.battlefield(P1, "Island");
    let plains = t.battlefield(P0, "Plains");
    tap(&mut t, P1, forest);
    // A triggered mana ability: it resolves at once, for the player who tapped it.
    assert_eq!(pool(&t, P1), vec![ManaType::G, ManaType::G]);
    tap(&mut t, P1, island);
    assert_eq!(pool(&t, P1), vec![ManaType::U, ManaType::G, ManaType::G]);
    tap(&mut t, P0, plains);
    assert_eq!(pool(&t, P0), vec![ManaType::W, ManaType::W]);
}

#[test]
fn cheering_crowd_adds_colorless_for_each_counter_if_they_do() {
    cr!("106.4", "603.5");
    assert_supported("Cheering Crowd");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let crowd = t.battlefield(P0, "Cheering Crowd");
        t.g.add_counters(Entity::Object(crowd), "+1/+1", 1, None);
        t.answer_yes(P1, yes);
        t.advance_to(P1, Step::PrecombatMain);
        t.resolve_all();
        if yes {
            assert_eq!(t.counters(crowd, "+1/+1"), 2);
            assert_eq!(pool(&t, P1), vec![ManaType::C, ManaType::C]);
        } else {
            assert_eq!(t.counters(crowd, "+1/+1"), 1);
            assert!(pool(&t, P1).is_empty());
        }
        assert!(pool(&t, P0).is_empty());
    }
}

#[test]
fn spoils_of_evil_counts_artifact_and_creature_cards() {
    cr!("106.4", "119.3");
    assert_supported("Spoils of Evil");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Mind Stone");
    t.graveyard(P1, "Memnite");
    t.graveyard(P1, "Forest");
    t.graveyard(P1, "Lightning Bolt");
    // Your own graveyard doesn't count.
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let spoils = t.hand(P0, "Spoils of Evil");
    t.cast(P0, spoils).target(P1).go();
    t.resolve_all();
    assert_eq!(pool(&t, P0), vec![ManaType::C; 3]);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn gauntlet_of_power_adds_mana_of_the_chosen_color_for_each_basic_land() {
    cr!("106.12a", "607.2d");
    assert_supported("Gauntlet of Power");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gauntlet of Power");
    t.g.objects[g.0 as usize].choices.color = Some(Color::Green);
    let f0 = t.battlefield(P0, "Forest");
    let f1 = t.battlefield(P1, "Forest");
    let plains = t.battlefield(P0, "Plains");
    let tomb = t.battlefield(P0, "Ancient Tomb");
    tap(&mut t, P0, f0);
    assert_eq!(pool(&t, P0), vec![ManaType::G, ManaType::G]);
    // Its controller adds it.
    tap(&mut t, P1, f1);
    assert_eq!(pool(&t, P1), vec![ManaType::G, ManaType::G]);
    // Not the chosen color; not a basic land.
    tap(&mut t, P0, plains);
    tap(&mut t, P0, tomb);
    assert_eq!(
        pool(&t, P0),
        vec![ManaType::W, ManaType::G, ManaType::G, ManaType::C, ManaType::C]
    );
}

#[test]
fn three_tree_city_adds_mana_of_a_chosen_color_for_creatures_of_its_type() {
    cr!("106.1b", "607.2d");
    assert_supported("Three Tree City");
    let mut t = TestGame::new(2);
    let city = t.battlefield(P0, "Three Tree City");
    t.g.objects[city.0 as usize].choices.creature_type = Some("Bear".into());
    t.lands(P0, "Plains", 2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    // An opponent's Bear doesn't count.
    t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, city, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::R, ManaType::R]);
}

#[test]
fn the_myriad_pools_turns_a_permanent_into_a_copy_of_the_spell_until_end_of_turn() {
    cr!("106.3", "707.2");
    ruling!(
        "The Everflowing Well // The Myriad Pools",
        "The triggered ability of The Myriad Pools will resolve before the spell that caused it to trigger."
    );
    assert_supported("The Everflowing Well // The Myriad Pools");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let well = t.battlefield(P0, "The Everflowing Well // The Myriad Pools");
    assert!(mtg_engine::dfc::transform(&mut t.g, well));
    t.g.recompute();
    let memnite = t.battlefield(P0, "Memnite");
    t.lands(P0, "Island", 1);
    let sable = t.hand(P0, "Bronze Sable");
    t.answer_targets(P0, &[Entity::Object(memnite)]);
    t.cast(P0, sable).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    t.g.recompute();
    assert_eq!(t.obj_now(memnite).chars.name, "Bronze Sable");
    assert_eq!(t.pt(memnite), (2, 1));
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    assert_eq!(t.obj_now(memnite).chars.name, "Memnite");
}

#[test]
fn primeval_spawn_is_exiled_unless_cast_with_mana() {
    cr!("614.1c", "614.12");
    ruling!(
        "Primeval Spawn",
        "As long as mana was paid for any cost to cast Primeval Spawn, it enters the battlefield as it resolves."
    );
    assert_supported("Primeval Spawn");
    // Put onto the battlefield: exiled instead.
    let mut t = TestGame::new(2);
    let id = t.g.create_card_object(card("Primeval Spawn"), P0, Zone::Hand(P0));
    let moved = t
        .g
        .move_object(id, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0));
    let now = t.g.current(moved.unwrap_or(id));
    assert_eq!(t.zone(now), Zone::Exile);
    // Cast with mana: it enters.
    let mut t = TestGame::new(2);
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, l, 2);
    }
    let spawn = t.hand(P0, "Primeval Spawn");
    t.cast(P0, spawn).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(spawn)));
    // Cast without paying its mana cost: no mana was spent, so it's exiled.
    let mut t = TestGame::new(2);
    let spawn = t.hand(P0, "Primeval Spawn");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![spawn],
        Duration::EndOfTurn,
        true,
        None,
    );
    t.cast(P0, spawn).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(spawn)), Zone::Exile);
}

#[test]
fn unnatural_aggression_exiles_the_opponents_creature_if_it_would_die() {
    cr!("701.14a", "614.1a");
    assert_supported("Unnatural Aggression");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "Unnatural Aggression");
    t.cast(P0, spell)
        .target(giant).target(bears)
        .go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Exile);
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    // Your own creature isn't covered.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "Unnatural Aggression");
    t.cast(P0, spell)
        .target(bears).target(giant)
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(giant));
}

#[test]
fn stunning_reversal_replaces_the_next_loss_this_turn() {
    cr!("104.3b", "614.1a", "704.5a");
    ruling!(
        "Stunning Reversal",
        "For your life total to become 1, you gain or lose the appropriate amount of life."
    );
    assert_supported("Stunning Reversal");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    for _ in 0..10 {
        t.library_top(P0, "Forest");
    }
    let sr = t.hand(P0, "Stunning Reversal");
    t.cast(P0, sr).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(sr)), Zone::Exile);
    let hand = t.hand_size(P0);
    // At 3 life, a Lightning Bolt would make P0 lose (CR 704.5a).
    t.g.players[P0.idx()].life = 3;
    bolt(&mut t);
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 1);
    assert_eq!(t.hand_size(P0), hand + 7);
    // Used up: the next loss this turn happens.
    bolt(&mut t);
    assert!(t.has_lost(P0));
}

/// P1 casts Lightning Bolt at P0, and it resolves.
fn bolt(t: &mut TestGame) {
    t.lands(P1, "Mountain", 1);
    let b = t.hand(P1, "Lightning Bolt");
    t.cast(P1, b).target(P0).go();
    t.resolve_all();
}

#[test]
fn stunning_reversal_ends_with_the_turn() {
    cr!("514.2");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 4);
    let sr = t.hand(P0, "Stunning Reversal");
    t.cast(P0, sr).go();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.g.players[P0.idx()].life = 3;
    bolt(&mut t);
    assert!(t.has_lost(P0));
}

#[test]
fn camp_makes_junk_if_the_creature_shares_a_color_with_the_mana() {
    cr!("106.12a", "105.2");
    assert_supported("C.A.M.P.");
    for (land, junk) in [("Forest", 1), ("Mountain", 0)] {
        let mut t = TestGame::new(2);
        let camp = t.battlefield(P0, "C.A.M.P.");
        let l = t.battlefield(P0, land);
        t.g.attach(camp, Entity::Object(l));
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        tap(&mut t, P0, l);
        t.resolve_all();
        assert_eq!(t.counters(bears, "+1/+1"), 1, "{land}");
        let n = t
            .g
            .permanents_controlled_by(P0)
            .into_iter()
            .filter(|o| t.g.obj(*o).chars.has_subtype("Junk"))
            .count();
        assert_eq!(n, junk, "{land}");
    }
}
