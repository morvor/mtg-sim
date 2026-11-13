//! Rulings batch P121 — triggered abilities of "+1/+1 counters matter" cards: what
//! triggers them (attacks on players vs. planeswalkers, counters put after attacking,
//! life-gaining events, renown, CR 702.112), how often they do something (one token,
//! card or counter per event, not per counter), casting triggers resolving before the
//! spell (CR 603.3), intervening "if" clauses (CR 603.4) and values counted as an
//! ability resolves (CR 608.2h).

use crate::r_p121_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- what triggers them -------------------------------------------------------------------

#[test]
fn soulblade_corrupter_triggers_for_an_opponent_attacking_another_opponent() {
    cr!("508.1", "603.2", "506.2");
    ruling!(
        "Soulblade Corrupter",
        "Soulblade Corrupter's last ability will trigger if a creature an opponent controls with a +1/+1 counter on it attacks another player who's also your opponent."
    );
    supported("Soulblade Corrupter");
    let mut t = crate::r_p177_common::ffa(3);
    t.battlefield(P0, "Soulblade Corrupter");
    let bears = with_counters(&mut t, P1, "Grizzly Bears", 1);
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P2))]);
    t.resolve_all();
    assert!(has_kw(&mut t, bears, KeywordKind::Deathtouch));
}

#[test]
fn soulblade_corrupter_doesnt_trigger_for_an_attack_on_a_planeswalker() {
    cr!("508.1", "506.2");
    ruling!(
        "Soulblade Corrupter",
        "Soulblade Corrupter's last ability won't trigger if a creature attacks a planeswalker."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Soulblade Corrupter");
    let pw = t.battlefield(P1, "Jace Beleren");
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 1);
    attack_with(&mut t, &[(bears, obj(pw))]);
    assert_eq!(stack(&t), 0);
    assert!(!has_kw(&mut t, bears, KeywordKind::Deathtouch));
}

#[test]
fn soulblade_corrupter_doesnt_trigger_for_a_counter_put_after_attacking() {
    cr!("508.1", "603.2");
    ruling!(
        "Soulblade Corrupter",
        "Soulblade Corrupter's last ability won't trigger if a creature without any +1/+1 counters on it attacks and receives a +1/+1 counter any time after attacking."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Soulblade Corrupter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    give_plus1(&mut t, bears, 1);
    t.resolve_all();
    assert!(!has_kw(&mut t, bears, KeywordKind::Deathtouch));
}

#[test]
fn renown_from_aragorn_triggers_only_on_combat_damage_to_a_player() {
    cr!("702.112a", "510.2");
    ruling!(
        "Aragorn, Hornburg Hero",
        "Renown won't trigger when a creature deals damage to a planeswalker, a battle, or another creature. It also won't trigger when a creature deals noncombat damage to a player."
    );
    supported("Aragorn, Hornburg Hero");
    // Attacking a planeswalker, and being blocked by a creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aragorn, Hornburg Hero");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let pw = t.battlefield(P1, "Jace Beleren");
    let blocker = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(a, obj(pw)), (b, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(blocker, b)]);
    t.resolve_all();
    assert_eq!(t.counters(pw, "loyalty"), 1, "Jace was dealt 2 damage");
    assert_eq!((plus1(&t, a), plus1(&t, b)), (0, 0));
    // Noncombat damage dealt to a player by an attacking creature with renown.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aragorn, Hornburg Hero");
    let a = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.g.deal_damage(a, Entity::Player(P1), 1, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P1), 19);
    assert_eq!(stack(&t), 0);
    assert_eq!(plus1(&t, a), 0);
    // Combat damage to the player does make it renowned.
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(plus1(&t, a), 1);
}

#[test]
fn voice_of_the_blessed_triggers_once_per_life_gaining_event() {
    cr!("119.9", "603.2c");
    ruling!(
        "Voice of the Blessed",
        "The first ability of Voice of the Blessed triggers just once for each life-gaining event, no matter how much life was gained."
    );
    supported("Voice of the Blessed");
    let mut t = TestGame::new(2);
    let voice = t.battlefield(P0, "Voice of the Blessed");
    gain_life(&mut t, P0, 5);
    t.resolve_all();
    assert_eq!(plus1(&t, voice), 1);
}

#[test]
fn herald_of_war_gets_its_counter_before_blockers_and_damage() {
    cr!("508.2", "603.3", "510.1");
    ruling!(
        "Herald of War",
        "The +1/+1 counter is put on Herald of War before blockers are declared and combat damage is dealt."
    );
    supported("Herald of War");
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Herald of War");
    attack_with(&mut t, &[(herald, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(plus1(&t, herald), 1);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn sharktocrab_can_tap_an_already_tapped_creature_which_then_doesnt_untap() {
    cr!("115.1", "502.3");
    ruling!(
        "Sharktocrab",
        "Sharktocrab's triggered ability can target a creature that's already tapped. That creature won't untap during its controller's next untap step."
    );
    supported("Sharktocrab");
    let mut t = TestGame::new(2);
    let crab = t.battlefield(P0, "Sharktocrab");
    let theirs = crate::r_p062_common::tapped_creature(&mut t, P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(theirs)]);
    give_plus1(&mut t, crab, 1);
    t.resolve_all();
    assert!(t.obj_now(theirs).tapped);
    crate::r_p062_common::misses_one_untap(&mut t, theirs, P1);
}

// --- one per event ------------------------------------------------------------------------

#[test]
fn one_token_card_counter_or_life_gain_per_event_not_per_counter() {
    cr!("603.2c");
    ruling!(
        "Rite of the Serpent",
        "Rite of the Serpent won’t create more than one Snake token, even if the target creature had more than one +1/+1 counter on it."
    );
    ruling!(
        "Rite of the Serpent",
        "You get the Snake token no matter who controlled the target creature."
    );
    ruling!(
        "Wildwood Scourge",
        "Wildwood Scourge gets just one +1/+1 counter when its last ability resolves, no matter how many counters were put on the non-Hydra creature."
    );
    ruling!(
        "Swarm Shambler",
        "You create just one Insect token, no matter how many +1/+1 counters the target creature has on it."
    );
    ruling!(
        "Alharu, Solemn Ritualist",
        "You create just one Spirit token, not one for each +1/+1 counter that was on the creature that died."
    );
    ruling!(
        "Skyclave Shadowcat",
        "You draw just one card each time Skyclave Shadowcat’s last ability resolves, no matter how many +1/+1 counters the creature had on it."
    );
    ruling!(
        "Gladehart Cavalry",
        "You gain 2 life per creature, not per +1/+1 counter."
    );
    ruling!(
        "Thoughtbound Phantasm",
        "You put only one +1/+1 counter on Thoughtbound Phantasm each time you surveil, no matter how many cards you looked at when you surveilled."
    );
    for name in [
        "Rite of the Serpent",
        "Wildwood Scourge",
        "Swarm Shambler",
        "Alharu, Solemn Ritualist",
        "Skyclave Shadowcat",
        "Gladehart Cavalry",
        "Thoughtbound Phantasm",
    ] {
        supported(name);
    }
    // Rite of the Serpent on an opponent's creature with three counters.
    let mut t = TestGame::new(2);
    let theirs = with_counters(&mut t, P1, "Grizzly Bears", 3);
    cast_resolve(&mut t, P0, "Rite of the Serpent", &[obj(theirs)]);
    assert_eq!(tokens_named(&t, P0, "Snake"), 1);
    assert_eq!(tokens_named(&t, P1, "Snake"), 0);
    // Wildwood Scourge: three counters on another creature.
    let mut t = TestGame::new(2);
    let scourge = with_counters(&mut t, P0, "Wildwood Scourge", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 3);
    t.resolve_all();
    assert_eq!(plus1(&t, scourge), 2);
    // Swarm Shambler: a creature with three counters targeted by an opponent's spell.
    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Swarm Shambler", 1);
    let bears = with_counters(&mut t, P0, "Hill Giant", 3);
    cast_resolve(&mut t, P1, "Shock", &[obj(bears)]);
    assert_eq!(tokens_named(&t, P0, "Insect"), 1);
    // Alharu: a nontoken creature with three counters dies.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alharu, Solemn Ritualist");
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 3);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Spirit"), 1);
    // Skyclave Shadowcat: a creature with three counters dies.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skyclave Shadowcat");
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 3);
    let hand = t.hand_size(P0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Gladehart Cavalry: two creatures with three counters each die.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gladehart Cavalry");
    with_counters(&mut t, P0, "Grizzly Bears", 3);
    with_counters(&mut t, P0, "Grizzly Bears", 3);
    crate::r_p119_common::wrath(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    // Thoughtbound Phantasm: surveil 2.
    let mut t = TestGame::new(2);
    let ph = t.battlefield(P0, "Thoughtbound Phantasm");
    crate::r_p030_common::surveil(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(plus1(&t, ph), 1);
}

#[test]
fn noosegraf_mob_removes_exactly_one_counter_per_spell() {
    cr!("603.2", "608.2c");
    ruling!(
        "Noosegraf Mob",
        "You can’t remove more than one +1/+1 counter from Noosegraf Mob at once to get more Zombie tokens, and you can’t choose not to remove a counter."
    );
    supported("Noosegraf Mob");
    let mut t = TestGame::new(2);
    let mob = with_counters(&mut t, P0, "Noosegraf Mob", 5);
    cast_new(&mut t, P1, "Opt", &[]);
    t.resolve_all();
    assert_eq!(plus1(&t, mob), 4);
    assert_eq!(tokens_named(&t, P0, "Zombie"), 1);
}

#[test]
fn benthic_biomancer_draws_and_discards_while_resolving() {
    cr!("608.2c", "608.2");
    ruling!(
        "Benthic Biomancer",
        "You draw a card and discard a card all while Benthic Biomancer's triggered ability is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    supported("Benthic Biomancer");
    let mut t = TestGame::new(2);
    let bio = t.battlefield(P0, "Benthic Biomancer");
    assert_eq!(t.hand_size(P0), 0);
    give_plus1(&mut t, bio, 1);
    assert_eq!(stack(&t), 1);
    let from = t.asked().len();
    t.resolve();
    // The drawn card was the only card to discard; nobody got priority in between.
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 1);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Priority { .. })));
}

// --- casting triggers ---------------------------------------------------------------------

#[test]
fn emperor_of_palamecia_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "701.6a");
    ruling!(
        "The Emperor of Palamecia // The Lord Master of Hell",
        "The Emperor of Palamecia's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("The Emperor of Palamecia // The Lord Master of Hell");
    for countered in [false, true] {
        let mut t = TestGame::new(2);
        let emp = t.battlefield(P0, "The Emperor of Palamecia // The Lord Master of Hell");
        let spell = cast_new(&mut t, P0, "Concentrate", &[]);
        t.settle();
        assert_eq!(stack(&t), 2);
        if countered {
            counter_spell(&mut t, spell);
        } else {
            t.resolve();
            assert_eq!(plus1(&t, emp), 1);
            assert_eq!(stack(&t), 1, "Concentrate is still on the stack");
        }
        t.resolve_all();
        assert_eq!(plus1(&t, emp), 1, "countered = {countered}");
    }
}

#[test]
fn zimone_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "701.6a");
    ruling!(
        "Zimone, Infinite Analyst",
        "Zimone's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Zimone, Infinite Analyst");
    for countered in [false, true] {
        let mut t = TestGame::new(2);
        let z = t.battlefield(P0, "Zimone, Infinite Analyst");
        t.lands(P0, "Wastes", 2);
        let one = t.hand(P0, "Endless One");
        let spell = t.cast(P0, one).x(2).go();
        t.settle();
        assert_eq!(stack(&t), 2);
        if countered {
            counter_spell(&mut t, spell);
        } else {
            t.resolve();
            assert_eq!(plus1(&t, z), 2);
            assert_eq!(stack(&t), 1, "Endless One is still on the stack");
        }
        t.resolve_all();
        assert_eq!(plus1(&t, z), 2, "countered = {countered}");
    }
}

#[test]
fn zimone_counters_for_a_spell_dont_reduce_that_spells_cost() {
    cr!("601.2f", "603.3");
    ruling!(
        "Zimone, Infinite Analyst",
        "Since Zimone's last ability won't trigger until you've finished casting your first spell with {X} in its mana cost each turn, the counters from that ability won't be placed on Zimone until after that spell's cost has been determined. The counters put on Zimone from casting your first spell with {X} in its mana cost do not reduce that same spell's cost."
    );
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zimone, Infinite Analyst");
    t.lands(P0, "Wastes", 2);
    let one = t.hand(P0, "Endless One");
    t.cast(P0, one).x(2).go();
    // All of {2} was paid.
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
    assert_eq!(plus1(&t, z), 0);
    t.resolve_all();
    assert_eq!(plus1(&t, z), 2);
}

// --- intervening "if" and values counted on resolution ------------------------------------

#[test]
fn warriors_resolve_checks_the_counters_when_it_would_trigger() {
    cr!("603.4");
    ruling!(
        "Warrior's Resolve",
        "Warrior's Resolve's last ability checks at the moment it would trigger to see if any creatures you currently control with one or more +1/+1 counters on them attacked this turn. If none did, the ability won't trigger at all. It doesn't matter if the creatures had +1/+1 counters on them when they attacked."
    );
    supported("Warrior's Resolve");
    for counter_before in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Warrior's Resolve");
        let bears = t.battlefield(P0, "Grizzly Bears");
        if counter_before {
            give_plus1(&mut t, bears, 1);
        }
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        block_and_finish(&mut t, P1, &[]);
        if counter_before {
            remove_plus1(&mut t, bears, 1);
        } else {
            give_plus1(&mut t, bears, 1);
        }
        let hand = t.hand_size(P0);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(
            t.hand_size(P0),
            hand + usize::from(!counter_before),
            "counter before attacking: {counter_before}"
        );
    }
}

#[test]
fn chronicler_of_heroes_checks_for_a_counter_on_resolution() {
    cr!("608.2c", "603.4");
    ruling!(
        "Chronicler of Heroes",
        "Whether you control a creature with a +1/+1 counter on it is checked only when Chronicler of Heroes's ability resolves."
    );
    supported("Chronicler of Heroes");
    for (before, after) in [(false, true), (true, false)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        if before {
            give_plus1(&mut t, bears, 1);
        }
        t.enter(P0, "Chronicler of Heroes");
        t.settle();
        assert_eq!(stack(&t), 1);
        if after {
            give_plus1(&mut t, bears, 1);
        } else {
            remove_plus1(&mut t, bears, 1);
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + usize::from(after));
    }
}

#[test]
fn armorcraft_judge_counts_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Armorcraft Judge",
        "The number of creatures you control with +1/+1 counters on them is counted only as Armorcraft Judge's triggered ability resolves. Players may respond to the triggered ability by trying to change that number."
    );
    supported("Armorcraft Judge");
    let mut t = TestGame::new(2);
    let a = with_counters(&mut t, P0, "Grizzly Bears", 1);
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Armorcraft Judge");
    t.settle();
    give_plus1(&mut t, b, 1);
    give_plus1(&mut t, c, 1);
    remove_plus1(&mut t, a, 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn canopy_crawler_counts_its_counters_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Canopy Crawler",
        "The number of +1/+1 counters for the ability is checked when the ability resolves."
    );
    supported("Canopy Crawler");
    let mut t = TestGame::new(2);
    let crawler = with_counters(&mut t, P0, "Canopy Crawler", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, crawler, 0, &[obj(bears)]).expect("activate");
    give_plus1(&mut t, crawler, 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
}
