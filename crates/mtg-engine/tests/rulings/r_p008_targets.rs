//! Rulings batch P008 — burn that can hit planeswalkers, with targets that become illegal:
//! a spell or ability whose only target (or every target) is illegal doesn't resolve and
//! none of its other effects happen (CR 608.2b); with some targets still legal, it
//! resolves and affects them; a triggered ability with no legal target isn't put on the
//! stack (CR 603.3d).

use crate::r_p007_common::*;
use crate::r_p008_common::*;
use crate::r_p113_common::{cast_modes, spoil, Spoil};
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s05_common::move_to;
use crate::r_s07_common::resolved;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

/// A single-target spell aimed at P1's creature whose target gains hexproof before it
/// resolves: it doesn't resolve, deals no damage and its other effects don't happen.
#[test]
fn creature_or_planeswalker_burn_with_a_rider_doesnt_resolve() {
    cr!("608.2b");
    ruling!(
        "Broadside Barrage",
        "If the creature or planeswalker isn’t a legal target as this spell tries to resolve, it won’t resolve. You won’t get to draw a card and discard a card."
    );
    ruling!(
        "Consuming Corruption",
        "If the target creature or planeswalker is an illegal target as Consuming Corruption tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    ruling!(
        "Tundra Fumarole",
        "If the target creature or planeswalker is an illegal target as Tundra Fumarole tries to resolve, it won’t resolve and none of its effects will happen. You won’t add mana."
    );
    ruling!(
        "Fry",
        "If the target creature or planeswalker is an illegal target by the time Fry tries to resolve, the spell won’t resolve and will be put into its owner’s graveyard with no effect. This is true even though it can’t be countered."
    );
    ruling!(
        "Volcanic Spite",
        "If the target creature, planeswalker, or battle is an illegal target at the time Volcanic Spite tries to resolve, it won’t resolve and none of its effects will happen. You won’t get to put a card from your hand on the bottom of your library, and you won’t draw a card."
    );
    ruling!(
        "Defibrillating Current",
        "If the target permanent is an illegal target as Defibrillating Current tries to resolve, it won’t resolve and none of its effects will happen. You won’t gain life."
    );
    // (spell, lands to pay with, the creature targeted)
    let cases: [(&str, &[&str], &str); 6] = [
        ("Broadside Barrage", &["Wastes", "Island", "Mountain"], "Craw Wurm"),
        ("Consuming Corruption", &["Swamp", "Swamp"], "Craw Wurm"),
        (
            "Tundra Fumarole",
            &["Snow-Covered Mountain", "Snow-Covered Mountain", "Snow-Covered Mountain"],
            "Craw Wurm",
        ),
        ("Fry", &["Wastes", "Mountain"], "Serra Angel"),
        ("Volcanic Spite", &["Wastes", "Mountain"], "Craw Wurm"),
        (
            "Defibrillating Current",
            &["Mountain", "Plains", "Swamp"],
            "Craw Wurm",
        ),
    ];
    for (name, lands, victim) in cases {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, victim);
        let spare = t.hand(P0, "Grizzly Bears");
        let s = cast_paying(&mut t, P0, name, lands, NORMAL, &[obj(wurm)]);
        let (hand, library) = (t.hand_size(P0), t.library_size(P0));
        spoil(&mut t, wurm, Spoil::Hexproof);
        // Volcanic Spite's and Broadside Barrage's choices, if they were asked.
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(spare)]);
        t.resolve_all();
        assert!(!resolved(&t, s), "{name}");
        assert!(t.in_graveyard(P0, name), "{name}");
        assert_eq!(dmg(&t, wurm), 0, "{name}");
        assert_eq!(
            (t.hand_size(P0), t.library_size(P0), t.life(P0)),
            (hand, library, 20),
            "{name}: no draw, discard, bottom or life gain"
        );
        assert_eq!(t.g.players[0].mana_pool.total(), 0, "{name}: no mana");
    }
}

/// The target player becomes illegal: nothing is dealt damage and nothing else happens.
#[test]
fn player_or_planeswalker_burn_doesnt_resolve_for_an_illegal_player() {
    cr!("608.2b");
    ruling!(
        "Bonfire of the Damned",
        "If the player is an illegal target when Bonfire of the Damned tries to resolve, the spell won’t resolve and none of its effects will happen. No damage will be dealt to any permanent or player."
    );
    ruling!(
        "Lavalanche",
        "If the targeted player has become an illegal target by the time Lavalanche resolves, Lavalanche doesn't resolve. Nothing will be dealt damage."
    );
    ruling!(
        "Concussive Bolt",
        "If the player is an illegal target when Concussive Bolt resolves, the spell won’t resolve. None of its effects will happen, even if you control three or more artifacts."
    );
    ruling!(
        "Skullcrack",
        "Skullcrack targets only the player or planeswalker. If that player or planeswalker is an illegal target when Skullcrack tries to resolve, it won't resolve and none of its effects will happen."
    );
    for (name, x) in [
        ("Bonfire of the Damned", Some(2)),
        ("Lavalanche", Some(2)),
        ("Concussive Bolt", None),
        ("Skullcrack", None),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let theirs = pw(&mut t, P1, "Chandra, Torch of Defiance", 4);
        for _ in 0..3 {
            t.battlefield(P0, "Ornithopter");
        }
        if name == "Bonfire of the Damned" {
            t.lands(P0, "Wastes", 2);
        }
        let s = cast_x(&mut t, P0, name, NORMAL, x, &[pl(P1)]);
        player_hexproof(&mut t, P1);
        t.resolve_all();
        assert!(!resolved(&t, s), "{name}");
        assert_eq!(
            (t.life(P1), dmg(&t, bears), t.counters(theirs, "loyalty")),
            (20, 0, 4),
            "{name}: nothing is dealt damage"
        );
        // No "can't block" (metalcraft) and no "can't gain life / can't be prevented".
        assert!(t.g.can_block_at_all(bears), "{name}");
        t.g.gain_life(P1, 2);
        assert_eq!(t.life(P1), 22, "{name}: P1 can still gain life");
    }
}

#[test]
fn carnival_and_carnage_with_an_illegal_target() {
    cr!("608.2b", "709.3");
    ruling!(
        "Carnival // Carnage",
        "If the target permanent is an illegal target by the time Carnival tries to resolve, the spell doesn't resolve. No player is dealt damage."
    );
    ruling!(
        "Carnival // Carnage",
        "If the target player is an illegal target by the time Carnage tries to resolve, the spell doesn't resolve. No player discards."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let s = cast_paying(
        &mut t,
        P0,
        "Carnival // Carnage",
        &["Mountain"],
        CastMethod::Half(0),
        &[obj(bears)],
    );
    spoil(&mut t, bears, Spoil::Hexproof);
    t.resolve_all();
    assert!(!resolved(&t, s));
    assert_eq!((t.life(P1), dmg(&t, bears)), (20, 0));

    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    let s = cast_paying(
        &mut t,
        P0,
        "Carnival // Carnage",
        &["Wastes", "Wastes", "Swamp", "Mountain"],
        CastMethod::Half(1),
        &[pl(P1)],
    );
    player_hexproof(&mut t, P1);
    t.resolve_all();
    assert!(!resolved(&t, s));
    assert_eq!((t.life(P1), t.hand_size(P1)), (20, 2));
}

#[test]
fn pyretic_rebirth_with_one_target_illegal() {
    cr!("608.2b");
    ruling!(
        "Pyretic Rebirth",
        "If the target card in your graveyard is an illegal target when Pyretic Rebirth tries to resolve, no damage will be dealt."
    );
    ruling!(
        "Pyretic Rebirth",
        "If the target creature or planeswalker is an illegal target when Pyretic Rebirth tries to resolve, you'll still return the target artifact or creature card from your graveyard to your hand."
    );
    // The card leaves the graveyard: no damage (its mana value isn't used).
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let card = t.graveyard(P0, "Hill Giant");
    let s = cast_x(
        &mut t,
        P0,
        "Pyretic Rebirth",
        NORMAL,
        None,
        &[obj(card), obj(wurm)],
    );
    move_to(&mut t, card, Zone::Exile);
    t.resolve_all();
    assert!(resolved(&t, s), "the creature target is still legal");
    assert_eq!(dmg(&t, wurm), 0);
    // The creature gains hexproof: the card still returns.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let card = t.graveyard(P0, "Hill Giant");
    cast_x(
        &mut t,
        P0,
        "Pyretic Rebirth",
        NORMAL,
        None,
        &[obj(card), obj(wurm)],
    );
    spoil(&mut t, wurm, Spoil::Hexproof);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(dmg(&t, wurm), 0);
}

#[test]
fn domris_ambush_and_brokers_charm_with_one_target_illegal() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Domri's Ambush",
        "If the creature you control is an illegal target as Domri’s Ambush tries to resolve, you won’t put a +1/+1 counter on it. If that creature is a legal target but the other target isn’t, you’ll still put the counter on the creature you control."
    );
    ruling!(
        "Brokers Charm",
        "If you choose the first mode and your creature is not a legal target as Brokers Charm resolves, no damage will be dealt."
    );
    ruling!(
        "Brokers Charm",
        "If you choose the first mode and your opponent's creature or planeswalker is not a legal target as Brokers Charm resolves, your creature will still get +1/+0"
    );
    // (your creature illegal?)
    for mine_illegal in [true, false] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let wurm = t.battlefield(P1, "Craw Wurm");
        cast_x(
            &mut t,
            P0,
            "Domri's Ambush",
            NORMAL,
            None,
            &[obj(mine), obj(wurm)],
        );
        spoil(&mut t, if mine_illegal { mine } else { wurm }, Spoil::Hexproof);
        t.resolve_all();
        if mine_illegal {
            assert_eq!(t.counters(mine, "+1/+1"), 0);
            assert_eq!(dmg(&t, wurm), 0);
        } else {
            assert_eq!(t.counters(mine, "+1/+1"), 1);
            assert_eq!(dmg(&t, wurm), 0);
        }

        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let wurm = t.battlefield(P1, "Craw Wurm");
        cast_modes(&mut t, P0, "Brokers Charm", &[0], &[obj(mine), obj(wurm)]);
        spoil(&mut t, if mine_illegal { mine } else { wurm }, Spoil::Hexproof);
        t.resolve_all();
        if mine_illegal {
            assert_eq!(t.pt(mine), (2, 2));
            assert_eq!(dmg(&t, wurm), 0, "no damage is dealt");
        } else {
            assert_eq!(t.pt(mine), (3, 2), "+1/+0 still");
            assert_eq!(dmg(&t, wurm), 0);
        }
    }
}

#[test]
fn tandem_takedown_without_its_last_target_or_both_first_targets() {
    cr!("608.2b");
    ruling!(
        "Tandem Takedown",
        "If the last target is an illegal target as Tandem Takedown resolves, or if both of the first targets are illegal targets, nothing deals or is dealt damage."
    );
    for last_illegal in [true, false] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Hill Giant");
        let wurm = t.battlefield(P1, "Craw Wurm");
        let start = t.g.turn_events.len();
        t.answer_targets(P0, &[obj(a), obj(b)]);
        cast_x(&mut t, P0, "Tandem Takedown", NORMAL, None, &[obj(wurm)]);
        if last_illegal {
            spoil(&mut t, wurm, Spoil::Hexproof);
        } else {
            spoil(&mut t, a, Spoil::Hexproof);
            spoil(&mut t, b, Spoil::Hexproof);
        }
        t.resolve_all();
        assert!(
            damage_events(&t, start).is_empty(),
            "last_illegal {last_illegal}: no damage"
        );
        assert!(t.on_battlefield(wurm));
    }
}

#[test]
fn shower_of_sparks_still_damages_the_player() {
    cr!("608.2b");
    ruling!(
        "Shower of Sparks",
        "If the creature leaves the battlefield before this resolves, it still damages the player."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_x(
        &mut t,
        P0,
        "Shower of Sparks",
        NORMAL,
        None,
        &[obj(bears), pl(P1)],
    );
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn triggered_and_activated_burn_with_an_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Aria of Flame",
        "If the target player or planeswalker is an illegal target by the time Aria of Flame’s last ability tries to resolve, the ability doesn’t resolve. You don’t put a verse counter on Aria of Flame."
    );
    ruling!(
        "Cragganwick Cremator",
        "If the target player or planeswalker is an illegal target by the time the ability tries to resolve, the ability won't resolve. You won't discard a card."
    );
    ruling!(
        "Sarkhan the Mad",
        "If the targeted creature is an illegal target by the time the second ability resolves, the ability doesn't resolve. The player won't get a Dragon token."
    );
    ruling!(
        "Vengeful Archon",
        "If the targeted player is an illegal target by the time the ability resolves, the entire ability doesn’t resolve. No damage will be prevented."
    );
    // Aria of Flame: an instant is cast; its trigger targets P1, who gains hexproof.
    supported("Aria of Flame");
    let mut t = TestGame::new(2);
    let aria = t.battlefield(P0, "Aria of Flame");
    cast_x(&mut t, P0, "Opt", NORMAL, None, &[]);
    t.answer_targets(P0, &[pl(P1)]);
    t.settle();
    player_hexproof(&mut t, P1);
    t.resolve_all();
    assert_eq!((t.counters(aria, "verse"), t.life(P1)), (0, 20));

    // Cragganwick Cremator: no random discard.
    supported("Cragganwick Cremator");
    let mut t = TestGame::new(2);
    t.hand(P0, "Craw Wurm");
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Cragganwick Cremator");
    t.settle();
    player_hexproof(&mut t, P1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Craw Wurm"));
    assert_eq!(t.life(P1), 20);

    // Sarkhan the Mad −2: the creature leaves; no Dragon.
    supported("Sarkhan the Mad");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let s = pw(&mut t, P0, "Sarkhan the Mad", 7);
    loyalty(&mut t, P0, s, 1, &[obj(wurm)]);
    spoil(&mut t, wurm, Spoil::Hexproof);
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());

    // Vengeful Archon: no shield, so P0 is dealt the damage.
    supported("Vengeful Archon");
    let mut t = TestGame::new(2);
    let archon = t.battlefield(P0, "Vengeful Archon");
    let src = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, archon, 0, &[pl(P1)]).expect("Vengeful Archon");
    player_hexproof(&mut t, P1);
    t.resolve_all();
    hit_player(&mut t, src, P0, 2);
    assert_eq!((t.life(P0), t.life(P1)), (18, 20));
}

#[test]
fn sarkhan_the_mad_gives_a_dragon_even_if_the_sacrifice_cant_happen() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Sarkhan the Mad",
        "If the targeted creature is a legal target by the time the second ability resolves but its controller can't sacrifice it (due to Tajuru Preserver, perhaps), the ability continues to resolve. That player gets a Dragon token."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P1, "Tajuru Preserver");
    let s = pw(&mut t, P0, "Sarkhan the Mad", 7);
    loyalty(&mut t, P0, s, 1, &[obj(wurm)]);
    t.resolve_all();
    assert!(t.on_battlefield(wurm), "can't be sacrificed");
    let dragons = crate::r_s01_common::tokens(&t, P1);
    assert_eq!(dragons.len(), 1);
    assert_eq!(t.pt(dragons[0]), (5, 5));
}

#[test]
fn etb_burn_with_no_legal_opponent_isnt_put_on_the_stack() {
    cr!("603.3d", "608.2b");
    // (Manticore of the Gauntlet's ruling was written for "deals 3 damage to target
    // opponent"; since the Dominaria Oracle update it says "target opponent or
    // planeswalker", so "If you can't choose a target opponent" no longer decides it:
    // manticore_of_the_gauntlet_targets_a_planeswalker_when_its_opponents_have_hexproof,
    // docs/rulings-exemptions/rulings-P008.tsv. What it says happens when the ability has
    // no legal target still holds, tested here with no planeswalker on the battlefield.)
    ruling!(
        "Manticore of the Gauntlet",
        "the ability is removed from the stack and you won't put a -1/-1 counter on any creature."
    );
    ruling!(
        "Rakdos Firewheeler",
        "If you can't target any opponent with Rakdos Firewheeler's ability, the ability doesn't go on the stack. You can't have it deal 2 damage to any creature or planeswalker. On the other hand, if you do choose two legal targets but either target is illegal when the ability tries to resolve, the other is still affected as appropriate."
    );
    supported("Manticore of the Gauntlet");
    supported("Rakdos Firewheeler");
    // Manticore: P1 has hexproof, and there's no planeswalker.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    player_hexproof(&mut t, P1);
    t.answer_targets(P0, &[obj(mine)]);
    t.enter(P0, "Manticore of the Gauntlet");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.counters(mine, "-1/-1"), 0);
    assert_eq!(t.pt(mine), (3, 3));

    // Rakdos Firewheeler: P1 has hexproof; the creature isn't dealt damage.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Grizzly Bears");
    player_hexproof(&mut t, P1);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Rakdos Firewheeler");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 0);
    assert!(t.on_battlefield(wurm));
    // Two legal targets; the creature then leaves: the opponent is still dealt 2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Rakdos Firewheeler");
    t.settle();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // The opponent gains hexproof: the creature is still dealt 2.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Rakdos Firewheeler");
    t.settle();
    player_hexproof(&mut t, P1);
    t.resolve_all();
    assert_eq!((t.life(P1), dmg(&t, wurm)), (20, 2));
}

/// Manticore of the Gauntlet under its current Oracle text ("This creature deals 3 damage
/// to target opponent or planeswalker", errata of the Dominaria Oracle update, 2018): with
/// its opponents hexproof, the ability can still target a planeswalker, an opponent's or,
/// with no other choice, its controller's own, so it's put on the stack (CR 603.3d). Its
/// 2017 ruling's "If you can't choose a target opponent" is outdated
/// (`docs/rulings-exemptions/rulings-P008.tsv`, `docs/RULING_DISCREPANCIES.md`).
#[test]
fn manticore_of_the_gauntlet_targets_a_planeswalker_when_its_opponents_have_hexproof() {
    cr!("603.3d", "601.2c");
    supported("Manticore of the Gauntlet");
    // P1 has hexproof and a planeswalker: the planeswalker is the target.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = pw(&mut t, P1, "Chandra, Torch of Defiance", 4);
    player_hexproof(&mut t, P1);
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(theirs)]);
    t.enter(P0, "Manticore of the Gauntlet");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(
        (t.counters(mine, "-1/-1"), t.counters(theirs, "loyalty")),
        (1, 1)
    );
    // P1 has hexproof and no planeswalker: P0's own planeswalker is the only choice.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let own = pw(&mut t, P0, "Chandra, Pyrogenius", 5);
    player_hexproof(&mut t, P1);
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(own)]);
    t.enter(P0, "Manticore of the Gauntlet");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!((t.counters(mine, "-1/-1"), t.counters(own, "loyalty")), (1, 2));
}

#[test]
fn burning_suns_avatar_can_target_any_creature() {
    cr!("115.1", "601.2c");
    ruling!(
        "Burning Sun's Avatar",
        "The ability of Burning Sun’s Avatar can target any creature, not just one controlled by the target opponent."
    );
    // Three players: P1 is the target opponent; the creature is P2's (then P0's own).
    for owner in [P2, P0] {
        let mut t = TestGame::new(3);
        let wurm = t.battlefield(owner, "Craw Wurm");
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_targets(P0, &[obj(wurm)]);
        t.enter(P0, "Burning Sun's Avatar");
        t.settle();
        t.resolve_all();
        assert_eq!((t.life(P1), dmg(&t, wurm)), (17, 3), "{owner:?}");
    }
}

#[test]
fn soul_sear_can_target_a_permanent_without_indestructible() {
    cr!("115.1");
    ruling!(
        "Soul Sear",
        "Soul Sear can target a creature or planeswalker that doesn't have indestructible."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let theirs = pw(&mut t, P1, "Chandra, Torch of Defiance", 4);
    cast_x(&mut t, P0, "Soul Sear", NORMAL, None, &[obj(wurm)]);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    cast_x(&mut t, P0, "Soul Sear", NORMAL, None, &[obj(theirs)]);
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
}
