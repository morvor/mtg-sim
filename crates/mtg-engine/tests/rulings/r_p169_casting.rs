//! Rulings batch P169 — casting: several conspire instances (CR 702.78), exchanging
//! control of a spell (CR 701.12b, 115.7), casting without paying the mana cost (CR
//! 118.9, 601.2b, 601.2f), casting during the resolution of an ability (CR 608.2g), and
//! several affinity instances (CR 702.41b).

use crate::r_p169_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn raiding_schemes_conspire_instances_are_paid_individually() {
    cr!("702.78a", "702.78b");
    ruling!(
        "Raiding Schemes",
        "If a spell has multiple instances of conspire (perhaps from multiple Raiding Schemes), you may choose to pay for each of those abilities individually. Each conspire ability triggers only if you tap two creatures specifically for that ability."
    );
    supported("Raiding Schemes");
    // `pay`: whether to pay each conspire instance. Returns (copies, tapped Merfolk).
    let run = |pay: &[bool]| {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Raiding Schemes");
        t.battlefield(P0, "Raiding Schemes");
        let merfolk: Vec<ObjectId> = (0..4)
            .map(|_| t.battlefield(P0, "Merfolk of the Pearl Trident"))
            .collect();
        for p in pay {
            t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(*p));
        }
        let hand = t.hand_size(P0);
        cast_opt(&mut t, P0);
        t.resolve_all();
        // Opt and each copy draw a card.
        let copies_made = t.hand_size(P0) - hand - 1;
        let tapped = merfolk.iter().filter(|m| t.obj_now(**m).tapped).count();
        (copies_made, tapped)
    };
    assert_eq!(run(&[true, true]), (2, 4));
    assert_eq!(run(&[true, false]), (1, 2));
    assert_eq!(run(&[false, false]), (0, 0));
}

#[test]
fn sudden_substitution_without_an_exchange_still_lets_the_controller_retarget() {
    cr!("701.12b", "115.7", "608.2b");
    ruling!(
        "Sudden Substitution",
        "If the exchange doesn't happen but the spell is still a legal target, its controller may choose new targets for it."
    );
    supported("Sudden Substitution");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P1, ManaType::R, 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(bears).go();
    // Both targets are P1's: no exchange.
    mana(&mut t, P0, ManaType::U, 4);
    let ss = t.hand(P0, "Sudden Substitution");
    t.answer_targets(P0, &[obj(spell)]);
    t.answer_targets(P0, &[obj(giant)]);
    // P1 chooses a new target for Shock: P0.
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let from = t.asked().len();
    t.cast(P0, ss).go();
    t.resolve();
    assert_eq!(t.obj_now(spell).controller, P1);
    assert_eq!(t.obj_now(giant).controller, P1);
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseTargets { .. })));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P0), 18);
    // The creature target is gone (an illegal target): no exchange, and P1 may still
    // choose new targets for Shock.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    mana(&mut t, P1, ManaType::R, 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(bears).go();
    mana(&mut t, P0, ManaType::U, 4);
    let ss = t.hand(P0, "Sudden Substitution");
    t.answer_targets(P0, &[obj(spell)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.cast(P0, ss).go();
    let g = t.g.current(giant);
    t.g.move_object(g, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve();
    assert_eq!(t.obj_now(spell).controller, P1);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P0), 18);
}

/// P0's Yue, the Moon Spirit activates its waterbend ability (`mana` generic mana in the
/// pool, tapping nothing for it), with `answers` queued for its resolution, and the stack
/// resolves.
fn yue(t: &mut TestGame, mana_n: u32, answers: impl FnOnce(&mut TestGame)) {
    supported("Yue, the Moon Spirit");
    let yue = t.battlefield(P0, "Yue, the Moon Spirit");
    mana(t, P0, ManaType::C, mana_n);
    t.answer_choose(P0, &[]);
    answers(t);
    t.activate(P0, yue, 0, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn yue_free_casts_pay_additional_but_not_alternative_costs() {
    cr!("118.9", "118.9a", "601.2b");
    ruling!(
        "Yue, the Moon Spirit",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Cathartic Reunion, you must pay those to cast the card."
    );
    supported("Cathartic Reunion");
    // Cathartic Reunion: the two cards are discarded.
    let mut t = TestGame::new(2);
    let cr = t.hand(P0, "Cathartic Reunion");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    yue(&mut t, 5, |t| {
        t.answer_choose(P0, &[obj(cr)]);
        t.answer_choose(P0, &[obj(a), obj(b)]);
    });
    assert!(t.in_graveyard(P0, "Cathartic Reunion"));
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), 3);
    // Without two other cards it can't be cast.
    let mut t = TestGame::new(2);
    let cr = t.hand(P0, "Cathartic Reunion");
    t.hand(P0, "Grizzly Bears");
    yue(&mut t, 5, |t| {
        t.answer_choose(P0, &[obj(cr)]);
    });
    assert!(t.in_hand(P0, "Cathartic Reunion"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Burst Lightning's kicker can be paid.
    let mut t = TestGame::new(2);
    let bl = t.hand(P0, "Burst Lightning");
    yue(&mut t, 9, |t| {
        t.answer_choose(P0, &[obj(bl)]);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 16);
    // Force of Will's alternative cost isn't offered (P0 would take it), so it isn't used:
    // no life paid, no card exiled.
    let mut t = TestGame::new(2);
    mana(&mut t, P1, ManaType::R, 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(Entity::Player(P0)).go();
    let fow = t.hand(P0, "Force of Will");
    t.hand(P0, "Opt");
    let from = t.asked().len();
    yue(&mut t, 5, |t| {
        t.answer_choose(P0, &[obj(fow)]);
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        t.answer_targets(P0, &[obj(spell)]);
    });
    assert!(!t.asked()[from..].iter().any(|(_, d)| matches!(
        d,
        Decision::ChooseOption { .. } | Decision::ChooseCastingMethod { .. }
    )));
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.life(P0), 20);
    assert!(t.in_hand(P0, "Opt"));
    assert!(t.in_graveyard(P0, "Force of Will"));
}

#[test]
fn vadrok_casts_the_card_while_its_ability_resolves() {
    cr!("608.2g", "702.140d");
    ruling!(
        "Vadrok, Apex of Thunder",
        "If you wish to cast the target card, you do so as part of the resolution of Vadrok’s ability. You can’t wait to cast it later. Timing permissions based on a card’s type are ignored."
    );
    supported("Vadrok, Apex of Thunder");
    // `yes`: whether P0 casts Divination. Returns (hand size, Divination castable after).
    let run = |yes: bool| {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let div = t.graveyard(P0, "Divination");
        mana(&mut t, P0, ManaType::W, 1);
        mana(&mut t, P0, ManaType::R, 3);
        let v = t.hand(P0, "Vadrok, Apex of Thunder");
        t.answer_targets(P0, &[obj(bears)]);
        t.cast(P0, v)
            .method(CastMethod::Keyword(KeywordKind::Mutate))
            .go();
        t.answer_targets(P0, &[obj(div)]);
        t.answer_yes(P0, yes);
        // Resolve the mutating spell, then the trigger.
        t.resolve();
        assert_eq!(stack_triggers(&t).len(), 1);
        t.resolve();
        if yes {
            // Divination is on the stack (a sorcery, cast while the ability resolved).
            assert_eq!(t.stack_len(), 1);
        }
        t.resolve_all();
        mana(&mut t, P0, ManaType::U, 3);
        let later = castable(&mut t, P0, div);
        (t.hand_size(P0), later)
    };
    assert_eq!(run(true), (2, false));
    assert_eq!(run(false), (0, false));
}

#[test]
fn don_and_raph_affinity_instances_are_cumulative() {
    cr!("702.41b", "601.2f");
    ruling!(
        "Don & Raph, Hard Science",
        "In the rare case where Don & Raph's last ability resolves more than once in the same turn (probably because there were multiple combat phases), a single noncreature spell might have multiple instances of affinity for artifacts. Those abilities are cumulative. For example, if you attacked twice with Don & Raph in the same turn without casting any noncreature spells in between those attacks, the next noncreature spell you cast that turn would cost {2} less to cast for each artifact you controlled."
    );
    supported("Don & Raph, Hard Science");
    // `attacks`: combats Don & Raph attacks in. Two artifacts; Opportunity is {4}{U}{U}.
    let run = |attacks: usize| {
        let mut t = TestGame::new(2);
        let dr = t.battlefield(P0, "Don & Raph, Hard Science");
        t.battlefield(P0, "Ornithopter");
        t.battlefield(P0, "Ornithopter");
        let mut decl = vec![(dr, Entity::Player(P1))];
        if attacks == 2 {
            // Desert Were-Worm (12 power with six Mountains) gives an additional combat.
            t.lands(P0, "Mountain", 6);
            let worm = t.battlefield(P0, "Desert Were-Worm");
            decl.push((worm, Entity::Player(P1)));
        }
        t.answer(P0, DecisionKind::Attackers, Answer::Attackers(decl.clone()));
        if attacks == 2 {
            t.answer(
                P0,
                DecisionKind::Attackers,
                Answer::Attackers(vec![(dr, Entity::Player(P1))]),
            );
        }
        t.advance_to(P0, Step::PostcombatMain);
        let attack_triggers = triggers_of(&t, dr);
        mana(&mut t, P0, ManaType::U, 2);
        let opp = t.hand(P0, "Opportunity");
        let ok = t.cast(P0, opp).target(Entity::Player(P0)).try_go().is_ok();
        (attack_triggers, ok)
    };
    assert_eq!(run(1), (1, false));
    assert_eq!(run(2), (2, true));
}

#[test]
fn sudden_substitution_control_ends_when_its_controller_leaves() {
    cr!("800.4a", "701.12b");
    ruling!(
        "Sudden Substitution",
        "In a multiplayer game, if the controller of one of the objects leaves the game after Sudden Substitution resolves, the effect that gave them control of the spell or permanent expires. If the spell is still on the stack, the player who gains control of it this way can't choose new targets for it."
    );
    let mut t = TestGame::new(3);
    let wurm = t.battlefield(P2, "Colossal Dreadmaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P1's Shock targets P0's Bears.
    mana(&mut t, P1, ManaType::R, 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(bears).go();
    // P0 exchanges P1's Shock and P2's Dreadmaw: P2 controls Shock, P1 the Dreadmaw.
    mana(&mut t, P0, ManaType::U, 4);
    let ss = t.hand(P0, "Sudden Substitution");
    t.answer_targets(P0, &[obj(spell)]);
    t.answer_targets(P0, &[obj(wurm)]);
    // P2 keeps Shock's target.
    t.answer_yes(P2, false);
    t.cast(P0, ss).go();
    t.resolve();
    assert_eq!(t.obj_now(spell).controller, P2);
    assert_eq!(t.obj_now(wurm).controller, P1);
    // P2 leaves the game (and P2's Dreadmaw with it): P1 controls Shock again and isn't
    // offered new targets (P1 would aim it at P0).
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let from = t.asked().len();
    t.g.lose_game(P2);
    t.g.flush_events();
    t.settle();
    assert!(t.g.stack.contains(&spell));
    assert_eq!(t.obj_now(spell).controller, P1);
    assert!(t.named_on_battlefield("Colossal Dreadmaw").is_empty());
    t.resolve_all();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1
            && matches!(d, Decision::ChooseTargets { .. } | Decision::YesNo { .. })));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 20);

    // The other one leaves: P1 (controlling P2's Dreadmaw) leaves, and the Dreadmaw goes
    // back to P2 (P1's own Shock leaves the game with P1).
    let mut t = TestGame::new(3);
    let wurm = t.battlefield(P2, "Colossal Dreadmaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P1, ManaType::R, 1);
    let shock = t.hand(P1, "Shock");
    let spell = t.cast(P1, shock).target(bears).go();
    mana(&mut t, P0, ManaType::U, 4);
    let ss = t.hand(P0, "Sudden Substitution");
    t.answer_targets(P0, &[obj(spell)]);
    t.answer_targets(P0, &[obj(wurm)]);
    t.answer_yes(P2, false);
    t.cast(P0, ss).go();
    t.resolve();
    assert_eq!(t.obj_now(wurm).controller, P1);
    t.g.lose_game(P1);
    t.g.flush_events();
    t.settle();
    t.g.recompute();
    assert_eq!(t.obj_now(wurm).controller, P2);
    assert!(!t.g.stack.contains(&spell));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}
