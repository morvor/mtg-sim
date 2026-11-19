//! Rulings batch P103 — triggered abilities that mill: when they trigger (CR 603.2, 603.6,
//! 603.10a), when they're put on the stack and resolve (CR 603.3, 117.5, 601.2i), what
//! they count (CR 608.2h), and the delayed triggers of Duskmantle Guildmage (CR 603.7).

use crate::r_p103_common::*;
use crate::r_p189_common::{set_library, triggers_of};
use crate::r_s01_common::{supported, triggers_on_stack, with_subtype};
use crate::r_s02_common::{can_cast, create_token, destroy};
use crate::r_s07_common::{cast_methods, resolved};
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Mills `n` cards of `p`'s library (as a resolving effect would), then settles.
fn mill(t: &mut TestGame, p: PlayerId, n: u32) {
    crate::r_p104_common::mill(t, p, n);
}

#[test]
fn chronic_flooding_needs_the_land_to_become_tapped() {
    cr!("603.2e", "303.4a");
    ruling!(
        "Chronic Flooding",
        "Chronic Flooding becoming attached to a tapped land won’t cause its ability to immediately trigger. That land must go from untapped to tapped."
    );
    supported("Chronic Flooding");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Island");
    t.g.tap(land);
    cast_new(&mut t, P0, "Chronic Flooding", &[obj(land)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Chronic Flooding").len(), 1);
    assert_eq!(t.graveyard_size(P1), 0);
    t.g.untap(land);
    t.g.tap(land);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 3);
}

#[test]
fn drowned_secrets_triggers_only_on_the_battlefield_and_resolves_first() {
    cr!("603.6a", "603.3", "601.2i");
    ruling!(
        "Drowned Secrets",
        "Drowned Secrets has to be on the battlefield for its ability to trigger. Casting it won’t trigger its own ability."
    );
    ruling!(
        "Drowned Secrets",
        "Drowned Secrets’s triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Drowned Secrets");
    let mut t = TestGame::new(2);
    // Casting Drowned Secrets (a blue spell) doesn't trigger it.
    cast_new(&mut t, P0, "Drowned Secrets", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // A blue spell: the trigger goes on the stack above it.
    t.answer_targets(P0, &[pl(P1)]);
    let opt = cast_new(&mut t, P0, "Opt", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P1 counters Opt; the trigger still resolves.
    cast_new(&mut t, P1, "Counterspell", &[obj(opt)]);
    t.resolve();
    assert_eq!(t.zone(opt), Zone::Graveyard(P0));
    t.resolve_all();
    assert!(!resolved(&t, opt));
    assert_eq!(t.graveyard_size(P1), 3, "two milled cards and Counterspell");
}

#[test]
fn duskmantle_guildmage_each_activation_is_its_own_delayed_trigger() {
    cr!("603.7a", "603.7c", "603.2c");
    ruling!(
        "Duskmantle Guildmage",
        "Each time the first ability resolves, a delayed triggered ability is created. Whenever a card is put into an opponent’s graveyard that turn, each of those abilities will trigger."
    );
    supported("Duskmantle Guildmage");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Duskmantle Guildmage");
    let mana = |t: &mut TestGame, generic: usize| {
        t.lands(P0, "Island", 1);
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Wastes", generic);
    };
    for _ in 0..2 {
        mana(&mut t, 1);
        t.activate(P0, gm, 0, &[]).unwrap();
        t.resolve_all();
    }
    mana(&mut t, 2);
    t.activate(P0, gm, 1, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 2);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn duskmantle_guildmage_tokens_dont_count_and_every_opponent_does() {
    cr!("603.7c", "111.7", "102.2");
    ruling!(
        "Duskmantle Guildmage",
        "The delayed triggered ability created won’t trigger if a token permanent is put into an opponent’s graveyard."
    );
    ruling!(
        "Duskmantle Guildmage",
        "The first ability considers each of your opponents. You don’t choose just one of them."
    );
    let mut t = TestGame::new(3);
    let gm = t.battlefield(P0, "Duskmantle Guildmage");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    let token = create_token(&mut t, P1, "Soldier");
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.activate(P0, gm, 0, &[]).unwrap();
    t.resolve_all();
    // A token dying: no trigger.
    destroy(&mut t, token);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Cards put into either opponent's graveyard, from anywhere.
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P2), 19);
    mill(&mut t, P1, 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Not P0's own graveyard.
    mill(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn draw_triggers_trigger_per_card_after_the_spell_and_not_without_draw() {
    cr!("603.2c", "603.3", "121.2");
    ruling!(
        "Jace's Erasure",
        "If a spell or ability causes you to draw multiple cards, Jace’s Erasure’s ability triggers that many times. (The spell or ability finishes resolving before you put Jace’s Erasure’s abilities on the stack.)"
    );
    ruling!(
        "Jace's Erasure",
        "If a spell or ability causes you to put cards in your hand without specifically using the word “draw,” Jace’s Erasure’s ability won’t trigger."
    );
    ruling!(
        "Psychic Corrosion",
        "If a spell or ability causes you to put cards into your hand without specifically using the word “draw,” Psychic Corrosion’s ability won’t trigger."
    );
    for name in ["Jace's Erasure", "Psychic Corrosion"] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        // Divination: "Draw two cards."
        let div = cast_new(&mut t, P0, "Divination", &[]);
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_targets(P0, &[pl(P1)]);
        t.g.resolve_top();
        // Divination finished resolving before the triggers went on the stack.
        assert_eq!(t.zone(div), Zone::Graveyard(P0));
        assert_eq!(t.stack_len(), 0, "{name}");
        t.settle();
        assert_eq!(triggers_of(&t, src), 2, "{name}");
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.resolve_all();
        let per = if name == "Psychic Corrosion" { 2 } else { 1 };
        assert_eq!(t.graveyard_size(P1), 2 * per, "{name}");
        // Raise Dead returns a card to the hand without drawing it: no trigger.
        let bears = t.graveyard(P0, "Grizzly Bears");
        cast_new(&mut t, P0, "Raise Dead", &[obj(bears)]);
        t.resolve_all();
        assert_eq!(t.zone(bears), Zone::Hand(P0), "{name}");
        assert_eq!(triggers_of(&t, src), 2, "{name}");
    }
}

#[test]
fn circu_triggers_twice_for_a_blue_and_black_spell() {
    cr!("603.2", "105.2", "115.3");
    ruling!(
        "Circu, Dimir Lobotomist",
        "If you cast a blue and black spell, both of Circu's triggered abilities trigger. You can target two different players or target the same player twice."
    );
    supported("Circu, Dimir Lobotomist");
    for targets in [[P1, P2], [P1, P1]] {
        let mut t = TestGame::new(3);
        let circu = t.battlefield(P0, "Circu, Dimir Lobotomist");
        t.answer_targets(P0, &[pl(targets[0])]);
        t.answer_targets(P0, &[pl(targets[1])]);
        // Baleful Strix: a blue and black creature spell.
        cast_new(&mut t, P0, "Baleful Strix", &[]);
        t.settle();
        assert_eq!(triggers_of(&t, circu), 2);
        let (l1, l2) = (t.library_size(P1), t.library_size(P2));
        t.resolve_all();
        let n1 = targets.iter().filter(|p| **p == P1).count();
        let n2 = targets.iter().filter(|p| **p == P2).count();
        assert_eq!(t.library_size(P1), l1 - n1, "{targets:?}");
        assert_eq!(t.library_size(P2), l2 - n2, "{targets:?}");
    }
}

/// Circu exiles the top card of P1's library (`name`), with P0 casting Opt.
fn circu_exiles(t: &mut TestGame, name: &str) -> ObjectId {
    supported("Circu, Dimir Lobotomist");
    t.battlefield(P0, "Circu, Dimir Lobotomist");
    let top = t.library_top(P1, name);
    t.answer_targets(P0, &[pl(P1)]);
    cast_new(t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Exile);
    top
}

#[test]
fn circu_stops_every_opponent_not_just_the_owner() {
    cr!("101.2", "601.3");
    ruling!(
        "Circu, Dimir Lobotomist",
        "Circu's last ability applies to all of its controller's opponents, not just the owner of the exiled card."
    );
    let mut t = TestGame::new(3);
    circu_exiles(&mut t, "Shock");
    for p in [P1, P2] {
        t.lands(p, "Mountain", 1);
        let shock = t.hand(p, "Shock");
        assert!(!can_cast(&mut t, p, shock, CastMethod::Normal), "{p:?}");
    }
    // Circu's controller isn't stopped.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    assert!(can_cast(&mut t, P0, shock, CastMethod::Normal));
}

#[test]
fn circu_stops_both_halves_of_an_exiled_split_card() {
    cr!("601.3", "709.4", "201.2a");
    ruling!(
        "Circu, Dimir Lobotomist",
        "If a split card is exiled this way, opponents can't cast either half of the split card."
    );
    let mut t = TestGame::new(2);
    circu_exiles(&mut t, "Fire // Ice");
    t.lands(P1, "Mountain", 2);
    t.lands(P1, "Island", 2);
    let fi = t.hand(P1, "Fire // Ice");
    assert!(cast_methods(&mut t, P1, fi).is_empty());
    // Without Circu's exile, P1 could cast either half.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Circu, Dimir Lobotomist");
    t.lands(P1, "Mountain", 2);
    t.lands(P1, "Island", 2);
    let fi = t.hand(P1, "Fire // Ice");
    let methods = cast_methods(&mut t, P1, fi);
    assert!(methods.contains(&CastMethod::Half(0)) && methods.contains(&CastMethod::Half(1)));
}

#[test]
fn sacrificing_as_a_cost_puts_the_trigger_above_the_ability() {
    cr!("603.3", "117.5", "602.2b");
    ruling!(
        "Fleeting Memories",
        "If you sacrifice a Clue to pay the cost of a spell or ability, Fleeting Memories's second ability will resolve before that spell or ability."
    );
    ruling!(
        "Scavenger's Talent",
        "If you sacrifice a permanent as part of casting a spell or activating an ability, Scavenger's Talent's level 2 class ability will resolve before that spell or ability."
    );
    for (name, token, milled, payoff) in [
        ("Fleeting Memories", "Clue", 3, "draw"),
        ("Scavenger's Talent", "Food", 2, "life"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        if name == "Scavenger's Talent" {
            mtg_engine::classes::set_level(&mut t.g, src, 2);
            t.g.recompute();
        }
        let tok = create_token(&mut t, P0, token);
        t.lands(P0, "Wastes", 2);
        t.answer_targets(P0, &[pl(P1)]);
        t.activate(P0, tok, 0, &[]).unwrap();
        t.settle();
        assert_eq!(t.stack_len(), 2, "{name}");
        let hand = t.hand_size(P0);
        t.resolve();
        // The trigger resolved first; the token's ability hasn't yet.
        assert_eq!(t.graveyard_size(P1), milled, "{name}");
        assert_eq!(t.stack_len(), 1, "{name}");
        assert_eq!(t.hand_size(P0), hand, "{name}");
        assert_eq!(t.life(P0), 20, "{name}");
        t.resolve();
        match payoff {
            "draw" => assert_eq!(t.hand_size(P0), hand + 1),
            _ => assert_eq!(t.life(P0), 23),
        }
    }
}

#[test]
fn scavengers_talent_sees_itself_sacrificed_with_others() {
    cr!("603.10a", "603.6c", "701.21a");
    ruling!(
        "Scavenger's Talent",
        "Scavenger's Talent's level 2 class ability will trigger when you sacrifice it. If you sacrifice other permanents at the same time, it triggers for them as well."
    );
    supported("Scavenger's Talent");
    let mut t = TestGame::new(2);
    let talent = t.battlefield(P0, "Scavenger's Talent");
    mtg_engine::classes::set_level(&mut t.g, talent, 2);
    t.g.recompute();
    let bears = t.battlefield(P0, "Grizzly Bears");
    // All Is Dust: "Each player sacrifices all permanents they control that are one or
    // more colors."
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    cast_new(&mut t, P0, "All Is Dust", &[]);
    t.resolve();
    assert!(!t.on_battlefield(talent) && !t.on_battlefield(bears));
    // Two sacrifice triggers (and level 1's Food trigger for the Bears dying).
    assert_eq!(triggers_on_stack(&t, "Whenever you sacrifice a permanent"), 2);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 4);
}

#[test]
fn scavengers_talent_may_return_a_creature_it_just_sacrificed() {
    cr!("716.2a", "608.2c", "400.7");
    ruling!(
        "Scavenger's Talent",
        "The creature card you return from your graveyard with Scavenger's Talent's level 3 class ability may be one of the nonland permanents you sacrificed."
    );
    supported("Scavenger's Talent");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Scavenger's Talent");
    mtg_engine::classes::set_level(&mut t.g, class, 3);
    t.g.recompute();
    let giant = t.battlefield(P0, "Hill Giant");
    let fodder = [
        giant,
        t.battlefield(P0, "Ornithopter"),
        t.battlefield(P0, "Ornithopter"),
    ];
    t.answer_yes(P0, true);
    t.answer_choose(P0, &fodder.iter().map(|f| obj(*f)).collect::<Vec<_>>());
    t.answer_choose_moved(P0, &[obj(giant)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.g.current(giant);
    assert_ne!(back, giant);
    assert!(t.on_battlefield(back));
    assert_eq!(t.counters(back, mtg_engine::types::counters::FINALITY), 1);
}

#[test]
fn masters_councillors_triggers_on_the_second_draw_only() {
    cr!("603.2c", "603.2h", "121.2");
    ruling!(
        "Master's Councillors",
        "The triggered ability can trigger only once each turn. It doesn't matter whether Master's Councillors was on the battlefield when the first card was drawn. If it's not on the battlefield when the second card is drawn, the ability can't trigger at all that turn. It won't trigger when the third or fourth card is drawn."
    );
    supported("Master's Councillors");
    let draw = |t: &mut TestGame| {
        t.g.draw_cards(P0, 1);
        t.g.flush_events();
        t.settle();
    };
    // On the battlefield all turn: one trigger, for the second card.
    let mut t = TestGame::new(2);
    let mc = t.battlefield(P0, "Master's Councillors");
    draw(&mut t);
    assert_eq!(triggers_of(&t, mc), 0);
    draw(&mut t);
    assert_eq!(triggers_of(&t, mc), 1);
    draw(&mut t);
    draw(&mut t);
    assert_eq!(triggers_of(&t, mc), 1);
    // Arrives after the first draw: still triggers for the second.
    let mut t = TestGame::new(2);
    draw(&mut t);
    let mc = t.battlefield(P0, "Master's Councillors");
    draw(&mut t);
    assert_eq!(triggers_of(&t, mc), 1);
    // Arrives after the second draw: no trigger that turn.
    let mut t = TestGame::new(2);
    draw(&mut t);
    draw(&mut t);
    let mc = t.battlefield(P0, "Master's Councillors");
    draw(&mut t);
    draw(&mut t);
    assert_eq!(triggers_of(&t, mc), 0);
}

#[test]
fn patient_rebuilding_counts_lands_from_this_resolution_only() {
    cr!("608.2h", "701.17c");
    ruling!(
        "Patient Rebuilding",
        "Each time Patient Rebuilding’s triggered ability resolves, it counts only the land cards put into that graveyard during that ability’s resolution."
    );
    supported("Patient Rebuilding");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Patient Rebuilding");
    for lands in [2usize, 3] {
        if lands == 3 {
            t.advance_to(P1, Step::Upkeep);
        }
        t.advance_to(P0, Step::Upkeep);
        let mut top: Vec<&str> = vec!["Island"; lands];
        top.extend(vec!["Grizzly Bears"; 3 - lands]);
        top.extend(vec!["Grizzly Bears"; 5]);
        set_library(&mut t, P1, &top);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + lands, "{lands}");
    }
}

#[test]
fn patient_rebuilding_draws_nothing_for_cards_exiled_instead() {
    cr!("614.1a", "701.17c");
    ruling!(
        "Patient Rebuilding",
        "If your opponent moves cards from their library but they aren’t put into their graveyard (perhaps because you control Rest in Peace), you won’t draw any cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Patient Rebuilding");
    t.battlefield(P0, "Rest in Peace");
    t.advance_to(P0, Step::Upkeep);
    set_library(&mut t, P1, &["Island", "Island", "Island", "Island"]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 1);
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn altar_of_the_brood_triggers_for_permanents_entering_with_it() {
    cr!("603.6a", "603.2");
    ruling!(
        "Altar of the Brood",
        "If Altar of the Brood enters the battlefield under your control at the same time as other permanents, its ability will trigger for each of those permanents."
    );
    supported("Altar of the Brood");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Altar of the Brood");
    t.graveyard(P0, "Ornithopter");
    t.graveyard(P0, "Memnite");
    // Open the Vaults: "Return all artifact and enchantment cards from all graveyards to
    // the battlefield under their owners' control."
    cast_new(&mut t, P0, "Open the Vaults", &[]);
    t.resolve();
    let altar = t.named_on_battlefield("Altar of the Brood")[0];
    assert_eq!(triggers_of(&t, altar), 2);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn mystic_redaction_in_cleanup_gets_priority_then_another_cleanup() {
    cr!("514.3a", "514.1");
    ruling!(
        "Mystic Redaction",
        "If Mystic Redaction's last ability triggers during the cleanup step because you are discarding due to maximum hand size, players will be able to respond to that ability. Then, when all players have passed priority in succession with the stack empty, a new cleanup step will occur."
    );
    supported("Mystic Redaction");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mystic Redaction");
    for _ in 0..9 {
        t.hand(P0, "Grizzly Bears");
    }
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        |g| (g.turn.step, g.stack.len()),
    );
    // A second cleanup step begins in P0's turn, after the triggers resolved.
    let cleanups = |g: &mtg_engine::game::Game| {
        g.turn_events
            .iter()
            .filter(|e| matches!(e, Event::StepBegan { step: Step::Cleanup, active } if *active == P0))
            .count()
    };
    let ok = t
        .g
        .run_until(10_000, |g| g.turn.active == P0 && cleanups(g) >= 2);
    assert!(ok, "no second cleanup step");
    assert_eq!(t.g.turn.active, P0);
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.graveyard_size(P1), 4);
    assert_eq!(t.stack_len(), 0);
    // P1 got priority in the first cleanup step, with the triggers on the stack.
    let seen = seen.lock().unwrap().clone();
    assert!(seen.iter().any(|(s, n)| *s == Step::Cleanup && *n > 0));
}

#[test]
fn necromaster_dragon_pays_two_once_per_trigger() {
    cr!("603.2", "510.3a");
    ruling!(
        "Necromaster Dragon",
        "Necromaster Dragon’s last ability will trigger once for each player it deals combat damage to. For each of these abilities, you may pay {2} only once."
    );
    supported("Necromaster Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Necromaster Dragon");
    t.lands(P0, "Wastes", 4);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.attack(&[(dragon, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(triggers_of(&t, dragon), 1);
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 1);
    assert_eq!(t.graveyard_size(P1), 2);
    let paid = crate::r_s07_common::count_asked(&t, from, |d| {
        matches!(d, Decision::YesNo { .. } | Decision::OptionalCost { .. })
    });
    assert_eq!(paid, 1);
}

#[test]
fn smeagol_needn_t_have_seen_the_creature_die() {
    cr!("603.4", "603.6a");
    ruling!(
        "Sméagol, Helpful Guide",
        "Sméagol, Helpful Guide doesn't need to have been on the battlefield when the creature died."
    );
    supported("Sméagol, Helpful Guide");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    // Sméagol arrives after the creature died.
    cast_new(&mut t, P0, "Sméagol, Helpful Guide", &[]);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "died under your control"), 1);
}

#[test]
fn enter_the_god_eternals_triggers_wait_until_it_finishes() {
    cr!("603.3", "608.2n", "704.3");
    ruling!(
        "Enter the God-Eternals",
        "Abilities that trigger while Enter the God-Eternals is resolving or that trigger as the target creature dies are all put onto the stack after the spell has finished resolving and the target creature has (if lethally damaged) died."
    );
    let mut t = TestGame::new(2);
    // Ajani's Pridemate triggers on the life gain; Doomed Dissenter dies to the damage.
    t.battlefield(P0, "Ajani's Pridemate");
    let dissenter = t.battlefield(P1, "Doomed Dissenter");
    let spell = cast_new(&mut t, P0, "Enter the God-Eternals", &[obj(dissenter), pl(P1)]);
    t.g.resolve_top();
    // The spell finished (the Army has its counters) before anything was put on the
    // stack; the Dissenter hasn't died yet (state-based actions).
    assert_eq!(t.zone(spell), Zone::Graveyard(P0));
    assert_eq!(t.stack_len(), 0);
    let army = with_subtype(&t, P0, "Army")[0];
    assert_eq!(t.counters(army, "+1/+1"), 4);
    assert!(t.on_battlefield(dissenter));
    t.settle();
    assert_eq!(t.zone(dissenter), Zone::Graveyard(P1));
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn the_master_of_lake_town_counts_its_own_card_in_the_graveyard() {
    cr!("603.10a", "608.2h");
    ruling!(
        "The Master of Lake-town",
        "The Master of Lake-town will usually be in a graveyard as its last ability resolves."
    );
    supported("The Master of Lake-town");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "The Master of Lake-town");
    for _ in 0..6 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let hand = t.hand_size(P0);
    destroy(&mut t, master);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 7);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn tinybones_joins_up_discards_are_chosen_in_turn_order_then_made_together() {
    cr!("101.4", "701.9a", "608.2e");
    ruling!(
        "Tinybones Joins Up",
        "When Tinybones Joins Up’s first ability resolves, the next target player in turn order (or, if it’s a target player’s turn, that player) chooses a card in hand without revealing it, then each other target player in turn order (if any) does the same. All chosen cards are then discarded at the same time."
    );
    supported("Tinybones Joins Up");
    // (active player, targets, the order the targets choose in)
    for (active, targets, order) in [
        (P0, vec![P2, P1], vec![P1, P2]),
        (P1, vec![P0, P2], vec![P2, P0]),
        (P2, vec![P0, P1, P2], vec![P2, P0, P1]),
    ] {
        let mut t = TestGame::new(3);
        t.set_step(active, Step::PrecombatMain);
        let cards: Vec<ObjectId> = [P0, P1, P2]
            .iter()
            .map(|p| t.hand(*p, "Grizzly Bears"))
            .collect();
        for p in [P0, P1, P2] {
            t.hand(p, "Island");
            t.answer_choose(p, &[obj(cards[p.idx()])]);
        }
        // Whenever a player chooses, whether any chosen card has left its hand yet.
        let seen: Vec<_> = [P0, P1, P2]
            .iter()
            .map(|p| {
                crate::r_s01_common::watch(
                    &mut t,
                    *p,
                    |d| matches!(d, Decision::ChooseEntities { .. }),
                    |g| g.players.iter().map(|q| q.graveyard.len()).sum::<usize>(),
                )
            })
            .collect();
        t.answer_targets(P0, &targets.iter().map(|p| pl(*p)).collect::<Vec<_>>());
        let from = t.asked().len();
        t.enter(P0, "Tinybones Joins Up");
        t.g.flush_events();
        t.resolve_all();
        let choosers: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(choosers, order, "{active:?}");
        for p in &targets {
            assert_eq!(t.zone(cards[p.idx()]), Zone::Graveyard(*p));
        }
        // Nothing was discarded until everyone had chosen.
        for s in seen {
            assert!(s.lock().unwrap().iter().all(|n| *n == 0));
        }
    }
}
