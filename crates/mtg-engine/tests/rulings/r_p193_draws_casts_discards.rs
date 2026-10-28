//! Rulings batch P193 — abilities that count the cards drawn or discarded, or the spells
//! cast, this turn: only actual draws count (CR 121.2), the whole turn counts (not just
//! while the ability's source was on the battlefield), and a spell counts itself.

use crate::r_p216_common::{foretell, FORETELL};
use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use crate::r_s20_common::tap_for_mana;
use crate::r_s29_common::cast_and_resolve;
use crate::r_s31_common::put_together;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn draw(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.draw_cards(p, n);
    t.g.flush_events();
    t.settle();
}

/// A card put into `p`'s hand without being drawn.
fn put_in_hand(t: &mut TestGame, p: PlayerId) {
    let c = t.g.player(p).library.last().copied().unwrap();
    move_to(t, c, Zone::Hand(p));
}

#[test]
fn kydele_counts_every_draw_this_turn_and_only_draws() {
    cr!("121.1", "605.1a");
    ruling!(
        "Kydele, Chosen of Kruphix",
        "Cards put into your hand without using the word \"draw\" don't count as cards drawn for Kydele's ability."
    );
    ruling!(
        "Kydele, Chosen of Kruphix",
        "If you haven't drawn any cards this turn, you can activate Kydele's mana ability, but it won't produce any mana."
    );
    ruling!(
        "Kydele, Chosen of Kruphix",
        "Kydele's ability counts every card you've drawn this turn, whether or not they're still in your hand and whether or not Kydele was on the battlefield or under your control at the time they were drawn."
    );
    supported("Kydele, Chosen of Kruphix");
    let mut t = TestGame::new(2);
    let kydele = t.battlefield(P0, "Kydele, Chosen of Kruphix");
    put_in_hand(&mut t, P0);
    t.g.turn.priority = Some(P0);
    let uid = crate::r_s27_common::ability_containing(&mut t, kydele, "{C}");
    assert!(t.g.activate_ability(P0, kydele, uid).is_ok());
    assert!(t.obj_now(kydele).tapped);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // Two cards drawn before Kydele was on the battlefield, one of them discarded since.
    let mut t = TestGame::new(2);
    draw(&mut t, P0, 2);
    let c = t.g.player(P0).hand[0];
    move_to(&mut t, c, Zone::Graveyard(P0));
    put_in_hand(&mut t, P0);
    let kydele = t.battlefield(P0, "Kydele, Chosen of Kruphix");
    assert!(tap_for_mana(&mut t, P0, kydele, "{C}"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

#[test]
fn fists_of_flame_and_thundering_djinn_dont_count_cards_put_into_hand() {
    cr!("121.1", "608.2h");
    ruling!(
        "Fists of Flame",
        "If a spell or ability causes you to put cards into your hand without specifically using the word \"draw,\" Fists of Flame's effect doesn't count them."
    );
    ruling!(
        "Thundering Djinn",
        "If a spell or ability causes you to put cards into your hand without specifically using the word “draw,” Thundering Djinn’s effect doesn’t count them."
    );
    supported("Fists of Flame");
    supported("Thundering Djinn");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    draw(&mut t, P0, 1);
    put_in_hand(&mut t, P0);
    cast_and_resolve(&mut t, P0, "Fists of Flame", &[Entity::Object(bears)]);
    assert_eq!(t.pt(bears), (4, 2));
    let mut t = TestGame::new(2);
    let djinn = t.battlefield(P0, "Thundering Djinn");
    draw(&mut t, P0, 1);
    put_in_hand(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.attack(&[(djinn, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20 - 1 - 3);
}

#[test]
fn green_goblin_counts_every_discard_this_turn_including_its_own() {
    cr!("701.9a", "608.2h");
    ruling!(
        "Green Goblin, Revenant",
        "Green Goblin's last ability counts the number of cards you have discarded previously this turn, including the one you discard to Green Goblin's own ability. It doesn't matter if Green Goblin was on the battlefield at the time the cards were discarded."
    );
    supported("Green Goblin, Revenant");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Island");
    t.hand(P0, "Island");
    t.g.discard(P0, a, None);
    t.g.flush_events();
    t.settle();
    let goblin = t.battlefield(P0, "Green Goblin, Revenant");
    let before = t.hand_size(P0);
    t.attack(&[(goblin, Entity::Player(P1))], &[]);
    // Discarded 1 (hand 0), then drew 2.
    assert_eq!(t.hand_size(P0), before - 1 + 2);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn dihadas_ploy_still_discards_and_counts_earlier_discards_when_draws_are_replaced() {
    cr!("614.1a", "701.9a", "608.2c");
    ruling!(
        "Dihada's Ploy",
        "If the card draws are replaced by another effect, you will still discard a card and gain life. If you don't have any cards to discard, it will still count any cards previously discarded this turn."
    );
    supported("Dihada's Ploy");
    supported("Asmodeus the Archfiend");
    // "If you would draw a card, exile the top card of your library face down instead."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Asmodeus the Archfiend");
    let a = t.hand(P0, "Island");
    t.g.discard(P0, a, None);
    t.g.flush_events();
    t.settle();
    cast_and_resolve(&mut t, P0, "Dihada's Ploy", &[]);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.life(P0), 21);
    // With a card in hand, it's discarded and counted.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Asmodeus the Archfiend");
    t.hand(P0, "Island");
    cast_and_resolve(&mut t, P0, "Dihada's Ploy", &[]);
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Island"));
    assert_eq!(t.life(P0), 21);
}

#[test]
fn profts_eidetic_memory_doesnt_trigger_with_one_card_drawn() {
    cr!("603.4");
    ruling!(
        "Proft's Eidetic Memory",
        "If you haven't drawn more than one card by the time a beginning of combat step begins on your turn, Proft's Eidetic Memory's last ability won't trigger at all."
    );
    supported("Proft's Eidetic Memory");
    for drawn in [1, 3] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Proft's Eidetic Memory");
        let bears = t.battlefield(P0, "Grizzly Bears");
        draw(&mut t, P0, drawn);
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        assert_eq!(t.stack_len(), usize::from(drawn > 1));
        t.resolve_all();
        assert_eq!(t.counters(bears, counters::PLUS1), drawn - 1);
    }
}

#[test]
fn impending_flux_and_surge_of_brilliance_count_themselves_cast_from_exile() {
    cr!("702.143a", "601.2i");
    ruling!(
        "Impending Flux",
        "If you cast Impending Flux from exile, it counts itself when determining the value of X."
    );
    ruling!(
        "Surge of Brilliance",
        "If you cast Surge of Brilliance from exile, it counts itself when counting the number of spells cast from anywhere other than your hand."
    );
    supported("Impending Flux");
    supported("Surge of Brilliance");
    let mut t = TestGame::new(2);
    let flux = t.hand(P0, "Impending Flux");
    let surge = t.hand(P0, "Surge of Brilliance");
    let flux = foretell(&mut t, P0, flux);
    let surge = foretell(&mut t, P0, surge);
    let ogre = t.battlefield(P1, "Centaur Courser");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    // Impending Flux: X = 1 + 1 (itself).
    t.lands(P0, "Mountain", 3);
    t.cast(P0, flux).method(FORETELL).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.obj_now(ogre).damage, 2);
    // Surge of Brilliance: two spells from exile now, itself included.
    let hand = t.hand_size(P0);
    t.lands(P0, "Island", 2);
    t.cast(P0, surge).method(FORETELL).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn shadow_of_the_grave_finds_a_madness_card_only_if_it_wasnt_cast() {
    cr!("702.35a", "702.35c", "400.7k");
    ruling!(
        "Shadow of the Grave",
        "If you discard a card with madness but don’t cast it, Shadow of the Grave can find it in your graveyard. If you do cast that card, Shadow of the Grave can’t find it in your graveyard."
    );
    supported("Shadow of the Grave");
    supported("Fiery Temper");
    for cast in [false, true] {
        let mut t = TestGame::new(2);
        let temper = t.hand(P0, "Fiery Temper");
        if cast {
            t.lands(P0, "Mountain", 1);
            t.answer_targets(P0, &[Entity::Player(P1)]);
        }
        t.answer_yes(P0, cast);
        t.g.discard(P0, temper, None);
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        assert_eq!(t.life(P1), if cast { 17 } else { 20 });
        assert!(t.in_graveyard(P0, "Fiery Temper"));
        cast_and_resolve(&mut t, P0, "Shadow of the Grave", &[]);
        assert_eq!(t.in_hand(P0, "Fiery Temper"), !cast, "cast: {cast}");
    }
}

#[test]
fn geralf_counts_spells_cast_before_it_was_on_the_battlefield() {
    cr!("603.2", "700.4");
    ruling!(
        "Geralf, the Fleshwright",
        "Geralf’s first ability will consider spells you cast before it was on the battlefield."
    );
    supported("Geralf, the Fleshwright");
    let mut t = TestGame::new(2);
    let geralf = t.hand(P0, "Geralf, the Fleshwright");
    t.lands(P0, "Island", 3);
    t.cast(P0, geralf).go();
    t.resolve_all();
    assert!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Zombie").is_empty());
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Zombie").len(),
        1
    );
}

#[test]
fn geralf_zombies_entering_together_see_each_other() {
    cr!("603.6a", "608.2h");
    ruling!(
        "Geralf, the Fleshwright",
        "If multiple Zombies enter the battlefield under your control at the same time, Geralf’s last ability will trigger for each of those Zombies. Each of those triggered abilities will see each of those other Zombies"
    );
    supported("Walking Corpse");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Geralf, the Fleshwright");
    let first = t.enter(P0, "Walking Corpse");
    t.resolve_all();
    assert_eq!(t.counters(first, counters::PLUS1), 0);
    let a = t.graveyard(P0, "Walking Corpse");
    let b = t.graveyard(P0, "Walking Corpse");
    let ids = put_together(&mut t, &[a, b]);
    t.resolve_all();
    for id in ids {
        assert_eq!(t.counters(id.unwrap(), counters::PLUS1), 2);
    }
    // An opponent's Zombie entering doesn't count or trigger.
    t.enter(P1, "Walking Corpse");
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn sawblade_skinripper_needs_a_sacrifice_before_the_end_step() {
    cr!("603.4", "608.2h");
    ruling!(
        "Sawblade Skinripper",
        "If you haven't sacrificed any permanents during your turn by the time your end step begins, Sawblade Skinripper's last ability won't trigger at all."
    );
    supported("Sawblade Skinripper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sawblade Skinripper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    crate::r_p216_common::sacrifice(&mut t, P0, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // A sacrifice before the end step: 1 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sawblade Skinripper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_p216_common::sacrifice(&mut t, P0, bears);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn mana_abilities_counting_cards_drawn_are_mana_abilities() {
    cr!("605.1a", "605.3b");
    // "{T}: Add {U} for each card you've drawn this turn. Spend this mana only to cast
    // noncreature spells." counts draws without drawing: it doesn't use the stack.
    supported("Immortus, Master of Eternity");
    let mut t = TestGame::new(2);
    let immortus = t.battlefield(P0, "Immortus, Master of Eternity");
    draw(&mut t, P0, 2);
    assert!(tap_for_mana(&mut t, P0, immortus, "{U}"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}
