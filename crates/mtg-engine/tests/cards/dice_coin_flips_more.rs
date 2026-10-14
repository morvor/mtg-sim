//! Coin flips (CR 705) compiled with `oracle/patterns/dice_coin_ext.rs` and
//! `r705_706_dice.rs`: optional flips, flips by other players, "if both coins come up
//! heads", "for each flip you win", and flips in delayed triggered abilities.

use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn calls(t: &TestGame) -> usize {
    t.asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Call the coin flip"))
        .count()
}

#[test]
fn you_may_flip_a_coin_then_win_or_lose() {
    cr!("705.2");
    compiles("Chaotic Goo");
    compiles("Goblin Bomb");
    // "At the beginning of your upkeep, you may flip a coin. If you win the flip, put a
    // fuse counter on this enchantment. If you lose the flip, remove a fuse counter from
    // this enchantment."
    for (flip, coin, fuses) in [(true, Some(true), 3), (false, None, 2), (true, Some(false), 1)] {
        let mut t = TestGame::new(2);
        let bomb = t.battlefield(P0, "Goblin Bomb");
        t.g.objects[bomb.0 as usize]
            .counters
            .insert("fuse".into(), 2);
        // Call heads (the default).
        t.g.dice.loaded_coins.extend(coin);
        t.answer_yes(P0, flip);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.counters(bomb, "fuse"), fuses, "{flip} {coin:?}");
        // Declining to flip: no flip at all.
        assert_eq!(calls(&t), flip as usize);
    }
}

#[test]
fn both_coins_come_up_heads_or_tails() {
    cr!("705.2");
    compiles("Two-Headed Giant");
    // "Whenever this creature attacks, flip two coins. If both coins come up heads, this
    // creature gains double strike until end of turn. If both coins come up tails, this
    // creature gains menace until end of turn."
    for (coins, ds, menace) in [
        ([true, true], true, false),
        ([false, false], false, true),
        ([true, false], false, false),
    ] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Two-Headed Giant");
        t.g.dice.loaded_coins.extend(coins);
        t.set_step(P0, Step::PrecombatMain);
        t.attack(&[(giant, Entity::Player(P1))], &[]);
        let o = t.obj_now(giant);
        assert_eq!(o.has_keyword(KeywordKind::DoubleStrike), ds, "{coins:?}");
        assert_eq!(o.has_keyword(KeywordKind::Menace), menace, "{coins:?}");
        // Only heads or tails matters: nobody calls these flips.
        assert_eq!(calls(&t), 0);
    }
}

#[test]
fn that_player_flips_a_coin_and_loses() {
    cr!("705.2");
    compiles("Planar Chaos");
    // "Whenever a player casts a spell, that player flips a coin. If they lose the flip,
    // counter that spell."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Planar Chaos");
    t.lands(P1, "Mountain", 1);
    t.set_step(P1, Step::PrecombatMain);
    // P1 calls heads (the default) and the coin comes up tails.
    t.g.dice.loaded_coins.push_back(false);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // The caster called the flip.
    assert!(t.asked().iter().any(|(p, d)| *p == P1
        && matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Call the coin flip")));
    // Winning the flip: the spell resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Planar Chaos");
    t.lands(P1, "Mountain", 1);
    t.set_step(P1, Step::PrecombatMain);
    t.g.dice.loaded_coins.push_back(true);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn each_player_whose_coin_comes_up_tails() {
    cr!("705.2");
    compiles("Goblin Assassin");
    // "Whenever this creature or another Goblin enters, each player flips a coin. Each
    // player whose coin comes up tails sacrifices a creature of their choice."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P0's coin comes up heads, P1's tails.
    t.g.dice.loaded_coins.extend([true, false]);
    let assassin = t.enter(P0, "Goblin Assassin");
    t.resolve_all();
    assert!(t.on_battlefield(assassin));
    assert!(!t.on_battlefield(bears));
    assert_eq!(calls(&t), 0);
}

#[test]
fn for_each_flip_you_win_create_attacking_tokens() {
    cr!("705.2");
    compiles("Goblin Traprunner");
    compiles("Yusri, Fortune's Flame");
    // "Whenever this creature attacks, flip three coins. For each flip you win, create a
    // 1/1 red Goblin creature token that's tapped and attacking."
    let mut t = TestGame::new(2);
    let runner = t.battlefield(P0, "Goblin Traprunner");
    t.g.dice.loaded_coins.extend([true, false, true]);
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(runner, Entity::Player(P1))], &[]);
    let goblins = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Goblin"))
        .count();
    assert_eq!(goblins, 2);
    // The runner (4) and two tokens (1 each) dealt combat damage.
    assert_eq!(t.life(P1), 14);
}

#[test]
fn a_flip_at_the_beginning_of_the_next_end_step() {
    cr!("705.2", "603.7");
    compiles("Goblin Kites");
    compiles("Fickle Efreet");
    // "{R}: Target creature you control with toughness 2 or less gains flying until end of
    // turn. Flip a coin at the beginning of the next end step. If you lose the flip,
    // sacrifice that creature."
    for (heads, survives) in [(true, true), (false, false)] {
        let mut t = TestGame::new(2);
        let kites = t.battlefield(P0, "Goblin Kites");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Mountain", 1);
        t.set_step(P0, Step::PrecombatMain);
        t.activate(P0, kites, 0, &[Entity::Object(bears)]).unwrap();
        t.resolve_all();
        assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
        // No flip yet.
        assert_eq!(calls(&t), 0);
        t.g.dice.loaded_coins.push_back(heads);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(calls(&t), 1);
        assert_eq!(t.on_battlefield(bears), survives);
    }
}
