//! Rulings batch P062 — "Tap target creature. It doesn't untap during its controller's
//! next untap step." can target a creature that's already tapped: tapping it does nothing
//! more (CR 701.26a), and it still doesn't untap during its controller's next untap step
//! (CR 502.3); it untaps as normal in the untap step after that.

use crate::r_p062_common::*;
use crate::r_s01_common::supported;
use crate::r_s05_common::enter;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P1's tapped Grizzly Bears, the target of P0's spell or ability.
fn p1_tapped_bears(t: &mut TestGame) -> ObjectId {
    tapped_creature(t, P1, "Grizzly Bears")
}

/// P0 casts the real spell `name` targeting P1's tapped Bears; the Bears stay tapped
/// through P1's next untap step only.
fn spell_on_tapped_bears(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = p1_tapped_bears(&mut t);
    cast_and_resolve(&mut t, P0, name, &[obj(bears)]);
    assert!(is_tapped(&t, bears));
    misses_one_untap(&mut t, bears, P1);
}

/// The real creature `name` enters under P0's control and its enters ability targets
/// P1's tapped Bears; the Bears stay tapped through P1's next untap step only.
fn etb_on_tapped_bears(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = p1_tapped_bears(&mut t);
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, name);
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    misses_one_untap(&mut t, bears, P1);
}

#[test]
fn crippling_chill_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Crippling Chill",
        "Crippling Chill can target a creature that’s already tapped. It still won’t untap during its controller’s next untap step."
    );
    spell_on_tapped_bears("Crippling Chill");
}

#[test]
fn frost_breath_can_target_tapped_creatures() {
    cr!("701.26a", "502.3");
    ruling!(
        "Frost Breath",
        "Frost Breath can target tapped creatures. If a targeted creature is already tapped when the spell resolves, that creature just remains tapped and doesn't untap during its controller's next untap step."
    );
    spell_on_tapped_bears("Frost Breath");
}

#[test]
fn frostveil_ambush_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Frostveil Ambush",
        "Frostveil Ambush can target a creature that’s already tapped. That creature won’t untap during its controller’s next untap step."
    );
    spell_on_tapped_bears("Frostveil Ambush");
}

#[test]
fn grip_of_the_roil_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Grip of the Roil",
        "Grip of the Roil can target a creature that's already tapped. It still won't untap during its controller's next untap step."
    );
    spell_on_tapped_bears("Grip of the Roil");
}

#[test]
fn ojutais_breath_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Ojutai's Breath",
        "Ojutai’s Breath can target a creature that’s already tapped. It still won’t untap during its controller’s next untap step."
    );
    supported("Ojutai's Breath");
    let mut t = TestGame::new(2);
    let bears = p1_tapped_bears(&mut t);
    cast_and_resolve(&mut t, P0, "Ojutai's Breath", &[obj(bears)]);
    assert!(is_tapped(&t, bears));
    through_untap_step(&mut t, P1);
    assert!(is_tapped(&t, bears));
    // (P0 doesn't cast it again with rebound.)
    t.answer_yes(P0, false);
    untaps_next(&mut t, bears, P1);
}

#[test]
fn rush_of_ice_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Rush of Ice",
        "Rush of Ice can target a creature that’s already tapped. It still won’t untap during its controller’s next untap step."
    );
    spell_on_tapped_bears("Rush of Ice");
}

#[test]
fn sleep_of_the_dead_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Sleep of the Dead",
        "Sleep of the Dead can target a creature that's already tapped. That creature won't untap during its controller's next untap step."
    );
    spell_on_tapped_bears("Sleep of the Dead");
}

#[test]
fn take_into_custody_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Take into Custody",
        "Take into Custody can target a creature that’s already tapped. That creature won’t untap during its controller’s next untap step."
    );
    spell_on_tapped_bears("Take into Custody");
}

#[test]
fn icy_blast_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3", "107.3a");
    ruling!(
        "Icy Blast",
        "Icy Blast can target a creature that’s already tapped. It still won’t untap during its controller’s next untap step."
    );
    supported("Icy Blast");
    // Ferocious: P0 controls a creature with power 4 or greater (Craw Wurm, 6/4).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    let bears = p1_tapped_bears(&mut t);
    t.lands(P0, "Island", 2);
    let blast = t.hand(P0, "Icy Blast");
    t.cast(P0, blast).x(1).target(bears).go();
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    misses_one_untap(&mut t, bears, P1);
}

#[test]
fn rage_of_winter_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3", "715.3d");
    ruling!(
        "Queen of Ice // Rage of Winter",
        "Rage of Winter can target a creature that's already tapped. That creature won't untap during its controller's next untap step."
    );
    supported("Queen of Ice // Rage of Winter");
    let mut t = TestGame::new(2);
    let bears = p1_tapped_bears(&mut t);
    t.lands(P0, "Island", 2);
    let queen = t.hand(P0, "Queen of Ice // Rage of Winter");
    t.cast(P0, queen)
        .method(CastMethod::Half(1))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(t.zone(queen), mtg_engine::object::Zone::Exile);
    assert!(is_tapped(&t, bears));
    misses_one_untap(&mut t, bears, P1);
}

#[test]
fn chillbringers_ability_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Chillbringer",
        "Chillbringer’s ability can target a creature that’s already tapped. That creature won’t untap during its controller’s next untap step."
    );
    etb_on_tapped_bears("Chillbringer");
}

#[test]
fn frost_lynxs_ability_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Frost Lynx",
        "Frost Lynx’s triggered ability can target a creature that’s already tapped. That creature won’t untap during its controller’s next untap step."
    );
    etb_on_tapped_bears("Frost Lynx");
}

#[test]
fn frost_tricksters_ability_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Frost Trickster",
        "Frost Trickster's ability can target a creature that's already tapped. That creature won't untap during its controller's next untap step."
    );
    etb_on_tapped_bears("Frost Trickster");
}

#[test]
fn kor_hookmasters_ability_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Kor Hookmaster",
        "Kor Hookmaster’s ability can target a creature that’s already tapped. It still won’t untap during its controller’s next untap step."
    );
    etb_on_tapped_bears("Kor Hookmaster");
}

#[test]
fn spire_patrols_ability_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3");
    ruling!(
        "Spire Patrol",
        "Spire Patrol’s ability can target a creature that’s already tapped. That creature won’t untap during its controller’s next untap step."
    );
    etb_on_tapped_bears("Spire Patrol");
}

#[test]
fn dovins_second_ability_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3", "606.3", "606.4");
    ruling!(
        "Dovin, Architect of Law",
        "Dovin’s second ability can target a creature that’s already tapped. That creature won’t untap during its controller’s next untap step."
    );
    supported("Dovin, Architect of Law");
    let mut t = TestGame::new(2);
    let bears = p1_tapped_bears(&mut t);
    let dovin = t.battlefield(P0, "Dovin, Architect of Law");
    let loyalty = t.counters(dovin, "loyalty");
    crate::r_s06_common::activate_containing(&mut t, P0, dovin, "Tap target creature")
        .map(|_| ())
        .unwrap_or_else(|e| panic!("{e:?}"));
    // (the target is answered by the scripted default: the only other creature is the
    // Bears)
    t.resolve_all();
    assert_eq!(t.counters(dovin, "loyalty"), loyalty - 1);
    assert!(is_tapped(&t, bears));
    misses_one_untap(&mut t, bears, P1);
}

#[test]
fn kefnets_monuments_ability_doesnt_tap_and_does_nothing_to_an_untapped_creature() {
    cr!("502.3", "603.2");
    ruling!(
        "Kefnet's Monument",
        "The triggered ability of Kefnet's Monument doesn't tap the creature. It can target any creature, tapped or untapped. If that creature is already untapped at the beginning of its controller's next untap step, the effect won't do anything."
    );
    supported("Kefnet's Monument");
    // P0 casts two creature spells: the first trigger targets P1's tapped Bears, the
    // second P1's untapped Hill Giant.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kefnet's Monument");
    let bears = p1_tapped_bears(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    for target in [bears, giant] {
        t.lands(P0, "Forest", 2);
        let c = t.hand(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(target)]);
        t.cast_with(P0, c, &[]).expect("cast a creature spell");
        t.resolve_all();
    }
    // The ability didn't tap the Giant.
    assert!(!is_tapped(&t, giant));
    // P1's next untap step: the Bears stay tapped; the Giant was untapped.
    through_untap_step(&mut t, P1);
    assert!(is_tapped(&t, bears));
    assert!(!is_tapped(&t, giant));
    // The effect on the Giant is used up: tapped later, it untaps in P1's next untap
    // step, as do the Bears.
    tap(&mut t, giant);
    through_untap_step(&mut t, P1);
    assert!(!is_tapped(&t, giant));
    assert!(!is_tapped(&t, bears));
}

#[test]
fn skyline_cascades_ability_doesnt_tap_and_does_nothing_to_an_untapped_creature() {
    cr!("502.3", "603.2");
    ruling!(
        "Skyline Cascade",
        "Skyline Cascade's triggered ability doesn't tap the creature. It can target any creature, tapped or untapped. If that creature is already untapped at the beginning of its controller's next untap step, the effect won't do anything."
    );
    supported("Skyline Cascade");
    // Tapped target: it stays tapped through P1's next untap step.
    let mut t = TestGame::new(2);
    let bears = p1_tapped_bears(&mut t);
    t.answer_targets(P0, &[obj(bears)]);
    let land = enter(&mut t, P0, "Skyline Cascade");
    t.resolve_all();
    assert!(is_tapped(&t, land), "it enters tapped");
    misses_one_untap(&mut t, bears, P1);
    // Untapped target: not tapped; P1's next untap step passes with no effect, so tapped
    // later it untaps as normal.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    enter(&mut t, P0, "Skyline Cascade");
    t.resolve_all();
    assert!(!is_tapped(&t, giant));
    through_untap_step(&mut t, P1);
    assert!(!is_tapped(&t, giant));
    tap(&mut t, giant);
    untaps_next(&mut t, giant, P1);
}

#[test]
fn archipelagores_ability_can_target_a_tapped_creature() {
    cr!("701.26a", "502.3", "702.140d");
    ruling!(
        "Archipelagore",
        "Archipelagore’s triggered ability can target a creature that’s already tapped. That creature won’t untap during its controller’s next untap step."
    );
    supported("Archipelagore");
    // Archipelagore is cast for its mutate cost onto P0's Grizzly Bears (merging on top);
    // it has mutated once, so X is 1.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let bears = p1_tapped_bears(&mut t);
    t.lands(P0, "Island", 6);
    let a = t.hand(P0, "Archipelagore");
    t.cast(P0, a)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(mine)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(mine).chars.name, "Archipelagore");
    assert!(is_tapped(&t, bears));
    misses_one_untap(&mut t, bears, P1);
}
