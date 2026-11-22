//! Looking at another player's library, losing keywords, sacrificing groups, and the
//! controller of a destroyed object.

use crate::basic_effects_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The library of `p`, top card first.
fn top(t: &TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    t.g.player(p).library.iter().rev().take(n).copied().collect()
}

#[test]
fn natural_selection_you_reorder_the_targets_library() {
    cr!("401.4");
    assert_supported("Natural Selection");
    let mut t = TestGame::new(2);
    let c = t.library_top(P1, "Forest");
    let b = t.library_top(P1, "Island");
    let a = t.library_top(P1, "Swamp");
    assert_eq!(top(&t, P1, 3), vec![a, b, c]);
    t.lands(P0, "Forest", 1);
    let ns = t.hand(P0, "Natural Selection");
    // P0 (not the library's owner) puts them back: Forest, Swamp, Island.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![2, 0, 1]));
    t.answer_yes(P0, false);
    t.cast(P0, ns).target(P1).go();
    t.resolve();
    assert_eq!(top(&t, P1, 3), vec![c, a, b]);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
}

#[test]
fn thoughtpicker_witch_exiles_one_of_the_top_two() {
    cr!("406.3");
    assert_supported("Thoughtpicker Witch");
    let mut t = TestGame::new(2);
    let second = t.library_top(P1, "Forest");
    let first = t.library_top(P1, "Island");
    let witch = t.battlefield(P0, "Thoughtpicker Witch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, witch, 0, &[Entity::Player(P1)]).unwrap();
    t.answer_choose(P0, &[Entity::Object(second)]);
    t.resolve();
    assert!(t.in_exile("Forest"));
    assert_eq!(top(&t, P1, 1), vec![first]);
}

#[test]
fn scarwood_hag_removes_only_forestwalk() {
    cr!("702.14a", "613.1f");
    assert_supported("Scarwood Hag");
    let mut t = TestGame::new(2);
    let hag = t.battlefield(P0, "Scarwood Hag");
    let walker = t.battlefield(P1, "Shanodin Dryads");
    t.recompute();
    assert!(t.obj_now(walker).has_keyword(mtg_engine::keywords::KeywordKind::Landwalk));
    t.activate(P0, hag, 1, &[Entity::Object(walker)]).unwrap();
    t.resolve();
    t.recompute();
    assert!(!t.obj_now(walker).has_keyword(mtg_engine::keywords::KeywordKind::Landwalk));
}

#[test]
fn canopy_dragon_gains_flying_and_loses_trample() {
    cr!("613.1f");
    assert_supported("Canopy Dragon");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Canopy Dragon");
    t.lands(P0, "Forest", 2);
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve();
    t.recompute();
    use mtg_engine::keywords::KeywordKind as K;
    assert!(t.obj_now(d).has_keyword(K::Flying));
    assert!(!t.obj_now(d).has_keyword(K::Trample));
}

#[test]
fn walking_sponge_removes_the_chosen_keyword() {
    cr!("613.1f");
    assert_supported("Walking Sponge");
    let mut t = TestGame::new(2);
    let sponge = t.battlefield(P0, "Walking Sponge");
    let d = t.battlefield(P1, "Canopy Dragon");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.activate(P0, sponge, 0, &[Entity::Object(d)]).unwrap();
    t.resolve();
    t.recompute();
    assert!(!t
        .obj_now(d)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
}

#[test]
fn sword_of_feast_and_famine_untaps_your_lands() {
    cr!("701.26b");
    assert_supported("Sword of Feast and Famine");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Sword of Feast and Famine");
    assert!(t.g.attach(sword, Entity::Object(bears)));
    let lands = t.lands(P0, "Forest", 2);
    for l in &lands {
        t.g.tap(*l);
    }
    t.hand(P1, "Island");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped));
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn arcum_dagsson_the_controller_sacrifices_it() {
    cr!("701.21a");
    assert_supported("Arcum Dagsson");
    let mut t = TestGame::new(2);
    let arcum = t.battlefield(P0, "Arcum Dagsson");
    let golem = t.battlefield(P1, "Ornithopter");
    t.answer_yes(P1, false);
    t.activate(P0, arcum, 0, &[Entity::Object(golem)]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P1, "Ornithopter"));
}

#[test]
fn rakdos_the_defiler_sacrifices_half_rounded_up() {
    cr!("701.21a", "107.1a");
    assert_supported("Rakdos the Defiler");
    let mut t = TestGame::new(2);
    let rakdos = t.battlefield(P0, "Rakdos the Defiler");
    t.lands(P0, "Swamp", 3);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(rakdos, Entity::Player(P1))], &[]);
    // Three non-Demon permanents: two are sacrificed.
    assert_eq!(t.named_on_battlefield("Swamp").len(), 1);
}

#[test]
fn self_inflicted_wound_life_loss_only_if_a_creature_was_sacrificed() {
    cr!("118.12");
    assert_supported("Self-Inflicted Wound");
    for green in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, if green { "Grizzly Bears" } else { "Hill Giant" });
        t.lands(P0, "Swamp", 2);
        let s = t.hand(P0, "Self-Inflicted Wound");
        t.cast(P0, s).target(P1).go();
        t.resolve();
        assert_eq!(t.life(P1), if green { 18 } else { 20 }, "{green}");
    }
}

#[test]
fn kaervek_and_cinder_cloud_damage_the_creatures_controller() {
    cr!("700.4", "608.2h");
    assert_supported("Kaervek's Purge");
    assert_supported("Cinder Cloud");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 5);
    t.lands(P0, "Swamp", 1);
    let kp = t.hand(P0, "Kaervek's Purge");
    t.cast(P0, kp).x(4).target(giant).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);

    for white in [true, false] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, if white { "Savannah Lions" } else { "Grizzly Bears" });
        t.lands(P0, "Mountain", 5);
        let cc = t.hand(P0, "Cinder Cloud");
        t.cast(P0, cc).target(c).go();
        t.resolve();
        assert_eq!(t.life(P1), if white { 18 } else { 20 }, "{white}");
    }
    // The destroyed creature's power as it last existed on the battlefield (with its
    // counter), not the card's in the graveyard.
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P1, "Savannah Lions");
    t.g.add_counters(Entity::Object(lions), "+1/+1", 2, None);
    t.lands(P0, "Mountain", 5);
    let cc = t.hand(P0, "Cinder Cloud");
    t.cast(P0, cc).target(lions).go();
    t.resolve();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn eye_of_singularity_destroys_permanents_with_the_entering_permanents_name() {
    cr!("201.2", "603.2");
    assert_supported("Eye of Singularity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eye of Singularity");
    let old = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    let forest = t.battlefield(P1, "Forest");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(old));
    assert!(t.on_battlefield(other));
    assert!(t.on_battlefield(forest));
    // "All other permanents": the entering one stays.
    let _ = bears;
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn molten_rain_damages_only_for_a_nonbasic_land() {
    cr!("305.6");
    assert_supported("Molten Rain");
    for basic in [true, false] {
        let mut t = TestGame::new(2);
        let land = t.battlefield(P1, if basic { "Forest" } else { "Tropical Island" });
        t.lands(P0, "Mountain", 3);
        let mr = t.hand(P0, "Molten Rain");
        t.cast(P0, mr).target(land).go();
        t.resolve();
        assert!(!t.on_battlefield(land));
        assert_eq!(t.life(P1), if basic { 20 } else { 18 });
    }
}

#[test]
fn word_of_blasting_damages_the_walls_controller() {
    cr!("202.3");
    assert_supported("Word of Blasting");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Mountain", 2);
    let wb = t.hand(P0, "Word of Blasting");
    t.cast(P0, wb).target(wall).go();
    t.resolve();
    assert!(!t.on_battlefield(wall));
    // Wall of Stone's mana value is 3.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn mageta_destroys_all_other_creatures_without_regeneration() {
    cr!("701.19c");
    assert_supported("Mageta the Lion");
    let mut t = TestGame::new(2);
    let mageta = t.battlefield(P0, "Mageta the Lion");
    let other = t.battlefield(P1, "Drudge Skeletons");
    t.lands(P0, "Plains", 4);
    t.lands(P1, "Swamp", 1);
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    t.activate(P0, mageta, 0, &[]).unwrap();
    // Regenerating the Skeletons doesn't save them.
    t.activate(P1, other, 0, &[]).unwrap();
    t.resolve();
    t.resolve();
    assert!(t.on_battlefield(mageta));
    assert!(!t.on_battlefield(other));
}

#[test]
fn moonlight_hunt_wolves_and_werewolves_deal_damage_to_that_creature() {
    cr!("120.3");
    assert_supported("Moonlight Hunt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Timber Wolves");
    t.battlefield(P0, "Grizzly Bears");
    let target = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Forest", 2);
    let mh = t.hand(P0, "Moonlight Hunt");
    t.cast(P0, mh).target(target).go();
    t.resolve();
    // Only the Wolf's 1 damage.
    assert_eq!(t.obj_now(target).damage, 1);
}

#[test]
fn compel_brutality_a_planeswalker_deals_damage_equal_to_its_loyalty() {
    cr!("306.5b");
    assert_supported("Compel Brutality");
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P0, "Sorin, Lord of Innistrad");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Forest", 2);
    let cb = t.hand(P0, "Compel Brutality");
    t.cast(P0, cb).modes(&[1]).target(pw).target(wurm).go();
    t.resolve();
    assert_eq!(t.obj_now(wurm).damage, 3);
}

#[test]
fn sorin_returns_the_destroyed_creatures_under_your_control() {
    cr!("606.3");
    assert_supported("Sorin, Lord of Innistrad");
    let mut t = TestGame::new(2);
    let sorin = t.battlefield(P0, "Sorin, Lord of Innistrad");
    t.g.objects[sorin.0 as usize]
        .counters
        .insert("loyalty".into(), 6);
    let theirs = t.battlefield(P1, "Craw Wurm");
    t.activate(P0, sorin, 2, &[Entity::Object(theirs)]).unwrap();
    t.resolve();
    let wurms = t.named_on_battlefield("Craw Wurm");
    assert_eq!(wurms.len(), 1);
    assert_eq!(t.obj_now(wurms[0]).controller, P0);
}

#[test]
fn sophina_investigates_for_each_nontoken_attacker() {
    cr!("701.16a");
    assert_supported("Sophina, Spearsage Deserter");
    let mut t = TestGame::new(2);
    let sophina = t.battlefield(P0, "Sophina, Spearsage Deserter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(
        &[(sophina, Entity::Player(P1)), (bears, Entity::Player(P1))],
        &[],
    );
    assert_eq!(t.named_on_battlefield("Clue Token").len(), 2);
}

#[test]
fn intellectual_offering_untaps_your_and_that_players_nonland_permanents() {
    cr!("701.26b");
    assert_supported("Intellectual Offering");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    for o in [mine, theirs, land] {
        t.g.tap(o);
    }
    t.lands(P0, "Island", 5);
    let io = t.hand(P0, "Intellectual Offering");
    t.cast(P0, io).modes(&[1]).go();
    t.resolve();
    assert!(!t.obj_now(mine).tapped && !t.obj_now(theirs).tapped);
    assert!(t.obj_now(land).tapped);
}

#[test]
fn eye_of_singularity_enters_destroying_only_duplicated_names() {
    cr!("201.2");
    assert_supported("Eye of Singularity");
    let mut t = TestGame::new(2);
    let lone = t.battlefield(P1, "Hill Giant");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let f1 = t.battlefield(P1, "Forest");
    let f2 = t.battlefield(P1, "Forest");
    let eye = t.enter(P0, "Eye of Singularity");
    t.resolve_all();
    assert!(t.on_battlefield(lone), "a permanent with a unique name stays");
    assert!(t.on_battlefield(eye));
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert!(t.on_battlefield(f1) && t.on_battlefield(f2), "basic lands are excepted");
}

#[test]
fn hint_of_insanity_discards_only_cards_sharing_a_name_with_another() {
    cr!("201.2");
    assert_supported("Hint of Insanity");
    let mut t = TestGame::new(2);
    let s1 = t.hand(P1, "Shock");
    let s2 = t.hand(P1, "Shock");
    let bolt = t.hand(P1, "Lightning Bolt");
    let f1 = t.hand(P1, "Forest");
    let f2 = t.hand(P1, "Forest");
    t.lands(P0, "Swamp", 3);
    let hint = t.hand(P0, "Hint of Insanity");
    t.cast(P0, hint).target(P1).go();
    t.resolve();
    let hand = t.g.player(P1).hand.clone();
    assert!(!hand.contains(&s1) && !hand.contains(&s2));
    assert!(hand.contains(&bolt), "a card with a unique name stays");
    assert!(hand.contains(&f1) && hand.contains(&f2), "lands stay");
}

#[test]
fn pattern_matcher_another_creature_is_other_than_itself() {
    cr!("201.2");
    ruling!(
        "Pattern Matcher",
        "you can have it look at the first Pattern Matcher"
    );
    assert_supported("Pattern Matcher");
    // Alone, it can't find a copy of itself.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Pattern Matcher");
    let hand = t.hand_size(P0);
    t.enter(P0, "Pattern Matcher");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // With another Pattern Matcher, it finds the third.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pattern Matcher");
    t.library_top(P0, "Pattern Matcher");
    let hand = t.hand_size(P0);
    t.enter(P0, "Pattern Matcher");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
