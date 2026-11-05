//! Rulings batch P179 — "toll" abilities: triggers on an opponent casting a spell,
//! drawing or searching (Hive Mind, Witchstalker, Hermit of the Natterknolls, Dragonlair
//! Spider, Golem's Heart, Consecrated Sphinx, Wan Shi Tong, Alms Collector, ...).

use crate::r_p179_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 casts the real card `name` (lands for its cost) during P0's turn; triggers are put
/// on the stack. Returns the spell.
fn p1_casts_on_p0s_turn(t: &mut TestGame, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    t.set_step(P0, Step::PrecombatMain);
    let spell = cast_targeting(t, P1, name, targets);
    t.settle();
    spell
}

#[test]
fn hive_mind_copies_are_not_cast() {
    cr!("707.10", "601.2");
    ruling!(
        "Hive Mind",
        "The copies that Hive Mind’s ability creates are created on the stack, so they’re not “cast.”"
    );
    supported("Hive Mind");
    supported("Dragonlair Spider");
    let mut t = TestGame::new(2);
    let hive = t.battlefield(P0, "Hive Mind");
    let spider = t.battlefield(P0, "Dragonlair Spider");
    // P1 casts Lightning Bolt at P0: Hive Mind and Dragonlair Spider both trigger.
    p1_casts_on_p0s_turn(&mut t, "Lightning Bolt", &[Entity::Player(P0)]);
    assert_eq!(triggers_from(&t, hive), 1);
    assert_eq!(triggers_from(&t, spider), 1);
    t.resolve_all();
    // P0's copy (keeping its target, P0) wasn't cast: Hive Mind didn't trigger again
    // (that would have given P1 a third Bolt) and the Spider made only one Insect.
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.life(P1), 20);
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn witchstalker_gets_its_counter_before_the_spell_resolves() {
    cr!("603.3", "405.5");
    ruling!(
        "Witchstalker",
        "The counter will be put on Witchstalker before the spell cast by an opponent resolves."
    );
    supported("Witchstalker");
    let mut t = TestGame::new(2);
    let ws = t.battlefield(P0, "Witchstalker");
    let opt = p1_casts_on_p0s_turn(&mut t, "Opt", &[]);
    assert_eq!(triggers_from(&t, ws), 1);
    t.resolve();
    assert_eq!(t.counters(ws, "+1/+1"), 1);
    assert_eq!(t.zone(opt), Zone::Stack);
}

/// Hermit of the Natterknolls (front or transformed): P1 casts Opt during P0's turn, P0
/// counters Opt above the trigger; the trigger still resolves. Returns P0's hand growth
/// from the trigger.
fn natterknolls_draw_after_counter(transformed: bool) -> usize {
    supported("Hermit of the Natterknolls");
    let mut t = TestGame::new(2);
    let hermit = t.battlefield(P0, "Hermit of the Natterknolls");
    if transformed {
        crate::r_s17_common::transform(&mut t, hermit);
        assert_eq!(
            t.obj_now(hermit).chars.name.as_str(),
            "Lone Wolf of the Natterknolls"
        );
    }
    let opt = p1_casts_on_p0s_turn(&mut t, "Opt", &[]);
    let hermit = t.g.current(hermit);
    assert_eq!(triggers_from(&t, hermit), 1);
    // P0 counters Opt in response to the trigger.
    cast_targeting(&mut t, P0, "Counterspell", &[obj(opt)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Opt"));
    let before = t.hand_size(P0);
    t.resolve_all();
    t.hand_size(P0) - before
}

#[test]
fn hermit_of_the_natterknolls_draws_even_if_the_spell_is_countered() {
    cr!("603.2", "701.6a");
    ruling!(
        "Hermit of the Natterknolls",
        "The first ability of Hermit of the Natterknolls will resolve even if the spell your opponent casts is countered."
    );
    assert_eq!(natterknolls_draw_after_counter(false), 1);
}

#[test]
fn lone_wolf_of_the_natterknolls_draws_even_if_the_spell_is_countered() {
    cr!("603.2", "701.6a");
    ruling!(
        "Hermit of the Natterknolls",
        "The first ability of Lone Wolf of the Natterknolls will resolve even if the spell your opponent casts is countered."
    );
    assert_eq!(natterknolls_draw_after_counter(true), 2);
}

#[test]
fn natterknolls_you_may_act_between_the_trigger_and_the_spell() {
    cr!("117.3b", "608.2");
    ruling!(
        "Hermit of the Natterknolls",
        "You may cast spells or activate abilities after the first ability of Hermit of the Natterknolls has resolved but before the spell that caused it to trigger has resolved."
    );
    ruling!(
        "Hermit of the Natterknolls",
        "You may cast spells or activate abilities after the first ability of Lone Wolf of the Natterknolls has resolved but before the spell that caused it to trigger has resolved."
    );
    for transformed in [false, true] {
        let mut t = TestGame::new(2);
        let hermit = t.battlefield(P0, "Hermit of the Natterknolls");
        if transformed {
            crate::r_s17_common::transform(&mut t, hermit);
        }
        let opt = p1_casts_on_p0s_turn(&mut t, "Opt", &[]);
        let hand = t.hand_size(P0);
        t.resolve();
        assert_eq!(t.hand_size(P0), hand + if transformed { 2 } else { 1 });
        assert_eq!(t.zone(opt), Zone::Stack);
        // P0 now casts Counterspell using a card it holds, with Opt still on the stack.
        let cs = cast_targeting(&mut t, P0, "Counterspell", &[obj(opt)]);
        assert_eq!(t.zone(cs), Zone::Stack);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Opt"));
    }
}

#[test]
fn dragonlair_spider_insect_is_created_before_the_spell_resolves_even_if_countered() {
    cr!("603.3", "608.1");
    ruling!(
        "Dragonlair Spider",
        "The triggered ability will resolve and the Insect will be created before the spell that caused it to trigger resolves."
    );
    ruling!(
        "Dragonlair Spider",
        "The triggered ability won’t be affected if the spell that caused it to trigger is countered."
    );
    supported("Dragonlair Spider");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dragonlair Spider");
    let opt = p1_casts_on_p0s_turn(&mut t, "Opt", &[]);
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.zone(opt), Zone::Stack);
    // Countered: the trigger already resolved; again with the counter cast above it.
    let opt2 = cast_targeting(&mut t, P1, "Opt", &[]);
    t.settle();
    cast_targeting(&mut t, P0, "Counterspell", &[obj(opt2)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Opt"));
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn golems_heart_triggers_on_any_players_artifact_spell_and_resolves_first() {
    cr!("603.3", "405.5");
    ruling!(
        "Golem's Heart",
        "Golem’s Heart’s ability triggers whenever any player casts an artifact spell, not just when you do."
    );
    ruling!(
        "Golem's Heart",
        "Whenever a player casts an artifact spell, Golem’s Heart’s ability triggers and goes on the stack on top of it."
    );
    supported("Golem's Heart");
    for caster in [P0, P1] {
        let mut t = TestGame::new(2);
        let heart = t.battlefield(P0, "Golem's Heart");
        t.set_step(caster, Step::PrecombatMain);
        let thopter = cast_targeting(&mut t, caster, "Ornithopter", &[]);
        t.settle();
        assert_eq!(triggers_from(&t, heart), 1);
        assert_eq!(*t.g.stack.first().unwrap(), thopter);
        t.answer_yes(P0, true);
        t.resolve();
        assert_eq!(t.life(P0), 21);
        assert_eq!(t.zone(thopter), Zone::Stack);
    }
    // A nonartifact spell doesn't trigger it.
    let mut t = TestGame::new(2);
    let heart = t.battlefield(P0, "Golem's Heart");
    cast_targeting(&mut t, P1, "Opt", &[]);
    t.settle();
    assert_eq!(triggers_from(&t, heart), 0);
}

#[test]
fn consecrated_sphinx_draws_two_or_none() {
    cr!("603.5");
    ruling!(
        "Consecrated Sphinx",
        "You may either draw two cards or not draw at all. You can't choose to draw only one card."
    );
    supported("Consecrated Sphinx");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Consecrated Sphinx");
        t.set_step(P1, Step::PrecombatMain);
        let from = t.asked().len();
        t.g.draw_cards(P1, 1);
        t.g.flush_events();
        t.settle();
        t.answer_yes(P0, yes);
        let before = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), before + if yes { 2 } else { 0 });
        // The only choice offered is yes or no: no number of cards.
        let asked = decisions_since(&t, P0, from);
        assert!(asked.iter().any(|d| d.starts_with("YesNo")), "{asked:?}");
        assert!(!asked.iter().any(|d| d.starts_with("ChooseNumber")));
    }
}

/// P0 activates Tergrid's Lantern targeting P1 (who controls a Plains, plus Grizzly Bears
/// if `bears`, and holds Opt if `card`), with P1's yes/no answers queued. Returns the
/// game and the yes/no prompts P1 was asked.
fn lantern(bears: bool, card: bool, answers: &[bool]) -> (TestGame, Vec<String>) {
    supported("Tergrid, God of Fright // Tergrid's Lantern");
    let mut t = TestGame::new(2);
    let lantern = battlefield_back(&mut t, P0, "Tergrid, God of Fright // Tergrid's Lantern");
    assert_eq!(t.obj_now(lantern).chars.name.as_str(), "Tergrid's Lantern");
    t.battlefield(P1, "Plains");
    if bears {
        t.battlefield(P1, "Grizzly Bears");
    }
    if card {
        t.hand(P1, "Opt");
    }
    for a in answers {
        t.answer_yes(P1, *a);
    }
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, lantern, "loses 3 life").unwrap();
    t.resolve_all();
    let asked = decisions_since(&t, P1, from)
        .into_iter()
        .filter(|d| d.starts_with("YesNo"))
        .collect();
    (t, asked)
}

#[test]
fn tergrids_lantern_target_may_lose_life_but_cant_sacrifice_or_discard_nothing() {
    cr!("118.12a");
    ruling!(
        "Tergrid, God of Fright // Tergrid's Lantern",
        "can choose to lose 3 life even if they have nonland permanents to sacrifice or cards to discard."
    );
    // With both a nonland permanent and a card, P1 may decline both and lose 3 life.
    let (t, asked) = lantern(true, true, &[false, false]);
    assert_eq!(asked.len(), 2, "{asked:?}");
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.hand_size(P1), 1);
    // Positive controls: sacrificing, or discarding, instead.
    let (t, _) = lantern(true, true, &[true]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let (t, _) = lantern(true, true, &[false, true]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Opt"));
    // Nothing to sacrifice (only a land) and no cards: no choice is offered, P1 loses 3
    // (a "yes" would be meaningless).
    let (t, asked) = lantern(false, false, &[true, true]);
    assert!(asked.is_empty(), "{asked:?}");
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.named_on_battlefield("Plains").len(), 1);
    // A card but no nonland permanent: only the discard is offered.
    let (t, asked) = lantern(false, true, &[true]);
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert!(asked[0].contains("Discard"), "{asked:?}");
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Opt"));
}

#[test]
fn nezahal_returns_before_cleanup_if_exiled_before_the_end_step() {
    cr!("402.2", "514.1", "603.7a");
    ruling!(
        "Nezahal, Primal Tide",
        "Your maximum hand size is checked only during the cleanup step on your turn. If Nezahal's last ability is activated before your turn's end step, it will return before your next cleanup step and you'll have no maximum hand size."
    );
    supported("Nezahal, Primal Tide");
    for during_end_step in [false, true] {
        let mut t = TestGame::new(2);
        let nez = t.battlefield(P0, "Nezahal, Primal Tide");
        fill_hand(&mut t, P0, 11);
        if during_end_step {
            t.advance_to(P0, Step::End);
        }
        activate_containing(&mut t, P0, nez, "Discard three cards").unwrap();
        t.resolve_all();
        assert_eq!(t.zone(nez), Zone::Exile);
        assert_eq!(t.hand_size(P0), 8);
        t.advance_to(P1, Step::Upkeep);
        if during_end_step {
            // The next end step is P1's: Nezahal is still in exile at P0's cleanup.
            assert_eq!(t.zone(nez), Zone::Exile);
            assert_eq!(t.hand_size(P0), 7);
        } else {
            assert!(t.on_battlefield(nez));
            assert!(t.obj_now(nez).tapped);
            assert_eq!(t.hand_size(P0), 8);
        }
    }
}

#[test]
fn prowl_draws_only_while_it_is_still_stoic_strategist() {
    cr!("607.2a", "712.8e");
    ruling!(
        "Prowl, Stoic Strategist // Prowl, Pursuit Vehicle",
        "The exiled cards' owners may play them for as long as they remain exiled, but the triggered ability that causes you to draw a card will trigger only if Prowl is still on the battlefield as Prowl, Stoic Strategist."
    );
    supported("Prowl, Stoic Strategist // Prowl, Pursuit Vehicle");
    for converted in [false, true] {
        let mut t = TestGame::new(2);
        let prowl = t.battlefield(P0, "Prowl, Stoic Strategist // Prowl, Pursuit Vehicle");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.objects[bears.0 as usize].tapped = true;
        t.answer_targets(P0, &[obj(bears)]);
        t.attack(&[(prowl, Entity::Player(P1))], &[]);
        assert_eq!(t.zone(bears), Zone::Exile);
        if converted {
            crate::r_s17_common::transform(&mut t, prowl);
            assert_eq!(
                t.obj_now(prowl).chars.name.as_str(),
                "Prowl, Pursuit Vehicle"
            );
        }
        t.set_step(P1, Step::PrecombatMain);
        give_mana_for(&mut t, P1, "Grizzly Bears");
        let hand = t.hand_size(P0);
        let exiled = t.g.current(bears);
        t.cast(P1, exiled).go();
        t.settle();
        t.resolve_all();
        assert!(t.on_battlefield(exiled));
        if converted {
            assert_eq!(t.hand_size(P0), hand);
        } else {
            assert_eq!(t.hand_size(P0), hand + 1);
            assert_eq!(
                t.obj_now(prowl).chars.name.as_str(),
                "Prowl, Pursuit Vehicle"
            );
        }
    }
}

#[test]
fn archaeomancers_map_rechecks_the_land_count_on_resolution() {
    cr!("603.4");
    ruling!(
        "Archaeomancer's Map",
        "The last ability triggers only if that opponent controls more lands than you after the land enters the battlefield."
    );
    supported("Archaeomancer's Map");
    // (P1 lands after playing one, destroy one of P1's lands in response) → triggers,
    // Forest put onto the battlefield.
    for (p1_lands, respond, triggers, puts) in [
        (1, false, true, true),
        (1, true, true, false),
        (0, false, false, false),
    ] {
        let mut t = TestGame::new(2);
        let map = t.battlefield(P0, "Archaeomancer's Map");
        t.battlefield(P0, "Plains");
        let p1 = t.lands(P1, "Mountain", p1_lands);
        let forest = t.hand(P0, "Forest");
        t.set_step(P1, Step::PrecombatMain);
        let land = t.hand(P1, "Mountain");
        t.play_land(P1, land).unwrap();
        t.g.flush_events();
        t.settle();
        assert_eq!(triggers_from(&t, map), triggers as usize, "{p1_lands}");
        if respond {
            destroy(&mut t, p1[0]);
        }
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(forest)]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(forest), puts, "{p1_lands} {respond}");
    }
}

/// Wan Shi Tong's search triggers on the stack.
fn wan_shi_tong_triggers(g: &mtg_engine::game::Game) -> usize {
    g.stack
        .iter()
        .filter(|id| {
            g.obj(**id).stack.as_ref().is_some_and(|si| {
                matches!(&si.kind, mtg_engine::object::StackKind::Triggered { ability, .. }
                    if ability.text.contains("searches their library"))
            })
        })
        .count()
}

#[test]
fn wan_shi_tong_librarian_triggers_after_the_search_finishes_resolving() {
    cr!("603.3", "701.23a");
    ruling!(
        "Wan Shi Tong, Librarian",
        "The second triggered ability won't be put on the stack until after the spell or ability causing the opponent to search their library finishes resolving."
    );
    supported("Wan Shi Tong, Librarian");
    supported("Evolving Wilds");
    let mut t = TestGame::new(2);
    let wst = t.battlefield(P0, "Wan Shi Tong, Librarian");
    let wilds = t.battlefield(P1, "Evolving Wilds");
    let forest = t.library_top(P1, "Forest");
    t.set_step(P1, Step::PrecombatMain);
    // While P1 chooses the card, nothing is on the stack for Wan Shi Tong yet.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |d| matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }),
        wan_shi_tong_triggers,
    );
    t.answer_choose(P1, &[obj(forest)]);
    activate_containing(&mut t, P1, wilds, "Search").unwrap();
    t.resolve();
    assert_eq!(*seen.lock().unwrap(), vec![0]);
    // The ability finished (the Forest is on the battlefield), then the trigger went on
    // the stack.
    assert!(t.on_battlefield(forest));
    assert_eq!(triggers_from(&t, wst), 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.counters(wst, "+1/+1"), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn wan_shi_tong_librarian_ignores_searches_of_other_players_libraries() {
    cr!("701.23a");
    ruling!(
        "Wan Shi Tong, Librarian",
        "won't trigger if you search an opponent's library or if an opponent searches another player's library."
    );
    supported("Bribery");
    // P0 searches P1's library with Bribery: no trigger.
    let mut t = ffa(3);
    let wst = t.battlefield(P0, "Wan Shi Tong, Librarian");
    let bears = t.library_top(P1, "Grizzly Bears");
    cast_targeting(&mut t, P0, "Bribery", &[Entity::Player(P1)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(triggers_from(&t, wst), 0);
    // P1 (an opponent) searches P2's library with Bribery: no trigger.
    let ogre = t.library_top(P2, "Gray Ogre");
    t.set_step(P1, Step::PrecombatMain);
    cast_targeting(&mut t, P1, "Bribery", &[Entity::Player(P2)]);
    t.answer_choose(P1, &[obj(ogre)]);
    t.resolve();
    assert!(t.on_battlefield(ogre));
    assert_eq!(triggers_from(&t, wst), 0);
    // Control: P2 searches their own library.
    let wilds = t.battlefield(P2, "Evolving Wilds");
    let forest = t.library_top(P2, "Forest");
    t.set_step(P2, Step::PrecombatMain);
    t.answer_choose(P2, &[obj(forest)]);
    activate_containing(&mut t, P2, wilds, "Search").unwrap();
    t.resolve();
    assert_eq!(triggers_from(&t, wst), 1);
}

#[test]
fn wan_shi_tong_all_knowing_ignores_scry_but_sees_cards_put_into_a_library() {
    cr!("701.22a", "603.2");
    ruling!(
        "Wan Shi Tong, All-Knowing",
        "triggers only when cards are put into a library from somewhere other than the library. It won't trigger when you scry"
    );
    supported("Wan Shi Tong, All-Knowing");
    let mut t = TestGame::new(2);
    let wst = t.battlefield(P0, "Wan Shi Tong, All-Knowing");
    // Opt: scry 1, putting the top card on the bottom. No trigger.
    let top = t.library_top(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Scry,
        mtg_engine::decision::Answer::Split(vec![], vec![top]),
    );
    cast_targeting(&mut t, P0, "Opt", &[]);
    t.resolve();
    assert_eq!(*t.g.player(P0).library.first().unwrap(), top);
    assert_eq!(triggers_from(&t, wst), 0);
    // Control: Brainstorm puts two cards from hand into the library: one trigger.
    cast_targeting(&mut t, P0, "Brainstorm", &[]);
    t.resolve();
    assert_eq!(triggers_from(&t, wst), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn alms_collector_counts_the_draw_instructions() {
    cr!("614.1a", "121.2");
    ruling!(
        "Alms Collector",
        "Alms Collector’s replacement effect watches for one “draw” that instructs a player to draw multiple cards (including instructions to “draw a card for each. . .”)."
    );
    supported("Alms Collector");
    supported("Collective Unconscious");
    // (spell P1 casts, creatures P1 controls, P0's draws, P1's draws)
    let twice = crate::r_s01_common::custom_card(
        "Twice Drawn",
        "Sorcery",
        "{U}",
        None,
        "Draw a card. Draw a card.",
    );
    for (name, creatures, p0, p1) in [
        ("Divination", 0, 1, 1),
        ("Collective Unconscious", 2, 1, 1),
        ("Collective Unconscious", 1, 0, 1),
        ("Twice Drawn", 0, 0, 2),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Alms Collector");
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Grizzly Bears", creatures);
        let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
        if name == "Twice Drawn" {
            let c = t.custom(P1, twice.clone(), Zone::Hand(P1));
            t.lands(P1, "Island", 1);
            t.cast(P1, c).go();
        } else {
            cast_targeting(&mut t, P1, name, &[]);
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0) - h0, p0, "{name} {creatures}");
        assert_eq!(t.hand_size(P1) - h1, p1, "{name} {creatures}");
    }
}

#[test]
fn hamletback_goliath_uses_power_on_resolution_or_last_known() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Hamletback Goliath",
        "The value of X is determined as the ability begins resolving. If the other creature is no longer on the battlefield, its last known existence on the battlefield is checked to determine its power."
    );
    supported("Hamletback Goliath");
    for kill in [false, true] {
        let mut t = TestGame::new(2);
        let goliath = t.battlefield(P0, "Hamletback Goliath");
        let bears = t.enter(P1, "Grizzly Bears");
        t.g.flush_events();
        t.settle();
        assert_eq!(triggers_from(&t, goliath), 1);
        cast_targeting(&mut t, P0, "Giant Growth", &[obj(bears)]);
        t.resolve();
        if kill {
            destroy(&mut t, bears);
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.counters(goliath, "+1/+1"), 5, "kill {kill}");
    }
}

#[test]
fn species_specialist_offers_only_creature_types() {
    cr!("205.3m", "607.2d");
    ruling!(
        "Species Specialist",
        "You must choose an existing creature type, such as Human or Warrior. Card types such as artifact and supertypes such as legendary can’t be chosen."
    );
    supported("Species Specialist");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Species Specialist");
    let options = options_asked_since(&t, P0, from);
    assert_eq!(options.len(), 1);
    let o = &options[0];
    assert!(o.iter().any(|x| x == "Human") && o.iter().any(|x| x == "Warrior"));
    for bad in ["Artifact", "Legendary", "Creature", "Equipment", "Forest"] {
        assert!(!o.iter().any(|x| x == bad), "{bad} offered");
    }
    // The chosen type works: a Bear dying draws (P0 says yes), a non-Bear doesn't trigger.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    destroy(&mut t, ogre);
    assert_eq!(t.g.stack.len(), 0);
    destroy(&mut t, bears);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
