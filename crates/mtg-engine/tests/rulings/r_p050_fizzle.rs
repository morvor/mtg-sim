//! Rulings batch P050 — "deals damage to target ... and you gain life" spells and
//! abilities: if every target is illegal as the spell or ability tries to resolve, it
//! doesn't resolve at all, so you don't gain life either (CR 608.2b).

use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s02_common::destroy;
use crate::r_s05_common::{enter, tokens_with_subtype};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

const BEARS: &str = "Grizzly Bears";

/// Casts `name` from P0's hand targeting P1's Grizzly Bears, destroys the Bears before it
/// resolves, and checks that nobody's life changed.
fn spell_fizzles_on_creature(name: &str, method: CastMethod, setup: fn(&mut TestGame)) {
    supported(name);
    let mut t = TestGame::new(2);
    setup(&mut t);
    let bears = t.battlefield(P1, BEARS);
    let spell = t.hand(P0, name);
    t.cast(P0, spell).method(method.clone()).target(bears).go();
    assert_eq!(t.stack_len(), 1, "{name}: cast");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "{name}: you don't gain life");
    assert_eq!(t.life(P1), 20, "{name}");
    assert!(t.in_graveyard(P0, name) || t.in_exile(name), "{name}: left the stack");
    // With the target still there, it would have dealt damage and gained life.
    let mut t = TestGame::new(2);
    setup(&mut t);
    let spell = t.hand(P0, name);
    t.cast(P0, spell).method(method).target(P1).go();
    t.resolve_all();
    assert!(t.life(P0) > 20, "{name}: gains life normally");
    assert!(t.life(P1) < 20, "{name}: damage normally");
}

/// Casts `name` targeting P1, then P1 gets hexproof (Leyline of Sanctity), and checks that
/// the spell did nothing.
fn spell_fizzles_on_player(name: &str, setup: fn(&mut TestGame)) {
    supported(name);
    supported("Leyline of Sanctity");
    let mut t = TestGame::new(2);
    setup(&mut t);
    let spell = t.hand(P0, name);
    t.cast(P0, spell).target(P1).go();
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "{name}: you don't gain life");
    assert_eq!(t.life(P1), 20, "{name}");
    assert!(t.in_graveyard(P0, name), "{name}: countered on resolution");
}

fn mana(t: &mut TestGame, name: &str) {
    give_mana_for(t, P0, name);
}

#[test]
fn lightning_helix_essence_drain_and_kin_fizzle() {
    cr!("608.2b");
    ruling!(
        "Lightning Helix",
        "If the chosen target is illegal when Lightning Helix tries to resolve, it won't resolve and none of its effects will happen. You won't gain 3 life."
    );
    ruling!(
        "Smiting Helix",
        "If the chosen target is an illegal target by the time Smiting Helix tries to resolve, the spell doesn’t resolve. You won’t gain 3 life."
    );
    ruling!(
        "Essence Drain",
        "If the target permanent or player is an illegal target when Essence Drain tries to resolve, it won’t resolve and none of its effects will happen. You won’t gain 3 life."
    );
    ruling!(
        "Dark Nourishment",
        "If the target permanent or player is an illegal target by the time Dark Nourishment resolves, the entire spell doesn’t resolve. You won’t gain life."
    );
    spell_fizzles_on_creature("Lightning Helix", CastMethod::Normal, |t| {
        mana(t, "Lightning Helix")
    });
    spell_fizzles_on_creature("Smiting Helix", CastMethod::Normal, |t| {
        mana(t, "Smiting Helix")
    });
    spell_fizzles_on_creature("Essence Drain", CastMethod::Normal, |t| {
        mana(t, "Essence Drain")
    });
    // "Dark Nourishment deals 3 damage to any target. You gain 3 life." (two sentences)
    spell_fizzles_on_creature("Dark Nourishment", CastMethod::Normal, |t| {
        mana(t, "Dark Nourishment")
    });
}

#[test]
fn intervention_fizzles() {
    cr!("608.2b");
    ruling!(
        "Integrity // Intervention",
        "If the chosen target is an illegal target by the time Intervention tries to resolve, the spell doesn't resolve. You don't gain 3 life."
    );
    spell_fizzles_on_creature("Integrity // Intervention", CastMethod::Half(1), |t| {
        t.lands(P0, "Mountain", 2);
        t.lands(P0, "Plains", 2);
    });
}

#[test]
fn zenith_flare_fizzles() {
    cr!("608.2b");
    ruling!(
        "Zenith Flare",
        "If the chosen target is an illegal target by the time Zenith Flare tries to resolve, the spell won’t resolve. You won’t gain X life."
    );
    spell_fizzles_on_creature("Zenith Flare", CastMethod::Normal, |t| {
        mana(t, "Zenith Flare");
        // X = 2 (cards with cycling in your graveyard).
        t.graveyard(P0, "Barren Moor");
        t.graveyard(P0, "Barren Moor");
    });
}

#[test]
fn covenant_of_blood_fizzles_and_convoked_creatures_stay_tapped() {
    cr!("608.2b", "702.51a");
    ruling!(
        "Covenant of Blood",
        "If the target permanent or player is an illegal target when Covenant of Blood tries to resolve, it won't resolve and none of its effects will happen. You won't gain any life. (Any creatures you tapped to cast Covenant of Blood remain tapped.)"
    );
    supported("Covenant of Blood");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let helper = t.battlefield(P0, "Walking Corpse");
    let bears = t.battlefield(P1, BEARS);
    let spell = t.hand(P0, "Covenant of Blood");
    t.cast(P0, spell).target(bears).go();
    assert!(t.obj_now(helper).tapped, "convoked");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P0, "Covenant of Blood"));
    assert!(t.obj_now(helper).tapped, "remains tapped");
}

#[test]
fn hurloon_battle_hymn_does_nothing_even_if_kicked() {
    cr!("608.2b");
    ruling!(
        "Hurloon Battle Hymn",
        "If the target creature or planeswalker is an illegal target as Hurloon Battle Hymn tries to resolve, it won't do anything, even if it was kicked."
    );
    supported("Hurloon Battle Hymn");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Plains", 1);
    let bears = t.battlefield(P1, BEARS);
    let spell = t.hand(P0, "Hurloon Battle Hymn");
    t.cast(P0, spell).kicked(true).target(bears).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "no life even though kicked");
    // Kicked, with the target still there: 4 life.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Plains", 1);
    let bears = t.battlefield(P1, BEARS);
    let spell = t.hand(P0, "Hurloon Battle Hymn");
    t.cast(P0, spell).kicked(true).target(bears).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert!(t.in_graveyard(P1, BEARS));
}

#[test]
fn certain_death_illegal_target_vs_indestructible() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Certain Death",
        "If the creature becomes an illegal target for Certain Death, no player gains or loses life. If it’s a legal target but isn’t destroyed"
    );
    supported("Certain Death");
    // Illegal target: no life changes.
    let mut t = TestGame::new(2);
    mana(&mut t, "Certain Death");
    let bears = t.battlefield(P1, BEARS);
    let spell = t.hand(P0, "Certain Death");
    t.cast(P0, spell).target(bears).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // Legal but indestructible: it survives, yet its controller loses 2 and you gain 2.
    let mut t = TestGame::new(2);
    mana(&mut t, "Certain Death");
    let myr = t.battlefield(P1, "Darksteel Myr");
    let spell = t.hand(P0, "Certain Death");
    t.cast(P0, spell).target(myr).go();
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn inevitable_defeat_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Inevitable Defeat",
        "If the target permanent is an illegal target as Inevitable Defeat tries to resolve, it won’t resolve and none of its effects will happen. No player will gain or lose life."
    );
    supported("Inevitable Defeat");
    let mut t = TestGame::new(2);
    mana(&mut t, "Inevitable Defeat");
    let bears = t.battlefield(P1, BEARS);
    let spell = t.hand(P0, "Inevitable Defeat");
    t.cast(P0, spell).target(bears).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // Legal: exiled, 3 life each way.
    let mut t = TestGame::new(2);
    mana(&mut t, "Inevitable Defeat");
    let bears = t.battlefield(P1, BEARS);
    let spell = t.hand(P0, "Inevitable Defeat");
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    assert!(t.in_exile(BEARS));
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
}

#[test]
fn player_targeting_drains_fizzle() {
    cr!("608.2b", "702.11c");
    ruling!(
        "Sorin's Vengeance",
        "If the player is an illegal target when Sorin’s Vengeance tries to resolve, Sorin’s Vengeance won’t resolve and none of its effects will happen. You won’t gain life."
    );
    ruling!(
        "Taste of Blood",
        "If the player is an illegal target when Taste of Blood tries to resolve, Taste of Blood won’t resolve and none of its effects will happen. You won’t gain life."
    );
    spell_fizzles_on_player("Sorin's Vengeance", |t| mana(t, "Sorin's Vengeance"));
    spell_fizzles_on_player("Taste of Blood", |t| mana(t, "Taste of Blood"));
}

#[test]
fn vampires_kiss_makes_no_blood_tokens() {
    cr!("608.2b", "702.11c");
    ruling!(
        "Vampire's Kiss",
        "If the target player is an illegal target as this spell tries to resolve, you won't gain 2 life and won't create any Blood tokens."
    );
    spell_fizzles_on_player("Vampire's Kiss", |t| mana(t, "Vampire's Kiss"));
    let mut t = TestGame::new(2);
    mana(&mut t, "Vampire's Kiss");
    let spell = t.hand(P0, "Vampire's Kiss");
    t.cast(P0, spell).target(P1).go();
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert!(tokens_with_subtype(&t, P0, "Blood").is_empty(), "no Blood tokens");
    // A legal target: 2 life and two Blood tokens.
    let mut t = TestGame::new(2);
    mana(&mut t, "Vampire's Kiss");
    let spell = t.hand(P0, "Vampire's Kiss");
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(tokens_with_subtype(&t, P0, "Blood").len(), 2);
}

#[test]
fn lorehold_command_third_mode_with_both_targets_illegal() {
    cr!("608.2b");
    ruling!(
        "Lorehold Command",
        "If the third mode is chosen and somehow both of its targets are illegal as Lorehold Command tries to resolve, the entire spell does nothing."
    );
    supported("Lorehold Command");
    supported("Leyline of Sanctity");
    // Modes: create a Spirit token, and "deals 3 damage to any target. Target player gains
    // 3 life." The damage targets P1's Bears; the life-gain target is P1, who then gets
    // hexproof; the Bears are destroyed.
    let mut t = TestGame::new(2);
    mana(&mut t, "Lorehold Command");
    let bears = t.battlefield(P1, BEARS);
    let spell = t.hand(P0, "Lorehold Command");
    t.cast(P0, spell)
        .modes(&[0, 2])
        .target(bears)
        .target(P1)
        .go();
    destroy(&mut t, bears);
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "no life gained");
    assert!(
        tokens_with_subtype(&t, P0, "Spirit").is_empty(),
        "the token mode doesn't happen either"
    );
    assert!(t.in_graveyard(P0, "Lorehold Command"));
    // Only one target illegal: the rest of the spell happens.
    let mut t = TestGame::new(2);
    mana(&mut t, "Lorehold Command");
    let bears = t.battlefield(P1, BEARS);
    let spell = t.hand(P0, "Lorehold Command");
    t.cast(P0, spell)
        .modes(&[0, 2])
        .target(bears)
        .target(P0)
        .go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(tokens_with_subtype(&t, P0, "Spirit").len(), 1);
}

#[test]
fn sorin_vampire_lord_and_sorin_markov_loyalty_abilities_fizzle() {
    cr!("608.2b");
    ruling!(
        "Sorin, Vampire Lord",
        "If the chosen target is an illegal target by the time Sorin's second ability tries to resolve, the ability doesn't resolve. You won't gain 4 life."
    );
    ruling!(
        "Sorin Markov",
        "If the targeted permanent or player is an illegal target by the time Sorin’s first ability resolves, the entire ability doesn’t resolve. You won’t gain life."
    );
    for (name, index, gain) in [("Sorin, Vampire Lord", 1, 4), ("Sorin Markov", 0, 2)] {
        supported(name);
        let mut t = TestGame::new(2);
        let sorin = t.battlefield(P0, name);
        let bears = t.battlefield(P1, BEARS);
        t.activate(P0, sorin, index, &[Entity::Object(bears)])
            .expect("activate");
        destroy(&mut t, bears);
        t.resolve_all();
        assert_eq!(t.life(P0), 20, "{name}: no life");
        // Legal target: damage and life.
        let mut t = TestGame::new(2);
        let sorin = t.battlefield(P0, name);
        t.activate(P0, sorin, index, &[Entity::Player(P1)])
            .expect("activate");
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + gain, "{name}: life");
    }
}

#[test]
fn rin_and_seri_ability_fizzles() {
    cr!("608.2b");
    ruling!(
        "Rin and Seri, Inseparable",
        "If the chosen target is an illegal target by the time Rin and Seri's last ability tries to resolve, the ability won't resolve. You won't gain life."
    );
    supported("Rin and Seri, Inseparable");
    let setup = |t: &mut TestGame| {
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Plains", 1);
        // Rin and Seri is a Dog and a Cat: 1 damage, 1 life.
        t.battlefield(P0, "Rin and Seri, Inseparable")
    };
    let mut t = TestGame::new(2);
    let rs = setup(&mut t);
    let bears = t.battlefield(P1, BEARS);
    t.activate(P0, rs, 0, &[Entity::Object(bears)]).unwrap();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    let mut t = TestGame::new(2);
    let rs = setup(&mut t);
    t.activate(P0, rs, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn blood_hustler_ability_fizzles() {
    cr!("608.2b", "702.11c");
    ruling!(
        "Blood Hustler",
        "If the target opponent is an illegal target as Blood Hustler’s activated ability tries to resolve, it won’t resolve and none of its effects will happen. You won’t gain life."
    );
    supported("Blood Hustler");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let h = t.battlefield(P0, "Blood Hustler");
    t.activate(P0, h, 0, &[Entity::Player(P1)]).unwrap();
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn fear_of_lost_teeth_trigger_fizzles() {
    cr!("608.2b");
    ruling!(
        "Fear of Lost Teeth",
        "If the target of Fear of Lost Teeth's ability is an illegal target as the ability tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    supported("Fear of Lost Teeth");
    let mut t = TestGame::new(2);
    let fear = t.battlefield(P0, "Fear of Lost Teeth");
    let bears = t.battlefield(P1, BEARS);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    destroy(&mut t, fear);
    assert_eq!(t.stack_len(), 1, "dies trigger");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn ukkima_leaves_trigger_fizzles() {
    cr!("608.2b", "702.11c");
    ruling!(
        "Ukkima, Stalking Shadow",
        "If the target player is an illegal target by the time Ukkima's last ability tries to resolve, the ability won't resolve. You won't gain any life."
    );
    supported("Ukkima, Stalking Shadow");
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Ukkima, Stalking Shadow");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, u);
    assert_eq!(t.stack_len(), 1, "leaves trigger");
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // Without hexproof: 2 damage, 2 life (its last known power).
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Ukkima, Stalking Shadow");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, u);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn sorin_imperious_bloodlord_reflexive_trigger_fizzles() {
    cr!("608.2b", "603.12");
    ruling!(
        "Sorin, Imperious Bloodlord",
        "If the target of the reflexive triggered ability is an illegal target as that ability tries to resolve, it doesn’t resolve. You won’t gain 3 life."
    );
    supported("Sorin, Imperious Bloodlord");
    let mut t = TestGame::new(2);
    let sorin = t.battlefield(P0, "Sorin, Imperious Bloodlord");
    let vamp = t.battlefield(P0, "Vampire Neonate");
    let bears = t.battlefield(P1, BEARS);
    t.activate(P0, sorin, 1, &[]).unwrap();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(vamp)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Vampire Neonate"), "sacrificed");
    assert_eq!(t.stack_len(), 1, "reflexive trigger");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn dread_presence_second_mode_fizzles() {
    cr!("608.2b", "700.2b");
    ruling!(
        "Dread Presence",
        "If you choose the second mode and the chosen target is an illegal target by the time Dread Presence's triggered ability tries to resolve, the ability doesn't resolve."
    );
    supported("Dread Presence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dread Presence");
    let bears = t.battlefield(P1, BEARS);
    t.answer(P0, DecisionKind::Modes, mtg_engine::decision::Answer::Indices(vec![1]));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let hand = t.hand_size(P0);
    enter(&mut t, P0, "Swamp");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "no life gained, no life lost");
    assert_eq!(t.hand_size(P0), hand, "it didn't switch to the first mode");
}
