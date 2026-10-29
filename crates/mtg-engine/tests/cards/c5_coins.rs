//! Coins (cards5-game-rules-dice, `coins`): a flip as the permanent enters choosing what
//! it enters as (Molten Sentry), flipping until a loss or a stop (Fiery Gambit), triggered
//! abilities for the flip's result (Breeches, the Blastmaker), and a source of your choice
//! (Desperate Gambit). Coin results are loaded (`DiceState::loaded_coins`, true = heads);
//! the flipper calls heads unless told otherwise, so heads wins.

use crate::combat2_common::{declare, supported, to_end_of_combat};
use crate::td_next_turn_durations::mana;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn ready(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].tapped = false;
    t.g.objects[id.0 as usize].summoning_sick = false;
}

// ---------------------------------------------------------------------------
// Molten Sentry
// ---------------------------------------------------------------------------

#[test]
fn molten_sentry_enters_as_what_the_coin_says() {
    cr!("614.1c", "705.2", "707.2");
    supported("Molten Sentry");
    for (heads, pt, kw) in [
        (true, (5, 2), KeywordKind::Haste),
        (false, (2, 5), KeywordKind::Defender),
    ] {
        let mut t = TestGame::new(2);
        let s = t.hand(P0, "Molten Sentry");
        mana(&mut t, P0);
        t.g.dice.loaded_coins.push_back(heads);
        t.cast(P0, s).go();
        t.resolve_all();
        let s = t.named_on_battlefield("Molten Sentry")[0];
        assert_eq!(t.pt(s), pt);
        assert!(t.obj_now(s).chars.has_keyword(kw));
        // A flip that only cares about heads or tails has no call (CR 705.2).
        assert!(!t.asked().iter().any(|(_, d)| matches!(d,
            Decision::ChooseOption { prompt, .. } if prompt == "Call the coin flip")));
    }
}

#[test]
fn a_creature_entering_as_a_copy_of_molten_sentry_flips_for_itself() {
    cr!("614.1c", "707.2");
    ruling!(
        "Molten Sentry",
        "If a creature enters copying Molten Sentry, that creature's controller flips a coin"
    );
    let mut t = TestGame::new(2);
    let s = t.hand(P0, "Molten Sentry");
    mana(&mut t, P0);
    t.g.dice.loaded_coins.push_back(true);
    t.cast(P0, s).go();
    t.resolve_all();
    let sentry = t.named_on_battlefield("Molten Sentry")[0];
    t.set_step(P1, Step::PrecombatMain);
    let clone = t.hand(P1, "Clone");
    mana(&mut t, P1);
    t.g.dice.loaded_coins.push_back(false);
    t.answer_choose(P1, &[Entity::Object(sentry)]);
    t.answer_yes(P1, true);
    t.cast(P1, clone).go();
    t.resolve_all();
    let copy = t
        .named_on_battlefield("Molten Sentry")
        .into_iter()
        .find(|o| *o != sentry)
        .expect("the Clone entered as a copy");
    assert_eq!(t.pt(copy), (2, 5));
    assert!(t.obj_now(copy).chars.has_keyword(KeywordKind::Defender));
    assert_eq!(t.pt(sentry), (5, 2));
}

// ---------------------------------------------------------------------------
// Fiery Gambit
// ---------------------------------------------------------------------------

/// Casts Fiery Gambit at P1's Hill Giant with the coins `coins` (true = heads, a win),
/// choosing to flip again `again` times after wins and then to stop.
fn gambit(coins: &[bool], again: usize) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    let lands = t.lands(P0, "Mountain", 3);
    for l in &lands {
        t.g.objects[l.0 as usize].tapped = true;
    }
    let g = t.hand(P0, "Fiery Gambit");
    mana(&mut t, P0);
    t.g.dice.loaded_coins.extend(coins.iter().copied());
    for _ in 0..again {
        t.answer_yes(P0, true);
    }
    t.answer_yes(P0, false);
    t.cast(P0, g).target(giant).go();
    t.resolve_all();
    (t, giant)
}

fn flips(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter(|e| matches!(e, mtg_engine::events::Event::CoinFlipped { .. }))
        .count()
}

#[test]
fn fiery_gambit_counts_the_flips_won_before_you_stop() {
    cr!("705.2", "608.2d");
    supported("Fiery Gambit");
    ruling!("Fiery Gambit", "After each flip, you choose whether to continue flipping.");
    // One win, then stop: 3 damage to the creature.
    let (t, giant) = gambit(&[true], 0);
    assert_eq!(flips(&t), 1);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 20);
    // Two wins: and 6 damage to each opponent.
    let (t, giant) = gambit(&[true, true], 1);
    assert_eq!(flips(&t), 2);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 14);
}

#[test]
fn fiery_gambit_three_wins_draw_nine_and_untap_lands() {
    cr!("705.2");
    ruling!(
        "Fiery Gambit",
        "If you win three flips, Fiery Gambit deals 3 damage to the target creature and 6 damage to each opponent, and you draw nine cards and untap all lands you control."
    );
    let hand = 0;
    let (t, giant) = gambit(&[true, true, true], 2);
    assert_eq!(flips(&t), 3);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.hand_size(P0), hand + 9);
    let lands: Vec<ObjectId> = t.named_on_battlefield("Mountain");
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped));
}

#[test]
fn fiery_gambit_has_no_effect_if_you_lose_a_flip() {
    cr!("705.2");
    ruling!(
        "Fiery Gambit",
        "but Fiery Gambit has no effect if you lose any of the flips. You can’t continue flipping if you lose a flip."
    );
    for (coins, again) in [(&[false][..], 0), (&[true, true, false][..], 2)] {
        let (t, giant) = gambit(coins, again);
        assert_eq!(flips(&t), coins.len(), "no flip after the loss");
        assert!(t.on_battlefield(giant));
        assert_eq!(t.life(P1), 20);
        assert_eq!(t.hand_size(P0), 0);
    }
}

#[test]
fn fiery_gambit_with_an_illegal_target_doesnt_flip() {
    cr!("608.2b");
    ruling!(
        "Fiery Gambit",
        "If that target isn’t legal on resolution, Fiery Gambit has no effect and you don’t even flip a coin."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    let g = t.hand(P0, "Fiery Gambit");
    mana(&mut t, P0);
    t.g.dice.loaded_coins.push_back(true);
    t.cast(P0, g).target(giant).go();
    t.g.move_object(giant, mtg_engine::object::Zone::Graveyard(P1), mtg_engine::events::MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(flips(&t), 0);
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// Breeches, the Blastmaker
// ---------------------------------------------------------------------------

/// Breeches and a Memnite on P0's battlefield; P0 casts Opt, then Shock at P1 (the second
/// spell), choosing whether to sacrifice the Memnite (`sac`) with the coin `coin`.
fn breeches(sac: bool, coin: bool) -> TestGame {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Breeches, the Blastmaker");
    t.battlefield(P0, "Memnite");
    let opt = t.hand(P0, "Opt");
    mana(&mut t, P0);
    t.cast(P0, opt).go();
    t.resolve_all();
    t.g.dice.loaded_coins.push_back(coin);
    let shock = t.hand(P0, "Shock");
    t.answer_yes(P0, sac);
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    // The losing flip's ability: 1 damage (Shock's mana value) to P1.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    t
}

#[test]
fn breeches_copies_the_spell_when_you_win_the_flip() {
    cr!("603.12", "705.2", "707.10");
    supported("Breeches, the Blastmaker");
    ruling!(
        "Breeches, the Blastmaker",
        "It sets up two delayed triggered abilities: one that triggers when you win the flip, and one that triggers when you lose the flip. Only one of these abilities will trigger"
    );
    let t = breeches(true, true);
    assert!(t.in_graveyard(P0, "Memnite"));
    assert_eq!(t.life(P1), 16, "Shock and its copy");
}

#[test]
fn breeches_deals_damage_equal_to_the_mana_value_when_you_lose_the_flip() {
    cr!("603.12", "705.2");
    ruling!(
        "Breeches, the Blastmaker",
        "If you lost the flip, you'll choose a target for that delayed triggered ability at that time."
    );
    let t = breeches(true, false);
    assert_eq!(t.life(P1), 17, "Shock and 1 damage");
}

#[test]
fn breeches_without_a_sacrifice_doesnt_flip() {
    cr!("603.12");
    ruling!(
        "Breeches, the Blastmaker",
        "If you don't sacrifice an artifact, you won't flip a coin, and neither delayed triggered ability will trigger."
    );
    let t = breeches(false, true);
    assert!(t.named_on_battlefield("Memnite").len() == 1);
    assert_eq!(t.g.dice.loaded_coins.len(), 1, "no coin was flipped");
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------
// Desperate Gambit
// ---------------------------------------------------------------------------

#[test]
fn desperate_gambit_win_doubles_the_next_damage_of_a_spell_you_chose() {
    cr!("609.7a", "705.2", "614.1a");
    supported("Desperate Gambit");
    ruling!(
        "Desperate Gambit",
        "The chosen source may be a permanent, a spell on the stack, or anything referred to by something on the stack"
    );
    let mut t = TestGame::new(2);
    let shock = t.hand(P0, "Shock");
    let dg = t.hand(P0, "Desperate Gambit");
    let giant = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0);
    let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.g.dice.loaded_coins.push_back(true);
    t.answer_choose(P0, &[Entity::Object(spell)]);
    t.cast(P0, dg).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "Shock dealt double damage");
    // The sources offered: yours (the spell and the permanent), not the opponent's.
    let offered = t
        .asked()
        .into_iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if p == P0 && prompt == "Choose a source of damage" => Some(candidates),
            _ => None,
        })
        .expect("a source was chosen");
    assert!(offered.contains(&Entity::Object(spell)));
    assert!(offered.contains(&Entity::Object(giant)));
    assert!(!offered.contains(&Entity::Object(theirs)));
}

#[test]
fn desperate_gambit_loss_prevents_the_next_damage_of_the_source() {
    cr!("609.7a", "705.2", "615.1a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    ready(&mut t, giant);
    let dg = t.hand(P0, "Desperate Gambit");
    mana(&mut t, P0);
    t.g.dice.loaded_coins.push_back(false);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, dg).go();
    t.resolve_all();
    declare(&mut t, P0, &[(giant, Entity::Player(P1))]);
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 20, "the combat damage was prevented");
    // Only the next time: a second instance of damage is dealt.
    let mut ctx = mtg_engine::eval::Ctx::new(Some(giant), P0);
    t.g.exec(
        &mtg_engine::ability::Effect::DealDamage {
            source: mtg_engine::ability::Sel::This,
            amount: mtg_engine::ability::Value::c(3),
            to: mtg_engine::ability::Sel::Players(mtg_engine::ability::PlayerRef::Player(P1)),
        },
        &mut ctx,
    );
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_creature_becoming_a_copy_of_molten_sentry_copies_what_it_entered_as() {
    cr!("707.2", "614.1c");
    ruling!(
        "Molten Sentry",
        "If a creature that's already on the battlefield copies Molten Sentry, it becomes a copy of whatever Molten Sentry already is"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let s = t.hand(P0, "Molten Sentry");
    mana(&mut t, P0);
    t.g.dice.loaded_coins.push_back(false);
    t.cast(P0, s).go();
    t.resolve_all();
    let sentry = t.named_on_battlefield("Molten Sentry")[0];
    assert_eq!(t.pt(sentry), (2, 5));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cyto = t.hand(P0, "Cytoshape");
    mana(&mut t, P0);
    // A coin that would make it 5/2 if one were flipped.
    t.g.dice.loaded_coins.push_back(true);
    t.answer_choose(P0, &[Entity::Object(sentry)]);
    t.cast(P0, cyto).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Molten Sentry");
    assert_eq!(t.pt(bears), (2, 5));
    assert!(t.obj_now(bears).chars.has_keyword(KeywordKind::Defender));
    assert!(!t.obj_now(bears).chars.has_keyword(KeywordKind::Haste));
    assert_eq!(t.g.dice.loaded_coins.len(), 1, "no coin was flipped");
}
