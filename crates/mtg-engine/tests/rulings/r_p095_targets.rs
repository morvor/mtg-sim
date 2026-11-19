//! Rulings batch P095 — spells that gain or lose life alongside a targeted effect: if every
//! target is illegal as the spell tries to resolve, it doesn't resolve and no life is
//! gained or lost (CR 608.2b); the owner versus the controller of a creature (CR 108.3,
//! 110.2); and amounts taken from a creature that has left the battlefield, which use its
//! last known information (CR 608.2h).

use crate::r_p095_common::*;
use crate::r_p015_common::gain_control;
use crate::r_s01_common::supported;
use crate::r_s29_common::put_counters;
use mtg_engine::testing::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Illegal targets: no life gained or lost
// ---------------------------------------------------------------------------------------

#[test]
fn spells_whose_target_became_illegal_gain_no_life() {
    cr!("608.2b", "115.1");
    ruling!(
        "Tidy Conclusion",
        "If the target creature becomes an illegal target before Tidy Conclusion resolves, the spell doesn't resolve. You don't gain any life."
    );
    ruling!(
        "Sephiroth's Intervention",
        "If the target creature is an illegal target as Sephiroth's Intervention tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    ruling!(
        "Lich's Caress",
        "If the target creature is an illegal target by the time Lich’s Caress tries to resolve, the spell doesn’t resolve. You won’t gain any life."
    );
    ruling!(
        "Path of Peace",
        "If the target creature is an illegal target by the time Path of Peace tries to resolve, the spell doesn't resolve. No player gains life."
    );
    ruling!(
        "Grapple with Death",
        "If the target is illegal as Grapple with Death tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    for card in [
        "Tidy Conclusion",
        "Sephiroth's Intervention",
        "Lich's Caress",
        "Path of Peace",
        "Grapple with Death",
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ornithopter");
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_new(&mut t, P0, card, &[Entity::Object(bears)]);
        bounce(&mut t, bears);
        t.resolve_all();
        assert!(t.in_graveyard(P0, card), "{card}");
        assert_eq!((t.life(P0), t.life(P1)), (20, 20), "{card}");
        // With a legal target, life is gained.
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_new(&mut t, P0, card, &[Entity::Object(bears)]);
        t.resolve_all();
        assert!(t.life(P0) > 20 || t.life(P1) > 20, "{card}");
    }
}

#[test]
fn sunshower_druid_trigger_with_an_illegal_target_gains_no_life() {
    cr!("608.2b", "603.3d");
    ruling!(
        "Sunshower Druid",
        "If the target creature is an illegal target as Sunshower Druid’s ability tries to resolve, it won’t resolve and none of its effects will happen. You won’t gain 1 life."
    );
    supported("Sunshower Druid");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let druid = t.enter(P0, "Sunshower Druid");
    t.settle();
    assert_eq!(abilities_from(&t, druid), 1);
    bounce(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn divine_favor_with_an_illegal_target_doesnt_enter() {
    cr!("608.3b", "303.4a");
    ruling!(
        "Divine Favor",
        "If the creature targeted by Divine Favor is an illegal target when Divine Favor tries to resolve, Divine Favor won't resolve. It won't enter the battlefield and its enters-the-battlefield ability won't trigger."
    );
    supported("Divine Favor");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Divine Favor", &[Entity::Object(bears)]);
    bounce(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divine Favor"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn tandem_tactics_with_no_targets_or_only_illegal_targets() {
    cr!("608.2b", "115.1d", "601.2c");
    ruling!(
        "Tandem Tactics",
        "You can cast Tandem Tactics with no targets. When it resolves, you’ll gain 2 life. However, if you cast Tandem Tactics with any targets and all of those targets are illegal as it tries to resolve, it won’t resolve and none of its effects will happen. You won’t gain 2 life in that case."
    );
    supported("Tandem Tactics");
    let mut t = TestGame::new(2);
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Tandem Tactics");
    let card = t.hand(P0, "Tandem Tactics");
    t.cast(P0, card).targets(&[]).go();
    t.clear_answers();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Two targets, both gone.
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Raging Goblin");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Tandem Tactics");
    let card = t.hand(P0, "Tandem Tactics");
    let spell = t
        .cast(P0, card)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    assert_eq!(crate::r_s25_common::targets_of(&t, spell).len(), 2);
    bounce(&mut t, a);
    bounce(&mut t, b);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

// ---------------------------------------------------------------------------------------
// Owner versus controller
// ---------------------------------------------------------------------------------------

#[test]
fn the_destroyed_creatures_owner_gains_the_life() {
    cr!("108.3", "110.2");
    ruling!(
        "Misfortune's Gain",
        "The creature's *owner* gains the life, regardless of who happens to control it."
    );
    ruling!(
        "Path of Peace",
        "You are a creature's owner if the card representing it began the game in your deck, or if it's a token that entered the battlefield under your control. If Path of Peace targets a creature that you own but an opponent controls, you'll gain 4 life."
    );
    for card in ["Misfortune's Gain", "Path of Peace"] {
        supported(card);
        // P0's Grizzly Bears, controlled by P1.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        gain_control(&mut t, P1, bears);
        assert_eq!(t.obj_now(bears).controller, P1);
        cast_new(&mut t, P0, card, &[Entity::Object(bears)]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
        assert_eq!((t.life(P0), t.life(P1)), (24, 20), "{card}");
        // A token P1 created, controlled by P0: P1 gains the life.
        let mut t = TestGame::new(2);
        let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
        gain_control(&mut t, P0, token);
        cast_new(&mut t, P0, card, &[Entity::Object(token)]);
        t.resolve_all();
        assert_eq!((t.life(P0), t.life(P1)), (20, 24), "{card}");
    }
}

// ---------------------------------------------------------------------------------------
// Counting after the destruction; last known information
// ---------------------------------------------------------------------------------------

#[test]
fn tidy_conclusion_counts_artifacts_after_destroying_the_creature() {
    cr!("608.2c", "610.3c");
    ruling!(
        "Tidy Conclusion",
        "The number of artifacts you control is counted only as Tidy Conclusion resolves, after the creature is destroyed. For example, if you destroy a Fairgrounds Warden that exiled your artifact creature, the artifact creature will return and then be counted when determining how many artifacts you control."
    );
    // P1's Fairgrounds Warden exiled P0's Ornithopter; P0 also controls a Bottle Gnomes.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bottle Gnomes");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P1, &[Entity::Object(thopter)]);
    let warden = t.enter(P1, "Fairgrounds Warden");
    t.resolve_all();
    assert!(t.in_exile("Ornithopter"));
    cast_new(&mut t, P0, "Tidy Conclusion", &[Entity::Object(warden)]);
    t.resolve();
    assert!(!t.named_on_battlefield("Ornithopter").is_empty());
    assert_eq!(t.life(P0), 22);
}

#[test]
fn life_lost_equal_to_toughness_uses_last_known_information() {
    cr!("608.2h", "613.4c");
    ruling!(
        "Vendetta",
        "The destroyed creature's last existence on the battlefield is checked to determine its toughness."
    );
    ruling!(
        "Dire Tactics",
        "Use the exiled creature’s toughness as it last existed on the battlefield to determine how much life you lose."
    );
    supported("Vendetta");
    supported("Dire Tactics");
    // Grizzly Bears with two +1/+1 counters (4/4).
    for card in ["Vendetta", "Dire Tactics"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        put_counters(&mut t, bears, "+1/+1", 2);
        cast_new(&mut t, P0, card, &[Entity::Object(bears)]);
        t.resolve_all();
        assert!(!t.on_battlefield(bears), "{card}");
        assert_eq!(t.life(P0), 16, "{card}");
    }
}

#[test]
fn dire_tactics_checks_for_a_human_after_exiling() {
    cr!("608.2c", "610.3c");
    ruling!(
        "Dire Tactics",
        "Whether you control a Human is checked only after exiling the target creature. If that creature had exiled a Human you controlled until that creature left the battlefield, that Human will return to the battlefield in time to be counted."
    );
    // P1's Fairgrounds Warden (1/3) exiled P0's Elite Vanguard (a Human).
    let mut t = TestGame::new(2);
    let vanguard = t.battlefield(P0, "Elite Vanguard");
    t.answer_targets(P1, &[Entity::Object(vanguard)]);
    let warden = t.enter(P1, "Fairgrounds Warden");
    t.resolve_all();
    assert!(t.in_exile("Elite Vanguard"));
    cast_new(&mut t, P0, "Dire Tactics", &[Entity::Object(warden)]);
    t.resolve_all();
    assert!(!t.named_on_battlefield("Elite Vanguard").is_empty());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn madame_null_uses_the_power_as_the_ability_resolves() {
    cr!("608.2h", "603.4");
    ruling!(
        "Madame Null, Power Broker",
        "Use the creature's power as Madame Null's last ability resolves to determine how much life you may pay. If that creature is no longer on the battlefield, use its power as it last existed on the battlefield. (You probably won't want to pay life in that case, though.)"
    );
    supported("Madame Null, Power Broker");
    // Grizzly Bears enters (2/2), then gets a +1/+1 counter before the trigger resolves:
    // pay 3, three counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Madame Null, Power Broker");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    put_counters(&mut t, bears, "+1/+1", 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.counters(bears, "+1/+1"), 4);
    // It gets a counter and leaves the battlefield: its last known power (3) is paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Madame Null, Power Broker");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    put_counters(&mut t, bears, "+1/+1", 1);
    bounce(&mut t, bears);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}
