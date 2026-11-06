//! Rulings batch P007 — burn with several targets or a player target: effects that
//! target only the player still affect that player's hexproof creatures (CR 115.10), the
//! spell or ability resolves for the targets still legal (CR 608.2b), one-sided fights
//! deal no damage if either target is illegal, and spells with all targets illegal don't
//! resolve.

use crate::r_p007_common::*;
use crate::r_p113_common::{spoil, Spoil};
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s06_common::damage;
use crate::r_s07_common::{damage_on, resolved};
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

#[test]
fn player_targeted_sweeps_hit_hexproof_and_shroud_creatures() {
    cr!("115.10", "702.11b", "702.18a");
    ruling!(
        "Bonfire of the Damned",
        "Bonfire of the Damned targets only the player. Creatures with shroud or hexproof that player controls will be dealt damage"
    );
    ruling!(
        "Chandra's Fury",
        "Chandra’s Fury targets only the player, not any creature. Chandra’s Fury will deal 1 damage to a creature with hexproof"
    );
    ruling!(
        "Lavalanche",
        "Because Lavalanche doesn't target any creatures, it will deal damage to creatures with shroud. Any damage it would deal to creatures with appropriate protection abilities will still be prevented"
    );
    // (spell, X, extra lands for X)
    for (name, x, extra) in [
        ("Bonfire of the Damned", Some(1), 1usize),
        ("Chandra's Fury", None, 0),
        ("Lavalanche", Some(1), 0),
    ] {
        let mut t = TestGame::new(2);
        let bogle = t.battlefield(P1, "Slippery Bogle");
        let shroud = t.battlefield(P1, "Gladecover Scout");
        let pro_red = t.battlefield(P1, "Kor Firewalker");
        t.lands(P0, "Wastes", extra);
        cast_x(&mut t, P0, name, NORMAL, x, &[pl(P1)]);
        // (Kor Firewalker's "you may gain 1 life" is declined.)
        t.answer_yes(P1, false);
        t.resolve_all();
        assert!(
            !t.on_battlefield(bogle),
            "{name}: hexproof creature dealt damage"
        );
        assert!(
            !t.on_battlefield(shroud),
            "{name}: shroud creature dealt damage"
        );
        assert!(
            t.on_battlefield(pro_red),
            "{name}: protection from red prevents it"
        );
        assert_eq!(damage_on(&t, pro_red), 0, "{name}");
        assert!(t.life(P1) < 20, "{name}");
    }
}

#[test]
fn planeswalker_abilities_targeting_only_the_player_hit_hexproof_permanents() {
    cr!("115.10", "702.11b");
    ruling!(
        "Angrath, Minotaur Pirate",
        "Angrath's first and last abilities target only the player. Creatures with hexproof that player controls will be affected."
    );
    ruling!(
        "Chandra, Bold Pyromancer",
        "Chandra's last ability targets only the player. Creatures and planeswalkers that player controls with hexproof will be dealt damage."
    );
    supported("Angrath, Minotaur Pirate");
    supported("Chandra, Bold Pyromancer");
    // Angrath +2: 1 damage to the player and each of their creatures.
    let mut t = TestGame::new(2);
    let bogle = t.battlefield(P1, "Slippery Bogle");
    let a = pw(&mut t, P0, "Angrath, Minotaur Pirate", 5);
    loyalty(&mut t, P0, a, 0, &[pl(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bogle));
    assert_eq!(t.life(P1), 19);
    // Angrath −11: destroys the hexproof creatures; damage equal to their total power.
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P1, "Thrun, the Last Troll");
    let a = pw(&mut t, P0, "Angrath, Minotaur Pirate", 11);
    loyalty(&mut t, P0, a, 2, &[pl(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(troll));
    assert_eq!(t.life(P1), 16, "Thrun's power 4");
    // Chandra −7: 10 damage to the player and each creature and planeswalker they
    // control, hexproof or not.
    let mut t = TestGame::new(2);
    let bogle = t.battlefield(P1, "Slippery Bogle");
    let theirs = pw(&mut t, P1, "Chandra, Bold Pyromancer", 5);
    spoil(&mut t, theirs, Spoil::Hexproof);
    let c = pw(&mut t, P0, "Chandra, Bold Pyromancer", 7);
    loyalty(&mut t, P0, c, 2, &[pl(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bogle));
    assert!(
        !t.on_battlefield(theirs),
        "the hexproof planeswalker is dealt 10"
    );
    assert_eq!(t.life(P1), 10);
}

#[test]
fn angrath_ultimate_kills_before_damage_without_triggering_their_creatures() {
    cr!("603.2", "603.3", "704.5a");
    ruling!(
        "Angrath, Minotaur Pirate",
        "Any abilities of the destroyed creatures that trigger when the player is dealt damage won't trigger."
    );
    ruling!(
        "Angrath, Minotaur Pirate",
        "Any abilities that trigger when creatures die while resolving Angrath's last ability won't be put on the stack until after the player is dealt damage. If the player's life total becomes 0 or less, those triggers won't resolve in time to save that player."
    );
    // Darien ("whenever you're dealt damage, create that many Soldiers") is destroyed first.
    let mut t = TestGame::new(2);
    let darien = t.battlefield(P1, "Darien, King of Kjeldor");
    let a = pw(&mut t, P0, "Angrath, Minotaur Pirate", 11);
    loyalty(&mut t, P0, a, 2, &[pl(P1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(darien));
    assert_eq!(t.life(P1), 17);
    assert!(
        crate::r_s01_common::tokens(&t, P1).is_empty(),
        "Darien's trigger doesn't trigger"
    );
    // Guardian Automaton ("when this dies, you gain 3 life") can't save a player at 3.
    let mut t = TestGame::new(2);
    t.g.players[1].life = 3;
    let auto = t.battlefield(P1, "Guardian Automaton");
    let a = pw(&mut t, P0, "Angrath, Minotaur Pirate", 11);
    loyalty(&mut t, P0, a, 2, &[pl(P1)]);
    t.resolve();
    assert!(!t.on_battlefield(auto));
    assert!(
        t.has_lost(P1),
        "the player loses before the dies trigger resolves"
    );
}

// ---------------------------------------------------------------------------
// One-sided fights: "target creature you control deals damage equal to its power to ..."
// ---------------------------------------------------------------------------

/// How a bite is started: a spell (name, method) or Vivien's −3.
#[derive(Clone, Copy)]
enum Bite {
    /// (name, the split card half cast, if any)
    Spell(&'static str, Option<usize>),
    Vivien,
}

/// P0's Hill Giant (3/3) bites P1's Colossal Dreadmaw (6/6); `bad` makes one target
/// illegal in response. Returns (game, mine, theirs).
fn bite(how: Bite, bad: Option<(bool, Spoil)>, eldrazi: bool) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Colossal Dreadmaw");
    if eldrazi {
        t.battlefield(P0, "Ulamog's Crusher");
    }
    match how {
        Bite::Spell(name, half) => {
            let method = half.map_or(NORMAL, |h| CastMethod::Half(h as u8));
            cast_x(&mut t, P0, name, method, None, &[obj(mine), obj(theirs)]);
        }
        Bite::Vivien => {
            supported("Vivien, Arkbow Ranger");
            let v = pw(&mut t, P0, "Vivien, Arkbow Ranger", 4);
            loyalty(&mut t, P0, v, 1, &[obj(mine), obj(theirs)]);
        }
    }
    if let Some((mine_bad, s)) = bad {
        spoil(&mut t, if mine_bad { mine } else { theirs }, s);
    }
    t.resolve_all();
    (t, mine, theirs)
}

#[test]
fn bites_deal_no_damage_if_either_target_is_illegal() {
    cr!("608.2b", "115.1");
    ruling!(
        "Bite Down",
        "If either target is an illegal target as Bite Down tries to resolve, the creature you control won't deal damage."
    );
    ruling!(
        "Cosmic Hunger",
        "If either target is an illegal target as Cosmic Hunger tries to resolve, the creature you control won’t deal damage."
    );
    ruling!(
        "Domri's Ambush",
        "If either target is an illegal target as Domri’s Ambush tries to resolve, the creature you control won’t deal damage."
    );
    ruling!(
        "Horrific Assault",
        "If either target is an illegal target as Horrific Assault tries to resolve, the creature you control won't deal damage. As long as either target is still legal and you control an Eldrazi, you'll still gain 3 life."
    );
    ruling!(
        "Stump Stomp // Burnwillow Clearing",
        "If either target is an illegal target as Stump Stomp tries to resolve, the creature you control won't deal damage."
    );
    ruling!(
        "Thrash // Threat",
        "If either target is an illegal target as Thrash tries to resolve, the creature you control won't deal damage."
    );
    ruling!(
        "Vivien, Arkbow Ranger",
        "If either target is an illegal target as Vivien’s second ability tries to resolve, the creature you control won’t deal damage."
    );
    // (bite, damage with both targets legal)
    let cases = [
        (Bite::Spell("Bite Down", None), 3),
        (Bite::Spell("Cosmic Hunger", None), 3),
        (Bite::Spell("Domri's Ambush", None), 4),
        (Bite::Spell("Horrific Assault", None), 3),
        (Bite::Spell("Stump Stomp // Burnwillow Clearing", None), 3),
        (Bite::Spell("Thrash // Threat", Some(0)), 3),
        (Bite::Vivien, 3),
    ];
    for (i, (how, n)) in cases.into_iter().enumerate() {
        let (t, mine, theirs) = bite(how, None, false);
        assert_eq!(damage_on(&t, theirs), n, "case {i}");
        assert_eq!(damage_on(&t, mine), 0, "case {i}");
        for s in [Spoil::Gone, Spoil::Hexproof, Spoil::Stolen] {
            let (t, _, theirs) = bite(how, Some((true, s)), false);
            assert_eq!(damage_on(&t, theirs), 0, "case {i} mine {s:?}");
        }
        for s in [Spoil::Gone, Spoil::Hexproof] {
            let (t, mine, _) = bite(how, Some((false, s)), false);
            assert_eq!(damage_on(&t, mine), 0, "case {i} theirs {s:?}");
            assert_eq!(t.life(P1), 20, "case {i} theirs {s:?}");
        }
    }
    // Domri's Ambush: the counter is still put on the creature you control.
    let (t, mine, _) = bite(
        Bite::Spell("Domri's Ambush", None),
        Some((false, Spoil::Hexproof)),
        false,
    );
    assert_eq!(t.pt(mine), (4, 4));
    // Horrific Assault with an Eldrazi: 3 life as long as either target is legal.
    let how = Bite::Spell("Horrific Assault", None);
    let (t, _, theirs) = bite(how, None, true);
    assert_eq!((damage_on(&t, theirs), t.life(P0)), (3, 23));
    let (t, _, theirs) = bite(how, Some((true, Spoil::Hexproof)), true);
    assert_eq!((damage_on(&t, theirs), t.life(P0)), (0, 23));
    let (t, mine, _) = bite(how, Some((false, Spoil::Gone)), true);
    assert_eq!((damage_on(&t, mine), t.life(P0)), (0, 23));
    // Both illegal: it doesn't resolve, so no life.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P0, "Ulamog's Crusher");
    let s = cast_x(
        &mut t,
        P0,
        "Horrific Assault",
        NORMAL,
        None,
        &[obj(mine), obj(theirs)],
    );
    spoil(&mut t, mine, Spoil::Gone);
    spoil(&mut t, theirs, Spoil::Gone);
    t.resolve_all();
    assert!(!resolved(&t, s));
    assert_eq!(t.life(P0), 20);
}

// ---------------------------------------------------------------------------
// Two or more targets: the legal ones are still affected.
// ---------------------------------------------------------------------------

#[test]
fn two_target_burn_still_affects_the_legal_target() {
    cr!("608.2b");
    interp!("dependent-target-checked-against-an-illegal-target");
    ruling!(
        "Punish the Enemy",
        "If one of Punish the Enemy’s targets is illegal when it tries to resolve, it will still deal damage to the remaining legal target."
    );
    ruling!(
        "Searing Blaze",
        "If either target is illegal by the time Searing Blaze resolves, it still deals damage to the other target."
    );
    ruling!(
        "Burning Sun's Avatar",
        "If the ability of Burning Sun’s Avatar has two targets and one becomes illegal, the other will still be dealt damage."
    );
    // (spell, damage to each target, player target first)
    for (name, n) in [("Punish the Enemy", 3), ("Searing Blaze", 1)] {
        // The creature is gone: the player is still dealt damage.
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        cast_x(&mut t, P0, name, NORMAL, None, &[pl(P1), obj(wurm)]);
        destroy(&mut t, wurm);
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - n, "{name}");
        // The player has hexproof: the creature is still dealt damage.
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        cast_x(&mut t, P0, name, NORMAL, None, &[pl(P1), obj(wurm)]);
        player_hexproof(&mut t, P1);
        t.resolve_all();
        assert_eq!((t.life(P1), dmg(&t, wurm)), (20, n as u32), "{name}");
    }
    // Burning Sun's Avatar: 3 to the opponent and 3 to up to one creature.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    supported("Burning Sun's Avatar");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Burning Sun's Avatar");
    t.settle();
    destroy(&mut t, wurm);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Burning Sun's Avatar");
    t.settle();
    player_hexproof(&mut t, P1);
    t.resolve_all();
    assert_eq!((t.life(P1), dmg(&t, wurm)), (20, 3));
}

#[test]
fn manticore_and_angraths_fury_with_one_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Manticore of the Gauntlet",
        "If either target becomes illegal after Manticore of the Gauntlet's triggered ability is put onto the stack but before it resolves, the other is still affected as appropriate."
    );
    ruling!(
        "Angrath's Fury",
        "If either target becomes illegal after you cast Angrath’s Fury but before it resolves, the other is still affected as appropriate and you’ll search for Angrath. However, if both targets become illegal, the spell doesn’t resolve and you won’t search."
    );
    supported("Manticore of the Gauntlet");
    // Manticore: the creature is gone, the opponent is still dealt 3.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Manticore of the Gauntlet");
    t.settle();
    destroy(&mut t, mine);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // The opponent has hexproof: the -1/-1 counter is still put on the creature.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Manticore of the Gauntlet");
    t.settle();
    player_hexproof(&mut t, P1);
    t.resolve_all();
    assert_eq!((t.life(P1), t.pt(mine)), (20, (2, 2)));

    // Angrath's Fury: with one target illegal, the other is affected and you search.
    for creature_gone in [true, false] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        let angrath = t.library_top(P0, "Angrath, Minotaur Pirate");
        let s = cast_x(
            &mut t,
            P0,
            "Angrath's Fury",
            NORMAL,
            None,
            &[obj(wurm), pl(P1)],
        );
        if creature_gone {
            spoil(&mut t, wurm, Spoil::Gone);
        } else {
            player_hexproof(&mut t, P1);
        }
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(angrath)]);
        t.resolve_all();
        assert!(resolved(&t, s));
        assert!(!t.on_battlefield(wurm));
        assert_eq!(t.life(P1), if creature_gone { 17 } else { 20 });
        assert!(
            t.in_hand(P0, "Angrath, Minotaur Pirate"),
            "creature_gone {creature_gone}"
        );
    }
    // Both illegal: it doesn't resolve and you don't search.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let angrath = t.library_top(P0, "Angrath, Minotaur Pirate");
    let s = cast_x(
        &mut t,
        P0,
        "Angrath's Fury",
        NORMAL,
        None,
        &[obj(wurm), pl(P1)],
    );
    spoil(&mut t, wurm, Spoil::Gone);
    player_hexproof(&mut t, P1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(angrath)]);
    t.resolve_all();
    assert!(!resolved(&t, s));
    assert!(!t.in_hand(P0, "Angrath, Minotaur Pirate"));
}

#[test]
fn channeled_force_and_tandem_takedown_with_one_illegal_target() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Channeled Force",
        "If one of Channeled Force’s two targets becomes illegal, the other is still affected as appropriate."
    );
    ruling!(
        "Tandem Takedown",
        "If one of the two target creatures you control is an illegal target as Tandem Takedown resolves, the other will still get +1/+0 and then deal damage equal to its power."
    );
    // Channeled Force, X = 2: the creature is gone, P1 still draws two.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![obj(a), obj(b)]),
    );
    cast_x(
        &mut t,
        P0,
        "Channeled Force",
        NORMAL,
        Some(2),
        &[pl(P1), obj(wurm)],
    );
    let hand = t.hand_size(P1);
    destroy(&mut t, wurm);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 2);
    // The player is illegal: the creature is still dealt 2.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![obj(a), obj(b)]),
    );
    cast_x(
        &mut t,
        P0,
        "Channeled Force",
        NORMAL,
        Some(2),
        &[pl(P1), obj(wurm)],
    );
    let hand = t.hand_size(P1);
    player_hexproof(&mut t, P1);
    t.resolve_all();
    assert_eq!((t.hand_size(P1), dmg(&t, wurm)), (hand, 2));

    // Tandem Takedown: one of the two creatures you control is gone.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    cast_x(&mut t, P0, "Tandem Takedown", NORMAL, None, &[obj(wurm)]);
    spoil(&mut t, a, Spoil::Hexproof);
    t.resolve_all();
    assert_eq!(t.pt(b), (4, 3));
    assert_eq!(t.pt(a), (2, 2));
    assert_eq!(dmg(&t, wurm), 0, "Craw Wurm (6/4) is dealt 4 and dies");
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn vivien_counters_for_an_illegal_target_are_lost() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Vivien, Arkbow Ranger",
        "If one of the two target creatures becomes an illegal target in response to Vivien’s first ability, the +1/+1 counter that would have been put on that creature is lost. It can’t be put on the legal target."
    );
    supported("Vivien, Arkbow Ranger");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let v = pw(&mut t, P0, "Vivien, Arkbow Ranger", 4);
    t.answer_targets(P0, &[obj(a), obj(b)]);
    divide(&mut t, P0, &[1, 1]);
    loyalty(&mut t, P0, v, 0, &[]);
    spoil(&mut t, a, Spoil::Hexproof);
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 0);
    assert_eq!(
        t.counters(b, "+1/+1"),
        1,
        "the lost counter isn't moved to b"
    );
}

// ---------------------------------------------------------------------------
// One target: illegal means the spell or ability doesn't resolve.
// ---------------------------------------------------------------------------

#[test]
fn single_target_spells_dont_resolve_if_the_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Exploding Borders",
        "Exploding Borders has only one target: the player. If that player becomes an illegal target by the time Exploding Borders would resolve, the entire spell doesn't resolve. You won't get to search for a basic land card."
    );
    ruling!(
        "Heated Debate",
        "If Heated Debate’s target is illegal (perhaps because it gained protection from red in response) when it tries to resolve, it will be removed from the stack and no damage will be dealt."
    );
    ruling!(
        "Unstable Footing",
        "If Unstable Footing is kicked and the targeted player is an illegal target by the time it resolves, the entire spell doesn't resolve. Damage can still be prevented."
    );
    ruling!(
        "Nicol Bolas, Planeswalker",
        "For the third ability, if the player is an illegal target when the ability tries to resolve, the ability doesn't resolve and none of its effects will happen."
    );
    // Exploding Borders: no search.
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let s = cast_x(&mut t, P0, "Exploding Borders", NORMAL, None, &[pl(P1)]);
    player_hexproof(&mut t, P1);
    t.answer_choose(P0, &[obj(forest)]);
    t.resolve_all();
    assert!(!resolved(&t, s));
    assert_eq!(t.zone(forest), mtg_engine::object::Zone::Library(P0));
    assert_eq!(t.life(P1), 20);
    // Heated Debate: the creature gains hexproof (protection from red works the same way).
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let s = cast_x(&mut t, P0, "Heated Debate", NORMAL, None, &[obj(wurm)]);
    spoil(&mut t, wurm, Spoil::Hexproof);
    t.resolve_all();
    assert!(!resolved(&t, s));
    assert_eq!(dmg(&t, wurm), 0);
    assert!(t.in_graveyard(P0, "Heated Debate"));
    // Unstable Footing, kicked: doesn't resolve, so damage can still be prevented.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let specter = t.battlefield(P0, "Hypnotic Specter");
    supported("Unstable Footing");
    t.lands(P0, "Mountain", 5);
    let card = t.hand(P0, "Unstable Footing");
    t.answer_targets(P0, &[pl(P1)]);
    let s = t.cast(P0, card).kicked(true).go();
    player_hexproof(&mut t, P1);
    t.resolve_all();
    assert!(!resolved(&t, s));
    assert_eq!(t.life(P1), 20);
    damage(&mut t, specter, 2, knight);
    assert_eq!(
        dmg(&t, knight),
        0,
        "protection from black still prevents damage"
    );
    // Nicol Bolas −9: no damage, no discard, no sacrifice.
    let mut t = TestGame::new(2);
    supported("Nicol Bolas, Planeswalker");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.hand(P1, "Grizzly Bears");
    let b = pw(&mut t, P0, "Nicol Bolas, Planeswalker", 9);
    loyalty(&mut t, P0, b, 2, &[pl(P1)]);
    player_hexproof(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.on_battlefield(wurm));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn exploding_borders_deals_damage_without_a_land() {
    cr!("608.2c", "701.23b");
    ruling!(
        "Exploding Borders",
        "Exploding Borders will still deal damage even if you don't put a land card onto the battlefield."
    );
    // No basic land in the library; P0 controls a Mountain and a Forest among its lands.
    let mut t = TestGame::new(2);
    cast_x(&mut t, P0, "Exploding Borders", NORMAL, None, &[pl(P1)]);
    t.resolve_all();
    // lands_for_cost paid {2}{R}{G} with Wastes, Wastes, Mountain, Forest: 2 types.
    assert_eq!(t.life(P1), 18);
}
