//! Rulings batch P088 — Suncleanser's "can't have counters put on it" / "can't get
//! counters" (CR 122.1, 614.17, 702.80, 702.90, 702.24), Spirit of the Labyrinth's draw
//! limit (CR 121.2b, 614.11), Frenzied Baloth's "combat damage can't be prevented"
//! (CR 615.12), Tunnel Ignus's lands entering (CR 603.2, 603.4) and Thresher Lizard's
//! damage (CR 704.5g).

use crate::r_p088_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Suncleanser enters, choosing its first mode for the creature `target` (or its
/// second for the player).
fn suncleanse(t: &mut TestGame, target: Entity) -> ObjectId {
    supported("Suncleanser");
    let mode = if matches!(target, Entity::Player(_)) {
        1
    } else {
        0
    };
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![mode]));
    t.answer_targets(P0, &[target]);
    let s = t.enter(P0, "Suncleanser");
    t.resolve_all();
    s
}

#[test]
fn suncleanser_and_infect() {
    cr!("702.90b", "702.90c", "122.1", "702.15b", "120.3");
    ruling!(
        "Suncleanser",
        "Damage from a source with infect has no effect on an affected creature or player. No -1/-1 counters are put on creatures, and no damage is marked on them. Players don’t get poison counters and they don’t lose life. The damage is still dealt for purposes of effects that care about damage, such as lifelink."
    );
    supported("Flensermite");
    // Flensermite: infect, lifelink.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    suncleanse(&mut t, obj(bears));
    let mite = t.battlefield(P0, "Flensermite");
    t.g.deal_damage(mite, obj(bears), 1, false);
    t.settle();
    assert_eq!(t.counters(bears, "-1/-1"), 0);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.life(P0), 21);
    // An affected player.
    let mut t = TestGame::new(2);
    suncleanse(&mut t, Entity::Player(P1));
    let mite = t.battlefield(P0, "Flensermite");
    t.g.deal_damage(mite, Entity::Player(P1), 2, false);
    t.settle();
    assert_eq!(t.g.player(P1).counter("poison"), 0);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn suncleanser_and_wither() {
    cr!("702.80a", "122.1", "120.3d", "603.2");
    ruling!(
        "Suncleanser",
        "Damage from a source with wither has no effect on an affected creature. No -1/-1 counters are put on them, and no damage is marked on them. The damage is still dealt for purposes of effects that care about damage, such as lifelink."
    );
    supported("Kulrath Knight");
    supported("Spirit Link");
    // Kulrath Knight: flying, wither. Spirit Link on it: "Whenever enchanted creature
    // deals damage, you gain that much life."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    suncleanse(&mut t, obj(bears));
    let knight = t.battlefield(P0, "Kulrath Knight");
    crate::r_s06_common::attach_new(&mut t, P0, "Spirit Link", knight);
    t.g.deal_damage(knight, obj(bears), 3, false);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(bears, "-1/-1"), 0);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn suncleanser_and_counter_replacement_effects() {
    cr!("614.6", "616.1", "122.1");
    ruling!(
        "Suncleanser",
        "If a replacement effect allows a player to modify or replace an event by putting counters on a creature or player affected by Suncleanser, that player may apply that replacement effect, but they won’t get counters. If the original event is entirely replaced (such as by applying Soul-Scar Mage’s replacement effect), the original event won’t happen."
    );
    supported("Soul-Scar Mage");
    supported("Hardened Scales");
    // Soul-Scar Mage: "If a source you control would deal noncombat damage to a creature
    // an opponent controls, put that many -1/-1 counters on that creature instead."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    suncleanse(&mut t, obj(bears));
    t.battlefield(P0, "Soul-Scar Mage");
    cast_new(&mut t, P0, "Lightning Bolt", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.counters(bears, "-1/-1"), 0);
    // Hardened Scales ("If one or more +1/+1 counters would be put on a creature you
    // control, that many plus one ... instead") on an affected creature: no counters.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    suncleanse(&mut t, obj(bears));
    t.battlefield(P0, "Hardened Scales");
    t.g.add_counters(obj(bears), "+1/+1", 1, None);
    t.settle();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn suncleanser_costs_and_choices_to_put_counters() {
    cr!("614.17b", "118.3", "122.1");
    ruling!(
        "Suncleanser",
        "If the cost of an ability or an additional cost of a spell requires putting counters on a creature or player affected by Suncleanser, that cost can’t be paid. If a resolving spell or ability says that a player may give counters to that creature or player, that player can’t choose to do so."
    );
    supported("Devoted Druid");
    supported("Ajani's Pridemate");
    // Devoted Druid: "Put a -1/-1 counter on this creature: Untap this creature."
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Devoted Druid");
    t.g.tap(druid);
    assert!(can_activate_containing(&mut t, P0, druid, "Untap"));
    suncleanse(&mut t, obj(druid));
    assert!(!can_activate_containing(&mut t, P0, druid, "Untap"));
    // Ajani's Pridemate: "Whenever you gain life, you may put a +1/+1 counter on this
    // creature." P0 can't choose to.
    let mut t = TestGame::new(2);
    let pm = t.battlefield(P0, "Ajani's Pridemate");
    suncleanse(&mut t, obj(pm));
    t.answer_yes(P0, true);
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(pm, "+1/+1"), 0);
}

#[test]
fn suncleanser_and_cumulative_upkeep() {
    cr!("702.24a", "122.1", "614.17");
    ruling!(
        "Suncleanser",
        "While resolving a cumulative upkeep trigger of an affected creature, you’ll fail to put a counter on that permanent, then you may pay for the age counters already on it. If it has no age counters on it, you may pay {0}."
    );
    supported("Gallowbraid");
    // Gallowbraid: "Cumulative upkeep—Pay 1 life."
    let mut t = TestGame::new(2);
    let gb = t.battlefield(P0, "Gallowbraid");
    t.g.obj_mut(gb).counters.insert("age".into(), 2);
    suncleanse(&mut t, obj(gb));
    assert_eq!(t.counters(gb, "age"), 0);
    t.answer_yes(P0, true);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(gb));
    assert_eq!(t.counters(gb, "age"), 0);
    assert_eq!(t.life(P0), 20);
}

/// The cards P0 draws when told to draw `n`.
fn draw(t: &mut TestGame, n: u32) -> usize {
    let r = t.g.draw_cards(P0, n).len();
    t.g.flush_events();
    t.settle();
    r
}

#[test]
fn spirit_of_the_labyrinth_draw_limit() {
    cr!("121.2b");
    ruling!(
        "Spirit of the Labyrinth",
        "If you haven't drawn any cards in a turn, and a spell or ability would cause you to draw multiple cards, you'll just draw one card."
    );
    ruling!(
        "Spirit of the Labyrinth",
        "You can draw a maximum of one card on each player's turn. Subsequent card draws are ignored."
    );
    ruling!(
        "Spirit of the Labyrinth",
        "If you draw a card, and then Spirit of the Labyrinth enters the battlefield, you won't be able to draw more cards that turn."
    );
    supported("Spirit of the Labyrinth");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Spirit of the Labyrinth");
    assert_eq!(draw(&mut t, 3), 1);
    assert_eq!(draw(&mut t, 1), 0);
    // On P1's turn, P0 may draw one card again.
    t.set_step(P1, Step::PrecombatMain);
    t.g.history.cards_drawn.clear();
    t.advance_to(P1, Step::End);
    assert_eq!(draw(&mut t, 2), 1);
    // Drawing a card before Spirit of the Labyrinth enters.
    let mut t = TestGame::new(2);
    assert_eq!(draw(&mut t, 1), 1);
    t.enter(P1, "Spirit of the Labyrinth");
    t.settle();
    assert_eq!(draw(&mut t, 1), 0);
}

#[test]
fn spirit_of_the_labyrinth_stops_replacing_a_draw_that_cant_happen() {
    cr!("121.2b", "614.11", "702.52a");
    ruling!(
        "Spirit of the Labyrinth",
        "If a replacement effect would try to replace a card that you can't draw, that effect can't apply."
    );
    supported("Stinkweed Imp");
    // Dredge 5 (Stinkweed Imp): "If you would draw a card, you may mill five cards
    // instead. If you do, return this card from your graveyard to your hand."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Spirit of the Labyrinth");
    assert_eq!(draw(&mut t, 1), 1);
    let imp = t.graveyard(P0, "Stinkweed Imp");
    let library = t.library_size(P0);
    let from = t.asked().len();
    yes(&mut t, P0, 2);
    t.answer(P0, DecisionKind::Replacement, Answer::Index(0));
    assert_eq!(draw(&mut t, 1), 0);
    assert_eq!(t.zone(imp), Zone::Graveyard(P0));
    assert_eq!(t.library_size(P0), library);
    assert!(!t.asked()[from..].iter().any(|(_, d)| matches!(
        d,
        Decision::ChooseReplacement { .. } | Decision::YesNo { .. }
    )));
    t.clear_answers();
}

#[test]
fn frenzied_baloth_only_stops_prevention() {
    cr!("615.12", "614.1", "510.2");
    ruling!(
        "Frenzied Baloth",
        "Frenzied Baloth’s last ability only stops combat damage from being prevented by effects that specifically use the word “prevent.”"
    );
    supported("Frenzied Baloth");
    supported("Holy Day");
    supported("Pariah");
    // Holy Day: "Prevent all combat damage that would be dealt this turn." Doesn't work.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let baloth = t.battlefield(P0, "Frenzied Baloth");
    crate::r_s01_common::attack_with(&mut t, &[(baloth, Entity::Player(P1))]);
    cast_new(&mut t, P1, "Holy Day", &[]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    // Pariah ("All damage that would be dealt to you is dealt to enchanted creature
    // instead."), a redirection that doesn't say "prevent": it still applies.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::attach_new(&mut t, P1, "Pariah", giant);
    t.set_step(P0, Step::BeginningOfCombat);
    let baloth = t.battlefield(P0, "Frenzied Baloth");
    crate::r_s01_common::attack_with(&mut t, &[(baloth, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn tunnel_ignus_counts_lands_that_entered_earlier_or_together() {
    cr!("603.2", "603.4", "603.2c");
    ruling!(
        "Tunnel Ignus",
        "Whenever a land an opponent controls enters, this ability triggers only if another land had already entered the battlefield under that opponent's control this turn. It doesn't matter if that other land is still on the battlefield, is still under that player's control, or is still a land."
    );
    ruling!(
        "Tunnel Ignus",
        "If multiple lands an opponent controls enter at the same time, Tunnel Ignus's ability triggers that many times, even if no other lands had entered the battlefield under that player's control earlier in the turn. That's because each ability takes into consideration the other land(s) that entered at the same time as the one that caused it to trigger."
    );
    supported("Tunnel Ignus");
    // One land, then (after it's destroyed) another: only the second triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tunnel Ignus");
    let f = t.enter(P1, "Forest");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    destroy(&mut t, f);
    t.enter(P1, "Forest");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Two lands at once, the first this turn: two triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tunnel Ignus");
    enter_together(&mut t, &[("Forest", P1), ("Island", P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn thresher_lizard_damage_can_become_lethal() {
    cr!("704.5g", "514.2", "613.4c");
    ruling!(
        "Thresher Lizard",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to Thresher Lizard while you have one or fewer cards in hand may become lethal if cards are put into your hand during that turn."
    );
    supported("Thresher Lizard");
    let mut t = TestGame::new(2);
    let lizard = t.battlefield(P0, "Thresher Lizard");
    assert_eq!(t.pt(lizard), (4, 4));
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(src, obj(lizard), 3, false);
    t.settle();
    assert!(t.on_battlefield(lizard));
    t.g.draw_cards(P0, 2);
    t.settle();
    assert!(!t.on_battlefield(lizard));
    let _ = ManaType::C;
}
