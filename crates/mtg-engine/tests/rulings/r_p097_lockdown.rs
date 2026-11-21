//! Rulings batch P097 — lockdown effects: permanents that don't untap during their
//! controller's untap step (CR 502.3), tapping a creature that's already tapped
//! (CR 701.26a), and upkeep payments to untap (Paralyze, Dream Tides).

use crate::r_p097_common::*;
use crate::r_s06_common::{activate_containing, attach_new};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Advances to `p`'s next upkeep (after their untap step), resolving what triggers there.
fn through_untap(t: &mut TestGame, p: PlayerId) {
    if t.g.turn.active == p {
        let q = if p == P0 { P1 } else { P0 };
        t.advance_to(q, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn lockdown_abilities_can_target_an_already_tapped_creature() {
    cr!("115.1", "701.26a", "502.3");
    ruling!(
        "Singing Bell Strike",
        "Singing Bell Strike can be cast targeting any creature, including one that's already tapped."
    );
    ruling!(
        "Deserter's Quarters",
        "The activated ability can target any creature, not just one that's untapped."
    );
    ruling!(
        "Merchant Raiders",
        "The triggered ability can target any creature, including one that is already tapped. In that case, the creature stays tapped and won't untap during its controller's untap step."
    );
    ruling!(
        "Shipbreaker Kraken",
        "The ability can target a tapped creature. If a targeted creature is already tapped when it resolves, that creature just remains tapped and doesn't untap during its controller's untap step."
    );
    for case in ["Singing Bell Strike", "Deserter's Quarters", "Merchant Raiders", "Shipbreaker Kraken"] {
        supported(case);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.tap(bears);
        match case {
            "Singing Bell Strike" => {
                let aura = t.hand(P0, case);
                pool(&mut t, P0, &[(ManaType::U, 2)]);
                t.cast(P0, aura).target(bears).go();
            }
            "Deserter's Quarters" => {
                let q = t.battlefield(P0, case);
                pool(&mut t, P0, &[(ManaType::C, 6)]);
                activate_containing(&mut t, P0, q, "Tap target").expect("activate");
            }
            "Merchant Raiders" => {
                t.answer_targets(P0, &[Entity::Object(bears)]);
                t.enter(P0, case);
            }
            _ => {
                let kraken = t.battlefield(P0, case);
                pool(&mut t, P0, &[(ManaType::U, 8)]);
                activate_containing(&mut t, P0, kraken, "Monstrosity").expect("activate");
                t.answer_targets(P0, &[Entity::Object(bears)]);
                t.resolve();
                assert_eq!(t.counters(kraken, "+1/+1"), 4);
            }
        }
        t.resolve_all();
        assert!(tapped(&t, bears), "{case}");
        through_untap(&mut t, P1);
        assert!(tapped(&t, bears), "{case}: doesn't untap");
    }
}

#[test]
fn shackles_doesnt_tap_the_creature() {
    cr!("303.4", "502.3");
    ruling!(
        "Shackles",
        "Shackles does not tap the creature when it enters."
    );
    supported("Shackles");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.hand(P0, "Shackles");
    pool(&mut t, P0, &[(ManaType::W, 1), (ManaType::C, 2)]);
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(bears)));
    assert!(!tapped(&t, bears));
}

#[test]
fn encrust_leaves_static_and_triggered_abilities_alone() {
    cr!("602.5", "604.1", "603.2");
    ruling!(
        "Encrust",
        "Static and triggered abilities are unaffected by Encrust."
    );
    supported("Encrust");
    let mut t = TestGame::new(2);
    let marshal = t.battlefield(P1, "Benalish Marshal");
    let warden = t.battlefield(P1, "Soul Warden");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    for c in [marshal, warden, sorcerer] {
        attach_new(&mut t, P0, "Encrust", c);
    }
    // Static: Benalish Marshal still gives other creatures +1/+1.
    assert_eq!(t.pt(warden), (2, 2));
    // Triggered: Soul Warden still gains life.
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
    // Activated: Prodigal Sorcerer's can't be activated (an unenchanted one's can).
    let other = t.battlefield(P1, "Prodigal Sorcerer");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    assert!(t.activate(P1, sorcerer, 0, &[]).is_err());
    t.answer_targets(P1, &[Entity::Player(P0)]);
    assert!(t.activate(P1, other, 0, &[]).is_ok());
}

/// Puts `n` Paralyzes on P1's tapped Grizzly Bears, gives P1 four Swamps, and runs P1's
/// next untap step and upkeep answering the Paralyze triggers with `pay`.
fn paralyzed(n: usize, pay: &[bool]) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    for _ in 0..n {
        attach_new(&mut t, P0, "Paralyze", bears);
    }
    t.lands(P1, "Swamp", 4);
    for p in pay {
        t.answer_yes(P1, *p);
    }
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    (t, bears)
}

#[test]
fn paralyze_untaps_once_per_turn_and_two_arent_cumulative() {
    cr!("502.3", "603.2", "118.12");
    ruling!(
        "Paralyze",
        "The creature can only be untapped using Paralyze once per turn."
    );
    ruling!(
        "Paralyze",
        "Two Paralyzes are not cumulative. Paying either one will untap the creature."
    );
    supported("Paralyze");
    // Not paying: it stays tapped.
    let (t, bears) = paralyzed(1, &[false]);
    assert!(tapped(&t, bears));
    // Paying {4} once untaps it; tapped again that turn, nothing more untaps it.
    let (mut t, bears) = paralyzed(1, &[true]);
    assert!(!tapped(&t, bears));
    t.g.tap(bears);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(tapped(&t, bears));
    // Two Paralyzes: paying for the first alone untaps it.
    let (t, bears) = paralyzed(2, &[true, false]);
    assert!(!tapped(&t, bears));
    // ... or paying for the second alone.
    let (t, bears) = paralyzed(2, &[false, true]);
    assert!(!tapped(&t, bears));
}

#[test]
fn dream_tides_untaps_nongreen_creatures_only_at_the_beginning_of_upkeep() {
    cr!("502.3", "603.2", "118.12");
    ruling!(
        "Dream Tides",
        "You can only untap nongreen creatures once, since this has a beginning of upkeep triggered ability."
    );
    supported("Dream Tides");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dream Tides");
    let warrior = t.battlefield(P1, "Phantom Warrior");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(warrior);
    t.g.tap(bears);
    t.lands(P1, "Island", 4);
    let from = t.asked().len();
    t.answer_choose(P1, &[Entity::Object(warrior)]);
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(!tapped(&t, warrior));
    assert!(tapped(&t, bears));
    // The green creature was never a choice.
    for c in crate::r_s03_common::choice_candidates(&t, from, "") {
        assert!(!c.contains(&Entity::Object(bears)));
    }
    // Tapped again later in the turn, it stays tapped: there's no other chance to pay.
    t.g.tap(warrior);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(tapped(&t, warrior));
}

#[test]
fn storage_matrix_doesnt_override_effects_that_stop_untapping() {
    cr!("502.3");
    ruling!(
        "Storage Matrix",
        "This card’s ability does not override effects which prevent a permanent from untapping."
    );
    supported("Storage Matrix");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Storage Matrix");
    let shackled = t.battlefield(P1, "Grizzly Bears");
    let free = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    for c in [shackled, free, forest] {
        t.g.tap(c);
    }
    attach_new(&mut t, P0, "Shackles", shackled);
    // P1 chooses creature.
    t.answer(
        P1,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    through_untap(&mut t, P1);
    assert!(!tapped(&t, free), "{}", t.dump_log());
    assert!(tapped(&t, shackled));
    assert!(tapped(&t, forest));
}
