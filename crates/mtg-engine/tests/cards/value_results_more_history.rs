//! Value grammar II, more history amounts: permanents that entered this turn, times a
//! creature attacked, cards drawn by opponents, card types among spells cast or permanents
//! sacrificed, counters you've put, cards put into a graveyard from a library, damage
//! dealt by sources, creatures that dealt combat damage.

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

fn count_subtype(t: &TestGame, p: PlayerId, sub: &str) -> usize {
    t.g.permanents_controlled_by(p)
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.has_subtype(sub))
        .count()
}

#[test]
fn kinbinding_counts_creatures_that_entered_under_your_control() {
    cr!("613.4c", "608.2h");
    let c = card("Kinbinding");
    assert!(
        !c.unsupported_text()
            .iter()
            .any(|u| u.contains("entered the battlefield")),
        "{:?}",
        c.unsupported_text()
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kinbinding");
    let bears = t.enter(P0, "Grizzly Bears");
    t.enter(P0, "Grizzly Bears");
    // An opponent's creature doesn't count.
    t.enter(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn moraug_counts_the_times_each_creature_attacked() {
    cr!("613.4c", "508.1");
    let c = card("Moraug, Fury of Akoum");
    assert!(
        !c.unsupported_text()
            .iter()
            .any(|u| u.contains("time it has attacked")),
        "{:?}",
        c.unsupported_text()
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moraug, Fury of Akoum");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(t.pt(other), (2, 2));
}

#[test]
fn thought_sponge_enters_with_the_most_cards_one_opponent_drew() {
    cr!("614.1c", "121.1");
    assert_supported("Thought Sponge");
    let mut t = TestGame::new(3);
    t.g.draw_cards(P1, 2);
    t.g.draw_cards(P2, 3);
    t.g.draw_cards(P0, 4);
    t.g.flush_events();
    let ts = t.enter(P0, "Thought Sponge");
    assert_eq!(t.counters(ts, "+1/+1"), 3);
}

#[test]
fn heliod_spells_cost_less_for_each_card_opponents_drew() {
    cr!("601.2f");
    let c = card("Heliod, the Radiant Dawn // Heliod, the Warped Eclipse");
    assert!(
        c.unsupported_text().is_empty(),
        "{:?}",
        c.unsupported_text()
    );
    let mut t = TestGame::new(2);
    let def = CardDef::custom(c.faces[1].chars.clone());
    t.custom(P0, def, mtg_engine::object::Zone::Battlefield);
    t.g.draw_cards(P1, 2);
    t.g.flush_events();
    // Hill Giant {3}{R} for {1}{R}.
    t.lands(P0, "Mountain", 2);
    let hg = t.hand(P0, "Hill Giant");
    t.cast(P0, hg).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn korvold_costs_less_for_each_card_type_among_permanents_sacrificed() {
    cr!("601.2f", "205.2a");
    assert_supported("Korvold, Gleeful Glutton");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let orni = t.battlefield(P0, "Ornithopter");
    let land = t.battlefield(P0, "Forest");
    for o in [bears, orni, land] {
        t.g.sacrifice(o, P0);
    }
    t.g.flush_events();
    // Creature, artifact (Ornithopter is both), land: three card types. {5}{B}{R}{G}
    // minus {3}.
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let k = t.hand(P0, "Korvold, Gleeful Glutton");
    t.cast(P0, k).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Korvold, Gleeful Glutton").len(), 1);
}

#[test]
fn iridescent_hornbeetle_counts_counters_you_put_on_your_creatures() {
    cr!("122.6");
    assert_supported("Iridescent Hornbeetle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Iridescent Hornbeetle");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // Counters you put on a creature you control count; on an opponent's, they don't.
    let src = t.g.permanents_controlled_by(P0)[0];
    t.g.add_counters(Entity::Object(bears), "+1/+1", 2, Some(src));
    t.g.add_counters(Entity::Object(theirs), "+1/+1", 1, Some(src));
    t.g.flush_events();
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // Two Insect tokens (the Hornbeetle is an Insect too).
    assert_eq!(count_subtype(&t, P0, "Insect"), 3);
}

#[test]
fn cruel_calculations_counts_cards_milled_into_the_target_players_graveyard() {
    cr!("701.17a", "608.2h");
    assert_supported("Cruel Calculations");
    let mut t = TestGame::new(2);
    t.g.mill(P1, 3);
    t.g.mill(P0, 2);
    t.g.flush_events();
    t.lands(P0, "Island", 3);
    let cc = t.hand(P0, "Cruel Calculations");
    let before = t.hand_size(P0);
    t.cast(P0, cc).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), before - 1 + 3);
}

#[test]
fn first_family_counts_colors_among_permanents_and_spells_cast() {
    cr!("105.2", "608.2h");
    assert_supported("First Family");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 2);
    let ff = t.hand(P0, "First Family");
    let before = t.hand_size(P0);
    t.cast(P0, ff).go();
    t.resolve();
    // Green (Bears), red (Lightning Bolt), green and blue (First Family itself was cast
    // this turn): three colors.
    assert_eq!(t.hand_size(P0), before - 1 + 3);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn april_oneil_draws_for_each_card_type_among_spells_cast() {
    cr!("205.2a");
    assert_supported("April O'Neil, Hacktivist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "April O'Neil, Hacktivist");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve();
    let before = t.hand_size(P0);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // An instant and a creature.
    assert_eq!(t.hand_size(P0), before + 2);
}

#[test]
fn indulge_excess_counts_creatures_that_dealt_combat_damage_to_a_player() {
    cr!("510.2");
    let c = card("Indulge // Excess");
    assert!(
        c.unsupported_text().is_empty(),
        "{:?}",
        c.unsupported_text()
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    let v = mtg_engine::ability::Value::CountSel(Box::new(mtg_engine::ability::Sel::ThisTurn(
        Box::new(mtg_engine::ability::TriggerCond::DealsDamage {
            source: mtg_engine::ability::Filter::and(vec![
                mtg_engine::ability::Filter::creature(),
                mtg_engine::ability::Filter::ControlledBy(mtg_engine::ability::PlayerRel::You),
            ]),
            to: mtg_engine::ability::DamageRecipient::Player(mtg_engine::ability::PlayerRel::Any),
            combat_only: true,
        }),
    )));
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert_eq!(t.g.eval_value(&v, &ctx), 2);
}

#[test]
fn blitzwing_converts_when_no_life_was_lost_this_way() {
    cr!("119.3", "608.2c");
    let c = card("Blitzwing, Cruel Tormentor // Blitzwing, Adaptive Assailant");
    assert!(
        !c.unsupported_text()
            .iter()
            .any(|u| u.contains("no life is lost")),
        "{:?}",
        c.unsupported_text()
    );
}

#[test]
fn cut_a_deal_draws_for_each_opponent_who_drew_this_way() {
    cr!("608.2c", "121.1");
    assert_supported("Cut a Deal");
    let mut t = TestGame::new(3);
    t.lands(P0, "Plains", 3);
    let cd = t.hand(P0, "Cut a Deal");
    let before = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    t.cast(P0, cd).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), before.1 + 1);
    assert_eq!(t.hand_size(P2), before.2 + 1);
    assert_eq!(t.hand_size(P0), before.0 - 1 + 2);
}

#[test]
fn smugglers_share_counts_opponents_over_a_threshold() {
    cr!("121.1", "608.2h");
    assert_supported("Smuggler's Share");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Smuggler's Share");
    t.g.draw_cards(P1, 2);
    t.g.draw_cards(P2, 1);
    t.g.flush_events();
    t.enter(P1, "Forest");
    t.enter(P1, "Forest");
    t.enter(P2, "Forest");
    let before = t.hand_size(P0);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // P1 drew two cards and had two lands enter; P2 only one of each.
    assert_eq!(t.hand_size(P0), before + 1);
    assert_eq!(count_subtype(&t, P0, "Treasure"), 1);
}

#[test]
fn kaya_orzhov_usurper_gains_the_damage_dealt() {
    cr!("120.4b", "608.2c");
    let c = card("Kaya, Orzhov Usurper");
    assert!(
        c.unsupported_text().is_empty(),
        "{:?}",
        c.unsupported_text()
    );
    let mut t = TestGame::new(2);
    let kaya = t.battlefield(P0, "Kaya, Orzhov Usurper");
    t.g.add_counters(Entity::Object(kaya), "loyalty", 5, None);
    t.exile(P1, "Grizzly Bears");
    t.exile(P1, "Grizzly Bears");
    t.exile(P0, "Grizzly Bears");
    t.g.flush_events();
    t.activate(P0, kaya, 2, &[Entity::Player(P1)])
        .expect("activate");
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}
