//! Rulings batch P157 — discard triggers that give no permission to discard (CR 701.9),
//! and Falkenrath Gorger granting madness (CR 702.35) to Vampire cards not on the
//! battlefield: its own card, the source leaving before the madness trigger resolves,
//! and two madness abilities.

use crate::r_p146_common::discard;
use crate::r_p157_common::*;
use crate::r_s04_common::add_mana;
use mtg_engine::decision::{Action, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether `p` has a legal action that uses an ability of `src`.
fn can_use(t: &mut TestGame, p: PlayerId, src: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p).iter().any(|a| match a {
        Action::Activate { source, .. } => *source == src,
        _ => false,
    })
}

#[test]
fn discard_triggers_dont_let_you_discard() {
    cr!("701.9a", "603.2");
    ruling!("Cunning Survivor", "An ability that triggers whenever you discard a card doesn’t give you permission to discard cards.");
    ruling!("Drake Haven", "An ability that triggers whenever you discard a card doesn’t give you permission to discard cards.");
    ruling!("Flameblade Adept", "An ability that triggers whenever you discard a card doesn’t give you permission to discard cards.");
    ruling!("Grisly Survivor", "An ability that triggers whenever you discard a card doesn’t give you permission to discard cards.");
    ruling!("Hekma Sentinels", "An ability that triggers whenever you discard a card doesn’t give you permission to discard cards.");
    ruling!("Ominous Sphinx", "An ability that triggers whenever you discard a card doesn’t give you permission to discard cards.");
    ruling!("Pitiless Vizier", "An ability that triggers whenever you discard a card doesn’t give you permission to discard cards.");
    ruling!("Ruthless Sniper", "An ability that triggers whenever you discard a card doesn’t give you permission to discard cards.");
    ruling!("Shadowstorm Vizier", "An ability that triggers whenever you discard a card doesn’t give you permission to discard cards.");
    ruling!("Zenith Seeker", "An ability that triggers whenever you discard a card doesn’t give you permission to discard cards.");
    for name in [
        "Cunning Survivor",
        "Drake Haven",
        "Flameblade Adept",
        "Grisly Survivor",
        "Hekma Sentinels",
        "Ominous Sphinx",
        "Pitiless Vizier",
        "Ruthless Sniper",
        "Shadowstorm Vizier",
        "Zenith Seeker",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let victim = t.battlefield(P1, "Craw Wurm");
        let card = t.hand(P0, "Island");
        crate::r_p016_common::lots(&mut t, P0);
        // Nothing lets P0 discard the Island: no ability of the permanent to use, and no
        // other action than passing or playing the Island.
        assert!(!can_use(&mut t, P0, src), "{name}");
        t.g.turn.priority = Some(P0);
        let acts = t.g.legal_actions(P0);
        assert!(
            acts.iter()
                .all(|a| matches!(a, Action::Pass | Action::PlayLand { .. })),
            "{name}: {acts:?}"
        );
        assert!(t.in_hand(P0, "Island"));
        // Another effect making P0 discard triggers it.
        t.answer_targets(P0, &[obj(victim)]);
        discard(&mut t, P0, card);
        assert_eq!(t.stack_len(), 1, "{name}: triggers on a discard");
    }
}

#[test]
fn falkenrath_gorger_doesnt_give_itself_madness() {
    cr!("702.35a", "611.3a", "113.6");
    ruling!(
        "Falkenrath Gorger",
        "Falkenrath Gorger's ability only applies while it's on the battlefield. If you discard it, it won't give itself madness."
    );
    supported("Falkenrath Gorger");
    // No Gorger on the battlefield: the discarded Gorger goes to the graveyard.
    let mut t = TestGame::new(2);
    let g = t.hand(P0, "Falkenrath Gorger");
    discard(&mut t, P0, g);
    assert!(t.in_graveyard(P0, "Falkenrath Gorger"));
    assert_eq!(t.stack_len(), 0);
    // With a Gorger on the battlefield, another Gorger card discarded has madness.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Falkenrath Gorger");
    let g = t.hand(P0, "Falkenrath Gorger");
    discard(&mut t, P0, g);
    assert!(t.in_exile("Falkenrath Gorger"));
    assert_eq!(triggers_on_stack(&t, "adness"), 1);
}

#[test]
fn falkenrath_gorger_leaving_doesnt_stop_the_madness_cast() {
    cr!("702.35a", "603.4", "113.7a");
    ruling!(
        "Falkenrath Gorger",
        "If Falkenrath Gorger leaves the battlefield before the madness trigger has resolved for a Vampire card that gained madness with its ability, the madness ability will still let you cast that Vampire card for the appropriate cost even though it no longer has madness."
    );
    let mut t = TestGame::new(2);
    let gorger = t.battlefield(P0, "Falkenrath Gorger");
    let v = t.hand(P0, "Vampire Interloper");
    discard(&mut t, P0, v);
    assert!(t.in_exile("Vampire Interloper"));
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, gorger);
    // Its mana cost ({1}{B}).
    add_mana(&mut t, P0, ManaType::B, 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Vampire Interloper").len(), 1);
    assert!(!t.in_exile("Vampire Interloper"));
}

#[test]
fn falkenrath_gorger_choose_which_madness_ability_exiles_it() {
    cr!("616.1", "702.35a");
    ruling!(
        "Falkenrath Gorger",
        "If you discard a Vampire creature card that already has a madness ability, you'll choose which madness ability exiles it. You may choose either the one it normally has or the one it gains from Falkenrath Gorger."
    );
    // Stromkirk Occultist: mana cost {2}{R}, madness {1}{R}. With {1}{R} available, it can
    // be cast after one choice (its own madness) and not after the other (the Gorger's,
    // {2}{R}): each choice's own madness ability triggers, for its own cost.
    let mut cast = vec![];
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Falkenrath Gorger");
        let v = t.hand(P0, "Stromkirk Occultist");
        let m = mark(&t);
        t.answer(P0, DecisionKind::Replacement, mtg_engine::decision::Answer::Index(pick));
        discard(&mut t, P0, v);
        let options: Vec<usize> = t.asked()[m..]
            .iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseReplacement { options, .. } if *p == P0 => Some(options.len()),
                _ => None,
            })
            .collect();
        assert_eq!(options, vec![2], "a choice between the two madness abilities");
        assert!(t.in_exile("Stromkirk Occultist"));
        assert_eq!(triggers_on_stack(&t, "adness"), 1, "only the one that exiled it");
        add_mana(&mut t, P0, ManaType::R, 1);
        add_mana(&mut t, P0, ManaType::C, 1);
        t.answer_yes(P0, true);
        t.resolve_all();
        cast.push(t.named_on_battlefield("Stromkirk Occultist").len() == 1);
        if !cast[pick] {
            assert!(t.in_graveyard(P0, "Stromkirk Occultist"));
        }
    }
    cast.sort();
    assert_eq!(cast, vec![false, true]);
}
