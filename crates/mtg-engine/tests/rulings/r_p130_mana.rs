//! Rulings batch P130: mana abilities and abilities that add mana (CR 605, 106): which are
//! mana abilities (no stack) and which aren't (loyalty abilities, abilities with targets,
//! triggers on non-mana events), spending restrictions, and mana-producing triggers.

use crate::r_p130_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

// --- Mana abilities: no stack -------------------------------------------------------------

#[test]
fn grinning_ignus_is_a_mana_ability_despite_its_timing_restriction() {
    cr!("605.1a", "605.3b");
    ruling!(
        "Grinning Ignus",
        "Although Grinning Ignus's ability has a timing restriction, it's still a mana ability."
    );
    supported("Grinning Ignus");
    let mut t = TestGame::new(2);
    let ignus = t.battlefield(P0, "Grinning Ignus");
    t.lands(P0, "Mountain", 1);
    t.activate(P0, ignus, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0), [0, 0, 0, 1, 0, 2]);
    assert!(t.in_hand(P0, "Grinning Ignus"));
}

#[test]
fn grinning_ignus_cast_in_the_main_phase_gives_you_priority_first() {
    cr!("117.3b", "608.3");
    ruling!(
        "Grinning Ignus",
        "If you cast Grinning Ignus during your main phase, it will enter the battlefield and you'll receive priority."
    );
    ruling!(
        "Grinning Ignus",
        "If you cast this as normal during your main phase, it will enter the battlefield and you'll receive priority."
    );
    let mut t = TestGame::new(2);
    let ignus = cast_new(&mut t, P0, "Grinning Ignus", &[]);
    t.lands(P0, "Mountain", 1);
    // Both players pass; the spell resolves; the active player receives priority.
    let ok = t.g.run_until(1000, |g| {
        g.stack.is_empty() && g.turn.stage == Stage::Priority && g.turn.priority.is_some()
    });
    assert!(ok);
    assert!(t.on_battlefield(ignus));
    assert_eq!(t.g.turn.priority, Some(P0));
    // P0 activates it at once.
    let ignus = t.g.current(ignus);
    t.activate(P0, ignus, 0, &[]).unwrap();
    assert!(t.in_hand(P0, "Grinning Ignus"));
}

#[test]
fn bag_end_banquets_mana_ability_doesnt_use_the_stack() {
    cr!("605.1a", "605.3b");
    ruling!(
        "Bag End Banquet",
        "Bag End Banquet's last ability is a mana ability."
    );
    supported("Bag End Banquet");
    let mut t = TestGame::new(2);
    let banquet = t.battlefield(P0, "Bag End Banquet");
    create_token(&mut t, P0, "Food");
    create_token(&mut t, P0, "Food");
    t.activate(P0, banquet, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0)[5], 2);
}

#[test]
fn skirge_familiars_ability_is_a_mana_ability() {
    cr!("605.1a", "605.3b");
    ruling!(
        "Skirge Familiar",
        "Skirge Familiar's activated ability is a mana ability."
    );
    supported("Skirge Familiar");
    let mut t = TestGame::new(2);
    let skirge = t.battlefield(P0, "Skirge Familiar");
    let card = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(card)]);
    t.activate(P0, skirge, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0)[2], 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

/// Casts Shimmerwilds Growth on a new Forest of P0's, choosing `color`; returns the land.
fn shimmerwilds_on_forest(t: &mut TestGame, color: usize) -> ObjectId {
    let forest = t.battlefield(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(color));
    t.lands(P0, "Forest", 2);
    let growth = t.hand(P0, "Shimmerwilds Growth");
    t.cast(P0, growth).target(forest).go();
    t.resolve_all();
    t.g.objects[forest.0 as usize].tapped = false;
    t.clear_answers();
    forest
}

#[test]
fn shimmerwilds_growths_triggered_mana_ability_doesnt_use_the_stack() {
    cr!("605.1b", "605.3b", "106.12a");
    ruling!(
        "Shimmerwilds Growth",
        "Shimmerwilds Growth's last ability is a mana ability."
    );
    supported("Shimmerwilds Growth");
    let mut t = TestGame::new(2);
    let forest = shimmerwilds_on_forest(&mut t, 0);
    let growth = t.named_on_battlefield("Shimmerwilds Growth")[0];
    let color = t.obj_now(growth).choices.color.expect("a color was chosen");
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool_total(&t, P0), 2);
    assert_eq!(pool(&t, P0)[4] >= 1, true);
    let extra = match color {
        Color::White => 0,
        Color::Blue => 1,
        Color::Black => 2,
        Color::Red => 3,
        Color::Green => 4,
    };
    assert!(pool(&t, P0)[extra] >= 1);
}

#[test]
fn shimmerwilds_growth_without_a_chosen_color_adds_no_mana() {
    cr!("605.1b", "106.5");
    ruling!(
        "Shimmerwilds Growth",
        "If Shimmerwilds Growth is somehow on the battlefield without a chosen color, its last ability won't add any mana."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let growth = t.battlefield(P0, "Shimmerwilds Growth");
    t.g.attach(growth, obj(forest));
    t.g.recompute();
    assert!(t.obj_now(growth).choices.color.is_none());
    t.activate(P0, forest, 0, &[]).unwrap();
    t.settle();
    assert_eq!(pool(&t, P0), [0, 0, 0, 0, 1, 0]);
}

// --- Not mana abilities: loyalty abilities, targets, triggers ---------------------------

/// Asserts that activating `pw`'s `index`th loyalty ability puts it on the stack (no mana
/// yet), that it can't be activated at instant speed, and that the mana arrives as it
/// resolves.
fn loyalty_mana_uses_the_stack(name: &str, index: usize, mana: [usize; 6]) {
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P0, name);
    // Not during combat (sorcery timing only).
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_activate(&mut t, P0, pw));
    // Not with something on the stack.
    t.set_step(P0, Step::PrecombatMain);
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert!(!can_activate(&mut t, P0, pw));
    t.resolve_all();
    assert!(!t.on_battlefield(bolt));
    assert!(can_activate(&mut t, P0, pw));
    t.activate(P0, pw, index, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(pool(&t, P0), mana);
    // Once per turn.
    assert!(!can_activate(&mut t, P0, pw));
}

#[test]
fn chandra_novice_pyromancers_mana_ability_is_a_loyalty_ability() {
    cr!("605.1a", "606.3", "606.6");
    ruling!(
        "Chandra, Novice Pyromancer",
        "Because it's a loyalty ability, Chandra's second ability isn't a mana ability."
    );
    supported("Chandra, Novice Pyromancer");
    loyalty_mana_uses_the_stack("Chandra, Novice Pyromancer", 1, [0, 0, 0, 2, 0, 0]);
}

#[test]
fn sarkhan_firebloods_mana_ability_is_a_loyalty_ability() {
    cr!("605.1a", "606.3", "606.6");
    ruling!(
        "Sarkhan, Fireblood",
        "Because it's a loyalty ability, Sarkhan's second ability isn't a mana ability."
    );
    supported("Sarkhan, Fireblood");
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan, Fireblood");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_activate(&mut t, P0, sarkhan));
    t.set_step(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, sarkhan, 1, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(pool_total(&t, P0), 2);
}

#[test]
fn chandra_torch_of_defiances_mana_ability_is_a_loyalty_ability() {
    cr!("605.1a", "606.3", "606.6");
    ruling!(
        "Chandra, Torch of Defiance",
        "Loyalty abilities can't be mana abilities. Chandra's second ability uses the stack"
    );
    supported("Chandra, Torch of Defiance");
    loyalty_mana_uses_the_stack("Chandra, Torch of Defiance", 1, [0, 0, 0, 2, 0, 0]);
}

#[test]
fn sarkhans_mana_can_be_spent_only_on_dragon_spells() {
    cr!("106.6", "205.3m");
    ruling!(
        "Sarkhan, Fireblood",
        "A \"Dragon spell\" refers only to a spell that has the Dragon subtype, regardless of its name."
    );
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan, Fireblood");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, sarkhan, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(pool_total(&t, P0), 2);
    // Two more lands: four mana in all. Dragon's Hoard ({3}) isn't a Dragon spell; Shivan
    // Dragon ({4}{R}{R}) is out of reach, but a Dragon costing four is castable.
    t.lands(P0, "Wastes", 2);
    let hoard = t.hand(P0, "Dragon's Hoard");
    assert!(t.cast(P0, hoard).try_go().is_err());
    assert_eq!(pool_total(&t, P0), 2);
    let whelp = t.hand(P0, "Dragon Whelp");
    t.cast(P0, whelp).go();
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn jetfires_ability_has_a_target_so_it_isnt_a_mana_ability() {
    cr!("605.1a", "602.2");
    ruling!(
        "Jetfire, Ingenious Scientist // Jetfire, Air Guardian",
        "Because it has a target, Jetfire, Ingenious Scientist's activated ability isn't a mana ability."
    );
    supported("Jetfire, Ingenious Scientist // Jetfire, Air Guardian");
    let mut t = TestGame::new(2);
    let jet = t.battlefield(P0, "Jetfire, Ingenious Scientist // Jetfire, Air Guardian");
    t.g.add_counters(obj(jet), counters::PLUS1, 2, None);
    t.g.recompute();
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, jet, 0, &[Entity::Player(P0)]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(pool(&t, P0)[5], 2);
}

#[test]
fn jetfires_mana_can_pay_for_activated_abilities() {
    cr!("106.6");
    ruling!(
        "Jetfire, Ingenious Scientist // Jetfire, Air Guardian",
        "Mana that you add with Jetfire, Ingenious Scientist's activated ability can be used on anything that isn't a nonartifact spell."
    );
    let mut t = TestGame::new(2);
    let jet = t.battlefield(P0, "Jetfire, Ingenious Scientist // Jetfire, Air Guardian");
    t.g.add_counters(obj(jet), counters::PLUS1, 1, None);
    t.g.recompute();
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    t.activate(P0, jet, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    assert_eq!(pool(&t, P0)[5], 1);
    // The Bonesplitter's equip ability ({1}) can be paid with it.
    let splitter = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, splitter, 0, &[obj(bears)]).unwrap();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn coal_stokers_ability_uses_the_stack() {
    cr!("605.1b", "603.3");
    ruling!(
        "Coal Stoker",
        "Coal Stoker's ability isn't a mana ability. It uses the stack and can be responded to."
    );
    supported("Coal Stoker");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Coal Stoker", &[]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(is_trigger(&t, top(&t)));
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(pool(&t, P0)[3], 3);
}

#[test]
fn a_clone_of_coal_stoker_cast_from_hand_adds_the_mana() {
    cr!("707.2", "603.4");
    ruling!(
        "Coal Stoker",
        "If a nontoken creature enters the battlefield as a copy of this creature, the copy's enters-the-battlefield ability will still trigger as long as you cast the spell that became that nontoken creature from your hand."
    );
    let mut t = TestGame::new(2);
    let stoker = t.battlefield(P0, "Coal Stoker");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(stoker)]);
    cast_new(&mut t, P0, "Clone", &[]);
    t.resolve();
    t.settle();
    assert_eq!(t.named_on_battlefield("Coal Stoker").len(), 2);
    t.resolve_all();
    assert_eq!(pool(&t, P0)[3], 3);
}

#[test]
fn a_token_copy_of_coal_stoker_wasnt_cast() {
    cr!("707.2", "603.4", "111.1");
    ruling!(
        "Coal Stoker",
        "If a token is created that's a copy of Coal Stoker, the token wasn't cast, and Coal Stoker's ability won't trigger."
    );
    let mut t = TestGame::new(2);
    let stoker = t.battlefield(P0, "Coal Stoker");
    cast_new(&mut t, P0, "Cackling Counterpart", &[obj(stoker)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Coal Stoker").len(), 2);
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn lotus_cobras_landfall_mana_uses_the_stack() {
    cr!("605.1b", "605.5a");
    ruling!(
        "Lotus Cobra",
        "Lotus Cobra's ability isn't a mana ability. Opponents may respond to it before you have that mana."
    );
    ruling!(
        "Lotus Cobra",
        "It isn't a mana ability because the event that causes it to trigger isn't a mana ability."
    );
    supported("Lotus Cobra");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lotus Cobra");
    let land = t.hand(P0, "Forest");
    t.play_land(P0, land).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(pool_total(&t, P0), 1);
}

#[test]
fn liturgy_of_blood_uses_the_stack_and_needs_its_target() {
    cr!("608.2b", "605.1a");
    ruling!(
        "Liturgy of Blood",
        "Liturgy of Blood isn't a mana ability and uses the stack."
    );
    ruling!(
        "Liturgy of Blood",
        "If the target creature is an illegal target when Liturgy of Blood tries to resolve, it won't resolve and none of its effects will happen. You won't add mana."
    );
    supported("Liturgy of Blood");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_new(&mut t, P0, "Liturgy of Blood", &[obj(bears)]);
        assert_eq!(t.stack_len(), 1);
        if respond {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        if respond {
            assert_eq!(pool_total(&t, P0), 0);
        } else {
            assert!(!t.on_battlefield(bears));
            assert_eq!(pool(&t, P0)[2], 3);
        }
    }
}

#[test]
fn traitorous_greed_with_an_illegal_target_adds_no_mana() {
    cr!("608.2b");
    ruling!(
        "Traitorous Greed",
        "If the target creature is an illegal target by the time Traitorous Greed tries to resolve, the spell won't resolve. You won't add two mana."
    );
    supported("Traitorous Greed");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Traitorous Greed", &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0);
}

// --- Mana Echoes --------------------------------------------------------------------------

#[test]
fn mana_echoes_counts_creatures_that_share_a_type() {
    cr!("700.13", "603.3");
    ruling!(
        "Mana Echoes",
        "Mana Echoes counts the number of creatures that share a type, not how many types they share."
    );
    supported("Mana Echoes");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Echoes");
    t.battlefield(P0, "Elite Vanguard");
    t.battlefield(P0, "Elite Vanguard");
    t.battlefield(P0, "Fearless Halberdier");
    t.battlefield(P0, "Moriok Reaver");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.enter(P0, "Dromoka Warrior");
    t.settle();
    t.resolve_all();
    assert_eq!(pool(&t, P0)[5], 5);
}

#[test]
fn mana_echoes_counts_as_it_resolves_and_uses_the_stack() {
    cr!("608.2h", "605.1b");
    ruling!(
        "Mana Echoes",
        "The amount of mana to add is determined only as the triggered ability of Mana Echoes resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Echoes");
    t.answer_yes(P0, true);
    t.enter(P0, "Elite Vanguard");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    // Another Human arrives before it resolves (without triggering it).
    t.battlefield(P0, "Elite Vanguard");
    t.resolve();
    assert_eq!(pool(&t, P0)[5], 2);
}

// --- Spending restrictions ----------------------------------------------------------------

#[test]
fn beastcaller_savants_mana_pays_a_creature_spells_kicker() {
    cr!("106.6", "601.2f");
    ruling!(
        "Beastcaller Savant",
        "Mana produced by Beastcaller Savant can be spent on any part of a creature spell’s total cost"
    );
    supported("Beastcaller Savant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Beastcaller Savant");
    t.lands(P0, "Forest", 4);
    // Kavu Titan ({1}{G}, kicker {2}{G}): five mana in all needs the Savant's.
    let kavu = t.hand(P0, "Kavu Titan");
    t.cast(P0, kavu).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.counters(kavu, counters::PLUS1), 3);
}

#[test]
fn beastcaller_savants_mana_cant_pay_for_abilities_or_token_making_spells() {
    cr!("106.6");
    ruling!(
        "Beastcaller Savant",
        "It can’t be spent to pay the costs of abilities of creatures you control."
    );
    ruling!(
        "Beastcaller Savant",
        "Mana produced by Beastcaller Savant can’t be used to activate an ability or cast an instant or sorcery spell that creates creature tokens."
    );
    let mut t = TestGame::new(2);
    let savant = t.battlefield(P0, "Beastcaller Savant");
    let shade = t.battlefield(P0, "Frozen Shade");
    // The Savant adds {B}.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.activate(P0, savant, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0)[2], 1);
    // Frozen Shade's "{B}: +1/+1" can't be paid with it.
    assert!(t.activate(P0, shade, 0, &[]).is_err());
    assert_eq!(pool(&t, P0)[2], 1);
    // Raise the Alarm ({1}{W}) creates creature tokens but isn't a creature spell.
    t.lands(P0, "Plains", 1);
    let alarm = t.hand(P0, "Raise the Alarm");
    assert!(t.cast(P0, alarm).try_go().is_err());
    assert_eq!(pool(&t, P0)[2], 1);
    // A creature spell ({1}{B}) can use it.
    let corpse = t.hand(P0, "Walking Corpse");
    t.cast(P0, corpse).go();
    assert_eq!(pool_total(&t, P0), 0);
}

// --- Triggered mana / mana in the pool ----------------------------------------------------

#[test]
fn tangleroots_mana_is_added_whether_wanted_or_not_and_empties() {
    cr!("106.4", "500.5");
    ruling!(
        "Tangleroot",
        "The player gets the mana whether they want it or not."
    );
    supported("Tangleroot");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tangleroot");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    t.resolve();
    // The trigger resolved first: P0 has {G}, and the Bears are still on the stack.
    assert_eq!(pool(&t, P0)[4], 1);
    t.resolve_all();
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn grand_warlord_radhas_mana_can_be_spent_before_blockers() {
    cr!("508.2", "117.3c");
    ruling!(
        "Grand Warlord Radha",
        "After Radha’s triggered ability resolves, you can cast spells and activate abilities before blockers are declared."
    );
    supported("Grand Warlord Radha");
    let mut t = TestGame::new(2);
    let radha = t.battlefield(P0, "Grand Warlord Radha");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    attack_with(
        &mut t,
        &[(radha, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve();
    assert_eq!(pool_total(&t, P0), 2);
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
}

#[test]
fn dawns_reflection_adds_two_mana_of_different_colors_chosen_by_the_lands_controller() {
    cr!("106.1a", "605.1b");
    ruling!(
        "Dawn's Reflection",
        "The two mana can be of two different colors. The controller of the land chooses the colors."
    );
    supported("Dawn's Reflection");
    let mut t = TestGame::new(2);
    // P0 enchants P1's Forest; P1 taps it.
    let forest = t.battlefield(P1, "Forest");
    cast_new(&mut t, P0, "Dawn's Reflection", &[obj(forest)]);
    t.resolve_all();
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.activate(P1, forest, 0, &[]).unwrap();
    t.settle();
    let asked: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("mana")))
        .map(|(p, _)| *p)
        .collect();
    assert!(asked.iter().all(|p| *p == P1));
    let p = pool(&t, P1);
    assert_eq!(pool_total(&t, P1), 3);
    assert_eq!(p[0], 1);
    assert_eq!(p[1], 1);
    assert_eq!(p[4], 1);
}

// --- Fellwar Stone ------------------------------------------------------------------------

/// The mana types P0's Fellwar Stone offers with P1 controlling `lands` (None if no mana
/// is added).
fn fellwar(lands: &[&str]) -> (TestGame, Vec<String>) {
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Fellwar Stone");
    for l in lands {
        t.battlefield(P1, l);
    }
    let from = t.asked().len();
    t.activate(P0, stone, 0, &[]).unwrap();
    let offered = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .next()
        .unwrap_or_default();
    (t, offered)
}

#[test]
fn fellwar_stone_checks_effects_not_costs() {
    cr!("106.7");
    ruling!(
        "Fellwar Stone",
        "Fellwar Stone checks the effects of all mana-producing abilities of lands your opponents control, but it doesn't check their costs."
    );
    supported("Fellwar Stone");
    // Vivid Crag without charge counters (and tapped) still could produce any color.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Fellwar Stone");
    let crag = t.battlefield(P1, "Vivid Crag");
    t.g.tap(crag);
    assert_eq!(t.counters(crag, counters::CHARGE), 0);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let from = t.asked().len();
    t.activate(P0, stone, 0, &[]).unwrap();
    let options: Vec<Vec<String>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .collect();
    assert_eq!(options.len(), 1);
    assert_eq!(options[0].len(), 5);
    assert_eq!(pool_total(&t, P0), 1);
}

#[test]
fn fellwar_stone_ignores_spending_restrictions_of_opponents_lands() {
    cr!("106.7");
    ruling!(
        "Fellwar Stone",
        "Fellwar Stone doesn't care about any restrictions or riders your opponents' lands"
    );
    let (t, offered) = fellwar(&["Ancient Ziggurat"]);
    assert_eq!(offered.len(), 5);
    // One unrestricted mana.
    assert_eq!(pool_total(&t, P0), 1);
    assert!(t.g.player(P0).mana_pool.mana[0].restriction.is_none());
}

#[test]
fn fellwar_stone_produces_only_one_mana() {
    cr!("106.7");
    ruling!(
        "Fellwar Stone",
        "It only produces one mana even if the land can produce more than one."
    );
    let (t, _) = fellwar(&["Swamp"]);
    assert_eq!(pool(&t, P0), [0, 0, 1, 0, 0, 0]);
    let (t, _) = fellwar(&["Gemstone Mine"]);
    assert_eq!(pool_total(&t, P0), 1);
}

#[test]
fn fellwar_stone_can_be_activated_when_it_cant_make_mana() {
    cr!("106.7", "605.3a");
    ruling!(
        "Fellwar Stone",
        "The ability can be activated if the opponent has no lands that produce mana, but the effect will not be able to generate any mana."
    );
    let (t, _) = fellwar(&[]);
    assert!(t.obj_now(t.named_on_battlefield("Fellwar Stone")[0]).tapped);
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn fellwar_stone_cant_make_colorless_mana() {
    cr!("106.7", "105.1");
    ruling!(
        "Fellwar Stone",
        "Fellwar Stone can't be tapped for colorless mana, even if a land an opponent controls could produce colorless mana."
    );
    let (t, _) = fellwar(&["Wastes"]);
    assert_eq!(pool_total(&t, P0), 0);
}
