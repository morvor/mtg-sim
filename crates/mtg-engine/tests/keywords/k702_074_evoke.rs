//! CR 702.74 Evoke.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::can_cast;
use crate::common_k702_052_066::stack_triggers;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

const EVOKE: CastMethod = CastMethod::Keyword(KeywordKind::Evoke);

/// Casts Mulldrifter for its evoke cost and resolves the spell (its two enters triggers
/// are then on the stack, ordered with `order`).
fn evoke_mulldrifter(t: &mut TestGame, order: Vec<usize>) -> ObjectId {
    t.lands(P0, "Island", 3);
    let md = t.hand(P0, "Mulldrifter");
    t.answer(P0, DecisionKind::Order, Answer::Indices(order));
    let spell = t.cast(P0, md).method(EVOKE).go();
    // The mana value is still that of its mana cost ({4}{U}).
    assert_eq!(t.g.obj(spell).chars.mana_value(), 5);
    t.resolve();
    spell
}

#[test]
fn an_evoked_creature_is_sacrificed_when_it_enters() {
    cr!("702.74", "702.74a");
    ruling!(
        "Mulldrifter",
        "The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    assert_supported("Mulldrifter");
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    evoke_mulldrifter(&mut t, vec![0, 1]);
    // Its own enters ability and the evoke sacrifice ability both trigger.
    assert_eq!(stack_triggers(&t, "Evoke").len(), 1);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.named_on_battlefield("Mulldrifter").is_empty());
    assert!(t.in_graveyard(P0, "Mulldrifter"));
    assert_eq!(t.hand_size(P0), hand + 2);
    // Paid with three lands, not five.
    assert_eq!(t.g.permanents().filter(|o| o.tapped).count(), 3);
}

#[test]
fn the_controller_orders_the_evoke_trigger_and_the_creatures_own_trigger() {
    cr!("702.74a");
    ruling!(
        "Shriekmaw",
        "own triggered ability resolve before the evoke triggered ability"
    );
    ruling!(
        "Solitude",
        "own triggered ability resolve before the evoke triggered ability"
    );
    ruling!(
        "Mulldrifter",
        "If you pay the evoke cost, you can have Mulldrifter's own triggered ability resolve before the evoke triggered ability."
    );
    let mut tops = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let hand = t.hand_size(P0);
        evoke_mulldrifter(&mut t, order);
        let evoke_on_top = stack_triggers(&t, "Evoke")
            .first()
            .is_some_and(|(id, _)| t.g.stack.last() == Some(id));
        tops.push(evoke_on_top);
        t.resolve();
        if evoke_on_top {
            // Sacrificed first; the draw ability still resolves.
            assert!(t.named_on_battlefield("Mulldrifter").is_empty());
            assert_eq!(t.hand_size(P0), hand);
        } else {
            // It draws two cards while still on the battlefield, then it's sacrificed.
            assert_eq!(t.named_on_battlefield("Mulldrifter").len(), 1);
            assert_eq!(t.hand_size(P0), hand + 2);
        }
        t.resolve_all();
        assert!(t.named_on_battlefield("Mulldrifter").is_empty());
        assert_eq!(t.hand_size(P0), hand + 2);
    }
    // Both orders were possible.
    assert_eq!(tops.len(), 2);
    assert_ne!(tops[0], tops[1]);
}

#[test]
fn a_creature_cast_for_its_mana_cost_isnt_sacrificed() {
    cr!("702.74a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let md = t.hand(P0, "Mulldrifter");
    t.cast(P0, md).go();
    t.resolve();
    assert!(stack_triggers(&t, "Evoke").is_empty());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mulldrifter").len(), 1);
}

#[test]
fn a_creature_put_onto_the_battlefield_without_being_cast_isnt_sacrificed() {
    cr!("702.74a");
    let mut t = TestGame::new(2);
    t.enter(P0, "Mulldrifter");
    t.settle();
    assert!(stack_triggers(&t, "Evoke").is_empty());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mulldrifter").len(), 1);
}

#[test]
fn a_blinked_evoked_creature_stays_on_the_battlefield() {
    cr!("702.74a", "400.7");
    assert_supported("Ephemerate");
    let mut t = TestGame::new(2);
    evoke_mulldrifter(&mut t, vec![0, 1]);
    // In response to the evoke trigger, the creature is exiled and returned: it's a new
    // object, which wasn't evoked.
    let md = t.named_on_battlefield("Mulldrifter")[0];
    t.lands(P0, "Plains", 1);
    let eph = t.hand(P0, "Ephemerate");
    t.cast(P0, eph).target(md).go();
    t.resolve();
    let back = t.named_on_battlefield("Mulldrifter");
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], md);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mulldrifter").len(), 1);
}

#[test]
fn an_evoke_cost_can_be_a_non_mana_cost() {
    cr!("702.74a");
    assert_supported("Solitude");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let solitude = t.hand(P0, "Solitude");
    let white = t.hand(P0, "Savannah Lions");
    // "Evoke—Exile a white card from your hand."
    t.answer_choose(P0, &[Entity::Object(white)]);
    t.cast(P0, solitude).method(EVOKE).go();
    assert!(t.in_exile("Savannah Lions"));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 22);
    assert!(t.in_graveyard(P0, "Solitude"));
}

#[test]
fn evoke_works_only_from_a_zone_the_card_could_be_cast_from() {
    cr!("702.74a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let in_gy = t.graveyard(P0, "Mulldrifter");
    assert!(!can_cast(&mut t, P0, in_gy, EVOKE));
    let in_hand = t.hand(P0, "Mulldrifter");
    assert!(can_cast(&mut t, P0, in_hand, EVOKE));
}

#[test]
fn cost_increases_apply_to_the_evoke_cost() {
    cr!("702.74a", "601.2f");
    ruling!(
        "Mulldrifter",
        "start with the mana cost or alternative cost you're paying (such as an evoke cost), add any cost increases, then apply any cost reductions"
    );
    assert_supported("Grand Arbiter Augustin IV");
    let mut t = TestGame::new(2);
    // "Spells your opponents cast cost {1} more to cast."
    t.battlefield(P1, "Grand Arbiter Augustin IV");
    t.lands(P0, "Island", 3);
    let md = t.hand(P0, "Mulldrifter");
    assert!(!can_cast(&mut t, P0, md, EVOKE));
    t.lands(P0, "Island", 1);
    assert!(can_cast(&mut t, P0, md, EVOKE));
    t.cast(P0, md).method(EVOKE).go();
    assert_eq!(t.g.permanents().filter(|o| o.tapped).count(), 4);
}
