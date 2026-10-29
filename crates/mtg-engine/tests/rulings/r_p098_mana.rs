//! Rulings batch P098 — mana abilities and mana effects: mana abilities with extra effects
//! (Avid Reclaimer, Bog Witch, Quirion Explorer), "whenever you tap this for mana" (Zhur-Taa
//! Druid), Magus Lucea Kane's copy rider, Horizon Stone, Celestial Dawn, Deep Water,
//! Damping Sphere and Selvala's last known power.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::{attach_new, give_control};
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn pool_of(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort_by_key(|m| format!("{m:?}"));
    v
}

#[test]
fn avid_reclaimer_mana_and_life_immediately_even_with_nothing_to_spend_on() {
    cr!("605.1a", "605.3a", "605.3b");
    ruling!(
        "Avid Reclaimer",
        "Avid Reclaimer's ability is a mana ability. It doesn't use the stack and can't be responded to. You'll immediately add mana and gain 2 life, if applicable."
    );
    ruling!(
        "Avid Reclaimer",
        "You can activate Avid Reclaimer's ability even if you don't have anything to spend that mana on. You'll still gain 2 life if you control a Nissa planeswalker."
    );
    supported("Avid Reclaimer");
    for nissa in [false, true] {
        let mut t = TestGame::new(2);
        let reclaimer = t.battlefield(P0, "Avid Reclaimer");
        if nissa {
            t.battlefield(P0, "Nissa, Who Shakes the World");
        }
        // P0's hand is empty: there's nothing to spend the mana on.
        assert_eq!(t.hand_size(P0), 0);
        assert!(tap_for_mana(&mut t, P0, reclaimer, "Add"));
        assert_eq!(t.stack_len(), 0, "nissa: {nissa}");
        assert_eq!(t.g.player(P0).mana_pool.total(), 1);
        assert_eq!(t.life(P0), if nissa { 22 } else { 20 });
    }
}

#[test]
fn bog_witch_discard_ability_is_a_mana_ability() {
    cr!("605.1a", "605.3b");
    ruling!("Bog Witch", "The ability is a mana ability.");
    supported("Bog Witch");
    let mut t = TestGame::new(2);
    let witch = t.battlefield(P0, "Bog Witch");
    t.hand(P0, "Grizzly Bears");
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::B, 1);
    assert!(tap_for_mana(&mut t, P0, witch, "Discard"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool_of(&t, P0), vec![ManaType::B; 3]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn quirion_explorer_with_only_colorless_lands_opposing_adds_nothing() {
    cr!("106.7", "605.1a");
    ruling!(
        "Quirion Explorer",
        "If the opponent only has lands that produce colorless or no mana, this card’s ability can still be activated; it just won’t produce any mana."
    );
    supported("Quirion Explorer");
    let mut t = TestGame::new(2);
    let explorer = t.battlefield(P0, "Quirion Explorer");
    t.battlefield(P1, "Wastes");
    t.recompute();
    let uid = t.obj_now(explorer).chars.abilities[0].uid;
    t.g.turn.priority = Some(P0);
    assert!(t.g.activate_ability(P0, explorer, uid).is_ok());
    assert!(t.obj_now(explorer).tapped);
    assert!(t.g.player(P0).mana_pool.is_empty());
    // With a Forest on P1's side it adds {G}.
    let mut t = TestGame::new(2);
    let explorer = t.battlefield(P0, "Quirion Explorer");
    t.battlefield(P1, "Forest");
    assert!(tap_for_mana(&mut t, P0, explorer, "Add"));
    assert_eq!(pool_of(&t, P0), vec![ManaType::G]);
}

#[test]
fn zhur_taa_druid_trigger_uses_the_stack_above_the_spell_it_paid_for() {
    cr!("605.4a", "603.3");
    ruling!(
        "Zhur-Taa Druid",
        "If you tap Zhur-Taa Druid for mana during the process of casting a spell or activating an ability, its triggered ability will resolve before that spell or ability."
    );
    ruling!(
        "Zhur-Taa Druid",
        "Zhur-Taa Druid’s last ability isn’t a mana ability. It uses the stack and can be responded to."
    );
    supported("Zhur-Taa Druid");
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Zhur-Taa Druid");
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    t.settle();
    assert!(t.obj_now(druid).tapped);
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_on_stack(&t, "deals 1 damage"), 1);
    let top = *t.g.stack.last().unwrap();
    assert!(matches!(
        t.g.obj(top).stack.as_ref().unwrap().kind,
        mtg_engine::object::StackKind::Triggered { .. }
    ));
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

#[test]
fn zhur_taa_druid_triggers_only_when_tapped_for_mana_by_any_mana_ability() {
    cr!("605.1a", "603.2");
    ruling!(
        "Zhur-Taa Druid",
        "Zhur-Taa Druid’s last ability doesn’t trigger if it becomes tapped for any other reason. However, if Zhur-Taa Druid gains another mana ability that requires you tap it, its ability will trigger."
    );
    supported("Paradise Mantle");
    // Tapped by attacking: no trigger.
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Zhur-Taa Druid");
    attack_with(&mut t, &[(druid, Entity::Player(P1))]);
    assert!(t.obj_now(druid).tapped);
    assert_eq!(triggers_on_stack(&t, "deals 1 damage"), 0);
    // Tapped by an effect: no trigger.
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Zhur-Taa Druid");
    t.g.tap(druid);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Paradise Mantle: "Equipped creature has '{T}: Add one mana of any color.'"
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Zhur-Taa Druid");
    attach_new(&mut t, P0, "Paradise Mantle", druid);
    assert!(tap_for_mana(&mut t, P0, druid, "any color"));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "deals 1 damage"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

/// P0 controls Magus Lucea Kane and has activated its mana ability this turn: "{T}: Add
/// {C}{C}. When you next cast a spell with {X} in its mana cost or activate an ability with
/// {X} in its activation cost this turn, copy that spell or ability."
fn magus(t: &mut TestGame) {
    supported("Magus Lucea Kane");
    let magus = t.battlefield(P0, "Magus Lucea Kane");
    assert!(tap_for_mana(t, P0, magus, "Add"));
    t.g.players[P0.idx()].mana_pool = Default::default();
}

#[test]
fn magus_lucea_kane_doesnt_copy_a_cost_with_x_that_isnt_the_mana_symbol() {
    cr!("707.10", "603.7c");
    ruling!(
        "Magus Lucea Kane",
        "A spell or ability with a cost that includes X, but not the mana symbol {X}, won't be copied."
    );
    ruling!(
        "Magus Lucea Kane",
        "Spells with additional costs that include {X} won't be affected by Magus Lucea Kane. {X} must be in the spell's mana cost."
    );
    supported("Brain in a Jar");
    supported("Waterbender's Restoration");
    let mut t = TestGame::new(2);
    magus(&mut t);
    // Brain in a Jar: "{3}, {T}, Remove X charge counters from this artifact: Scry X."
    let jar = t.battlefield(P0, "Brain in a Jar");
    t.g.objects[jar.0 as usize]
        .counters
        .insert(counters::CHARGE.into(), 2);
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    crate::r_s06_common::activate_containing(&mut t, P0, jar, "Scry X").expect("activate");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // Waterbender's Restoration: "As an additional cost to cast this spell, waterbend
    // {X}. Exile X target creatures you control. ..."
    let bears = t.battlefield(P0, "Grizzly Bears");
    let resto = t.hand(P0, "Waterbender's Restoration");
    t.lands(P0, "Island", 3);
    t.cast(P0, resto).x(1).target(bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // The rider is still waiting: Blaze ({X}{R}) is copied.
    let blaze = t.hand(P0, "Blaze");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, blaze).x(1).target(Entity::Player(P1)).go();
    t.settle();
    // Blaze and Magus's copy trigger.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn magus_lucea_kane_copy_makes_its_resolution_choices_separately() {
    cr!("707.10", "608.2c");
    ruling!(
        "Magus Lucea Kane",
        "Any choices made when the spell or ability resolves won't have been made yet when it's copied. Any such choices will be made separately when the copy resolves."
    );
    supported("Venarian Glimmer");
    let mut t = TestGame::new(2);
    magus(&mut t);
    let bolt = t.hand(P1, "Lightning Bolt");
    let shock = t.hand(P1, "Shock");
    // "Target player reveals their hand. You choose a nonland card with mana value X or
    // less from it. That player discards that card."
    let glimmer = t.hand(P0, "Venarian Glimmer");
    t.lands(P0, "Island", 2);
    t.cast(P0, glimmer).x(1).target(Entity::Player(P1)).go();
    t.settle();
    // The spell and Magus's trigger, which copies it as it resolves.
    assert_eq!(t.stack_len(), 2);
    let chooses = |t: &TestGame| {
        t.asked()
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. }))
            .count()
    };
    assert_eq!(chooses(&t), 0);
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(chooses(&t), 0);
    t.answer_choose(P0, &[Entity::Object(shock)]);
    t.resolve_all();
    // The copy's choice (Shock), then the original's: only Lightning Bolt was left.
    assert_eq!(chooses(&t), 2);
    assert!(t.in_graveyard(P1, "Shock"));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    let _ = bolt;
}

#[test]
fn horizon_stone_keeps_mana_as_colorless_with_its_restrictions() {
    cr!("500.5", "614.1a", "106.4");
    ruling!(
        "Horizon Stone",
        "As long as Horizon Stone remains under your control, you'll retain unspent mana as steps and phases end, although that mana will become colorless. This means you can add mana and spend it during a future step, phase, or turn. Once Horizon Stone leaves your control, you'll have until the end of the current step or phase to spend the mana before it is lost."
    );
    ruling!(
        "Horizon Stone",
        "If unspent mana you have has any restrictions or riders associated with it (for example, if it was produced by Jeweled Lotus), those restrictions or riders remain associated with that mana when it becomes colorless."
    );
    supported("Horizon Stone");
    supported("Jeweled Lotus");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Horizon Stone");
    let forest = t.battlefield(P0, "Forest");
    let lotus = t.battlefield(P0, "Jeweled Lotus");
    assert!(tap_for_mana(&mut t, P0, forest, "Add"));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    assert!(tap_for_mana(&mut t, P0, lotus, "Add"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 4);
    t.advance_to(P0, Step::BeginningOfCombat);
    let pool = &t.g.player(P0).mana_pool.mana;
    assert_eq!(pool.len(), 4);
    assert!(pool.iter().all(|m| m.ty == ManaType::C));
    // The Lotus mana still can only be spent to cast a commander.
    assert_eq!(pool.iter().filter(|m| m.restriction.is_some()).count(), 3);
    // Spendable in a later turn (on generic costs).
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.player(P0).mana_pool.total(), 4);
    // Horizon Stone leaves: the mana lasts until the end of this step.
    destroy(&mut t, stone);
    assert_eq!(t.g.player(P0).mana_pool.total(), 4);
    t.advance_to(P1, Step::Draw);
    assert!(t.g.player(P0).mana_pool.is_empty());
}

#[test]
fn celestial_dawn_characteristics_lands_and_symbols() {
    cr!("305.7", "611.3a", "105.3");
    ruling!(
        "Celestial Dawn",
        "Celestial Dawn does not change the type of lands which are not on the battlefield. Lands not on the battlefield are what they say they are."
    );
    ruling!(
        "Celestial Dawn",
        "Celestial Dawn no longer changes mana symbols on cards."
    );
    ruling!(
        "Celestial Dawn",
        "If a permanent enters while Celestial Dawn is on the battlefield, and then Celestial Dawn leaves the battlefield, the permanent's color will revert to the colors in its mana cost."
    );
    supported("Celestial Dawn");
    let mut t = TestGame::new(2);
    let dawn = t.battlefield(P0, "Celestial Dawn");
    let in_play = t.battlefield(P0, "Island");
    let in_hand = t.hand(P0, "Island");
    let in_library = t.library_top(P0, "Island");
    let bolt = t.hand(P0, "Lightning Bolt");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.recompute();
    let sub = |t: &TestGame, id: ObjectId, s: &str| t.obj_now(id).chars.has_subtype(s);
    assert!(sub(&t, in_play, "Plains") && !sub(&t, in_play, "Island"));
    for id in [in_hand, in_library] {
        assert!(sub(&t, id, "Island") && !sub(&t, id, "Plains"));
    }
    // Lightning Bolt is white but still costs {R}.
    assert_eq!(t.obj_now(bolt).chars.colors, ColorSet::single(Color::White));
    let cost = t.obj_now(bolt).chars.mana_cost.clone().unwrap();
    assert_eq!(format!("{cost}"), "{R}");
    assert_eq!(t.obj_now(bears).chars.colors, ColorSet::single(Color::White));
    destroy(&mut t, dawn);
    t.recompute();
    assert_eq!(t.obj_now(bears).chars.colors, ColorSet::single(Color::Green));
}

#[test]
fn celestial_dawn_nonbasic_lands_lose_their_enters_abilities_before_they_apply() {
    cr!("305.7", "614.12");
    ruling!(
        "Celestial Dawn",
        "If a nonbasic land has an ability that causes it to enter the battlefield tapped, it will lose that ability before it applies. The same is also true of any other abilities that modify how a land enters the battlefield or apply \"as\" a land enters the battlefield, such as the first ability of Cavern of Souls."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestial Dawn");
    let gate = t.hand(P0, "Dimir Guildgate");
    t.play_land(P0, gate).unwrap();
    t.settle();
    let gate = t.g.current(gate);
    assert_eq!(t.zone(gate), Zone::Battlefield);
    assert!(!t.obj_now(gate).tapped);
    // Cavern of Souls: no creature type is chosen as it enters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestial Dawn");
    let cavern = t.hand(P0, "Cavern of Souls");
    let from = t.asked().len();
    t.play_land(P0, cavern).unwrap();
    t.settle();
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseOption { .. } | Decision::NameCard { .. })));
    // Without Celestial Dawn, the Guildgate enters tapped.
    let mut t = TestGame::new(2);
    let gate = t.hand(P0, "Dimir Guildgate");
    t.play_land(P0, gate).unwrap();
    assert!(t.obj_now(t.g.current(gate)).tapped);
}

#[test]
fn celestial_dawn_unspent_nonwhite_mana_pays_only_generic() {
    cr!("609.4b");
    ruling!(
        "Celestial Dawn",
        "If you have unspent nonwhite mana when Celestial Dawn enters, you still have that mana, but you can spend it only on the colorless part of costs."
    );
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::G, 1);
    t.enter(P0, "Celestial Dawn");
    t.settle();
    assert_eq!(pool_of(&t, P0), vec![ManaType::G]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = t.hand(P0, "Giant Growth");
    assert!(t.cast(P0, growth).target(bears).try_go().is_err());
    let ring = t.hand(P0, "Sol Ring");
    t.cast(P0, ring).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sol Ring").len(), 1);
    assert!(t.g.player(P0).mana_pool.is_empty());
}

#[test]
fn deep_water_applies_to_lands_gained_later_this_turn() {
    cr!("106.12b", "611.2c");
    ruling!(
        "Deep Water",
        "Deep Waters affects lands you control when it resolves and any lands you gain control of this turn."
    );
    supported("Deep Water");
    let mut t = TestGame::new(2);
    let water = t.battlefield(P0, "Deep Water");
    let mine = t.battlefield(P0, "Forest");
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::U, 1);
    t.activate(P0, water, 0, &[]).expect("activate");
    t.resolve_all();
    let theirs = t.battlefield(P1, "Mountain");
    give_control(&mut t, theirs, P0);
    assert!(tap_for_mana(&mut t, P0, mine, "Add"));
    assert!(tap_for_mana(&mut t, P0, theirs, "Add"));
    assert_eq!(pool_of(&t, P0), vec![ManaType::U, ManaType::U]);
    // Next turn, it no longer applies.
    t.advance_to(P1, Step::PrecombatMain);
    let forest = t.battlefield(P0, "Forest");
    assert!(tap_for_mana(&mut t, P0, forest, "Add"));
    assert_eq!(pool_of(&t, P0), vec![ManaType::G]);
}

#[test]
fn damping_sphere_and_another_mana_replacement_the_controller_picks_the_order() {
    cr!("616.1", "616.1e", "106.12b");
    ruling!(
        "Damping Sphere",
        "If multiple replacement effects would modify the mana an ability you control produces, choose one to apply. After that, determine if any others are applicable. A replacement effect can’t apply to the same event more than once."
    );
    supported("Damping Sphere");
    // Ancient Tomb ({T}: Add {C}{C}) under Damping Sphere and an active Deep Water.
    let mut outcomes = Vec::new();
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Damping Sphere");
        let water = t.battlefield(P0, "Deep Water");
        let tomb = t.battlefield(P0, "Ancient Tomb");
        t.g.players[P0.idx()].mana_pool.add_type(ManaType::U, 1);
        t.activate(P0, water, 0, &[]).expect("activate");
        t.resolve_all();
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        assert!(tap_for_mana(&mut t, P0, tomb, "Add"));
        assert!(asked_since(&t, from)
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseReplacement { .. })));
        outcomes.push(pool_of(&t, P0));
    }
    outcomes.sort_by_key(|v| format!("{v:?}"));
    // Damping Sphere first: one {C}, which Deep Water turns into {U}. Deep Water first:
    // {U}{U}, then Damping Sphere (still a land tapped for two mana): one {C}.
    assert_eq!(outcomes, vec![vec![ManaType::C], vec![ManaType::U]]);
}

#[test]
fn selvala_uses_the_entering_creatures_last_known_power() {
    cr!("608.2h");
    ruling!(
        "Selvala, Heart of the Wilds",
        "If the new creature isn't on the battlefield as the first ability resolves, use its power when it left the battlefield to determine whether its controller may draw a card. Note that effects that reduced its power before it left the battlefield will apply."
    );
    supported("Selvala, Heart of the Wilds");
    for shrunk in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Selvala, Heart of the Wilds");
        let giant = t.enter(P1, "Hill Giant");
        t.settle();
        assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
        if shrunk {
            // Weakness (-2/-1): power 1, not greater than Selvala's 2.
            attach_new(&mut t, P0, "Weakness", giant);
            assert_eq!(t.pt(giant).0, 1);
        }
        destroy(&mut t, giant);
        t.answer_yes(P1, true);
        t.resolve_all();
        assert_eq!(t.hand_size(P1), usize::from(!shrunk), "shrunk: {shrunk}");
    }
}


/// Selvala's comparison is against each other creature (Selvala included), one at a time:
/// not against the total of their powers, nor excluding Selvala.
#[test]
fn selvala_compares_the_power_with_each_other_creature() {
    cr!("608.2h");
    // (creature entering, other creatures, draws)
    let cases: [(&str, &[&str], bool); 4] = [
        ("Hill Giant", &[], true),
        ("Hill Giant", &["Grizzly Bears", "Llanowar Elves"], true),
        ("Grizzly Bears", &[], false),
        ("Hill Giant", &["Hill Giant"], false),
    ];
    for (entering, others, draws) in cases {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Selvala, Heart of the Wilds");
        for o in others {
            t.battlefield(P0, o);
        }
        t.enter(P1, entering);
        t.answer_yes(P1, true);
        t.resolve_all();
        assert_eq!(t.hand_size(P1), usize::from(draws), "{entering} {others:?}");
    }
}
