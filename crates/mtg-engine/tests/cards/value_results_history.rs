//! Value grammar II, history amounts: values counted from this turn's events
//! (`Value::EventsThisTurn`): "the number of creatures that died under your control this
//! turn", "for each card you've discarded this turn", "for each opponent who lost life
//! this turn", "for each 1 life your opponents have lost this turn", "the damage dealt to
//! you this turn", cost reductions "for each creature that attacked this turn".

use mtg_engine::ability::*;
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

fn kill(t: &mut TestGame, id: ObjectId) {
    t.g.destroy(id, None);
    t.g.flush_events();
}

#[test]
fn body_count_counts_creatures_that_died_under_your_control() {
    cr!("700.4", "608.2h");
    assert_supported("Body Count");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P1, "Grizzly Bears");
    // A creature an opponent owns but you control dies under your control.
    let stolen = t.battlefield(P1, "Grizzly Bears");
    t.g.obj_mut(stolen).controller = P0;
    t.g.obj_mut(stolen).base_controller = P0;
    t.g.recompute();
    for id in [a, b, c, stolen] {
        kill(&mut t, id);
    }
    t.lands(P0, "Swamp", 3);
    let bc = t.hand(P0, "Body Count");
    let before = t.hand_size(P0);
    t.cast(P0, bc).go();
    t.resolve();
    // Three of the four died under your control.
    assert_eq!(t.hand_size(P0), before - 1 + 3);
}

#[test]
fn mahadi_counts_every_creature_that_died_this_turn_even_tokens() {
    cr!("700.4");
    assert_supported("Mahadi, Emporium Master");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mahadi, Emporium Master");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, a);
    kill(&mut t, b);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(count_subtype(&t, P0, "Treasure"), 2);
}

#[test]
fn gadrak_counts_only_nontoken_creatures_that_died() {
    cr!("700.4");
    assert_supported("Gadrak, the Crown-Scourge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gadrak, the Crown-Scourge");
    let bears = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, bears);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(count_subtype(&t, P0, "Treasure"), 1);
}

#[test]
fn change_of_fortune_counts_cards_discarded_this_turn_for_any_reason() {
    cr!("608.2h");
    assert_supported("Change of Fortune");
    ruling!(
        "Change of Fortune",
        "counts cards you discarded this turn for any reason"
    );
    let mut t = TestGame::new(2);
    // One card discarded earlier this turn.
    let early = t.hand(P0, "Grizzly Bears");
    t.g.discard(P0, early, None);
    t.g.flush_events();
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let cof = t.hand(P0, "Change of Fortune");
    t.cast(P0, cof).go();
    t.resolve();
    // Two discarded now plus one earlier: three cards.
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn hollow_one_costs_less_for_each_card_cycled_or_discarded() {
    cr!("601.2f");
    assert_supported("Hollow One");
    ruling!(
        "Hollow One",
        "cost is reduced even if the cards you've cycled or discarded aren't in your graveyard"
    );
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        let c = t.hand(P0, "Grizzly Bears");
        t.g.discard(P0, c, None);
    }
    t.g.flush_events();
    // {5} reduced by {2} for each of two discarded cards: {1}.
    t.lands(P0, "Mountain", 1);
    let h1 = t.hand(P0, "Hollow One");
    t.cast(P0, h1).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hollow One").len(), 1);
}

#[test]
fn rowdy_research_costs_less_for_each_creature_that_attacked() {
    cr!("601.2f", "508.1");
    assert_supported("Rowdy Research");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    // A creature that attacked and then left the battlefield still counts.
    kill(&mut t, a);
    t.advance_to(P0, Step::PostcombatMain);
    // {6}{U} minus {1} for each of the two attackers: {4}{U}.
    t.lands(P0, "Island", 5);
    let rr = t.hand(P0, "Rowdy Research");
    let before = t.hand_size(P0);
    t.cast(P0, rr).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), before - 1 + 3);
}

#[test]
fn teysa_investigates_for_each_opponent_who_lost_life() {
    cr!("119.3");
    assert_supported("Teysa, Opulent Oligarch");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Teysa, Opulent Oligarch");
    t.g.lose_life(P1, 2);
    t.g.lose_life(P1, 1);
    t.g.lose_life(P0, 1);
    t.g.flush_events();
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // Only P1 of the two opponents lost life (twice): one Clue; your own loss doesn't
    // count.
    assert_eq!(count_subtype(&t, P0, "Clue"), 1);
}

#[test]
fn neheb_adds_red_for_each_life_opponents_lost() {
    cr!("119.3", "106.4");
    assert_supported("Neheb, the Eternal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Neheb, the Eternal");
    t.g.lose_life(P1, 3);
    t.g.gain_life(P1, 5);
    t.g.flush_events();
    t.set_step(P0, Step::EndOfCombat);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    // Life lost counts, whatever life was gained.
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn archfiend_of_despair_each_opponent_loses_the_life_they_lost() {
    cr!("119.3", "608.2h");
    assert_supported("Archfiend of Despair");
    ruling!(
        "Archfiend of Despair",
        "counts only how much life was lost. It doesn’t care whether a player also gained life"
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Archfiend of Despair");
    t.g.lose_life(P1, 4);
    t.g.lose_life(P2, 1);
    t.g.lose_life(P0, 3);
    t.g.flush_events();
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // Each opponent loses what they lost: P1 4 more, P2 1 more; you don't.
    assert_eq!(t.life(P1), 12);
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn final_punishment_loses_the_damage_already_dealt_to_that_player() {
    cr!("120.3a", "608.2h");
    assert_supported("Final Punishment");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(bears, Entity::Player(P1), 2, false);
    t.g.deal_damage(bears, Entity::Player(P1), 3, true);
    t.g.lose_life(P1, 4);
    t.g.flush_events();
    t.lands(P0, "Swamp", 5);
    let fp = t.hand(P0, "Final Punishment");
    t.cast(P0, fp).target(P1).go();
    t.resolve();
    // 20 - 5 damage - 4 loss - 5 (damage, not other life loss).
    assert_eq!(t.life(P1), 6);
}

#[test]
fn chandras_incinerator_costs_less_by_noncombat_damage_to_opponents() {
    cr!("601.2f", "120.2b");
    assert_supported("Chandra's Incinerator");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(bears, Entity::Player(P1), 3, false);
    // Combat damage doesn't count.
    t.g.deal_damage(bears, Entity::Player(P1), 2, true);
    t.g.flush_events();
    // {5}{R} minus {3}: {2}{R}.
    t.lands(P0, "Mountain", 3);
    let ci = t.hand(P0, "Chandra's Incinerator");
    t.cast(P0, ci).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Chandra's Incinerator").len(), 1);
}

#[test]
fn gnoll_war_band_ignores_opponents_who_left_the_game() {
    cr!("601.2f", "800.4a");
    assert_supported("Gnoll War Band");
    ruling!(
        "Gnoll War Band",
        "Opponents who have left the game will not be counted"
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(bears, Entity::Player(P1), 1, false);
    t.g.deal_damage(bears, Entity::Player(P2), 1, false);
    t.g.flush_events();
    let v = Value::EventsThisTurn(
        Box::new(TriggerCond::PlayerDealtDamage {
            who: PlayerRel::Opponent,
            combat_only: false,
        }),
        Tally::Players,
    );
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert_eq!(t.g.eval_value(&v, &ctx), 2);
    t.g.player_mut(P2).has_lost = true;
    t.g.player_mut(P2).left_game = true;
    assert_eq!(t.g.eval_value(&v, &ctx), 1);
}
