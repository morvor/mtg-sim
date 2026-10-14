//! Rulings batch S03 — constellation (an ability word): "Constellation — Whenever [this
//! creature or another] enchantment you control enters, ..."

use crate::r_s01_common::*;
use crate::r_s03_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

const BESTOW: CastMethod = CastMethod::Keyword(KeywordKind::Bestow);

/// Casts the real card `name` from `p`'s hand (with the mana for it) with `targets`, and
/// resolves everything.
fn cast_and_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    let c = in_hand_with_mana(t, p, name);
    t.cast(p, c).targets(targets).go();
    t.resolve_all();
}

#[test]
fn constellation_triggers_for_any_enchantment_entering_under_your_control() {
    cr!("207.2c", "603.2", "603.6a");
    ruling!(
        "Eidolon of Blossoms",
        "A constellation ability triggers whenever an enchantment enters the battlefield under your control for any reason. Enchantments with other card types, such as enchantment creatures, will also cause constellation abilities to trigger."
    );
    supported("Eidolon of Blossoms");
    // Eidolon of Blossoms: "Constellation — Whenever this creature or another enchantment
    // you control enters, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eidolon of Blossoms");
    let hand = t.hand_size(P0);
    // A cast enchantment.
    cast_and_resolve(&mut t, P0, "Glorious Anthem", &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    // An enchantment put onto the battlefield by an effect.
    t.enter(P0, "Glorious Anthem");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // An enchantment creature.
    cast_and_resolve(&mut t, P0, "Nyxborn Rollicker", &[]);
    assert_eq!(t.hand_size(P0), hand + 3);
    // An enchantment entering under an opponent's control doesn't count, nor does a
    // creature that isn't an enchantment.
    t.enter(P1, "Glorious Anthem");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

/// Orders triggered abilities so Grim Guardian's is at the bottom of the stack and Eidolon
/// of Blossoms's on top.
fn guardian_first_eidolon_last(_g: &Game, d: &Decision) -> Option<Answer> {
    let Decision::Order { items, .. } = d else {
        return None;
    };
    let rank = |s: &String| {
        if s.starts_with("Grim Guardian") {
            0
        } else if s.starts_with("Eidolon of Blossoms") {
            2
        } else {
            1
        }
    };
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by_key(|i| rank(&items[*i]));
    Some(Answer::Indices(order))
}

#[test]
fn each_constellation_ability_triggers_and_they_go_on_the_stack_in_any_order() {
    cr!("603.3b", "405.2");
    ruling!(
        "Eidolon of Blossoms",
        "When an enchantment enters the battlefield under your control, each constellation ability of permanents you control will trigger. You can put these abilities on the stack in any order. The last ability you put on the stack will be the first one that resolves."
    );
    supported("Underworld Coinsmith");
    supported("Grim Guardian");
    // Grim Guardian: "... each opponent loses 1 life." Underworld Coinsmith: "... you gain
    // 1 life." Eidolon of Blossoms: "... draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eidolon of Blossoms");
    t.battlefield(P0, "Underworld Coinsmith");
    t.battlefield(P0, "Grim Guardian");
    respond(&mut t, P0, guardian_first_eidolon_last);
    let from = t.asked().len();
    t.enter(P0, "Glorious Anthem");
    t.settle();
    assert_eq!(t.stack_len(), 3);
    let orders: Vec<_> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Order { .. }))
        .cloned()
        .collect();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].0, P0);
    let before = t.g.turn_events.len();
    t.resolve_all();
    let order: Vec<&str> = t.g.turn_events[before..]
        .iter()
        .filter_map(|e| match e {
            Event::Drew { .. } => Some("draw"),
            Event::LifeGained { .. } => Some("gain"),
            Event::LifeLost { .. } => Some("lose"),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec!["draw", "gain", "lose"]);
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn an_aura_with_an_illegal_target_doesnt_enter_but_a_bestowed_one_does() {
    cr!("608.3b", "702.103e", "303.4a");
    ruling!(
        "Eidolon of Blossoms",
        "An Aura spell without bestow that has an illegal target when it tries to resolve won't resolve and will be put into its owner's graveyard. It won't enter the battlefield and constellation abilities won't trigger. An Aura spell with bestow won't be countered this way. It will revert to being an enchantment creature and resolve, entering the battlefield and triggering constellation abilities."
    );
    supported("Pacifism");
    supported("Nyxborn Rollicker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eidolon of Blossoms");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Pacifism's target is destroyed in response: it goes to the graveyard, no trigger.
    let pacifism = in_hand_with_mana(&mut t, P0, "Pacifism");
    t.cast(P0, pacifism).target(bears).go();
    let hand_after_cast = t.hand_size(P0);
    let b = t.g.current(bears);
    t.g.destroy(b, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Pacifism"));
    assert_eq!(t.hand_size(P0), hand_after_cast);
    // Nyxborn Rollicker cast bestowed: its target is destroyed in response; it resolves
    // as an enchantment creature, enters and triggers the Eidolon.
    let giant = t.battlefield(P1, "Hill Giant");
    let rollicker = t.hand(P0, "Nyxborn Rollicker");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, rollicker).method(BESTOW).target(giant).go();
    let hand = t.hand_size(P0);
    let g = t.g.current(giant);
    t.g.destroy(g, None);
    t.resolve_all();
    let r = t.named_on_battlefield("Nyxborn Rollicker");
    assert_eq!(r.len(), 1);
    assert!(t.obj(r[0]).is_creature());
    assert_eq!(t.obj(r[0]).attached_to, None);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn an_aura_spell_with_an_illegal_target_doesnt_trigger_constellation() {
    cr!("608.3b", "303.4a", "701.6a");
    ruling!(
        "Nexus Wardens",
        "An Aura spell that has an illegal target when it tries to resolve doesn’t resolve and is instead put into its owner’s graveyard. It doesn’t enter the battlefield, so constellation abilities don’t trigger."
    );
    ruling!(
        "Setessan Champion",
        "An Aura spell that has an illegal target when it tries to resolve doesn't resolve and is instead put into its owner's graveyard. It doesn't enter the battlefield, so constellation abilities don't trigger."
    );
    supported("Nexus Wardens");
    supported("Setessan Champion");
    // Nexus Wardens: "Constellation — Whenever an enchantment you control enters, you
    // gain 2 life." Setessan Champion: "... put a +1/+1 counter on this creature and draw
    // a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nexus Wardens");
    let champion = t.battlefield(P0, "Setessan Champion");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rancor = in_hand_with_mana(&mut t, P0, "Rancor");
    t.cast(P0, rancor).target(bears).go();
    let hand = t.hand_size(P0);
    // The target is no longer a creature on the battlefield: it's returned to its
    // owner's hand.
    let b = t.g.current(bears);
    t.g.move_object(
        b,
        mtg_engine::object::Zone::Hand(P1),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Rancor"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(champion, "+1/+1"), 0);
    assert_eq!(t.hand_size(P0), hand);
    // With a legal target, both trigger.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rancor = in_hand_with_mana(&mut t, P0, "Rancor");
    t.cast(P0, rancor).target(bears).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.counters(champion, "+1/+1"), 1);
}
