//! Rulings batch P042 — life totals that damage can't reduce below 1 (Worship, Sustaining
//! Spirit, Ali from Cairo, Serra's emblem; CR 120.3a, 119.7–119.8), effects that stop a
//! player from losing (CR 104.3), and enchantments that replace or prevent all damage to
//! you and then make you lose (Immortal Coil, Nine Lives, Nefarious Lich, Force Bubble,
//! Intervention Pact).

use crate::r_p042_common::*;
use mtg_engine::decision::Action;
use mtg_engine::game::GameResult;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Other ways to lose ----------------------------------------------------------------------

/// `p` concedes (CR 104.3a) and loses although an effect says they can't lose the game.
fn concede_and_lose(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].life = 0;
    t.settle();
    assert!(!t.has_lost(p), "the effect is in place: 0 life doesn't lose");
    t.take_action(p, Action::Concede);
    assert!(t.has_lost(p));
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P1])));
}

#[test]
fn you_can_still_concede_while_you_cant_lose_the_game() {
    cr!("104.3a");
    ruling!(
        "Cloudsteel Kirin",
        "Other circumstances can still cause you to lose the game. For example, you will lose a game if you concede"
    );
    ruling!(
        "Courageous Resolve",
        "Other circumstances can still cause you to lose the game. For example, you will lose a game if you concede"
    );
    ruling!(
        "Gideon of the Trials",
        "Other circumstances can still cause you to lose the game. You will lose a game if you concede"
    );
    supported("Cloudsteel Kirin");
    supported("Courageous Resolve");
    supported("Gideon of the Trials");
    // Cloudsteel Kirin attached to a creature (its reconfigure ability).
    let mut t = TestGame::new(2);
    let kirin = t.battlefield(P0, "Cloudsteel Kirin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 5);
    activate_resolve(&mut t, P0, kirin, 0, &[obj(bears)]);
    assert_eq!(t.obj(kirin).attached_to, Some(obj(bears)));
    concede_and_lose(&mut t, P0);
    // Courageous Resolve with fateful hour.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    t.answer_targets(P0, &[]);
    cast_resolve(&mut t, P0, "Courageous Resolve", &[]);
    concede_and_lose(&mut t, P0);
    // Gideon of the Trials' emblem and a Gideon.
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon of the Trials");
    activate_resolve(&mut t, P0, gideon, 2, &[]);
    concede_and_lose(&mut t, P0);
}

#[test]
fn courageous_resolve_life_loss_parts_of_spells_do_nothing() {
    cr!("101.2", "608.2c");
    ruling!(
        "Courageous Resolve",
        "Spells and abilities that would normally cause you to lose life still resolve, but the life-loss part simply has no effect."
    );
    supported("Sign in Blood");
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    t.answer_targets(P0, &[]);
    cast_resolve(&mut t, P0, "Courageous Resolve", &[]);
    let hand = t.hand_size(P0);
    // P1's Sign in Blood targeting P0: "Target player draws two cards and loses 2 life."
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Sign in Blood", &[pl(P0)]);
    assert!(t.in_graveyard(P1, "Sign in Blood"), "it resolved");
    assert_eq!(t.hand_size(P0), hand + 2, "the draw happened");
    assert_eq!(t.life(P0), 5, "the life loss didn't");
}

#[test]
fn courageous_resolve_you_can_pay_only_0_life_but_can_be_dealt_damage() {
    cr!("119.8", "120.3a", "101.2");
    ruling!(
        "Courageous Resolve",
        "You can't pay a cost that includes the payment of any amount of life other than 0 life. However, you can choose to be dealt damage, even though that damage won't result in your life total changing."
    );
    supported("Adarkar Wastes");
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    t.answer_targets(P0, &[]);
    cast_resolve(&mut t, P0, "Courageous Resolve", &[]);
    // Gitaxian Probe ({U/P}) with no mana: only 2 life would pay for it.
    let probe = t.hand(P0, "Gitaxian Probe");
    assert!(t.cast(P0, probe).target(pl(P1)).try_go().is_err());
    assert_eq!(t.life(P0), 5);
    // Adarkar Wastes: "{T}: Add {W} or {U}. This land deals 1 damage to you."
    let wastes = t.battlefield(P0, "Adarkar Wastes");
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, wastes, "{W}"));
    t.g.flush_events();
    t.settle();
    assert_eq!(damage_events(&t), vec![(wastes, pl(P0), 1)], "it was dealt");
    assert_eq!(t.life(P0), 5, "but the life total didn't change");
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

// --- Damage that can't reduce your life below 1 ----------------------------------------------

/// P0 (at `life`) controls Serra the Benevolent's emblem and a creature.
fn serra_emblem(life: i32) -> TestGame {
    supported("Serra the Benevolent");
    let mut t = TestGame::new(2);
    let serra = t.battlefield(P0, "Serra the Benevolent");
    set_loyalty(&mut t, serra, 6);
    activate_resolve(&mut t, P0, serra, 2, &[]);
    t.battlefield(P0, "Grizzly Bears");
    set_life(&mut t, P0, life);
    t
}

#[test]
fn serra_emblem_doesnt_prevent_damage_lifelink_still_gains() {
    cr!("120.3a", "120.3f", "702.15b");
    ruling!(
        "Serra the Benevolent",
        "Serra's emblem doesn't prevent damage. It only changes the result of damage dealt to you. For example, a creature with lifelink that deals damage to you will still cause its controller to gain life"
    );
    let mut t = serra_emblem(3);
    let knight = t.battlefield(P1, "Vampire Nighthawk");
    damage(&mut t, knight, pl(P0), 2);
    assert_eq!(t.life(P0), 1);
    damage(&mut t, knight, pl(P0), 2);
    assert_eq!(t.life(P0), 1);
    assert!(!t.has_lost(P0));
    assert_eq!(damage_events(&t), vec![(knight, pl(P0), 2); 2]);
    assert!(prevented_events(&t).is_empty(), "nothing was prevented");
    assert_eq!(t.life(P1), 24, "lifelink saw all 4 damage");
}

#[test]
fn serra_emblem_doesnt_stop_life_loss() {
    cr!("119.3", "704.5a");
    ruling!(
        "Serra the Benevolent",
        "Serra's emblem doesn't stop loss of life from effects that say that you lose life."
    );
    let mut t = serra_emblem(3);
    t.g.lose_life(P0, 5);
    t.settle();
    assert_eq!(t.life(P0), -2);
    assert!(t.has_lost(P0));
}

#[test]
fn worship_damage_is_dealt_and_counted_and_life_loss_bypasses_it() {
    cr!("120.3a", "119.3", "702.15b");
    ruling!(
        "Worship",
        "Worship does not prevent damage. It causes some damage to be unable to lower your life total. So any damage rendered useless by Worship was still dealt and is counted by effects that track the amount of damage done to a player. In addition, Worship does not prevent loss of life, so loss of life bypasses Worship."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Worship");
    t.battlefield(P0, "Grizzly Bears");
    set_life(&mut t, P0, 3);
    let knight = t.battlefield(P1, "Vampire Nighthawk");
    damage(&mut t, knight, pl(P0), 5);
    assert_eq!(t.life(P0), 1);
    assert_eq!(damage_dealt_to(&t, pl(P0)), 5, "all 5 damage was dealt");
    assert!(prevented_events(&t).is_empty());
    assert_eq!(t.life(P1), 25, "lifelink counts all 5");
    t.g.lose_life(P0, 1);
    t.settle();
    assert!(t.has_lost(P0), "life loss bypasses Worship");
}

#[test]
fn sustaining_spirit_spirit_link_sees_the_full_damage() {
    cr!("120.3a");
    ruling!(
        "Sustaining Spirit",
        "The ability doesn’t change how much damage is dealt; it just changes how much life that damage makes you lose. An effect such as Spirit Link will see the full amount of damage being dealt."
    );
    supported("Spirit Link");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sustaining Spirit");
    set_life(&mut t, P0, 3);
    let giant = t.battlefield(P1, "Hill Giant");
    // P1's Spirit Link on P1's Hill Giant: "Whenever enchanted creature deals damage, you
    // gain that much life."
    let link = t.battlefield(P1, "Spirit Link");
    assert!(t.g.attach(link, obj(giant)));
    t.g.recompute();
    damage(&mut t, giant, pl(P0), 3);
    t.resolve_all();
    assert_eq!(t.life(P0), 1);
    assert_eq!(t.life(P1), 23, "Spirit Link's controller gains 3");
}

#[test]
fn sustaining_spirit_life_loss_and_payments_go_below_1() {
    cr!("119.3", "119.4", "704.5a");
    ruling!(
        "Sustaining Spirit",
        "The second ability applies only if your life total is being reduced by damage. Other effects or costs (such as “lose 1 life” or “pay 1 life”) can reduce your life total below 1 as normal."
    );
    // Paying life: Gitaxian Probe's {U/P} from 2 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sustaining Spirit");
    set_life(&mut t, P0, 2);
    let probe = t.hand(P0, "Gitaxian Probe");
    t.cast(P0, probe).target(pl(P1)).go();
    assert_eq!(t.life(P0), 0);
    // Losing life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sustaining Spirit");
    set_life(&mut t, P0, 2);
    t.g.lose_life(P0, 3);
    t.settle();
    assert_eq!(t.life(P0), -1);
    assert!(t.has_lost(P0));
}

#[test]
fn sustaining_spirit_doesnt_stop_you_losing_at_0_life_or_otherwise() {
    cr!("704.5a", "704.5c");
    ruling!(
        "Sustaining Spirit",
        "Sustaining Spirit won’t prevent you from losing the game if your life total is 0 or less or some other effect causes you to lose the game."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sustaining Spirit");
    set_life(&mut t, P0, 0);
    t.settle();
    assert!(t.has_lost(P0));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sustaining Spirit");
    t.g.players[0].counters.insert("poison".into(), 10);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn sustaining_spirit_stops_unpreventable_damage_from_going_below_1() {
    cr!("120.3a", "615.12");
    ruling!(
        "Sustaining Spirit",
        "The second ability is not a prevention effect. It stops unpreventable damage from reducing your life total below 1."
    );
    supported("Banefire");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sustaining Spirit");
    set_life(&mut t, P0, 3);
    // Banefire with X = 5: "If X is 5 or more, this spell can't be countered and the damage
    // can't be prevented."
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 6);
    let fire = t.hand(P1, "Banefire");
    let fire = t.cast(P1, fire).target(pl(P0)).x(5).go();
    t.resolve_all();
    assert_eq!(damage_events(&t), vec![(fire, pl(P0), 5)]);
    assert_eq!(t.life(P0), 1);
    assert!(!t.has_lost(P0));
}

#[test]
fn ali_from_cairo_applies_to_damage_dealt_at_the_same_time_as_its_own() {
    cr!("120.3a", "704.5g", "614.1a");
    ruling!(
        "Ali from Cairo",
        "The ability works up until Ali enters the graveyard, so if he takes lethal damage or is destroyed at the same time you take damage, the ability helps you. If the damage occurs after it goes to the graveyard, however, it is not affected by the Ali which is no longer on the battlefield."
    );
    let mut t = TestGame::new(2);
    let ali = t.battlefield(P0, "Ali from Cairo");
    set_life(&mut t, P0, 3);
    let giant = t.battlefield(P1, "Hill Giant");
    // 5 damage to Ali and 5 to P0 at once.
    damage_at_once(&mut t, &[(giant, obj(ali), 5), (giant, pl(P0), 5)], false);
    assert!(t.in_graveyard(P0, "Ali from Cairo"));
    assert_eq!(t.life(P0), 1);
    assert!(!t.has_lost(P0));
    // Afterwards, it doesn't apply.
    damage(&mut t, giant, pl(P0), 1);
    assert_eq!(t.life(P0), 0);
    assert!(t.has_lost(P0));
}

#[test]
fn nefarious_lich_doesnt_stop_you_losing_at_0_life() {
    cr!("704.5a");
    ruling!(
        "Nefarious Lich",
        "This card will not prevent you from losing the game due to zero or less life."
    );
    supported("Nefarious Lich");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nefarious Lich");
    t.g.lose_life(P0, 20);
    t.settle();
    assert_eq!(t.life(P0), 0);
    assert!(t.has_lost(P0));
}

// --- Immortal Coil ---------------------------------------------------------------------------

/// P0 controls Immortal Coil and has `n` cards in their graveyard.
fn coil(n: usize) -> (TestGame, ObjectId) {
    supported("Immortal Coil");
    let mut t = TestGame::new(2);
    let coil = t.battlefield(P0, "Immortal Coil");
    for _ in 0..n {
        t.graveyard(P0, "Grizzly Bears");
    }
    (t, coil)
}

#[test]
fn immortal_coil_prevents_all_damage_whatever_the_graveyard() {
    cr!("615.1a", "603.8");
    ruling!(
        "Immortal Coil",
        "The second ability prevents all damage that would be dealt to you, regardless of how many cards are in your graveyard. For example, if you have three cards in your graveyard and would be dealt 5 damage, Immortal Coil prevents all 5 damage and exiles all the cards in your graveyard."
    );
    let (mut t, _) = coil(3);
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, pl(P0), 5);
    assert_eq!(t.life(P0), 20);
    assert_eq!(prevented_events(&t), vec![(giant, pl(P0), 5)]);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.g.exile.len(), 3);
    // Its last ability then makes P0 lose.
    t.resolve_all();
    assert!(t.has_lost(P0));
}

#[test]
fn immortal_coil_loses_even_if_a_card_is_put_into_the_graveyard_in_response() {
    cr!("603.8");
    ruling!(
        "Immortal Coil",
        "The third ability checks whether your graveyard is empty only at the time it triggers. Putting a card into your graveyard after that doesn’t help. You’ll lose the game when the ability resolves."
    );
    let (mut t, _) = coil(0);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the state trigger");
    t.graveyard(P0, "Grizzly Bears");
    assert_eq!(t.graveyard_size(P0), 1);
    t.resolve();
    assert!(t.has_lost(P0));
}

#[test]
fn immortal_coil_exiles_two_cards_as_a_cost() {
    cr!("602.2b", "601.2h", "118.3");
    ruling!(
        "Immortal Coil",
        "The first ability exiles two cards in your graveyard as a cost. You can’t activate this ability unless you have at least two cards in your graveyard. Exiling those two cards can’t be responded to."
    );
    let (mut t, coil) = coil(1);
    assert!(t.activate(P0, coil, 0, &[]).is_err(), "one card isn't enough");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Craw Wurm");
    let hand = t.hand_size(P0);
    activate(&mut t, P0, coil, 0, &[]);
    // The cards were exiled as the ability was activated, before anyone could respond.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.g.exile.len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(!t.has_lost(P0));
}

#[test]
fn immortal_coil_and_platinum_angel_with_an_empty_graveyard_is_a_draw() {
    cr!("104.4b", "603.8");
    ruling!(
        "Immortal Coil",
        "Similarly, if the third ability resolves but you don’t lose the game for some reason (because you control Platinum Angel, perhaps), it will immediately trigger again if your graveyard is still empty. Immortal Coil + Platinum Angel + an empty graveyard is an involuntary infinite loop. Unless a player disrupts it, the game will end in a draw."
    );
    supported("Platinum Angel");
    let (mut t, _) = coil(0);
    t.battlefield(P0, "Platinum Angel");
    // It triggers again after resolving.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(!t.has_lost(P0));
    assert_eq!(t.stack_len(), 1, "it triggered again");
    t.g.run_until(2000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

// --- Nine Lives, Force Bubble, Intervention Pact ---------------------------------------------

#[test]
fn nine_lives_state_trigger_doesnt_trigger_again_while_on_the_stack() {
    cr!("603.8");
    ruling!(
        "Nine Lives",
        "Once the first triggered ability is on the stack, it won't trigger again, even if more damage is prevented and more incarnation counters are put onto it."
    );
    let mut t = TestGame::new(2);
    let lives = t.battlefield(P0, "Nine Lives");
    let giant = t.battlefield(P1, "Hill Giant");
    for i in 1..=9 {
        damage(&mut t, giant, pl(P0), 1);
        assert_eq!(t.counters(lives, "incarnation"), i);
    }
    assert_eq!(t.stack_len(), 1, "nine counters: it triggered");
    damage(&mut t, giant, pl(P0), 1);
    assert_eq!(t.counters(lives, "incarnation"), 10);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.stack_len(), 1, "it didn't trigger again");
    // It resolves: Nine Lives is exiled and its last ability makes P0 lose.
    t.resolve_all();
    assert!(!t.on_battlefield(lives));
    assert!(t.has_lost(P0));
}

#[test]
fn nine_lives_leaving_the_battlefield_any_way_loses() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Nine Lives",
        "The last ability of Nine Lives triggers no matter how it leaves the battlefield."
    );
    supported("Disenchant");
    supported("Boomerang");
    for how in ["Disenchant", "Boomerang", "sacrifice"] {
        let mut t = TestGame::new(2);
        let lives = t.battlefield(P0, "Nine Lives");
        if how == "sacrifice" {
            t.g.sacrifice(lives, P0);
            t.g.flush_events();
        } else {
            // Nine Lives has hexproof: P0's own spells can target it.
            cast_resolve(&mut t, P0, how, &[obj(lives)]);
        }
        t.resolve_all();
        assert!(!t.on_battlefield(lives), "{how}");
        assert!(t.has_lost(P0), "{how}");
    }
}

#[test]
fn force_bubble_damage_is_neither_dealt_nor_prevented() {
    cr!("614.1a", "615.1");
    ruling!(
        "Force Bubble",
        "The effect does not prevent damage, it replaces it so the damage is neither dealt or prevented."
    );
    supported("Force Bubble");
    let mut t = TestGame::new(2);
    let bubble = t.battlefield(P0, "Force Bubble");
    let knight = t.battlefield(P1, "Vampire Nighthawk");
    damage(&mut t, knight, pl(P0), 2);
    assert_eq!(t.counters(bubble, "depletion"), 2);
    assert_eq!(t.life(P0), 20);
    assert!(damage_events(&t).is_empty(), "no damage was dealt");
    assert!(prevented_events(&t).is_empty(), "no damage was prevented");
    assert_eq!(t.life(P1), 20, "no lifelink: nothing was dealt");
}

#[test]
fn intervention_pact_upkeep_trigger_whether_or_not_damage_was_prevented() {
    cr!("603.7a");
    ruling!(
        "Intervention Pact",
        "The delayed triggered ability will trigger regardless of whether any damage is actually prevented."
    );
    supported("Intervention Pact");
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_choose(P0, &[obj(giant)]);
        cast_resolve(&mut t, P0, "Intervention Pact", &[]);
        if pay {
            t.lands(P0, "Plains", 3);
            t.answer_yes(P0, true);
        }
        assert!(prevented_events(&t).is_empty());
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.has_lost(P0), !pay, "paid: {pay}");
    }
}
