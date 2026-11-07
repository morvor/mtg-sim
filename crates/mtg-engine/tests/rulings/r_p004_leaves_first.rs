//! Rulings batch P004 — "exile ... until this leaves the battlefield" (CR 610.3a–b): if the
//! banishing permanent leaves the battlefield before its exile ability resolves, nothing
//! is exiled (the rest of the ability still happens); an illegal target makes the whole
//! ability not resolve (CR 608.2b).

use crate::r_p004_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn if_the_banisher_leaves_before_its_ability_resolves_nothing_is_exiled() {
    cr!("610.3b", "610.3a", "603.2");
    ruling!(
        "Alabaster Host Intercessor",
        "If Alabaster Host Intercessor leaves the battlefield before its enters-the-battlefield ability resolves, the target creature won’t be exiled at all."
    );
    ruling!(
        "Banishing Light",
        "If Banishing Light leaves the battlefield before its triggered ability resolves, the target permanent won’t be exiled."
    );
    ruling!(
        "Borrowed Time",
        "If Borrowed Time leaves the battlefield before its enters-the-battlefield ability resolves, the target permanent won't be exiled."
    );
    ruling!(
        "Chained to the Rocks",
        "If Chained to the Rocks leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    ruling!(
        "Chains of Custody",
        "If Chains of Custody leaves the battlefield before its second ability resolves, the target permanent won't be exiled."
    );
    ruling!(
        "Colossal Whale",
        "If Colossal Whale leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    ruling!(
        "Deputy of Detention",
        "If Deputy of Detention leaves the battlefield before its triggered ability resolves, nothing is exiled."
    );
    ruling!(
        "Dimensional Exile",
        "If Dimensional Exile leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    ruling!(
        "Fairgrounds Warden",
        "If Fairgrounds Warden leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    ruling!(
        "Faith Unbroken",
        "If Faith Unbroken leaves the battlefield before its triggered ability resolves, the targeted creature won't be exiled."
    );
    ruling!(
        "Gelatinous Cube",
        "If Gelatinous Cube leaves the battlefield before its Engulf ability resolves, the target non-Ooze creature won't be exiled."
    );
    ruling!(
        "Glass Casket",
        "If Glass Casket leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    ruling!(
        "Hieromancer's Cage",
        "If Hieromancer’s Cage leaves the battlefield before its triggered ability resolves, the target permanent won’t be exiled."
    );
    ruling!(
        "Hixus, Prison Warden",
        "If Hixus leaves the battlefield before its triggered ability resolves, the creature that dealt combat damage to you won't be exiled."
    );
    ruling!(
        "In the Trenches",
        "If In the Trenches leaves the battlefield before its activated ability resolves, the target permanent won't be exiled."
    );
    ruling!(
        "Ixalan's Binding",
        "If Ixalan's Binding leaves the battlefield before its triggered ability resolves, the target permanent won't be exiled."
    );
    ruling!(
        "Liminal Hold",
        "If Liminal Hold leaves the battlefield before its ability resolves, the target nonland permanent won't be exiled. You'll still gain life if that permanent remains a legal target."
    );
    ruling!(
        "Makeshift Binding",
        "If Makeshift Binding leaves the battlefield before its triggered ability resolves, the target creature won't be exiled, but you'll still gain 2 life."
    );
    ruling!(
        "Mystical Tether",
        "If Mystical Tether leaves the battlefield before its triggered ability resolves, the target artifact or creature won't be exiled at all."
    );
    ruling!(
        "Portable Hole",
        "If Portable Hole leaves the battlefield before its ability resolves, the target permanent won't be exiled."
    );
    ruling!(
        "Prayer of Binding",
        "If Prayer of Binding leaves the battlefield before its triggered ability resolves, the target permanent won't be exiled."
    );
    ruling!(
        "Seam Rip",
        "If Seam Rip leaves the battlefield before its triggered ability resolves, the target permanent won’t be exiled."
    );
    ruling!(
        "Silkwrap",
        "If Silkwrap leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    ruling!(
        "Stasis Snare",
        "If Stasis Snare leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    ruling!(
        "Thopter Arrest",
        "If Thopter Arrest leaves the battlefield before its triggered ability resolves, the target permanent won’t be exiled."
    );
    ruling!(
        "Trapjaw Tyrant",
        "If Trapjaw Tyrant leaves the battlefield before its triggered ability resolves (most likely because it was dealt lethal damage), the target creature won’t be exiled."
    );
    ruling!(
        "Werefox Bodyguard",
        "If Werefox Bodyguard leaves the battlefield before its triggered ability resolves, the target creature won't be exiled."
    );
    ruling!(
        "Stormplain Detainment",
        "If this enchantment leaves the battlefield before its ability resolves, the target nonland permanent won’t be exiled."
    );
    for &(name, _) in BANISHERS {
        // Control: the target is exiled.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        banish_trigger(&mut t, name, bears);
        assert!(t.stack_len() >= 1, "{name}: no ability on the stack");
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"), "{name}: not exiled");
        // The banisher leaves first: nothing is exiled.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let b = banish_trigger(&mut t, name, bears);
        destroy_now(&mut t, b);
        t.resolve_all();
        assert!(!t.in_exile("Grizzly Bears"), "{name}: exiled anyway");
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{name}");
    }
}

#[test]
fn the_life_gain_still_happens_when_the_banisher_left_but_the_target_is_legal() {
    cr!("610.3b", "608.2b");
    // (Cited in the test above.) Liminal Hold, Makeshift Binding and Prayer of Binding.
    for name in ["Liminal Hold", "Makeshift Binding", "Prayer of Binding"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let b = banish_trigger(&mut t, name, bears);
        destroy_now(&mut t, b);
        t.resolve_all();
        assert!(!t.in_exile("Grizzly Bears"), "{name}");
        assert_eq!(t.life(P0), 22, "{name}: no life gained");
    }
}

#[test]
fn an_illegal_target_means_none_of_the_effects_happen() {
    cr!("608.2b", "115.3");
    ruling!(
        "Makeshift Binding",
        "If the target creature is an illegal target as Makeshift Binding's ability tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    ruling!(
        "Deputy of Detention",
        "If the target nonland permanent is an illegal target by the time this ability tries to resolve, the ability doesn't resolve. Nothing is exiled."
    );
    ruling!(
        "Prayer of Binding",
        "You don't have to choose a target for Prayer of Binding's triggered ability. However, if you do and the target is illegal as the ability tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    for name in ["Makeshift Binding", "Prayer of Binding", "Liminal Hold"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        banish_trigger(&mut t, name, bears);
        destroy_now(&mut t, bears);
        t.resolve_all();
        assert_eq!(t.life(P0), 20, "{name}: gained life");
    }
    // Prayer of Binding with no target: you still gain 2 life.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    t.enter(P0, "Prayer of Binding");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Deputy of Detention: the target is gone, so the other permanents with its name
    // aren't exiled either.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    banish_trigger(&mut t, "Deputy of Detention", bears);
    destroy_now(&mut t, bears);
    t.resolve_all();
    assert!(!t.in_exile("Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
