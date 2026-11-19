//! Rulings batch P103 — targets: counterspells that mill aimed at a spell that can't be
//! countered (CR 101.2, 701.6a: still a legal target; the other effects happen), and
//! spells and abilities whose targets all became illegal (CR 608.2b: they don't resolve,
//! none of their effects happen).

use crate::r_p103_common::*;
use crate::r_s01_common::supported;
use crate::r_s07_common::resolved;
use crate::r_s25_common::cast_new;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P1 casts Abrupt Decay ("This spell can't be countered.") at P0's Grizzly Bears.
fn uncounterable(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let decay = cast_new(t, P1, "Abrupt Decay", &[obj(bears)]);
    (decay, bears)
}

#[test]
fn counterspells_that_mill_still_mill_for_an_uncounterable_spell() {
    cr!("101.2", "701.6a", "608.2c");
    ruling!(
        "Didn't Say Please",
        "A spell that can't be countered is a legal target for Didn't Say Please. The spell won't be countered when Didn't Say Please resolves, but its controller still puts the top three cards of their library into their graveyard."
    );
    ruling!(
        "Thought Collapse",
        "A spell that can’t be countered is a legal target for Thought Collapse. The spell won’t be countered when Thought Collapse resolves, but its controller will still put the top three cards of their library into their graveyard."
    );
    ruling!(
        "Countermand",
        "You may target a spell that can't be countered with Countermand. If you do, when Countermand resolves, the target spell won't be countered, but its controller will still put the top four cards of their library into their graveyard."
    );
    ruling!(
        "Psychic Strike",
        "You may target a spell that can’t be countered. When Psychic Strike resolves, the target spell will be unaffected, but its controller will still put the top two cards of their library into their graveyard."
    );
    for (name, n) in [
        ("Didn't Say Please", 3),
        ("Thought Collapse", 3),
        ("Countermand", 4),
        ("Psychic Strike", 2),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let (decay, bears) = uncounterable(&mut t);
        let cs = cast_new(&mut t, P0, name, &[obj(decay)]);
        t.resolve();
        assert!(resolved(&t, cs), "{name}");
        assert!(on_stack(&t, decay), "{name}: not countered");
        assert_eq!(t.graveyard_size(P1), n, "{name}");
        assert_eq!(t.graveyard_size(P0), 1, "{name}: only the counterspell");
        t.resolve_all();
        assert!(!t.on_battlefield(bears), "{name}: Abrupt Decay resolved");
    }
}

#[test]
fn fear_of_impostors_manifests_dread_for_an_uncounterable_spell() {
    cr!("101.2", "701.6a", "701.62a");
    ruling!(
        "Fear of Impostors",
        "A spell that can't be countered is a legal target for Fear of Impostors's last ability. The spell won't be countered when the ability resolves, but its controller will still manifest dread."
    );
    supported("Fear of Impostors");
    let mut t = TestGame::new(2);
    let (decay, _) = uncounterable(&mut t);
    let fear = cast_new(&mut t, P0, "Fear of Impostors", &[]);
    t.resolve();
    assert!(t.on_battlefield(fear));
    t.answer_targets(P0, &[obj(decay)]);
    t.resolve();
    assert!(on_stack(&t, decay));
    let face_down: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.controller == P1 && o.face_down)
        .map(|o| o.id)
        .collect();
    assert_eq!(face_down.len(), 1);
    assert_eq!(t.pt(face_down[0]), (2, 2));
    assert_eq!(t.graveyard_size(P1), 1);
}

#[test]
fn a_counterspell_whose_target_left_the_stack_doesnt_mill() {
    cr!("608.2b");
    ruling!(
        "Countermand",
        "If the target spell is an illegal target when Countermand tries to resolve, Countermand won't resolve and none of its effects will happen. The target's controller won't put any cards from the top of their library into their graveyard."
    );
    ruling!(
        "Psychic Strike",
        "If the target spell is an illegal target when Psychic Strike tries to resolve, Psychic Strike won’t resolve and none of its effects will happen. The target’s controller won’t put any cards from the top of their library into their graveyard."
    );
    for name in ["Countermand", "Psychic Strike"] {
        supported(name);
        let mut t = TestGame::new(2);
        let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
        let cs = cast_new(&mut t, P0, name, &[obj(shock)]);
        // Counterspell counters the Shock first.
        cast_new(&mut t, P0, "Counterspell", &[obj(shock)]);
        t.resolve();
        assert_eq!(t.zone(shock), Zone::Graveyard(P1));
        t.resolve();
        assert!(!resolved(&t, cs), "{name}");
        assert_eq!(t.graveyard_size(P1), 1, "{name}: only the Shock");
        assert_eq!(t.life(P0), 20);
    }
}

#[test]
fn spells_whose_creature_target_is_gone_do_nothing() {
    cr!("608.2b");
    ruling!(
        "Grisly Spectacle",
        "If the creature is an illegal target when Grisly Spectacle tries to resolve, it won’t resolve and none of its effects will happen. The creature’s controller won’t put any cards into their graveyard."
    );
    ruling!(
        "Unwanted Remake",
        "If the target creature is an illegal target as Unwanted Remake tries to resolve, it won't resolve and none of its effects will happen. The creature's controller won't manifest dread."
    );
    for name in ["Grisly Spectacle", "Unwanted Remake"] {
        supported(name);
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let spell = cast_new(&mut t, P0, name, &[obj(giant)]);
        // The creature leaves the battlefield (bounced) before the spell resolves.
        crate::r_p189_common::bounce(&mut t, giant);
        t.resolve();
        assert!(!resolved(&t, spell), "{name}");
        assert_eq!(t.graveyard_size(P1), 0, "{name}");
        assert_eq!(t.g.permanents().filter(|o| o.face_down).count(), 0, "{name}");
    }
}

#[test]
fn grisly_spectacle_uses_the_destroyed_creatures_last_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Grisly Spectacle",
        "Use the creature’s power the last time it was on the battlefield to determine how many cards its controller puts into their graveyard."
    );
    supported("Grisly Spectacle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Giant Growth: a 5/5 until end of turn.
    cast_new(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    cast_new(&mut t, P0, "Grisly Spectacle", &[obj(bears)]);
    t.resolve();
    // The Bears and the Giant Growth, then five milled cards.
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    assert_eq!(t.graveyard_size(P1), 2 + 5);
}

/// P1 gains hexproof (Leyline of Sanctity enters under P1's control).
fn hexproof(t: &mut TestGame, p: PlayerId) {
    t.battlefield(p, "Leyline of Sanctity");
}

#[test]
fn spells_whose_player_target_became_illegal_dont_draw() {
    cr!("608.2b", "702.11b");
    ruling!(
        "Thought Scour",
        "If the target player is an illegal target when Thought Scour tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    ruling!(
        "Thassa's Bounty",
        "If the target player is an illegal target when Thassa's Bounty tries to resolve, it won't resolve and none of its effects will happen. You won't draw three cards."
    );
    for name in ["Thought Scour", "Thassa's Bounty"] {
        supported(name);
        let mut t = TestGame::new(2);
        let spell = cast_new(&mut t, P0, name, &[pl(P1)]);
        hexproof(&mut t, P1);
        let hand = t.hand_size(P0);
        t.resolve();
        assert!(!resolved(&t, spell), "{name}");
        assert_eq!(t.hand_size(P0), hand, "{name}");
        assert_eq!(t.graveyard_size(P1), 0, "{name}");
    }
}

#[test]
fn jaces_plus_one_with_an_illegal_target_doesnt_draw() {
    cr!("608.2b", "606.3");
    ruling!(
        "Jace, Wielder of Mysteries",
        "If the target player is an illegal target when Jace’s first loyalty ability tries to resolve, it doesn’t resolve. You won’t draw a card."
    );
    supported("Jace, Wielder of Mysteries");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace, Wielder of Mysteries");
    t.activate(P0, jace, 0, &[pl(P1)]).unwrap();
    hexproof(&mut t, P1);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.graveyard_size(P1), 0);
    // The loyalty cost was paid.
    assert_eq!(t.counters(jace, "loyalty"), 5);
}
