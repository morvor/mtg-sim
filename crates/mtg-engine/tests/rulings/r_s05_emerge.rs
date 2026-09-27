//! Rulings batch S05 — emerge (CR 702.119): "You may cast this spell by paying [cost] and
//! sacrificing a creature rather than paying its mana cost", reduced by the sacrificed
//! creature's mana value.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const EMERGE: CastMethod = CastMethod::Keyword(KeywordKind::Emerge);

/// Casts Wretched Gryff (emerge {5}{U}) for its emerge cost sacrificing `sac`, with
/// `generic` Wastes and an Island for mana. Whether it could be cast.
fn emerge_gryff(t: &mut TestGame, sac: ObjectId, generic: usize) -> bool {
    supported("Wretched Gryff");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", generic);
    let gryff = t.hand(P0, "Wretched Gryff");
    t.answer_choose(P0, &[Entity::Object(sac)]);
    let ok = t.cast(P0, gryff).method(EMERGE).try_go().is_ok();
    t.clear_answers();
    ok
}

#[test]
fn a_cast_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3b", "702.119a");
    ruling!(
        "Wretched Gryff",
        "An ability that triggers when a player casts a spell resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    supported("Counterspell");
    // Wretched Gryff: "When you cast this spell, draw a card."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(emerge_gryff(&mut t, bears, 3));
    t.settle();
    assert_eq!(
        stack_items(&t),
        vec![
            "Wretched Gryff".to_string(),
            "ability: When you cast ~, draw a card".to_string()
        ]
    );
    // Countered in response: the trigger still resolves.
    let hand = t.hand_size(P0);
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    let spell = t.g.stack[0];
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Wretched Gryff"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn emerge_is_an_alternative_cost_reduced_by_the_sacrificed_creatures_mana_value() {
    cr!("702.119a", "118.9", "601.2f");
    ruling!(
        "Wretched Gryff",
        "Emerge represents two static abilities that function while the spell with emerge is on the stack. “Emerge [cost]” means “You may cast this spell by paying [cost] and sacrificing a creature rather than paying its mana cost” and “If you chose to pay this spell’s emerge cost, its total cost is reduced by an amount of generic mana equal to the sacrificed creature’s mana value.”"
    );
    // Wretched Gryff ({7}) for {5}{U}, sacrificing Grizzly Bears (mana value 2): {3}{U}.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!emerge_gryff(&mut t, bears, 2));
    assert!(t.on_battlefield(bears));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(emerge_gryff(&mut t, bears, 3));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(untapped_lands(&t, P0), 0);
    // Rather than its mana cost: without the sacrifice, the mana cost is {7}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 6);
    let gryff = t.hand(P0, "Wretched Gryff");
    assert!(!can_cast(&mut t, P0, gryff, CastMethod::Normal));
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, gryff, CastMethod::Normal));
}

#[test]
fn a_sacrificed_creature_with_x_in_its_cost_has_x_0() {
    cr!("702.119a", "202.3e");
    ruling!(
        "Wretched Gryff",
        "If you sacrifice a creature with {X} in its mana cost, that X is 0."
    );
    supported("Walking Ballista");
    // Walking Ballista ({X}{X}) with three counters (cast for X=3) has mana value 0.
    let mut t = TestGame::new(2);
    let ballista = t.battlefield(P0, "Walking Ballista");
    t.g.add_counters(Entity::Object(ballista), counters::PLUS1, 3, None);
    assert!(!emerge_gryff(&mut t, ballista, 4));
    let mut t = TestGame::new(2);
    let ballista = t.battlefield(P0, "Walking Ballista");
    t.g.add_counters(Entity::Object(ballista), counters::PLUS1, 3, None);
    assert!(emerge_gryff(&mut t, ballista, 5));
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn nothing_can_be_done_while_an_emerge_spell_is_being_cast() {
    cr!("601.2", "702.119a");
    ruling!(
        "Wretched Gryff",
        "Once you begin to cast a spell with emerge, no player may take actions until you’re done. Notably, opponents can’t try to remove the creature you wish to sacrifice."
    );
    // P0 casts the Gryff through the priority loop; P1 is never asked anything while the
    // Bears is chosen but not yet sacrificed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let gryff = t.hand(P0, "Wretched Gryff");
    let state = |g: &Game| {
        let bears_out = g
            .battlefield
            .iter()
            .any(|o| g.obj(*o).chars.name.as_str() == "Grizzly Bears");
        (bears_out, g.stack.len())
    };
    let seen = watch(&mut t, P1, |_| true, state);
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: gryff,
            method: EMERGE,
        }),
    );
    t.answer_choose(P0, &[Entity::Object(bears)]);
    // Through the priority loop until the Gryff has resolved.
    assert!(t.g.run_until(1000, |g| g.battlefield.iter().any(|o| g
        .obj(*o)
        .chars
        .name
        .as_str()
        == "Wretched Gryff")));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Wretched Gryff").len(), 1);
    let seen = seen.lock().unwrap();
    // P1 was asked (for priority with the Gryff on the stack), but only once the Bears
    // was gone.
    assert!(seen
        .iter()
        .any(|(bears_out, stack)| !*bears_out && *stack > 0));
    for (bears_out, stack) in seen.iter() {
        assert!(!(*bears_out && *stack > 0), "P1 was asked mid-cast");
    }
}

#[test]
fn only_printed_mana_symbols_count_toward_the_sacrificed_creatures_mana_value() {
    cr!("202.3", "202.3a", "702.119a");
    ruling!(
        "Wretched Gryff",
        "Ignore any alternative costs or additional costs (such as kicker) that were paid as the creature was cast."
    );
    ruling!(
        "Wretched Gryff",
        "If it’s a single-faced card with no mana symbols in its upper right corner (because it’s an animated land, for example), its mana value is 0."
    );
    supported("Kavu Titan");
    supported("Badgermole Cub");
    // Kavu Titan ({1}{G}) cast kicked ({2}{G} more) still has mana value 2: {3}{U} to pay.
    let kicked_titan = |t: &mut TestGame| {
        add_mana(t, P0, mtg_engine::mana::ManaType::G, 5);
        let titan = t.hand(P0, "Kavu Titan");
        t.cast(P0, titan).kicked(true).go();
        t.resolve_all();
        let titan = t.g.current(titan);
        assert_eq!(t.counters(titan, counters::PLUS1), 3);
        assert_eq!(t.g.mana_value_of(titan), 2);
        titan
    };
    let mut t = TestGame::new(2);
    let titan = kicked_titan(&mut t);
    assert!(!emerge_gryff(&mut t, titan, 2));
    let mut t = TestGame::new(2);
    let titan = kicked_titan(&mut t);
    assert!(emerge_gryff(&mut t, titan, 3));
    assert_eq!(untapped_lands(&t, P0), 0);
    // A land animated by earthbend (Badgermole Cub) has mana value 0: no reduction.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Wastes");
    t.answer_targets(P0, &[Entity::Object(land)]);
    enter(&mut t, P0, "Badgermole Cub");
    t.resolve_all();
    assert!(t.obj_now(land).chars.is(CardType::Creature));
    // (Tapped, so it can't also pay for the spell.)
    t.g.tap(land);
    assert!(!emerge_gryff(&mut t, land, 4));
    let mut t2 = TestGame::new(2);
    let land = t2.battlefield(P0, "Wastes");
    t2.answer_targets(P0, &[Entity::Object(land)]);
    enter(&mut t2, P0, "Badgermole Cub");
    t2.resolve_all();
    t2.g.tap(land);
    assert!(emerge_gryff(&mut t2, land, 5));
    assert_eq!(untapped_lands(&t2, P0), 0);
}

#[test]
fn a_transformed_permanent_has_its_front_faces_mana_value_and_a_copy_of_it_has_0() {
    cr!("202.3b", "202.3c", "712.8e", "702.119a");
    ruling!(
        "Wretched Gryff",
        "The mana value of the back face of a double-faced card is the mana value of its front face. The mana value of a melded permanent is the sum of the mana values of its front faces. A creature that’s a copy of either has a mana value of 0."
    );
    supported("Ulvenwald Captive");
    supported("Clone");
    // Ulvenwald Captive ({1}{G}) transformed into Ulvenwald Abomination: mana value 2.
    let put_transformed = |t: &mut TestGame| {
        let card = t.hand(P0, "Ulvenwald Captive");
        run_from(
            t,
            P0,
            None,
            Effect::Move {
                what: Sel::Target(0),
                to: Destination {
                    transformed: true,
                    ..Destination::battlefield()
                },
            },
            &[Entity::Object(card)],
        );
        let id = t.g.current(card);
        assert_eq!(t.obj(id).chars.name.as_str(), "Ulvenwald Abomination");
        id
    };
    let mut t = TestGame::new(2);
    let abomination = put_transformed(&mut t);
    assert_eq!(t.g.mana_value_of(abomination), 2);
    assert!(emerge_gryff(&mut t, abomination, 3));
    assert_eq!(untapped_lands(&t, P0), 0);
    // A Clone copying it has mana value 0.
    let mut t = TestGame::new(2);
    let abomination = put_transformed(&mut t);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(abomination)]);
    let clone = enter(&mut t, P0, "Clone");
    t.resolve_all();
    let clone = t.g.current(clone);
    assert_eq!(t.obj(clone).chars.name.as_str(), "Ulvenwald Abomination");
    assert_eq!(t.g.mana_value_of(clone), 0);
    assert!(!emerge_gryff(&mut t, clone, 4));
    assert!(t.on_battlefield(clone));
    // Chittering Host, melded from Graf Rats ({1}{B}) and Midnight Scavengers ({4}{B}), has
    // mana value 7: sacrificing it reduces {5}{U} to {U}.
    supported("Graf Rats");
    supported("Midnight Scavengers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Graf Rats");
    t.battlefield(P0, "Midnight Scavengers");
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.advance_to_step(mtg_engine::turn::Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    let host = t.named_on_battlefield("Chittering Host")[0];
    assert_eq!(t.g.mana_value_of(host), 7);
    t.set_step(P0, mtg_engine::turn::Step::PostcombatMain);
    assert!(emerge_gryff(&mut t, host, 0));
    assert!(!t.on_battlefield(host));
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn an_emerge_spell_can_be_kicked_but_has_no_other_alternative_cost() {
    cr!("702.119a", "118.9", "118.8", "702.33a");
    ruling!(
        "Wretched Gryff",
        "If you cast a spell for another cost “rather than paying its mana cost,” such as an emerge cost, you can’t choose to cast it for any other alternative costs. You can, however, pay additional costs, such as kicker costs."
    );
    // A creature with emerge and kicker (compiled from its text): casting it for its emerge
    // cost, kicker can be paid on top.
    let def = custom_card(
        "Kicked Emerger",
        "Creature — Eldrazi",
        "{7}",
        Some((3, 3)),
        "Emerge {5}{U}\nKicker {2}\nWhen you cast this spell, if it was kicked, draw a card.",
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 5);
    let card = t.custom(P0, def, Zone::Hand(P0));
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.cast(P0, card).method(EMERGE).kicked(true).go();
    // {5}{U} - 2 + {2}: all six lands.
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Only one way to pay rather than the mana cost was used: no other method was asked.
    assert_eq!(
        asked_of_since(&t, P0, from, |d| matches!(
            d,
            Decision::ChooseCastingMethod { .. }
        )),
        0
    );
    t.resolve_all();
    // It left the hand, and its "if it was kicked" cast trigger drew a card.
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.named_on_battlefield("Kicked Emerger").len(), 1);
}

#[test]
fn a_mandatory_additional_cost_is_paid_when_casting_for_the_emerge_cost() {
    cr!("702.119a", "118.8", "601.2b");
    ruling!(
        "Wretched Gryff",
        "If the spell has any mandatory additional costs, those must be paid to cast it."
    );
    // A creature with emerge and "As an additional cost to cast this spell, discard a
    // card" (compiled from its text).
    let def = custom_card(
        "Hungry Emerger",
        "Creature — Eldrazi",
        "{7}",
        Some((3, 3)),
        "Emerge {5}{U}\nAs an additional cost to cast this spell, discard a card.",
    );
    // With no other card in hand, it can't be cast for its emerge cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.custom(P0, def.clone(), Zone::Hand(P0));
    assert!(!can_cast(&mut t, P0, card, EMERGE));
    // With one, it's discarded as the Bears is sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.custom(P0, def, Zone::Hand(P0));
    t.hand(P0, "Hill Giant");
    assert!(can_cast(&mut t, P0, card, EMERGE));
    t.cast(P0, card).method(EMERGE).go();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(untapped_lands(&t, P0), 0);
}
