//! Rulings batch P159 — total costs (CR 601.2f: mana or alternative cost, plus increases,
//! minus reductions; mana value from the mana cost only, CR 202.3), cost reductions of
//! activated abilities (CR 602.2b), costs paid as part of activating (CR 602.2), dividing
//! damage (CR 601.2d), and copies of spells (CR 707.10: not cast, additional costs and
//! choices copied).

use crate::r_p044_common::cast_only;
use crate::r_p046_common::{fill_pool, pool, tap_lands};
use crate::r_s01_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s08_common::{legal_cast_methods, mana_value};
use crate::r_s25_common::{spell_copies, targets_of};
use crate::r_s29_common::divide;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The amount of mana `f` spends out of a pool of plenty of every type (`p`'s lands
/// tapped first), and whether it succeeded.
fn spent(t: &mut TestGame, p: PlayerId, f: impl FnOnce(&mut TestGame) -> bool) -> Option<usize> {
    tap_lands(t, p);
    fill_pool(t, p, 15);
    let before = pool(t, p);
    let ok = f(t);
    ok.then(|| before - pool(t, p))
}

#[test]
fn total_cost_adds_increases_then_reductions_and_mana_value_ignores_it() {
    cr!("601.2f", "202.3", "702.33d", "702.34a");
    ruling!(
        "The Destined Warrior",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions"
    );
    ruling!(
        "Rowan, Scholar of Sparks // Will, Scholar of Frost",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you’re paying, add any cost increases, then apply any cost reductions"
    );
    supported("The Destined Warrior");
    supported("Rowan, Scholar of Sparks // Will, Scholar of Frost");
    // The Destined Warrior and a kicked Sea Gate Stormcaller (a Wizard): {1}{U} + {4}{U}
    // - {1} = 6 mana; its mana value is still 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Destined Warrior");
    let s = t.hand(P0, "Sea Gate Stormcaller");
    let paid = spent(&mut t, P0, |t| t.cast(P0, s).kicked(true).try_go().is_ok());
    assert_eq!(paid, Some(6));
    assert_eq!(mana_value(&t, t.g.stack[0]), 2);
    // Rowan and Think Twice cast with flashback: {2}{U} - {1} = 2 mana; mana value 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rowan, Scholar of Sparks // Will, Scholar of Frost");
    let tt = t.graveyard(P0, "Think Twice");
    fill_pool(&mut t, P0, 15);
    let m = legal_cast_methods(&mut t, P0, tt)
        .into_iter()
        .find(|m| *m != CastMethod::Normal)
        .expect("flashback");
    let paid = spent(&mut t, P0, |t| t.cast(P0, tt).method(m).try_go().is_ok());
    assert_eq!(paid, Some(2));
    assert_eq!(mana_value(&t, t.g.stack[0]), 2);
}

#[test]
fn eowyn_reduces_only_generic_mana_of_equip_costs() {
    cr!("601.2f", "602.2b", "702.6a");
    ruling!(
        "Éowyn, Lady of Rohan",
        "Éowyn, Lady of Rohan's last ability reduces only the amount of generic mana in equip abilities."
    );
    supported("Éowyn, Lady of Rohan");
    for (equipment, cost) in [("Bonesplitter", 0), ("Greatsword of Tyr", 1)] {
        let mut t = TestGame::new(2);
        let eowyn = t.battlefield(P0, "Éowyn, Lady of Rohan");
        let e = t.battlefield(P0, equipment);
        t.answer_targets(P0, &[obj(eowyn)]);
        let paid = spent(&mut t, P0, |t| {
            activate_containing(t, P0, e, "Equip").is_ok()
        });
        assert_eq!(paid, Some(cost), "{equipment}");
        t.resolve_all();
        assert_eq!(crate::r_s06_common::attached_to(&t, e), Some(obj(eowyn)));
    }
}

#[test]
fn agatha_reduces_only_activated_abilities_of_creatures_on_the_battlefield() {
    cr!("602.2b", "601.2f", "702.29a");
    ruling!(
        "Agatha of the Vile Cauldron",
        "Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") are unaffected by the cost reduction ability of Agatha of the Vile Cauldron."
    );
    ruling!(
        "Agatha of the Vile Cauldron",
        "Agatha of the Vile Cauldron's first ability affects only abilities of creatures you control on the battlefield."
    );
    supported("Agatha of the Vile Cauldron");
    // Agatha (power 1): Elder of Laurels' {3}{G} costs 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Agatha of the Vile Cauldron");
    let elder = t.battlefield(P0, "Elder of Laurels");
    t.answer_targets(P0, &[obj(elder)]);
    let paid = spent(&mut t, P0, |t| {
        activate_containing(t, P0, elder, "+X/+X").is_ok()
    });
    assert_eq!(paid, Some(3));
    // Unassuming Sage's "you may pay {2}" (a triggered ability): still {2}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Agatha of the Vile Cauldron");
    tap_lands(&mut t, P0);
    fill_pool(&mut t, P0, 15);
    let before = pool(&t, P0);
    t.answer_yes(P0, true);
    t.enter(P0, "Unassuming Sage");
    t.resolve_all();
    assert_eq!(before - pool(&t, P0), 2);
    // Cycling a creature card in hand (Yoked Plowbeast, cycling {2}): still {2}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Agatha of the Vile Cauldron");
    let beast = t.hand(P0, "Yoked Plowbeast");
    let paid = spent(&mut t, P0, |t| {
        activate_containing(t, P0, beast, "Cycling").is_ok()
    });
    assert_eq!(paid, Some(2));
}

#[test]
fn spending_mana_as_though_any_color_still_needs_the_colored_part() {
    cr!("609.4b", "601.2f");
    ruling!(
        "Fan Favorite",
        "If an effect allows a player to cast a spell spending mana \"as though it were mana of any color\" or \"of any type,\" that player must still pay for the colored mana in that spell's total cost."
    );
    supported("Fan Favorite");
    supported("Celestial Dawn");
    // Under Celestial Dawn ("You may spend white mana as though it were mana of any color.
    // You may spend other mana only as though it were colorless mana."): Fan Favorite's
    // {B} can be paid with white mana, but not with black mana, which can be spent only as
    // colorless. (Mana from the pool: Celestial Dawn makes lands Plains.)
    for (color, castable) in [(ManaType::W, true), (ManaType::B, false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Celestial Dawn");
        t.g.players[0].mana_pool.add_type(ManaType::C, 3);
        t.g.players[0].mana_pool.add_type(color, 1);
        let f = t.hand(P0, "Fan Favorite");
        assert_eq!(t.cast(P0, f).try_go().is_ok(), castable, "{color:?}");
    }
}

#[test]
fn hazel_can_tap_summoning_sick_creature_tokens() {
    cr!("302.6", "118.3", "605.1a");
    ruling!(
        "Hazel of the Rootbloom",
        "You can tap any untapped tokens you control, including creature tokens you control that you haven't controlled continuously since the beginning of your most recent turn"
    );
    supported("Hazel of the Rootbloom");
    let mut t = TestGame::new(2);
    let hazel = t.battlefield(P0, "Hazel of the Rootbloom");
    crate::r_p044_common::cast_spell(&mut t, P0, "Raise the Alarm", &[]);
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    assert!(toks.iter().all(|x| t.obj_now(*x).summoning_sick));
    tap_lands(&mut t, P0);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &toks.iter().map(|x| obj(*x)).collect::<Vec<_>>());
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, hazel, "Pay 2 life"));
    assert_eq!(pool(&t, P0), 2);
    assert!(toks.iter().all(|x| t.obj_now(*x).tapped));
    assert_eq!(t.life(P0), 18);
}

#[test]
fn stormscale_anarch_with_too_little_mana_discards_nothing() {
    cr!("602.2b", "601.2g", "601.2h");
    ruling!(
        "Stormscale Anarch",
        "You can't discard a card at random, see what it is, and then decide whether or not to pay the mana."
    );
    supported("Stormscale Anarch");
    // Not enough mana: the activation is illegal and the hand is untouched.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Stormscale Anarch");
    t.lands(P0, "Mountain", 2);
    t.hand(P0, "Lightning Helix");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(activate_containing(&mut t, P0, a, "Discard").is_err());
    assert_eq!(t.hand_size(P0), 1);
    // Enough mana: the (multicolored) card is discarded and the ability goes on: 4 damage.
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, a, "Discard").expect("activate");
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn costs_paid_while_activating_cant_be_responded_to() {
    cr!("602.2b", "118.3", "602.2");
    ruling!(
        "Scarred Vinebreeder",
        "If you activate the ability, the Elf card is exiled as a cost. This action can't be responded to."
    );
    ruling!(
        "Battlefield Scrounger",
        "Check the number of cards in your graveyard before activating the ability. If activating the ability causes that number to fall below seven, the ability still resolves as normal."
    );
    supported("Scarred Vinebreeder");
    supported("Battlefield Scrounger");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Scarred Vinebreeder");
    let elf = t.graveyard(P0, "Llanowar Elves");
    t.lands(P0, "Swamp", 3);
    t.answer_choose(P0, &[obj(elf)]);
    activate_containing(&mut t, P0, v, "Exile an Elf").expect("activate");
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.zone(elf), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.pt(v), (4, 4));
    // Battlefield Scrounger with seven cards: three go to the library as the cost, and the
    // ability resolves with four left.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Battlefield Scrounger");
    let gy: Vec<ObjectId> = (0..7).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
    t.answer_choose(P0, &gy[..3].iter().map(|x| obj(*x)).collect::<Vec<_>>());
    activate_containing(&mut t, P0, s, "bottom").expect("activate");
    assert_eq!(t.graveyard_size(P0), 4);
    t.resolve_all();
    assert_eq!(t.pt(s), (6, 6));
}

#[test]
fn emiels_payment_is_made_once_as_it_resolves() {
    cr!("603.5", "608.2c", "117.1");
    ruling!(
        "Emiel the Blessed",
        "While resolving Emiel's triggered ability, you can't pay {G/W} more than once to put more counters on the creature."
    );
    ruling!(
        "Emiel the Blessed",
        "You choose whether to pay for Emiel's triggered ability while it's resolving."
    );
    supported("Emiel the Blessed");
    for (creature, n) in [("Grizzly Bears", 1), ("Regal Unicorn", 2)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Emiel the Blessed");
        t.lands(P0, "Plains", 3);
        let from = t.asked().len();
        let c = crate::r_s05_common::enter(&mut t, P0, creature);
        assert_eq!(t.stack_len(), 1);
        // Nothing has been asked yet: the payment is chosen as it resolves.
        let yes_no = |t: &TestGame| {
            asked_since(t, from)
                .iter()
                .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
                .count()
        };
        assert_eq!(yes_no(&t), 0);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(yes_no(&t), 1, "{creature}");
        assert_eq!(t.counters(c, "+1/+1"), n, "{creature}");
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 1, "{creature}");
    }
}

#[test]
fn entish_restoration_sacrifices_on_resolution() {
    cr!("608.2c", "701.21a", "701.6a");
    ruling!(
        "Entish Restoration",
        "You sacrifice the lands as part of the resolution of Entish Restoration. It isn't an additional cost."
    );
    supported("Entish Restoration");
    // Countered: no land is sacrificed.
    let mut t = TestGame::new(2);
    let s = cast_only(&mut t, P0, "Entish Restoration", &[]);
    let p0_lands = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is(CardType::Land))
            .count()
    };
    let lands = p0_lands(&t);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Cancel");
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(s).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Entish Restoration"));
    assert_eq!(p0_lands(&t), lands);
    // Resolved: one land is sacrificed as it resolves and two basics are found.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Forest", "Forest"]);
    cast_only(&mut t, P0, "Entish Restoration", &[]);
    let mine = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is(CardType::Land))
            .count()
    };
    let before = mine(&t);
    t.resolve_all();
    assert_eq!(mine(&t), before - 1 + 2);
}

#[test]
fn targets_and_divisions_are_chosen_while_casting_or_activating() {
    cr!("601.2c", "601.2d", "602.2b");
    ruling!(
        "Avacyn's Judgment",
        "You announce how the damage will be divided as part of casting Avacyn's Judgment. Each chosen target must receive at least 1 damage."
    );
    ruling!(
        "Throw from the Saddle",
        "You can’t cast Throw from the Saddle unless you choose a creature you control and a creature you don’t control as targets."
    );
    ruling!(
        "Once and Future",
        "You choose which card will be returned to your hand and which will be put on top of your library as you choose those cards as targets"
    );
    ruling!(
        "Goblin Charbelcher",
        "You must choose a target for Goblin Charbelcher's ability as you activate it, before you reveal any cards."
    );
    for c in [
        "Avacyn's Judgment",
        "Throw from the Saddle",
        "Once and Future",
        "Goblin Charbelcher",
    ] {
        supported(c);
    }
    // Avacyn's Judgment: the division is asked while casting, at least 1 each.
    let mut t = TestGame::new(3);
    let from = t.asked().len();
    divide(&mut t, P0, &[1, 1]);
    let j = t.hand(P0, "Avacyn's Judgment");
    give_mana_for(&mut t, P0, "Avacyn's Judgment");
    t.cast(P0, j)
        .targets(&[Entity::Player(P1), Entity::Player(P2)])
        .go();
    let divs: Vec<u32> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide { min_each, .. } => Some(min_each),
            _ => None,
        })
        .collect();
    assert_eq!(divs, vec![1]);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (19, 19));

    // Throw from the Saddle: no creature P0 doesn't control, no casting.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Throw from the Saddle");
    let th = t.hand(P0, "Throw from the Saddle");
    t.answer_targets(P0, &[obj(mine)]);
    assert!(t.cast(P0, th).try_go().is_err());
    assert!(t.in_hand(P0, "Throw from the Saddle"));

    // Once and Future: the first target goes to hand, the second on top of the library.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(b)]);
    t.answer_targets(P0, &[obj(a)]);
    cast_only(&mut t, P0, "Once and Future", &[]);
    t.resolve_all();
    assert_eq!(t.zone(b), Zone::Hand(P0));
    assert_eq!(t.zone(a), Zone::Library(P0));
    assert_eq!(t.g.players[0].library.last().copied(), Some(t.g.current(a)));

    // Goblin Charbelcher: the target is chosen on activation; nothing is revealed yet.
    let mut t = TestGame::new(2);
    let gc = t.battlefield(P0, "Goblin Charbelcher");
    let top = stack_library(&mut t, P0, &["Grizzly Bears", "Mountain"]);
    t.lands(P0, "Wastes", 3);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, gc, "Reveal").expect("activate");
    assert_eq!(targets_of(&t, t.g.stack[0]), vec![Entity::Player(P1)]);
    // Nothing has been revealed or moved yet: the library is as it was.
    let lib = t.g.players[0].library.clone();
    assert_eq!(lib[lib.len() - 2..].to_vec(), {
        let mut v = top.clone();
        v.reverse();
        v
    });
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    // One nonland card, then a Mountain: double damage.
    assert_eq!(t.life(P1), 18);
}

// --- Copies of spells ------------------------------------------------------------------

/// P0 casts Shock at P1, then copies it with `copier`: Increasing Vengeance (from hand or,
/// with `flashback`, from the graveyard) or Tomb of Horrors Adventurer (Shock being the
/// second spell). Young Pyromancer watches the casts.
fn copy_a_shock(copier: &str, flashback: bool) -> TestGame {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let shock = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 7);
    if copier == "Tomb of Horrors Adventurer" {
        t.battlefield(P0, copier);
        let opt = t.hand(P0, "Opt");
        t.lands(P0, "Island", 1);
        t.cast(P0, opt).go();
        t.resolve_all();
        let s = t.cast(P0, shock).target(Entity::Player(P1)).go();
        let _ = s;
    } else {
        let s = t.cast(P0, shock).target(Entity::Player(P1)).go();
        let v = if flashback {
            t.graveyard(P0, copier)
        } else {
            t.hand(P0, copier)
        };
        t.settle();
        let m = if flashback {
            legal_cast_methods(&mut t, P0, v)
                .into_iter()
                .find(|m| *m != CastMethod::Normal)
                .expect("flashback")
        } else {
            CastMethod::Normal
        };
        t.cast(P0, v).method(m).target(s).go();
    }
    t.settle();
    t
}

#[test]
fn copies_are_created_not_cast() {
    cr!("707.10", "707.10c", "603.2");
    ruling!(
        "Increasing Vengeance",
        "When Increasing Vengeance resolves, it creates one or two copies of a spell. You control each of the copies. Those copies are created on the stack, so they're not “cast.”"
    );
    ruling!(
        "Tomb of Horrors Adventurer",
        "When the triggered ability of Tomb of Horrors Adventurer resolves, it creates one or two copies of a spell. You control each of the copies. Those copies are created on the stack, so they're not “cast.”"
    );
    supported("Increasing Vengeance");
    supported("Tomb of Horrors Adventurer");
    // (copier, flashback, copies, spells cast)
    for (copier, flashback, copies, cast) in [
        ("Increasing Vengeance", false, 1, 2),
        ("Increasing Vengeance", true, 2, 2),
        ("Tomb of Horrors Adventurer", false, 1, 2),
    ] {
        let mut t = copy_a_shock(copier, flashback);
        let mut most = 0;
        let mut guard = 0;
        while !t.g.stack.is_empty() && guard < 50 {
            most = most.max(spell_copies(&t).len());
            for c in spell_copies(&t) {
                assert_eq!(t.g.obj(c).controller, P0);
            }
            t.g.resolve_top();
            t.settle();
            guard += 1;
        }
        assert_eq!(most, copies, "{copier} {flashback}");
        // Young Pyromancer: one token per spell cast; none for the copies.
        let goblins = tokens(&t, P0).len();
        assert_eq!(goblins, cast, "{copier} {flashback}");
        assert_eq!(t.life(P1), 20 - 2 * (1 + copies as i32), "{copier} {flashback}");
    }
}

#[test]
fn copies_copy_the_additional_costs_paid() {
    cr!("707.10", "601.2b", "702.33d");
    ruling!(
        "Increasing Vengeance",
        "You can't choose to pay any additional costs for the copies. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copies too. For example, if you sacrifice a 3/3 creature to cast Fling and then copy it with Increasing Vengeance"
    );
    ruling!(
        "Tomb of Horrors Adventurer",
        "You can't choose to pay any additional costs for the copies. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copies too. For example, if you sacrifice a 3/3 creature to cast Fling and then copy it with Tomb of Horrors Adventurer"
    );
    ruling!(
        "Sea Gate Stormcaller",
        "Notably, if the spell you copy was kicked, the copy or copies will also be kicked."
    );
    supported("Sea Gate Stormcaller");
    // Fling sacrificing a 3/3, copied: 3 damage each.
    for copier in ["Increasing Vengeance", "Tomb of Horrors Adventurer"] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        t.lands(P0, "Mountain", 4);
        let fling = t.hand(P0, "Fling");
        if copier == "Tomb of Horrors Adventurer" {
            t.battlefield(P0, copier);
            t.lands(P0, "Island", 1);
            let opt = t.hand(P0, "Opt");
            t.cast(P0, opt).go();
            t.resolve_all();
        }
        t.answer_choose(P0, &[obj(giant)]);
        let f = t.cast(P0, fling).target(Entity::Player(P1)).go();
        assert!(!t.on_battlefield(giant));
        if copier == "Increasing Vengeance" {
            let v = t.hand(P0, copier);
            t.cast(P0, v).target(f).go();
        }
        t.resolve_all();
        assert_eq!(t.life(P1), 14, "{copier}");
    }
    // Sea Gate Stormcaller copies a kicked Burst Lightning: 4 damage each.
    let mut t = TestGame::new(2);
    t.enter(P0, "Sea Gate Stormcaller");
    t.resolve_all();
    t.lands(P0, "Mountain", 5);
    let b = t.hand(P0, "Burst Lightning");
    t.cast(P0, b).kicked(true).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
}

#[test]
fn a_copy_of_sundering_stroke_deals_the_damage_as_divided() {
    cr!("707.10", "601.2d", "601.2h");
    ruling!(
        "Sundering Stroke",
        "You must divide the damage as you announce Sundering Stroke, even if you intend to pay seven red mana to cast it. If you copy Sundering Stroke, no mana was spent to cast the copy at all, so the copy will deal the damage as divided."
    );
    supported("Sundering Stroke");
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 9);
    let from = t.asked().len();
    divide(&mut t, P0, &[4, 3]);
    let s = t.hand(P0, "Sundering Stroke");
    let s = t
        .cast(P0, s)
        .targets(&[Entity::Player(P1), Entity::Player(P2)])
        .go();
    assert!(asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::Divide { total: 7, .. })));
    let v = t.hand(P0, "Increasing Vengeance");
    t.cast(P0, v).target(s).go();
    // Vengeance, then the copy (divided: 4 and 3), then the original (7 red: 7 each).
    t.resolve();
    t.resolve();
    assert_eq!((t.life(P1), t.life(P2)), (16, 17));
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (9, 10));
}
