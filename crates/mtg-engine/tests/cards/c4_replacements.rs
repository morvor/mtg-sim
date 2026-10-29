//! Replacement and prevention effects, round 4 (cards4-costs-mana-replacements,
//! `oracle/patterns/replacements_5.rs`, `prevention_5.rs`, `kw/untap_replacement.rs`):
//! next-draw shields written "instead [effect]", a draw replaced by looking at cards and
//! then drawing, untap replacements on a group, and damage that can't be prevented or
//! redirected.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn ring_of_maruf_replaces_the_next_draw_with_a_card_from_outside_the_game() {
    cr!("614.1a", "614.11", "400.11b");
    interp!("outside-the-game-is-the-sideboard");
    ruling!("Ring of Ma'rûf", "this ability replaces your next draw");
    assert_supported(&["Ring of Ma'rûf"]);
    let mut t = TestGame::new(2);
    let side = t.g.add_to_sideboard(P0, vec![card("Grizzly Bears")]);
    let ring = t.battlefield(P0, "Ring of Ma'rûf");
    t.lands(P0, "Wastes", 5);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.activate(P0, ring, 0, &[]).unwrap();
    t.resolve();
    // Nothing yet: it waits for the next draw.
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    let lib = t.library_size(P0);
    t.g.draw_cards(P0, 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.library_size(P0), lib, "the draw was replaced");
    // Used up: the next draw is a draw.
    t.g.draw_cards(P0, 1);
    assert_eq!(t.library_size(P0), lib - 1);
}

#[test]
fn aladdins_lamp_looks_at_x_cards_keeps_one_on_top_and_draws_it() {
    cr!("614.1a", "614.5", "121.6");
    ruling!("Aladdin's Lamp", "If you use more than one Lamp effect");
    assert_supported(&["Aladdin's Lamp"]);
    let mut t = TestGame::new(2);
    let lamp = t.battlefield(P0, "Aladdin's Lamp");
    t.lands(P0, "Wastes", 3);
    let a = t.library_top(P0, "Grizzly Bears");
    let b = t.library_top(P0, "Hill Giant");
    let c = t.library_top(P0, "Lightning Bolt");
    let lib = t.library_size(P0);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, lamp, 0, &[]).unwrap();
    t.resolve();
    let hand = t.hand_size(P0);
    t.g.draw_cards(P0, 1);
    // One card drawn, one of the three looked at; the other two went to the bottom.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.library_size(P0), lib - 1);
    let drawn: Vec<ObjectId> = [a, b, c]
        .iter()
        .map(|o| t.g.current(*o))
        .filter(|o| matches!(t.zone(*o), mtg_engine::object::Zone::Hand(_)))
        .collect();
    assert_eq!(drawn.len(), 1);
    let top = t.g.library_top(P0).expect("a library");
    assert_eq!(t.g.obj(top).chars.name, "Filler");
    // The draw it made wasn't replaced again: the next one is an ordinary draw.
    t.g.draw_cards(P0, 1);
    assert_eq!(t.library_size(P0), lib - 2);
}

#[test]
fn freyalises_winds_untap_is_replaced_by_removing_the_wind_counters() {
    cr!("502.3", "614.1a");
    ruling!(
        "Freyalise's Winds",
        "first you determine which of your permanents will untap"
    );
    assert_supported(&["Freyalise's Winds"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Freyalise's Winds");
    let forest = t.battlefield(P0, "Forest");
    let other = t.battlefield(P0, "Mountain");
    t.g.tap(forest);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(forest, "wind"), 1);
    // A tapped permanent without wind counters untaps as usual.
    t.g.objects[other.0 as usize].tapped = true;
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.obj(forest).tapped, "its untap was replaced");
    assert_eq!(t.counters(forest, "wind"), 0);
    assert!(!t.g.obj(other).tapped);
    // Without counters now, it untaps in the next untap step.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.g.obj(forest).tapped);
}

#[test]
fn whippoorwill_makes_damage_to_the_creature_unpreventable() {
    cr!("615.12", "614.9");
    assert_supported(&["Whippoorwill"]);
    let mut t = TestGame::new(2);
    let bird = t.battlefield(P0, "Whippoorwill");
    t.lands(P0, "Forest", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, bird, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    // A prevention shield on the Bears prevents nothing.
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve)
        .modes(&[1])
        .target(Entity::Object(bears))
        .go();
    t.resolve();
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Object(bears)).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears") || t.in_exile("Grizzly Bears"));
}

#[test]
fn lava_burst_damage_to_a_creature_cant_be_prevented_but_to_a_player_can() {
    cr!("615.12");
    ruling!(
        "Lava Burst",
        "they will work on damage that Lava Burst would deal to a player"
    );
    assert_supported(&["Lava Burst"]);
    // To a creature: the shield prevents nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve)
        .modes(&[1])
        .target(Entity::Object(bears))
        .go();
    t.resolve();
    t.lands(P0, "Mountain", 3);
    let burst = t.hand(P0, "Lava Burst");
    t.cast(P0, burst).x(2).target(Entity::Object(bears)).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // To a player: prevented.
    let mut t = TestGame::new(2);
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve)
        .modes(&[1])
        .target(Entity::Player(P1))
        .go();
    t.resolve();
    t.lands(P0, "Mountain", 3);
    let burst = t.hand(P0, "Lava Burst");
    t.cast(P0, burst).x(2).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn rock_hydra_prevents_damage_up_to_its_counters_removing_one_for_each() {
    cr!("615.7", "615.1a");
    interp!("per-point-prevention-batch");
    assert_supported(&["Rock Hydra"]);
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Rock Hydra");
    t.g.add_counters(Entity::Object(hydra), "+1/+1", 2, None);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(hydra).go();
    t.resolve();
    // 3 damage, 2 counters: 2 prevented and both counters removed, 1 dealt.
    assert_eq!(t.counters(hydra, "+1/+1"), 0);
    assert_eq!(t.g.obj(hydra).damage, 1);
    // No counters: nothing is prevented.
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Rock Hydra");
    t.g.add_counters(Entity::Object(hydra), "+1/+1", 5, None);
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(hydra).go();
    t.resolve();
    assert_eq!(t.counters(hydra, "+1/+1"), 3);
    assert_eq!(t.g.obj(hydra).damage, 0);
}

#[test]
fn bloatfly_swarm_turns_damage_into_counters_removed_and_rad_counters() {
    cr!("615.5", "122.1");
    ruling!(
        "Bloatfly Swarm",
        "If damage that can't be prevented is dealt to Bloatfly Swarm, you still remove"
    );
    assert_supported(&["Bloatfly Swarm"]);
    let mut t = TestGame::new(2);
    let fly = t.battlefield(P0, "Bloatfly Swarm");
    t.g.add_counters(Entity::Object(fly), "+1/+1", 5, None);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(fly).go();
    t.resolve();
    assert_eq!(t.g.obj(fly).damage, 0);
    assert_eq!(t.counters(fly, "+1/+1"), 2);
    assert_eq!(t.g.player(P0).counter("rad"), 3);
    assert_eq!(t.g.player(P1).counter("rad"), 3);
}

#[test]
fn bronze_horse_prevents_damage_from_spells_that_target_it() {
    cr!("615.1a", "115.9b");
    assert_supported(&["Bronze Horse"]);
    // With another creature: a Shock targeting it is prevented.
    let mut t = TestGame::new(2);
    let horse = t.battlefield(P0, "Bronze Horse");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(horse).go();
    t.resolve();
    assert_eq!(t.g.obj(horse).damage, 0);
    // Damage from a spell that doesn't target it isn't prevented.
    t.lands(P1, "Mountain", 3);
    t.set_step(P1, Step::PrecombatMain);
    let pyro = t.hand(P1, "Pyroclasm");
    t.cast(P1, pyro).go();
    t.resolve();
    assert_eq!(t.g.obj(horse).damage, 2);
    // Without another creature: not prevented.
    let mut t = TestGame::new(2);
    let horse = t.battlefield(P0, "Bronze Horse");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(horse).go();
    t.resolve();
    assert_eq!(t.g.obj(horse).damage, 2);
}

#[test]
fn rain_of_gore_turns_life_a_spell_gives_its_controller_into_life_loss() {
    cr!("614.1a", "119.9");
    ruling!("Rain of Gore", "does not apply to life gain caused by combat damage");
    assert_supported(&["Rain of Gore"]);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rain of Gore");
    t.lands(P0, "Plains", 1);
    let salve = t.hand(P0, "Healing Salve");
    t.cast(P0, salve).modes(&[0]).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 17);
    // A spell giving life to another player isn't "its controller" gaining it.
    t.lands(P0, "Plains", 1);
    let salve = t.hand(P0, "Healing Salve");
    t.cast(P0, salve).modes(&[0]).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 23);
    // Lifelink combat damage isn't a spell or ability causing it.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rain of Gore");
    let lion = t.battlefield(P0, "Ajani's Sunstriker");
    t.attack(&[(lion, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn island_sanctuary_skips_a_draw_step_draw_and_keeps_out_ground_attackers() {
    cr!("614.1a", "504.1", "508.1c");
    ruling!("Island Sanctuary", "Since the draw is replaced");
    assert_supported(&["Island Sanctuary"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Island Sanctuary");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let crow = t.battlefield(P1, "Storm Crow");
    t.advance_to(P1, Step::Upkeep);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), hand, "the draw was skipped");
    // On P1's turn: the Bears can't attack P0.
    t.advance_to(P1, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    // Declined on P0's next draw step: the restriction is over by P1's next turn, and the
    // flying Crow could always attack.
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), hand + 1);
    t.advance_to(P1, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P0)), (crow, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 17);
    // A draw outside the draw step isn't replaced.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Island Sanctuary");
    let hand = t.hand_size(P0);
    t.g.draw_cards(P0, 1);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn mox_diamond_enters_only_if_a_land_card_is_discarded() {
    cr!("614.1c", "614.12");
    ruling!("Mox Diamond", "Mox Diamond never enters");
    assert_supported(&["Mox Diamond"]);
    let mut t = TestGame::new(2);
    let land = t.hand(P0, "Forest");
    let mox = t.hand(P0, "Mox Diamond");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(land)]);
    t.cast(P0, mox).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(mox)));
    assert!(t.in_graveyard(P0, "Forest"));
    // Declined: it goes to the graveyard without entering.
    let mut t = TestGame::new(2);
    t.hand(P0, "Forest");
    let mox = t.hand(P0, "Mox Diamond");
    t.answer_yes(P0, false);
    t.cast(P0, mox).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mox Diamond"));
    assert!(t.in_hand(P0, "Forest"));
}

#[test]
fn eye_for_an_eye_deals_as_much_damage_to_the_sources_controller() {
    cr!("609.7a", "614.1a");
    ruling!("Eye for an Eye", "The damage dealt to you is still being dealt by the original source");
    assert_supported(&["Eye for an Eye"]);
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Plains", 2);
    let eye = t.hand(P0, "Eye for an Eye");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, eye).go();
    t.resolve();
    t.resolve();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 17);
    // Used up: the next damage isn't answered.
    t.lands(P1, "Mountain", 1);
    let bolt2 = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt2).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn cover_of_winter_prevents_x_of_each_creatures_combat_damage() {
    cr!("615.7", "510.2");
    interp!("per-source-prevention-division");
    ruling!("Cover of Winter", "This prevention is applied creature by creature");
    assert_supported(&["Cover of Winter"]);
    let mut t = TestGame::new(2);
    let cover = t.battlefield(P0, "Cover of Winter");
    t.g.add_counters(Entity::Object(cover), "age", 2, None);
    let small = t.battlefield(P1, "Memnite");
    let big = t.battlefield(P1, "Hill Giant");
    t.advance_to(P1, Step::PrecombatMain);
    // Keep Cover of Winter: pay its cumulative upkeep if asked.
    t.attack(&[(small, Entity::Player(P0)), (big, Entity::Player(P0))], &[]);
    // Memnite's 1 is all prevented, 2 of the Giant's 3: P0 is dealt 1.
    assert_eq!(t.life(P0), 19);
    // One creature's damage to several recipients: X in all, divided among them.
    let mut t = TestGame::new(2);
    let cover = t.battlefield(P0, "Cover of Winter");
    t.g.add_counters(Entity::Object(cover), "age", 2, None);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    let wall = t.battlefield(P0, "Wall of Wood");
    t.advance_to(P1, Step::PrecombatMain);
    t.attack(&[(wurm, Entity::Player(P0))], &[(wall, wurm)]);
    let dealt = (20 - t.life(P0)) + t.obj_now(wall).damage as i32;
    assert_eq!(dealt, 4, "6 damage, 2 of it prevented");
}
