//! The counter grammar (`oracle/patterns/counter_grammar.rs`), putting counters (CR 122):
//! several puts in one instruction, choices among counters, chosen holders ("a creature
//! you control", "one of them") that later pronouns refer to, putting a counter or
//! removing one, counter-qualified object phrases ("with three or more +1/+1 counters on
//! them"), conditions and state triggers on the counters on ~, and effects that last "for
//! as long as it has a [kind] counter on it" (CR 611.2b).

use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn add(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    t.g.add_counters(Entity::Object(id), kind, n, None);
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).chars.has_keyword(k)
}

#[test]
fn incremental_growth_puts_different_numbers_on_three_targets() {
    cr!("115.3", "122.1a");
    ruling!(
        "Incremental Growth",
        "You must choose three different targets in order to cast Incremental Growth."
    );
    assert_supported(&["Incremental Growth"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Forest", 5);
    let g = t.hand(P0, "Incremental Growth");
    t.cast(P0, g)
        .targets(&[Entity::Object(a), Entity::Object(b), Entity::Object(c)])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 2);
    assert_eq!(t.counters(c, "+1/+1"), 3);
    // The same creature can't be chosen for two of the targets.
    let g2 = t.hand(P0, "Incremental Growth");
    t.lands(P0, "Forest", 5);
    let r = t
        .cast(P0, g2)
        .targets(&[Entity::Object(a), Entity::Object(a), Entity::Object(c)])
        .try_go();
    if r.is_ok() {
        t.resolve_all();
        assert_ne!(t.counters(a, "+1/+1"), 1 + 1 + 2);
    }
}

#[test]
fn wicked_slumber_can_put_both_stun_counters_on_one_creature() {
    cr!("122.1d", "608.2c");
    ruling!(
        "Wicked Slumber",
        "The two stun counters can end up on the same creature"
    );
    assert_supported(&["Wicked Slumber"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 4);
    let w = t.hand(P0, "Wicked Slumber");
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.cast(P0, w)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert_eq!(t.counters(b, "stun"), 2);
    assert_eq!(t.counters(a, "stun"), 0);
}

#[test]
fn kaylas_command_chosen_creature_gains_double_strike() {
    cr!("608.2c", "700.2");
    ruling!(
        "Kayla's Command",
        "you don't have to choose a creature for the second mode until the spell resolves"
    );
    assert_supported(&["Kayla's Command"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 4);
    let k = t.hand(P0, "Kayla's Command");
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.cast(P0, k).modes(&[1, 3]).go();
    t.resolve_all();
    assert_eq!(t.counters(b, "+1/+1"), 1);
    assert!(has(&t, b, KeywordKind::DoubleStrike));
    assert!(!has(&t, a, KeywordKind::DoubleStrike));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn dwarven_armorer_puts_the_chosen_kind_of_counter() {
    cr!("122.1a");
    assert_supported(&["Dwarven Armorer"]);
    let mut t = TestGame::new(2);
    let armorer = t.battlefield(P0, "Dwarven Armorer");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, armorer, 0, &[Entity::Object(bear)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+0"), 1);
    assert_eq!(t.pt(bear), (3, 2));
}

#[test]
fn plague_boiler_removes_only_if_there_is_a_counter_and_triggers_at_three() {
    cr!("603.8", "122.1");
    ruling!(
        "Plague Boiler",
        "You can’t choose to remove a counter if there isn’t one there."
    );
    assert_supported(&["Plague Boiler"]);
    let mut t = TestGame::new(2);
    let boiler = t.battlefield(P0, "Plague Boiler");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    t.lands(P0, "Swamp", 6);
    t.lands(P0, "Forest", 3);
    // No counter: a counter is put on it (asking to remove one isn't an option).
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, boiler, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(boiler, "plague"), 1);
    // With one: removing it can be chosen.
    t.clear_answers();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, boiler, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(boiler, "plague"), 0);
    // Three counters: it's sacrificed and every nonland permanent is destroyed.
    add(&mut t, boiler, "plague", 3);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Plague Boiler"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{:?}", t.zone(bear));
    assert!(t.on_battlefield(forest));
}

#[test]
fn homarid_is_smaller_with_one_tide_counter_and_resets_at_four() {
    cr!("603.8", "613.4c");
    assert_supported(&["Homarid"]);
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Homarid");
    add(&mut t, h, "tide", 1);
    t.g.recompute();
    assert_eq!(t.pt(h), (1, 1));
    add(&mut t, h, "tide", 1);
    t.g.recompute();
    assert_eq!(t.pt(h), (2, 2));
    add(&mut t, h, "tide", 1);
    t.g.recompute();
    assert_eq!(t.pt(h), (3, 3));
    // Four: the state trigger removes them all.
    add(&mut t, h, "tide", 1);
    t.resolve_all();
    assert_eq!(t.counters(h, "tide"), 0);
    t.g.recompute();
    assert_eq!(t.pt(h), (2, 2));
}

#[test]
fn ironscale_hydra_prevents_combat_damage_and_grows_once_per_creature() {
    cr!("615.5", "510.2");
    ruling!(
        "Ironscale Hydra",
        "You put only one +1/+1 counter on Ironscale Hydra per creature whose combat damage is prevented"
    );
    assert_supported(&["Ironscale Hydra"]);
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P1, "Ironscale Hydra");
    let giant = t.battlefield(P0, "Craw Wurm");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[(hydra, giant)]);
    assert!(t.on_battlefield(hydra));
    assert_eq!(t.obj_now(hydra).damage, 0);
    assert_eq!(t.counters(hydra, "+1/+1"), 1);
}

#[test]
fn nine_lives_prevents_damage_to_you_and_is_exiled_at_nine() {
    cr!("615.5", "603.8");
    ruling!(
        "Nine Lives",
        "If more than one source deals damage to you at once, prevent the damage from each of them"
    );
    assert_supported(&["Nine Lives"]);
    let mut t = TestGame::new(2);
    let lives = t.battlefield(P0, "Nine Lives");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(
        &[(a, Entity::Player(P0)), (b, Entity::Player(P0))],
        &[],
    );
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(lives, "incarnation"), 2);
    // Nine counters: it's exiled (and its controller then loses the game).
    add(&mut t, lives, "incarnation", 7);
    t.resolve_all();
    assert!(t.in_exile("Nine Lives"));
}

#[test]
fn aven_mimeomancer_effect_lasts_while_the_feather_counter_does() {
    cr!("611.2b", "613.4b");
    ruling!(
        "Aven Mimeomancer",
        "The effect continues even if Aven Mimeomancer leaves the battlefield."
    );
    ruling!(
        "Aven Mimeomancer",
        "The first creature stops being affected by the ability because it no longer has a feather counter on it."
    );
    assert_supported(&["Aven Mimeomancer"]);
    let mut t = TestGame::new(2);
    let aven = t.battlefield(P0, "Aven Mimeomancer");
    let giant = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.set_step(P1, Step::Cleanup);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(giant, "feather"), 1);
    assert_eq!(t.pt(giant), (3, 1));
    assert!(has(&t, giant, KeywordKind::Flying));
    t.g.destroy(aven, None);
    t.settle();
    assert_eq!(t.pt(giant), (3, 1));
    // The counter is gone: so is the effect, even if it gets another one later.
    t.g.remove_counters(Entity::Object(giant), "feather", 1);
    t.g.recompute();
    assert_eq!(t.pt(giant), (6, 4));
    assert!(!has(&t, giant, KeywordKind::Flying));
    add(&mut t, giant, "feather", 1);
    t.g.recompute();
    assert_eq!(t.pt(giant), (6, 4));
}

#[test]
fn aquitects_will_land_is_an_island_while_it_has_a_flood_counter() {
    cr!("611.2b", "305.6");
    assert_supported(&["Aquitect's Will"]);
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Plains");
    t.lands(P0, "Island", 1);
    let w = t.hand(P0, "Aquitect's Will");
    t.cast(P0, w).target(land).go();
    t.resolve_all();
    assert!(t.obj_now(land).chars.has_subtype("Island"));
    assert!(t.obj_now(land).chars.has_subtype("Plains"));
    t.g.remove_counters(Entity::Object(land), "flood", 1);
    t.g.recompute();
    assert!(!t.obj_now(land).chars.has_subtype("Island"));
}

#[test]
fn runadi_gives_haste_to_creatures_with_three_or_more_counters() {
    cr!("122.1a", "613.1f");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Runadi, Behemoth Caller");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    add(&mut t, a, "+1/+1", 3);
    add(&mut t, b, "+1/+1", 2);
    t.g.recompute();
    assert!(has(&t, a, KeywordKind::Haste));
    assert!(!has(&t, b, KeywordKind::Haste));
}

#[test]
fn champions_drake_needs_a_creature_with_three_level_counters() {
    cr!("122.1");
    assert_supported(&["Champion's Drake"]);
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Champion's Drake");
    let bear = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, bear, "level", 2);
    t.g.recompute();
    assert_eq!(t.pt(drake), (1, 1));
    add(&mut t, bear, "level", 1);
    t.g.recompute();
    assert_eq!(t.pt(drake), (4, 4));
}

#[test]
fn bulwark_ox_protects_only_creatures_with_counters() {
    cr!("611.2c");
    assert_supported(&["Bulwark Ox"]);
    let mut t = TestGame::new(2);
    let ox = t.battlefield(P0, "Bulwark Ox");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    add(&mut t, a, "-1/-1", 1);
    t.activate(P0, ox, 0, &[]).unwrap();
    t.resolve_all();
    assert!(has(&t, a, KeywordKind::Hexproof));
    assert!(has(&t, a, KeywordKind::Indestructible));
    assert!(!has(&t, b, KeywordKind::Hexproof));
}

#[test]
fn heartless_act_destroys_only_a_creature_with_no_counters() {
    cr!("115.1", "608.2b");
    ruling!(
        "Heartless Act",
        "If you choose the first mode and the creature gains a counter in response, you can't remove counters from it instead"
    );
    assert_supported(&["Heartless Act"]);
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    add(&mut t, giant, "+1/+1", 1);
    t.lands(P0, "Swamp", 4);
    let h = t.hand(P0, "Heartless Act");
    t.cast(P0, h).modes(&[0]).target(bear).go();
    // A creature with a counter wasn't a legal target for the first mode.
    let offered: Vec<Entity> = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("targets were chosen");
    assert!(offered.contains(&Entity::Object(bear)));
    assert!(!offered.contains(&Entity::Object(giant)));
    // It gains a counter in response: the spell doesn't resolve.
    add(&mut t, bear, "+1/+1", 1);
    t.resolve_all();
    assert!(t.on_battlefield(bear));
    assert_eq!(t.counters(bear, "+1/+1"), 1);
}

#[test]
fn ajani_resolute_gets_loyalty_when_you_gain_life() {
    cr!("122.1e");
    assert_supported(&["Ajani Resolute"]);
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Resolute");
    let before = t.counters(ajani, "loyalty");
    t.g.gain_life(P0, 2);
    t.resolve_all();
    assert_eq!(t.counters(ajani, "loyalty"), before + 1);
}

#[test]
fn political_triumph_sacrifices_draws_and_counters_each_creature() {
    cr!("122.7", "608.2c");
    assert_supported(&["Political Triumph"]);
    let mut t = TestGame::new(2);
    let pt = t.battlefield(P0, "Political Triumph");
    let bear = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, pt, "plan", 3);
    t.settle();
    let hand = t.hand_size(P0);
    add(&mut t, pt, "plan", 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Political Triumph"));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(bear, "+1/+1"), 1);
}

#[test]
fn recursive_recruitment_counts_groups_of_three_cards_when_cast_from_a_graveyard() {
    cr!("702.34a", "107.1a");
    assert_supported(&["Recursive Recruitment"]);
    let mut t = TestGame::new(2);
    for _ in 0..7 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let rr = t.graveyard(P0, "Recursive Recruitment");
    t.lands(P0, "Island", 4);
    t.lands(P0, "Swamp", 4);
    t.cast(P0, rr)
        .method(mtg_engine::object::CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve_all();
    let cadets = t.named_on_battlefield("Cadet");
    assert_eq!(cadets.len(), 2);
    // Seven Grizzly Bears in the graveyard as it resolves (the spell is on the stack):
    // two groups of three.
    for c in cadets {
        assert_eq!(t.counters(c, "+1/+1"), 2);
    }
}

#[test]
fn captain_america_counters_that_hero_and_himself() {
    cr!("603.2", "122.6");
    assert_supported(&["Captain America, Team Leader"]);
    let mut t = TestGame::new(2);
    let cap = t.battlefield(P0, "Captain America, Team Leader");
    let hero = t.enter(P0, "Amateur Hero");
    t.resolve_all();
    assert_eq!(t.counters(hero, "+1/+1"), 1);
    assert_eq!(t.counters(cap, "+1/+1"), 1);
    assert!(has(&t, hero, KeywordKind::Vigilance));
}
