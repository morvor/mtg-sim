//! Rulings batch P037 — soft and reusable counterspells: "counter ... unless its controller
//! pays" with other effects that happen regardless (CR 608.2b, 118.12), Strict Proctor's
//! counter of triggered abilities (CR 603.2, 603.3b, 614.1c), and Declaration of Naught's
//! chosen name and split cards (CR 709.3, 702.102d, 201.4b).

use crate::r_p037_common::*;
use crate::r_s01_common::supported;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

// --- "Counter unless pays", other effects happen regardless -------------------------------

/// P1 casts Shock at P0 (with `spare` Wastes left to pay a tax), P0 answers it with the
/// real counterspell `name` (with `x` as its X if it has one); P1 pays or not. Returns
/// (whether Shock resolved, P0's life gained over 20 after Shock, P0's hand size change,
/// P1's life).
fn soft_counter(name: &str, x: Option<i64>, pay: bool) -> (bool, i32, i32, i32) {
    supported(name);
    let mut t = TestGame::new(2);
    t.lands(P1, "Wastes", 4);
    let s = shock(&mut t, P1, P0);
    crate::r_s25_common::lands_for_cost(&mut t, P0, name);
    if let Some(x) = x {
        t.lands(P0, "Wastes", x as usize);
        t.answer(P0, DecisionKind::X, Answer::Number(x));
    }
    let hand = t.hand_size(P0) as i32;
    let c = t.hand(P0, name);
    t.cast_with(P0, c, &[obj(s)]).unwrap();
    t.answer_yes(P1, pay);
    t.resolve();
    let resolved = t.stack.contains(&s);
    let after = t.hand_size(P0) as i32 - hand;
    t.resolve_all();
    let shock_damage = if resolved { 2 } else { 0 };
    (resolved, t.life(P0) + shock_damage - 20, after, t.life(P1))
}

#[test]
fn soft_counters_draw_or_gain_life_whether_or_not_the_tax_is_paid() {
    cr!("608.2b", "118.12");
    ruling!("Runeboggle", "You draw a card even if the player pays {1}.");
    ruling!(
        "Offering to Asha",
        "You gain 4 life whether or not the targeted spell’s controller pays {4} and whether or not the targeted spell is countered."
    );
    ruling!(
        "Overrule",
        "You gain the life whether or not the spell’s controller pays {X} and whether or not the spell is countered."
    );
    // (card, X, life gained, cards drawn)
    let cases: [(&str, Option<i64>, i32, i32); 3] = [
        ("Runeboggle", None, 0, 1),
        ("Offering to Asha", None, 4, 0),
        ("Overrule", Some(3), 3, 0),
    ];
    for (name, x, life, draw) in cases {
        for pay in [true, false] {
            let (resolved, gained, drew, _) = soft_counter(name, x, pay);
            assert_eq!(resolved, pay, "{name}: pay {pay}");
            assert_eq!(gained, life, "{name}: pay {pay}");
            assert_eq!(drew, draw, "{name}: pay {pay}");
        }
    }
}

#[test]
fn disrupt_draws_whether_or_not_the_spell_is_countered() {
    cr!("608.2b", "118.12");
    ruling!(
        "Disrupt",
        "You draw a card regardless of whether the targeted spell was countered or {1} was paid."
    );
    for pay in [true, false] {
        let (resolved, _, drew, _) = soft_counter("Disrupt", None, pay);
        assert_eq!(resolved, pay);
        assert_eq!(drew, 1);
    }
}

#[test]
fn mindswipe_deals_damage_even_if_the_controller_pays() {
    cr!("608.2b", "118.12");
    ruling!(
        "Mindswipe",
        "If the controller of the target spell pays {X}, the spell won’t be countered. Mindswipe will still deal damage to that player."
    );
    for pay in [true, false] {
        let (resolved, _, _, p1_life) = soft_counter("Mindswipe", Some(3), pay);
        assert_eq!(resolved, pay);
        assert_eq!(p1_life, 17);
    }
}

#[test]
fn spell_contortion_draws_for_each_kick_regardless() {
    cr!("608.2b", "702.33c", "702.33d");
    ruling!(
        "Spell Contortion",
        "If Spell Contortion resolves, you draw a card for each time Spell Contortion was kicked, regardless of whether the targeted spell's controller pays {2} or whether the targeted spell is countered this way."
    );
    supported("Spell Contortion");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P1, "Wastes", 2);
        let s = shock(&mut t, P1, P0);
        t.lands(P0, "Island", 3);
        t.lands(P0, "Wastes", 4);
        let c = t.hand(P0, "Spell Contortion");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
        t.cast_with(P0, c, &[obj(s)]).unwrap();
        let hand = t.hand_size(P0);
        t.answer_yes(P1, pay);
        t.resolve();
        assert_eq!(t.stack.contains(&s), pay);
        assert_eq!(t.hand_size(P0), hand + 2, "pay {pay}");
    }
}

/// P1 casts Shock; P0 casts the real soft counter `name` at it (X = 3 if it has X); then
/// P1's Shock is countered by Counterspell before `name` resolves. Returns the game.
fn countered_first(name: &str, x: bool) -> TestGame {
    let mut t = TestGame::new(2);
    let s = shock(&mut t, P1, P0);
    crate::r_s25_common::lands_for_cost(&mut t, P0, name);
    if x {
        t.lands(P0, "Wastes", 3);
        t.answer(P0, DecisionKind::X, Answer::Number(3));
    }
    let c = t.hand(P0, name);
    t.cast_with(P0, c, &[obj(s)]).unwrap();
    cast_new(&mut t, P0, "Counterspell", &[obj(s)]);
    t.resolve(); // Counterspell counters Shock.
    assert!(!t.stack.contains(&s));
    t
}

#[test]
fn soft_counters_with_an_illegal_target_do_nothing() {
    cr!("608.2b", "701.6a");
    ruling!(
        "Mindswipe",
        "If the target spell is an illegal target as Mindswipe tries to resolve (perhaps because it was countered by another spell or ability), Mindswipe won’t resolve and none of its effects will happen. No damage will be dealt."
    );
    ruling!(
        "Spell Contortion",
        "If the targeted spell is an illegal target by the time Spell Contortion resolves, Spell Contortion doesn't resolve. You won't draw any cards."
    );
    let mut t = countered_first("Mindswipe", true);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Mindswipe"));

    let mut t = TestGame::new(2);
    let s = shock(&mut t, P1, P0);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 3);
    let c = t.hand(P0, "Spell Contortion");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.cast_with(P0, c, &[obj(s)]).unwrap();
    cast_new(&mut t, P0, "Counterspell", &[obj(s)]);
    t.resolve();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Spell Contortion"));
}

#[test]
fn spell_rupture_uses_greatest_power_at_resolution_and_zero_can_be_paid() {
    cr!("608.2h", "118.5", "118.12");
    ruling!(
        "Spell Rupture",
        "The value of X is the greatest power among creatures you control when Spell Rupture resolves. If you control no creatures at that time, X will be 0. The target spell’s controller can choose to pay {0} (simply by indicating they wish to do so). That player can also choose to not pay {0} and the spell will be countered."
    );
    supported("Spell Rupture");
    // No creatures: P1 pays {0} (no lands needed), or declines and the spell is countered.
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let s = shock(&mut t, P1, P0);
        cast_new(&mut t, P0, "Spell Rupture", &[obj(s)]);
        t.answer_yes(P1, pay);
        t.resolve();
        assert_eq!(t.stack.contains(&s), pay, "pay {pay}");
    }
    // X is the greatest power when it resolves: Hill Giant (3) arrives after casting; P1
    // has only two lands, so it can't pay.
    let mut t = TestGame::new(2);
    let s = shock(&mut t, P1, P0);
    let wastes = t.lands(P1, "Wastes", 2);
    t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Spell Rupture", &[obj(s)]);
    t.battlefield(P0, "Hill Giant");
    t.answer_yes(P1, true);
    t.resolve();
    assert!(!t.stack.contains(&s));
    assert!(wastes.iter().all(|w| !t.obj(*w).tapped));
}

#[test]
fn izzet_charm_draws_two_then_discards_two_while_resolving() {
    cr!("608.2c", "700.2");
    ruling!(
        "Izzet Charm",
        "If you choose the last mode, you draw two cards and discard two cards all while Izzet Charm is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    supported("Izzet Charm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let c = t.hand(P0, "Izzet Charm");
    t.cast(P0, c).modes(&[2]).go();
    assert_eq!(t.hand_size(P0), 0);
    let gy = t.graveyard_size(P0);
    // At every decision P0 makes while it resolves (the discard), Izzet Charm is still on
    // the stack and both drawn cards are in hand.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |_| true,
        |g| (g.stack.len(), g.player(P0).hand.len()),
    );
    t.resolve();
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty(), "a discard choice was asked");
    assert!(seen.iter().all(|&(stack, hand)| stack == 1 && hand == 2), "{seen:?}");
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), gy + 3);
}

#[test]
fn frightful_delusion_needs_a_target_spell() {
    cr!("601.2c", "115.1");
    ruling!(
        "Frightful Delusion",
        "You must target a spell in order to cast Frightful Delusion. You can’t cast it without a legal target just to make a player discard a card."
    );
    supported("Frightful Delusion");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let c = t.hand(P0, "Frightful Delusion");
    assert!(t.cast_with(P0, c, &[]).is_err());
    assert!(t.in_hand(P0, "Frightful Delusion"));
}

// --- Strict Proctor ------------------------------------------------------------------------

#[test]
fn strict_proctor_s_trigger_goes_on_the_stack_after_the_trigger_it_counters() {
    cr!("603.2", "603.3b");
    ruling!(
        "Strict Proctor",
        "Abilities that trigger when something else causes an ability to trigger will always go on the stack after the ability that caused them to trigger. For example, Strict Proctor’s ability will always resolve before the triggered ability that caused it to trigger does. This is a change from previous rules."
    );
    supported("Strict Proctor");
    supported("Elvish Visionary");
    let mut t = TestGame::new(2);
    let proctor = t.battlefield(P0, "Strict Proctor");
    // Elvish Visionary: "When this creature enters, draw a card."
    let ev = t.enter(P1, "Elvish Visionary");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let top = *t.stack.last().unwrap();
    assert_eq!(
        crate::r_s25_common::abilities_from(&t, proctor),
        vec![top],
        "Proctor's ability is on top"
    );
    let bottom = t.stack[0];
    assert_eq!(crate::r_s25_common::abilities_from(&t, ev), vec![bottom]);
    let hand = t.hand_size(P1);
    t.answer_yes(P1, false);
    t.resolve();
    assert!(!t.stack.contains(&bottom), "the Visionary's trigger was countered");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand);
}

#[test]
fn strict_proctor_ignores_effects_that_modify_how_permanents_enter() {
    cr!("614.1c", "614.12", "603.6a");
    ruling!(
        "Strict Proctor",
        "Effects that modify how a creature enters the battlefield are not triggered abilities and are not affected by Strict Proctor’s ability. These include (but aren’t limited to) effects that have a permanent enter the battlefield as a copy of another permanent, effects that have a permanent enter with counters, and effects that have a permanent enter tapped."
    );
    supported("Gemstone Mine");
    supported("Azorius Guildgate");
    supported("Clone");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Strict Proctor");
    t.battlefield(P0, "Grizzly Bears");
    // Enters with counters.
    let mine = t.enter(P1, "Gemstone Mine");
    t.settle();
    assert_eq!(t.counters(mine, "mining"), 3);
    // Enters tapped.
    let gate = t.enter(P1, "Azorius Guildgate");
    t.settle();
    assert!(t.obj_now(gate).tapped);
    // Enters as a copy.
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    t.answer_choose(P1, &[obj(bears)]);
    t.answer_yes(P1, true);
    let clone = t.enter(P1, "Clone");
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
    assert_eq!(t.stack_len(), 0, "nothing triggered");
}

// --- Declaration of Naught -----------------------------------------------------------------

#[test]
fn declaration_of_naught_naming_one_half_of_a_split_card() {
    cr!("709.3", "702.102d", "201.4b");
    ruling!(
        "Declaration of Naught",
        "You can name either half of a split card, but not both. If a player casts the half with the chosen name, it can be targeted by the second ability. In addition, if a player casts a fused split spell with the chosen name, it can be targeted by the second ability."
    );
    supported("Declaration of Naught");
    supported("Turn // Burn");
    // (how Turn // Burn is cast, its targets, whether Declaration can target it)
    let fused = CastMethod::Keyword(KeywordKind::Fuse);
    for (method, can) in [
        (CastMethod::Half(1), true),
        (fused, true),
        (CastMethod::Half(0), false),
    ] {
        let mut t = TestGame::new(2);
        t.answer(P0, DecisionKind::Name, Answer::Text("Burn".to_string()));
        let decl = t.enter(P0, "Declaration of Naught");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P1, "Volcanic Island", 6);
        let tb = t.hand(P1, "Turn // Burn");
        let targets: Vec<Entity> = match method {
            CastMethod::Half(0) => vec![obj(bears)],
            CastMethod::Half(_) => vec![Entity::Player(P0)],
            _ => vec![obj(bears), Entity::Player(P0)],
        };
        let mut b = t.cast(P1, tb).method(method.clone());
        for e in targets {
            b = b.target(e);
        }
        let spell = b.go();
        let cands = ability_targets(&mut t, decl, "Counter target spell");
        assert_eq!(cands.contains(&obj(spell)), can, "{method:?}");
    }
}
