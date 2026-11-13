//! The counter grammar (`oracle/patterns/counter_grammar.rs`, `r122_move_counters.rs`):
//! choosing a kind of counter ("choose a counter on target permanent; put an additional
//! counter of that kind ..."), moving counters between an object and a group (CR 122.5)
//! or between targets with the same controller, exiling a card with counters on it, the
//! same number of each kind of counter, and conditions on the counters an instruction
//! just changed.

use mtg_engine::ability::{Duration, Effect, PlayerRef, Sel};
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

#[test]
fn clockspinning_removes_or_adds_a_counter_of_the_chosen_kind() {
    cr!("122.1", "702.62b");
    ruling!(
        "Clockspinning",
        "The type of counter isn't chosen until resolution."
    );
    ruling!(
        "Clockspinning",
        "If the target is a permanent and it has no counter on it when Clockspinning resolves, nothing happens."
    );
    assert_supported(&["Clockspinning"]);
    let mut t = TestGame::new(2);
    let rift = t.exile(P1, "Rift Bolt");
    add(&mut t, rift, "time", 3);
    t.lands(P0, "Island", 2);
    let c = t.hand(P0, "Clockspinning");
    // Remove that counter.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, c).target(rift).go();
    t.resolve_all();
    assert_eq!(t.counters(rift, "time"), 2);
    // A permanent with counters of two kinds: put another of the chosen kind.
    let giant = t.battlefield(P0, "Hill Giant");
    add(&mut t, giant, "+1/+1", 1);
    add(&mut t, giant, "charge", 1);
    t.lands(P0, "Island", 2);
    let c2 = t.hand(P0, "Clockspinning");
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // "charge"
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // put another
    t.cast(P0, c2).target(giant).go();
    t.resolve_all();
    assert_eq!(t.counters(giant, "charge"), 2);
    assert_eq!(t.counters(giant, "+1/+1"), 1);
    // No counters: nothing happens.
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let c3 = t.hand(P0, "Clockspinning");
    t.cast(P0, c3).target(bear).go();
    t.resolve_all();
    assert!(t.obj_now(bear).counters.is_empty());
}

#[test]
fn ichormoon_gauntlet_adds_a_counter_of_the_chosen_kind() {
    cr!("122.1", "603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ichormoon Gauntlet");
    let giant = t.battlefield(P0, "Hill Giant");
    add(&mut t, giant, "+1/+1", 1);
    add(&mut t, giant, "flying", 1);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // "flying"
    t.resolve_all();
    assert_eq!(t.counters(giant, "flying"), 2);
    assert_eq!(t.counters(giant, "+1/+1"), 1);
}

#[test]
fn contractual_safeguard_can_spread_the_shield_counter_from_its_addendum() {
    cr!("207.2c", "122.1c");
    ruling!(
        "Contractual Safeguard",
        "If you cast this spell during your main phase, you may choose the shield counter for the second part of the spell."
    );
    assert_supported(&["Contractual Safeguard"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Craw Wurm");
    let theirs = t.battlefield(P1, "Hill Giant");
    add(&mut t, b, "+1/+1", 1);
    t.lands(P0, "Plains", 3);
    let cs = t.hand(P0, "Contractual Safeguard");
    // The addendum: a shield counter on A. Then the kind is chosen from A: shield.
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.cast(P0, cs).go();
    t.resolve_all();
    assert_eq!(t.counters(a, "shield"), 1);
    assert_eq!(t.counters(b, "shield"), 1);
    assert_eq!(t.counters(c, "shield"), 1);
    assert_eq!(t.counters(theirs, "shield"), 0);
}

#[test]
fn aven_courier_copies_a_kind_only_onto_a_permanent_without_one() {
    cr!("122.1", "508.1m");
    ruling!(
        "Aven Courier",
        "You choose the kind of counter from a permanent as that ability resolves."
    );
    assert_supported(&["Aven Courier"]);
    let mut t = TestGame::new(2);
    let courier = t.battlefield(P0, "Aven Courier");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, a, "+1/+1", 2);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(courier, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(b, "+1/+1"), 1);
    assert_eq!(t.counters(a, "+1/+1"), 2);
}

#[test]
fn animation_module_gives_a_player_another_counter() {
    cr!("122.1f");
    let mut t = TestGame::new(2);
    let module = t.battlefield(P0, "Animation Module");
    t.g.add_counters(Entity::Player(P1), "poison", 3, None);
    t.lands(P0, "Plains", 3);
    t.activate(P0, module, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.player(P1).counter("poison"), 4);
}

#[test]
fn spike_cannibal_takes_every_plus_one_counter() {
    cr!("122.5", "122.6");
    assert_supported(&["Spike Cannibal"]);
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    add(&mut t, mine, "+1/+1", 2);
    add(&mut t, theirs, "+1/+1", 3);
    add(&mut t, theirs, "charge", 1);
    let spike = t.enter(P0, "Spike Cannibal");
    t.resolve_all();
    assert_eq!(t.counters(spike, "+1/+1"), 1 + 2 + 3);
    assert_eq!(t.counters(mine, "+1/+1"), 0);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
    assert_eq!(t.counters(theirs, "charge"), 1);
}

#[test]
fn aetherborn_marauder_moves_counters_from_several_permanents() {
    cr!("122.5", "107.1c");
    ruling!(
        "Aetherborn Marauder",
        "You may move +1/+1 counters from any number of other permanents you control, not just from one."
    );
    assert_supported(&["Aetherborn Marauder"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Craw Wurm");
    add(&mut t, a, "+1/+1", 3);
    add(&mut t, b, "+1/+1", 2);
    // How many from each (in the order the permanents are found): 2 from A, all 2 from B.
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let m = t.enter(P0, "Aetherborn Marauder");
    t.resolve_all();
    assert_eq!(t.counters(m, "+1/+1"), 4);
    assert_eq!(t.counters(a, "+1/+1") + t.counters(b, "+1/+1"), 1);
}

#[test]
fn bioshift_moves_nothing_once_the_controllers_differ() {
    cr!("122.5", "608.2b");
    ruling!(
        "Bioshift",
        "if the creatures are controlled by different players at that time, no counters will move"
    );
    ruling!("Bioshift", "You decide how many counters to move when Bioshift resolves.");
    assert_supported(&["Bioshift"]);
    for steal in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Hill Giant");
        let b = t.battlefield(P0, "Grizzly Bears");
        add(&mut t, a, "+1/+1", 3);
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Island", 1);
        let s = t.hand(P0, "Bioshift");
        t.answer(P0, DecisionKind::Number, Answer::Number(2));
        t.cast(P0, s)
            .targets(&[Entity::Object(a), Entity::Object(b)])
            .go();
        if steal {
            let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
            ctx.targets = vec![vec![Entity::Object(b)]];
            t.g.exec(
                &Effect::GainControl {
                    what: Sel::Target(0),
                    who: PlayerRef::You,
                    duration: Duration::Permanent,
                },
                &mut ctx,
            );
            t.g.recompute();
        }
        t.resolve_all();
        if steal {
            assert_eq!(t.counters(a, "+1/+1"), 3);
            assert_eq!(t.counters(b, "+1/+1"), 0);
        } else {
            assert_eq!(t.counters(a, "+1/+1"), 1);
            assert_eq!(t.counters(b, "+1/+1"), 2);
        }
    }
}

#[test]
fn black_panther_gains_life_equal_to_the_counters_moved() {
    cr!("122.5", "608.2c");
    let mut t = TestGame::new(2);
    let panther = t.battlefield(P0, "Black Panther, Wakandan King");
    let land = t.battlefield(P0, "Forest");
    let bear = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, land, "+1/+1", 3);
    t.lands(P0, "Plains", 3);
    let hand = t.hand_size(P0);
    t.activate(
        P0,
        panther,
        0,
        &[Entity::Object(land), Entity::Object(bear)],
    )
    .unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 3);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn epochrasite_is_exiled_with_time_counters_and_suspend() {
    cr!("702.62b", "406.3");
    ruling!(
        "Epochrasite",
        "If an effect refers to a \"suspended card,\" that means a card that (1) has suspend, (2) is in exile, and (3) has one or more time counters on it."
    );
    assert_supported(&["Epochrasite"]);
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Epochrasite");
    t.g.destroy(e, None);
    t.resolve_all();
    assert!(t.in_exile("Epochrasite"));
    let card = t.g.exile.iter().copied().find(|o| t.obj_now(*o).chars.name == "Epochrasite").unwrap();
    assert_eq!(t.counters(card, "time"), 3);
    assert!(t.obj_now(card).chars.has_keyword(KeywordKind::Suspend));
}

#[test]
fn blood_spatter_analysis_is_sacrificed_only_by_its_own_ability() {
    cr!("608.2c", "603.12");
    ruling!(
        "Blood Spatter Analysis",
        "If you put a fifth counter on it some other way, it won't be immediately sacrificed."
    );
    let mut t = TestGame::new(2);
    let bsa = t.battlefield(P0, "Blood Spatter Analysis");
    t.graveyard(P0, "Hill Giant");
    add(&mut t, bsa, "bloodstain", 5);
    t.resolve_all();
    assert!(t.on_battlefield(bsa));
    // A creature dies: a sixth counter, then it's sacrificed and a creature card returns.
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bear, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blood Spatter Analysis"));
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn denry_klin_copies_its_counters_onto_an_entering_creature() {
    cr!("122.6", "603.4");
    assert_supported(&["Denry Klin, Editor in Chief"]);
    let mut t = TestGame::new(2);
    let denry = t.battlefield(P0, "Denry Klin, Editor in Chief");
    add(&mut t, denry, "+1/+1", 2);
    add(&mut t, denry, "flying", 1);
    let bear = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 2);
    assert_eq!(t.counters(bear, "flying"), 1);
    assert_eq!(t.counters(denry, "+1/+1"), 2);
}

#[test]
fn evolved_spinoderm_has_trample_or_hexproof_by_its_oil_counters() {
    cr!("122.1", "611.3a");
    assert_supported(&["Evolved Spinoderm"]);
    let mut t = TestGame::new(2);
    let s = t.enter(P0, "Evolved Spinoderm");
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.counters(s, "oil"), 4);
    assert!(t.obj_now(s).chars.has_keyword(KeywordKind::Hexproof));
    assert!(!t.obj_now(s).chars.has_keyword(KeywordKind::Trample));
    t.g.remove_counters(Entity::Object(s), "oil", 2);
    t.g.recompute();
    assert!(t.obj_now(s).chars.has_keyword(KeywordKind::Trample));
    assert!(!t.obj_now(s).chars.has_keyword(KeywordKind::Hexproof));
}

#[test]
fn persistent_constrictor_shrinks_a_creature_of_the_upkeep_player() {
    cr!("122.1a", "503.1a");
    assert_supported(&["Persistent Constrictor"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Persistent Constrictor");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.counters(theirs, "-1/-1"), 1);
}

#[test]
fn dark_depths_makes_marit_lage_when_its_last_ice_counter_is_removed() {
    cr!("603.8");
    ruling!(
        "Dark Depths",
        "It won't trigger again while the ability is on the stack"
    );
    let mut t = TestGame::new(2);
    let depths = t.battlefield(P0, "Dark Depths");
    t.g.remove_counters(Entity::Object(depths), "ice", 10);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Dark Depths"));
    assert_eq!(t.named_on_battlefield("Marit Lage").len(), 1);
}

#[test]
fn agents_toolkit_moves_a_counter_of_the_chosen_kind() {
    cr!("122.5");
    ruling!(
        "Agent's Toolkit",
        "If Agent's Toolkit somehow ends up with counters on it other than the ones it entered with, those may also be moved from it by its triggered ability."
    );
    let mut t = TestGame::new(2);
    let kit = t.battlefield(P0, "Agent's Toolkit");
    add(&mut t, kit, "charge", 1);
    t.answer_yes(P0, true);
    // The kinds on it: +1/+1 (if it entered with them), charge, ... in order; choose
    // "charge".
    let kinds: Vec<String> = t.obj_now(kit).counters.keys().map(|k| k.to_string()).collect();
    let i = kinds.iter().position(|k| k == "charge").unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    let bear = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(bear, "charge"), 1);
    assert_eq!(t.counters(kit, "charge"), 0);
}
