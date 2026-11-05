//! Rulings batch P033 — Panoptic Projektor ("If turning a face-down permanent face up
//! causes a triggered ability of a permanent you control to trigger, that ability triggers
//! an additional time.", CR 603.2d) and Obscuring Aether ("Face-down creature spells you
//! cast cost {1} less to cast. {1}{G}: Turn this enchantment face down.", CR 708).

use crate::r_p033_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s06_common::activate_containing;
use crate::r_s11_common::{can_turn_face_up, turn_face_up};
use crate::r_s12_common::{morph, MORPH};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, StackKind};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The targets of the triggered abilities on the stack, top first.
fn trigger_targets(t: &TestGame) -> Vec<Vec<Entity>> {
    t.g.stack
        .iter()
        .rev()
        .filter_map(|id| {
            let o = t.g.obj(*id);
            let si = o.stack.as_ref()?;
            matches!(si.kind, StackKind::Triggered { .. }).then(|| {
                si.chosen
                    .iter()
                    .flat_map(|c| c.targets.iter().flatten().copied())
                    .collect()
            })
        })
        .collect()
}

#[test]
fn each_panoptic_projektor_adds_one_more_trigger() {
    cr!("603.2d", "702.37e", "116.2b");
    ruling!(
        "Panoptic Projektor",
        "If you control two copies of Panoptic Projektor, a face-down permanent turning face up causes abilities to trigger three times, not four. A third Panoptic Projektor causes abilities to trigger four times, a fourth causes abilities to trigger five times, and so on."
    );
    supported("Panoptic Projektor");
    supported("Coral Trickster");
    // Coral Trickster: "Morph {U}", "When this creature is turned face up, you may tap or
    // untap target permanent."
    for (projektors, triggers) in [(0, 1), (1, 2), (2, 3), (3, 4)] {
        let mut t = main_phase(2);
        for _ in 0..projektors {
            t.battlefield(P0, "Panoptic Projektor");
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        let fd = morph(&mut t, P0, "Coral Trickster");
        add(&mut t, P0, ManaType::U, 1);
        for _ in 0..triggers + 1 {
            t.answer_targets(P0, &[obj(bears)]);
        }
        assert!(turn_face_up(&mut t, P0, fd));
        t.settle();
        assert_eq!(
            triggers_on_stack(&t, "tap or untap"),
            triggers,
            "{projektors} Projektors"
        );
    }
}

#[test]
fn panoptic_projektors_extra_trigger_has_its_own_targets_and_choices() {
    cr!("603.2d", "603.3d", "608.2d");
    ruling!(
        "Panoptic Projektor",
        "Panoptic Projektor’s last ability doesn’t copy the triggered ability; it just causes the ability to trigger twice. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    let mut t = main_phase(2);
    t.battlefield(P0, "Panoptic Projektor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.objects[wurm.0 as usize].tapped = true;
    let fd = morph(&mut t, P0, "Coral Trickster");
    add(&mut t, P0, ManaType::U, 1);
    // A target for each instance: the Bears for one, the Wurm for the other.
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(wurm)]);
    assert!(turn_face_up(&mut t, P0, fd));
    t.settle();
    let targets = trigger_targets(&t);
    assert_eq!(targets.len(), 2);
    let mut all: Vec<Entity> = targets.iter().flatten().copied().collect();
    all.sort_by_key(|e| format!("{e:?}"));
    let mut want = vec![obj(bears), obj(wurm)];
    want.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(all, want, "each instance has its own target");
    // On resolution: the top one taps or untaps its target (whichever changes it); the
    // other one's controller declines.
    let top = targets[0][0].object().unwrap();
    let top_was_tapped = t.obj(top).tapped;
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(if top_was_tapped { 1 } else { 0 }),
    );
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(
        t.obj(top).tapped,
        !top_was_tapped,
        "the top instance's choice"
    );
    let other = if top == bears { wurm } else { bears };
    assert_eq!(
        t.obj(other).tapped,
        other == wurm,
        "the other instance's choice"
    );
}

#[test]
fn panoptic_projektor_doesnt_double_as_turned_face_up_effects() {
    cr!("708.11", "603.2d", "614.1e");
    ruling!(
        "Panoptic Projektor",
        "Replacement effects are unaffected by Panoptic Projektor’s last ability. For example, abilities that apply “as [this permanent] is turned face up,” such as that of Crowd-Control Warden, are unaffected."
    );
    supported("Crowd-Control Warden");
    // Crowd-Control Warden: "As this creature enters or is turned face up, put X +1/+1
    // counters on it, where X is the number of other creatures you control." Two other
    // creatures: two counters, Projektor or not.
    for projektor in [false, true] {
        let mut t = main_phase(2);
        if projektor {
            t.battlefield(P0, "Panoptic Projektor");
        }
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Kird Ape");
        let card = t.hand(P0, "Crowd-Control Warden");
        add(&mut t, P0, ManaType::C, 3);
        let spell = t
            .cast(P0, card)
            .method(CastMethod::FaceDown(KeywordKind::Disguise))
            .go();
        t.resolve_all();
        let warden = t.g.current(spell);
        assert!(t.obj(warden).face_down);
        add(&mut t, P0, ManaType::G, 5);
        assert!(turn_face_up(&mut t, P0, warden));
        t.resolve_all();
        assert_eq!(t.counters(warden, "+1/+1"), 2, "Projektor: {projektor}");
    }
}

#[test]
fn obscuring_aether_cant_turn_itself_face_up_and_doesnt_reduce_manifesting() {
    cr!("702.37e", "708.2", "701.40a", "601.2f");
    ruling!(
        "Obscuring Aether",
        "Note that Obscuring Aether doesn't have a morph cost. Once it's turned face down, it doesn't have a way to turn itself face up."
    );
    ruling!(
        "Obscuring Aether",
        "Obscuring Aether has no effect on spells that instruct you to manifest cards."
    );
    supported("Obscuring Aether");
    supported("Soul Summons");
    let mut t = main_phase(2);
    let aether = t.battlefield(P0, "Obscuring Aether");
    // A face-down creature spell (morph) costs {2}.
    let card = t.hand(P0, "Coral Trickster");
    let paid = cast_paying(&mut t, P0, card, MORPH).expect("morph");
    assert_spent(&paid.spent, "{2}", "face-down Coral Trickster");
    t.resolve_all();
    // Soul Summons ({1}{W}: "Manifest the top card of your library.") costs {1}{W}, and
    // it manifests that card: a face-down 2/2 creature.
    let top = t.library_top(P0, "Grizzly Bears");
    let ss = t.hand(P0, "Soul Summons");
    let paid = cast_normal(&mut t, P0, ss);
    assert_spent(&paid.spent, "{1}{W}", "Soul Summons");
    t.resolve_all();
    assert!(t.on_battlefield(top) && t.obj_now(top).face_down);
    assert_eq!(t.pt(top), (2, 2));
    // "{1}{G}: Turn this enchantment face down." Then it's a face-down 2/2 that can't be
    // turned face up, even with mana to spare.
    t.lands(P0, "Forest", 2);
    activate_containing(&mut t, P0, aether, "face down").expect("turn face down");
    t.resolve_all();
    let aether = t.g.current(aether);
    assert!(t.obj(aether).face_down && t.obj(aether).is_creature());
    crate::r_p046_common::fill_pool(&mut t, P0, 10);
    assert!(!can_turn_face_up(&mut t, P0, aether));
    assert!(!turn_face_up(&mut t, P0, aether));
    assert!(t.obj(aether).face_down);
}
