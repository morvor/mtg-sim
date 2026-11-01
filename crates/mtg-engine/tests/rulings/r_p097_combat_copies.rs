//! Rulings batch P097 — combat, copies and game-loss rulings among the "low mana value
//! matters" cards: creatures returned or put onto the battlefield during combat weren't
//! declared as attackers (CR 506.3, 508.4), attacking alone (CR 506.5), Dual Strike's
//! delayed copy trigger (CR 707.10, 603.7), and Withengar Unbound's loss trigger
//! (CR 104.3, 800.4).

use crate::r_p097_common::*;
use crate::r_s01_common::{attack_with, block_and_finish};
use mtg_engine::decision::Action;
use mtg_engine::game::GameConfig;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn attacking(t: &TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.g.is_attacking(id)
}

#[test]
fn a_creature_returned_during_combat_isnt_attacking_and_doesnt_trigger_attack_abilities() {
    cr!("506.3", "508.1", "603.2");
    ruling!(
        "Bishop of Rebirth",
        "All attackers are chosen at once. You can’t attack with Bishop of Rebirth, return a creature card to the battlefield, and then attack with that creature."
    );
    ruling!(
        "Bishop of Rebirth",
        "If the creature returned to the battlefield has any abilities that trigger when creatures attack, those abilities won’t trigger."
    );
    ruling!(
        "Devoted Crop-Mate",
        "All attackers are chosen at once. You can't attack with Devoted Crop-Mate, return a creature card to the battlefield, and then attack with that creature."
    );
    for case in ["Bishop of Rebirth", "Devoted Crop-Mate"] {
        supported(case);
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, case);
        // Battle cry: "Whenever this creature attacks, each other attacking creature gets
        // +1/+0 until end of turn."
        let paladin = t.graveyard(P0, "Accorder Paladin");
        let power = t.pt(a).0;
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Object(paladin)]);
        attack_with(&mut t, &[(a, Entity::Player(P1))]);
        t.resolve_all();
        assert!(t.on_battlefield(paladin), "{case}: {}", t.dump_log());
        assert!(!attacking(&t, paladin));
        assert!(attacking(&t, a));
        assert_eq!(t.pt(a).0, power, "{case}: battle cry didn't trigger");
        block_and_finish(&mut t, P1, &[]);
        assert_eq!(t.life(P1), 20 - power);
    }
}

#[test]
fn kinscaer_sentrys_creature_enters_attacking_without_attack_triggers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Kinscaer Sentry",
        "Although the creatures put onto the battlefield with Kinscaer Sentry enter as attacking creatures, they were never declared as attacking creatures. Abilities that trigger whenever a creature attacks won't trigger when the creatures enter attacking."
    );
    supported("Kinscaer Sentry");
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Kinscaer Sentry");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let paladin = t.hand(P0, "Accorder Paladin");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(paladin)]);
    attack_with(
        &mut t,
        &[(sentry, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(attacking(&t, paladin), "{}", t.dump_log());
    assert!(tapped(&t, paladin));
    // No battle cry: the other attackers keep their power.
    assert_eq!(t.pt(sentry).0, 2);
    assert_eq!(t.pt(bears).0, 2);
}

#[test]
fn attacking_alone_means_the_only_creature_declared_as_an_attacker() {
    cr!("506.5", "508.1", "508.4");
    ruling!(
        "Seifer Almasy",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step"
    );
    ruling!(
        "Squall, SeeD Mercenary",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step"
    );
    for case in ["Seifer Almasy", "Squall, SeeD Mercenary"] {
        supported(case);
        // Declared alone: double strike.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, case);
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        t.resolve_all();
        assert!(t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike), "{case}");
        // Two declared, one removed from combat: no trigger.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        t.battlefield(P0, case);
        attack_with(
            &mut t,
            &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
        );
        assert_eq!(t.stack_len(), 0, "{case}");
        t.g.destroy(giant, None);
        t.resolve_all();
        assert!(!t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
    }
}

/// P0 casts Dual Strike ({R}{R}) and has it resolve.
fn dual_strike(t: &mut TestGame) {
    pool(t, P0, &[(ManaType::R, 2)]);
    let s = t.hand(P0, "Dual Strike");
    t.cast(P0, s).go();
    t.resolve_all();
}

#[test]
fn dual_strike_copies_even_if_the_original_is_gone_and_the_copy_resolves_first() {
    cr!("707.10", "603.7c", "405.5");
    ruling!(
        "Dual Strike",
        "A copy is created even if the spell you cast has been countered or otherwise left the stack by the time the copy ability resolves. The copy will resolve before the original spell."
    );
    supported("Dual Strike");
    // The copy resolves first: it is on top of the original.
    let mut t = TestGame::new(2);
    dual_strike(&mut t);
    pool(&mut t, P0, &[(ManaType::R, 1)]);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    let top = *t.g.stack.last().unwrap();
    assert_ne!(top, bolt);
    assert_eq!(t.obj(top).kind, mtg_engine::object::ObjKind::SpellCopy);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.g.stack, vec![bolt]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // The original countered before the trigger resolves: still copied.
    let mut t = TestGame::new(2);
    dual_strike(&mut t);
    pool(&mut t, P0, &[(ManaType::R, 1)]);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(P1).go();
    t.settle();
    t.g.move_object(bolt, Zone::Graveyard(P0), events::MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
}

#[test]
fn dual_strike_copies_any_instant_or_sorcery_with_low_mana_value() {
    cr!("707.10", "202.3", "107.3");
    ruling!(
        "Dual Strike",
        "Dual Strike will copy any instant or sorcery spell with mana value 4 or less, not just one that requires targets."
    );
    ruling!(
        "Dual Strike",
        "If a spell has {X} in its mana cost, use the value of X to determine whether its mana value is 4 or less. If the spell is copied, the copy will have the same value of X."
    );
    // Divination: no targets. Draws four.
    let mut t = TestGame::new(2);
    dual_strike(&mut t);
    pool(&mut t, P0, &[(ManaType::U, 1), (ManaType::C, 2)]);
    let s = t.hand(P0, "Divination");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
    // Blaze ({X}{R}) with X = 3: mana value 4, copied with X = 3. With X = 4, not copied.
    for (x, life) in [(3, 14), (4, 16)] {
        let mut t = TestGame::new(2);
        dual_strike(&mut t);
        pool(&mut t, P0, &[(ManaType::R, 1), (ManaType::C, x as u32)]);
        let s = t.hand(P0, "Blaze");
        t.cast(P0, s).x(x).target(P1).go();
        t.resolve_all();
        assert_eq!(t.life(P1), life, "X={x}: {}", t.dump_log());
    }
}

#[test]
fn withengar_triggers_however_a_player_loses() {
    cr!("104.3a", "104.3b", "104.3c", "104.3d", "104.3e", "603.2");
    ruling!(
        "Elbrus, the Binding Blade // Withengar Unbound",
        "Withengar's triggered ability will trigger no matter how a player loses the game"
    );
    supported("Elbrus, the Binding Blade // Withengar Unbound");
    let mut t = TestGame::new(7);
    // (Seven players: the game goes on after five of them lose.)
    let w = t.battlefield(P0, "Elbrus, the Binding Blade // Withengar Unbound");
    mtg_engine::dfc::transform(&mut t.g, w);
    t.g.recompute();
    assert_eq!(t.obj_now(w).chars.name.as_str(), "Withengar Unbound");
    let w = t.g.current(w);
    // Life 0 (SBA).
    t.g.players[1].life = 0;
    t.resolve_all();
    assert!(t.has_lost(P1));
    assert_eq!(t.counters(w, "+1/+1"), 13);
    // Ten poison counters (SBA).
    t.g.players[2].counters.insert("poison".into(), 10);
    t.resolve_all();
    assert!(t.has_lost(PlayerId(2)));
    assert_eq!(t.counters(w, "+1/+1"), 26);
    // A concession.
    t.g.turn.priority = Some(PlayerId(3));
    t.g.perform_action(PlayerId(3), Action::Concede).unwrap();
    t.resolve_all();
    assert!(t.has_lost(PlayerId(3)));
    assert_eq!(t.counters(w, "+1/+1"), 39);
    // An effect that says the player loses the game.
    t.g.lose_game(PlayerId(4));
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 52);
    // Drawing from an empty library (SBA).
    let n = t.library_size(PlayerId(5)) as u32;
    t.g.draw_cards(PlayerId(5), n + 1);
    t.resolve_all();
    assert!(t.has_lost(PlayerId(5)));
    assert_eq!(t.counters(w, "+1/+1"), 65);
}

#[test]
fn withengar_triggers_when_a_win_makes_players_in_range_lose() {
    cr!("104.3h", "801.14", "801.7", "603.2");
    ruling!(
        "Elbrus, the Binding Blade // Withengar Unbound",
        "In a multiplayer game using the limited range of influence option (such as a Grand Melee game), if a spell or ability says that you win the game, it instead causes all of your opponents within your range of influence to lose the game. This is another way by which Withengar's ability can trigger."
    );
    supported("Felidar Sovereign");
    let config = GameConfig {
        range_of_influence: Some(1),
        ..GameConfig::free_for_all()
    };
    let mut t = TestGame::with_config(5, config);
    // P1's range of influence covers P0 and P2; P3's covers P2 and P4.
    let withengar = |t: &mut TestGame, p: PlayerId| {
        let w = t.battlefield(p, "Elbrus, the Binding Blade // Withengar Unbound");
        mtg_engine::dfc::transform(&mut t.g, w);
        t.g.recompute();
        t.g.current(w)
    };
    let w = withengar(&mut t, P1);
    let w3 = withengar(&mut t, PlayerId(3));
    t.battlefield(P1, "Felidar Sovereign");
    t.g.players[1].life = 40;
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.has_lost(P0) && t.has_lost(P2), "{}", t.dump_log());
    assert!(!t.has_lost(P1) && !t.has_lost(PlayerId(3)));
    // P3's Withengar sees only P2's loss (CR 801.7).
    assert_eq!(t.counters(w3, "+1/+1"), 13);
    assert_eq!(t.counters(w, "+1/+1"), 26, "{}", t.dump_log());
}

#[test]
fn elbrus_and_withengar_have_different_names_for_the_legend_rule() {
    cr!("704.5j", "201.2");
    ruling!(
        "Elbrus, the Binding Blade // Withengar Unbound",
        "The \"legend rule\" checks only for permanents with exactly the same name."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Elbrus, the Binding Blade // Withengar Unbound");
    let b = t.battlefield(P0, "Elbrus, the Binding Blade // Withengar Unbound");
    mtg_engine::dfc::transform(&mut t.g, b);
    t.g.recompute();
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    assert_eq!(t.obj_now(b).chars.name.as_str(), "Withengar Unbound");
    // Two Elbruses: one goes.
    let c = t.battlefield(P0, "Elbrus, the Binding Blade // Withengar Unbound");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.settle();
    assert_eq!(t.on_battlefield(a) as u8 + t.on_battlefield(c) as u8, 1);
}

