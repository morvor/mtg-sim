//! Rulings batch P138 — It That Betrays and Nazgûl Battle-Mace: "Whenever an opponent
//! sacrifices a nontoken permanent, put that card onto the battlefield under your control"
//! (CR 603.2, 603.10a look-back for leaves-the-battlefield triggers, 400.7 new objects,
//! 303.4f Auras put onto the battlefield without being cast).

use crate::r_p138_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts Nazgûl Battle-Mace on the battlefield under P0's control attached to a Hill Giant
/// (or the creature named), returning the creature.
fn mace_on(t: &mut TestGame, bearer: &str) -> ObjectId {
    let c = t.battlefield(P0, bearer);
    let mace = t.battlefield(P0, "Nazgûl Battle-Mace");
    assert!(t.g.attach(mace, obj(c)));
    t.g.recompute();
    c
}

#[test]
fn stolen_when_sacrificed_to_pay_a_cost_before_the_ability_resolves() {
    cr!("603.3", "405.5", "601.2h", "602.2b");
    ruling!(
        "It That Betrays",
        "If an opponent sacrifices a nontoken permanent as part of paying the cost of a spell or ability, the second ability triggers and goes on the stack on top of it."
    );
    ruling!(
        "Nazgûl Battle-Mace",
        "If an opponent sacrifices a nontoken permanent as part of paying the cost of a spell or ability, the last ability granted by Nazgûl Battle-Mace will trigger and go on the stack on top of that spell or ability."
    );
    for mace in [false, true] {
        let mut t = TestGame::new(2);
        if mace {
            mace_on(&mut t, "Hill Giant");
        } else {
            t.battlefield(P0, "It That Betrays");
        }
        let fanatic = t.battlefield(P1, "Mogg Fanatic");
        // "Sacrifice Mogg Fanatic: It deals 1 damage to any target."
        t.activate(P1, fanatic, 0, &[pl(P0)]).expect("activated");
        t.settle();
        assert_eq!(t.stack_len(), 2, "mace {mace}: the trigger is above the ability");
        if mace {
            t.answer_yes(P1, false);
        }
        t.resolve();
        // The card is back (under P0's control) while the Fanatic's ability still waits.
        assert!(controls(&t, P0, fanatic), "mace {mace}");
        assert_eq!(t.stack_len(), 1, "mace {mace}");
        assert_eq!(t.life(P0), 20);
        t.resolve_all();
        assert_eq!(t.life(P0), 19, "mace {mace}: the ability still resolves");
    }
}

#[test]
fn it_doesnt_matter_whose_graveyard_the_card_goes_to() {
    cr!("603.10a", "404.1");
    ruling!(
        "It That Betrays",
        "It doesn’t matter whose graveyard the permanent is put into, only that it was last controlled by, and sacrificed by, an opponent."
    );
    ruling!(
        "Nazgûl Battle-Mace",
        "If an opponent sacrifices a nontoken permanent, it doesn't matter which graveyard it goes to. The last ability granted by Nazgûl Battle-Mace will trigger."
    );
    for mace in [false, true] {
        let mut t = TestGame::new(2);
        if mace {
            mace_on(&mut t, "Hill Giant");
        } else {
            t.battlefield(P0, "It That Betrays");
        }
        // A creature P0 owns that P1 controls goes to P0's graveyard.
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.objects[bears.0 as usize].controller = P1;
        t.g.objects[bears.0 as usize].base_controller = P1;
        t.g.recompute();
        sac(&mut t, P1, bears);
        assert_eq!(t.zone(bears), Zone::Graveyard(P0));
        assert_eq!(triggers_on_stack(&t, "onto the battlefield"), 1, "mace {mace}");
        if mace {
            t.answer_yes(P1, false);
        }
        t.resolve_all();
        assert!(controls(&t, P0, bears), "mace {mace}");
        // P0 sacrificing its own permanent doesn't count.
        sac(&mut t, P0, bears);
        assert_eq!(t.stack_len(), 0, "mace {mace}");
    }
}

#[test]
fn an_aura_is_attached_as_it_returns_even_to_a_permanent_with_shroud() {
    cr!("303.4f", "303.4a", "702.18a");
    ruling!(
        "It That Betrays",
        "If an opponent sacrifices an Aura, you’ll choose what it enchants as you return it to the battlefield. No player can respond to the choice. Since an Aura doesn’t target anything if it isn’t cast as a spell, you can enchant a permanent with shroud this way."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "It That Betrays");
    let target = t.battlefield(P1, "Grizzly Bears");
    let shroud = t.battlefield(P1, "Argothian Enchantress");
    let pacifism = t.battlefield(P1, "Pacifism");
    assert!(t.g.attach(pacifism, obj(target)));
    sac(&mut t, P1, pacifism);
    assert_eq!(t.stack_len(), 1);
    t.answer_choose(P0, &[obj(shroud)]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(controls(&t, P0, pacifism));
    let now = t.g.current(pacifism);
    assert_eq!(t.obj(now).attached_to, Some(obj(shroud)));
    // P0 made the choice.
    assert!(first_offer(&t, P0, from).contains(&shroud));
}

#[test]
fn mace_aura_may_go_on_hexproof_but_not_protection_and_otherwise_stays() {
    cr!("303.4f", "702.11a", "702.16c");
    ruling!(
        "Nazgûl Battle-Mace",
        "If you put an Aura on the battlefield using the last ability granted by Nazgûl Battle-Mace, you choose what the Aura will enchant just before it enters the battlefield. An Aura put onto the battlefield this way doesn't target the player or permanent it will enchant, so permanents or players with hexproof may be chosen; however, the chosen recipient must be able to legally be enchanted by the Aura, so a player or permanent with protection from one of the Aura's qualities can't be chosen this way. If there's nothing legal for the Aura to enchant, it stays in the graveyard."
    );
    // Black Knight has protection from white; Pacifism is white.
    let mut t = TestGame::new(2);
    mace_on(&mut t, "Black Knight");
    let knight = t.battlefield(P1, "Black Knight");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let pacifism = t.battlefield(P1, "Pacifism");
    assert!(t.g.attach(pacifism, obj(scout)));
    sac(&mut t, P1, pacifism);
    t.answer_yes(P1, false);
    t.answer_choose(P0, &[obj(scout)]);
    let from = t.asked().len();
    t.resolve_all();
    let offer = first_offer(&t, P0, from);
    assert!(offer.contains(&scout), "hexproof may be chosen: {offer:?}");
    assert!(!offer.contains(&knight), "protection can't be chosen");
    assert!(controls(&t, P0, pacifism));
    assert_eq!(t.obj_now(pacifism).attached_to, Some(obj(scout)));

    // Nothing it could enchant: it stays in the graveyard.
    let mut t = TestGame::new(2);
    mace_on(&mut t, "Black Knight");
    t.battlefield(P1, "Black Knight");
    // (Unattached, it would be put into the graveyard by state-based actions: P1
    // sacrifices it before the game checks them.)
    let pacifism = t.battlefield(P1, "Pacifism");
    t.g.sacrifice(pacifism, P1);
    t.g.flush_events();
    t.settle();
    t.answer_yes(P1, false);
    t.resolve_all();
    assert_eq!(t.zone(pacifism), Zone::Graveyard(P1));
}

#[test]
fn mace_card_that_left_the_graveyard_isnt_returned_but_life_may_still_be_paid() {
    cr!("400.7", "603.10a", "608.2b");
    ruling!(
        "Nazgûl Battle-Mace",
        "If the card leaves the graveyard before the last ability granted by Nazgûl Battle-Mace resolves, the opponent will still have the option to pay 3 life. Regardless of their choice, you can't put the card onto the battlefield, even if it returns to the graveyard before the ability resolves."
    );
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        mace_on(&mut t, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        sac(&mut t, P1, bears);
        assert_eq!(t.stack_len(), 1);
        // The card leaves the graveyard and comes back: a new object.
        move_to(&mut t, bears, Zone::Exile);
        move_to(&mut t, bears, Zone::Graveyard(P1));
        t.answer_yes(P1, pay);
        let from = t.asked().len();
        t.resolve_all();
        assert!(asked_yes_no(&t, P1, from), "pay {pay}: still offered");
        assert_eq!(t.life(P1), if pay { 17 } else { 20 });
        assert_eq!(t.zone(bears), Zone::Graveyard(P1), "pay {pay}");
    }
}

#[test]
fn it_that_betrays_does_nothing_once_another_returned_the_card() {
    cr!("400.7", "603.10a", "101.4");
    ruling!(
        "It That Betrays",
        "If the sacrificed permanent that caused the second ability to trigger somehow leaves the graveyard before the ability resolves (possibly because it was returned to the battlefield by the ability of another It That Betrays), the ability simply won’t do anything when it resolves."
    );
    // P0 and P2 each control It That Betrays; P1 sacrifices. P2's trigger (put on the
    // stack last, APNAP) resolves first and takes the card.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "It That Betrays");
    t.battlefield(P2, "It That Betrays");
    let bears = t.battlefield(P1, "Grizzly Bears");
    sac(&mut t, P1, bears);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(controls(&t, P2, bears));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.graveyard_size(P1), 0);
}
