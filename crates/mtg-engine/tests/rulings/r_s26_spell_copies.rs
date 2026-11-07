//! Rulings batch S26 — copies of spells (CR 707.10): a copy isn't cast, it resolves
//! before the original, it has the original's modes, targets (unless new ones are
//! chosen, CR 707.10c, 115.7), value of X and the effects of its additional costs (CR
//! 707.2), and choices made on resolution are made separately for it (CR 608.2).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s04_common::stack_items;
use crate::r_s24_common::choose_creature_type;
use crate::r_s26_common::*;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The Elemental tokens Young Pyromancer made for `p` (one per instant or sorcery spell
/// cast).
fn elementals(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.name == "Elemental Token")
        .count()
}

#[test]
fn doublecast_copy_keeps_its_targets_unless_new_legal_ones_are_chosen() {
    cr!("707.10c", "115.7d");
    ruling!(
        "Doublecast",
        "The copy will have the same targets as the spell it’s copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    supported("Doublecast");
    supported("Arc Trail");
    // Arc Trail: 2 damage to any target and 1 damage to another target.
    let setup = || {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 4);
        let dc = t.hand(P0, "Doublecast");
        t.cast(P0, dc).go();
        t.resolve_all();
        let bears = t.battlefield(P1, "Grizzly Bears");
        let scout = t.battlefield(P1, "Gladecover Scout");
        let elves = t.battlefield(P1, "Llanowar Elves");
        (t, bears, scout, elves)
    };
    // No new targets: the copy has the same ones.
    let (mut t, bears, _, elves) = setup();
    let trail = t.hand(P0, "Arc Trail");
    t.cast(P0, trail)
        .target(P1)
        .target(elves)
        .go();
    t.answer_yes(P0, false);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(
        targets_on_stack(&t, copy),
        vec![Entity::Player(P1), Entity::Object(elves)]
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.on_battlefield(bears));
    // One of the two changed, the other left alone.
    let (mut t, bears, _, elves) = setup();
    let trail = t.hand(P0, "Arc Trail");
    t.cast(P0, trail)
        .target(P1)
        .target(elves)
        .go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(
        targets_on_stack(&t, copy),
        vec![Entity::Object(bears), Entity::Object(elves)]
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(!t.g.is_live(bears));
    // An illegal new target (the hexproof Scout) can't be chosen: the target stays.
    let (mut t, _, scout, elves) = setup();
    let trail = t.hand(P0, "Arc Trail");
    t.cast(P0, trail)
        .target(P1)
        .target(elves)
        .go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(scout)]);
    t.answer_targets(P0, &[]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(
        targets_on_stack(&t, copy),
        vec![Entity::Player(P1), Entity::Object(elves)]
    );
    t.resolve_all();
    assert!(t.on_battlefield(scout));
}

#[test]
fn a_copy_made_by_lucky_clover_isnt_cast() {
    cr!("707.10", "601.2i", "603.2");
    ruling!(
        "Lucky Clover",
        "The copy is created on the stack, so it's not “cast.” Abilities that trigger when a player casts a spell won't trigger."
    );
    supported("Lucky Clover");
    supported("Young Pyromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lucky Clover");
    t.battlefield(P0, "Young Pyromancer");
    t.lands(P0, "Mountain", 2);
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    t.cast(P0, giant)
        .method(CastMethod::Half(1))
        .target(P1)
        .go();
    t.resolve_all();
    // Stomp and its copy each dealt 2 damage; only Stomp was cast.
    assert_eq!(t.life(P1), 16);
    assert_eq!(elementals(&t, P0), 1);
}

#[test]
fn a_copy_made_by_twincast_isnt_cast_and_resolves_first() {
    cr!("707.10", "405.5", "601.2i");
    ruling!(
        "Twincast",
        "The copy is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger. The copy will resolve before the original spell does."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Volcanic Island", 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(P1).go();
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(bolt).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    // Resolve the two Pyromancer triggers and Twincast: the copy is on top of the Bolt.
    t.resolve();
    t.resolve();
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(*t.g.stack.last().unwrap(), copy);
    t.resolve();
    assert!(!t.g.is_live(bears));
    assert_eq!(t.life(P1), 20);
    assert!(t.g.stack.contains(&bolt));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Bolt and Twincast were cast; the copy wasn't.
    assert_eq!(elementals(&t, P0), 2);
}

#[test]
fn a_copy_made_by_insidious_will_isnt_cast() {
    cr!("707.10", "601.2i");
    ruling!(
        "Insidious Will",
        "The copy is created on the stack, so it's not “cast.” Abilities that trigger when a player casts a spell won't trigger."
    );
    supported("Insidious Will");
    let mut t = TestGame::new(2);
    // P1's Young Pyromancer triggers on P1's spells.
    t.battlefield(P1, "Young Pyromancer");
    t.lands(P1, "Volcanic Island", 5);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    let will = t.hand(P1, "Insidious Will");
    t.cast(P1, will).modes(&[2]).target(bolt).go();
    t.answer_yes(P1, false);
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
    assert_eq!(elementals(&t, P1), 2);
}

/// Case of the Shifting Visage on the battlefield, solved: "Whenever you cast a
/// nonlegendary creature spell, copy that spell."
fn solved_visage(t: &mut TestGame) {
    supported("Case of the Shifting Visage");
    let case = t.battlefield(P0, "Case of the Shifting Visage");
    assert!(mtg_engine::cases::solve(&mut t.g, case));
    t.g.recompute();
}

#[test]
fn shifting_visage_copy_has_the_effects_of_the_originals_additional_costs() {
    cr!("707.2", "707.10", "702.33d");
    ruling!(
        "Case of the Shifting Visage",
        "You can’t choose to pay any additional costs for a copied spell. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    supported("Kavu Titan");
    // Kicked: the copy is kicked too.
    let mut t = TestGame::new(2);
    solved_visage(&mut t);
    t.lands(P0, "Forest", 5);
    let titan = t.hand(P0, "Kavu Titan");
    t.cast(P0, titan).kicked(true).go();
    t.resolve_all();
    let titans = t.named_on_battlefield("Kavu Titan");
    assert_eq!(titans.len(), 2);
    for k in &titans {
        assert_eq!(t.pt(*k), (5, 5));
        assert!(t.obj_now(*k).has_keyword(KeywordKind::Trample));
    }
    assert!(titans.iter().any(|k| t.obj_now(*k).is_token()));
    // Not kicked: the copy's controller can't kick the copy.
    let mut t = TestGame::new(2);
    solved_visage(&mut t);
    t.lands(P0, "Forest", 5);
    let titan = t.hand(P0, "Kavu Titan");
    t.cast(P0, titan).kicked(false).go();
    t.resolve_all();
    let titans = t.named_on_battlefield("Kavu Titan");
    assert_eq!(titans.len(), 2);
    for k in &titans {
        assert_eq!(t.pt(*k), (2, 2));
    }
}

#[test]
fn shifting_visage_copy_makes_its_own_resolution_choices() {
    cr!("707.10", "608.2", "614.12");
    ruling!(
        "Case of the Shifting Visage",
        "Any choices made when the spell resolves won’t have been made yet when it’s copied. Any such choices will be made separately when the copy resolves."
    );
    supported("Adaptive Automaton");
    let mut t = TestGame::new(2);
    solved_visage(&mut t);
    t.lands(P0, "Island", 3);
    let automaton = t.hand(P0, "Adaptive Automaton");
    t.cast(P0, automaton).go();
    // The copy resolves first and chooses Elf; the original chooses Bear.
    choose_creature_type(&mut t, P0, "Elf");
    choose_creature_type(&mut t, P0, "Bear");
    t.resolve_all();
    let autos = t.named_on_battlefield("Adaptive Automaton");
    assert_eq!(autos.len(), 2);
    let types: Vec<Option<String>> = autos
        .iter()
        .map(|a| {
            t.obj_now(*a)
                .choices
                .creature_type
                .as_ref()
                .map(|s| s.to_string())
        })
        .collect();
    assert!(types.contains(&Some("Elf".into())));
    assert!(types.contains(&Some("Bear".into())));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(elves), (2, 2));
}

#[test]
fn reflections_of_littjara_trigger_and_copy_resolve_before_the_spell() {
    cr!("707.10", "405.5", "608.3f");
    ruling!(
        "Reflections of Littjara",
        "The triggered ability and the copy it creates will resolve before the spell that caused the ability to trigger."
    );
    supported("Reflections of Littjara");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Reflections of Littjara");
    t.settle();
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The trigger resolves; the copy is on top of the original spell and resolves into a
    // token while the original is still on the stack.
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.obj_now(t.named_on_battlefield("Grizzly Bears")[0]).is_token());
    assert!(t.g.stack.contains(&spell));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn spelldancer_copy_keeps_an_illegal_target_when_no_new_one_is_legal() {
    cr!("707.10c", "115.7d", "608.2b");
    ruling!(
        "Mercurial Spelldancer",
        "If, for any target, you can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    supported("Mercurial Spelldancer");
    supported("Shatter");
    let mut t = TestGame::new(2);
    let dancer = t.battlefield(P0, "Mercurial Spelldancer");
    t.g.add_counters(Entity::Object(dancer), "oil", 2, None);
    t.g.recompute();
    let ring = t.battlefield(P1, "Sol Ring");
    t.answer_yes(P0, true);
    t.attack(&[(dancer, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.counters(dancer, "oil"), 0);
    t.advance_to(P0, Step::PostcombatMain);
    // Shatter targets Sol Ring; the delayed trigger copies it after Sol Ring is gone.
    t.lands(P0, "Mountain", 2);
    let shatter = t.hand(P0, "Shatter");
    let shatter = t.cast(P0, shatter).target(ring).go();
    t.settle();
    // Shatter, the delayed trigger and the oil counter trigger.
    assert_eq!(t.stack_len(), 3);
    while !stack_items(&t).last().unwrap().contains("delayed trigger") {
        t.resolve();
    }
    destroy(&mut t, ring);
    t.answer_yes(P0, true);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(targets_on_stack(&t, copy), vec![Entity::Object(ring)]);
    assert_eq!(targets_on_stack(&t, shatter), vec![Entity::Object(ring)]);
    t.resolve_all();
    assert!(t.g.stack.is_empty());
}

/// P0 has Double Stroke face up in the command zone, naming Skeletal Scrying.
fn double_stroke_naming_skeletal_scrying(t: &mut TestGame) {
    supported("Double Stroke");
    supported("Skeletal Scrying");
    let c = t.custom(P0, (*card::card("Double Stroke")).clone(), Zone::Command);
    t.answer(P0, DecisionKind::Name, Answer::Text("Skeletal Scrying".into()));
    mtg_engine::kw::hidden_agenda::as_put_into_command_zone(&mut t.g, P0, c);
    t.g.recompute();
    t.g.turn.priority = Some(P0);
    let action = Action::Special(SpecialAction::TurnFaceUp { obj: c });
    assert!(t.g.legal_actions(P0).contains(&action));
    t.g.take_action(P0, action);
    t.g.recompute();
}

#[test]
fn double_stroke_copy_has_the_effects_of_the_exiled_cards_without_exiling_more() {
    cr!("707.2", "707.10", "702.106a");
    ruling!(
        "Double Stroke",
        "Effects based on any additional costs that were paid for the original spell will be copied as though those same costs were paid for the copy."
    );
    let mut t = TestGame::new(2);
    double_stroke_naming_skeletal_scrying(&mut t);
    for _ in 0..5 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.lands(P0, "Swamp", 4);
    let scrying = t.hand(P0, "Skeletal Scrying");
    let hand = t.hand_size(P0) - 1;
    t.cast(P0, scrying).x(3).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    // Both drew 3 and lost 3 life; only the original's 3 cards were exiled.
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.hand_size(P0), hand + 6);
    assert_eq!(t.graveyard_size(P0), 2 + 1);
}

#[test]
fn double_stroke_copy_has_the_same_value_of_x() {
    cr!("707.2", "707.10", "107.3");
    ruling!(
        "Double Stroke",
        "If the spell being copied has an X whose value was determined as it was cast (like Skeletal Scrying has), the copy will have the same value of X."
    );
    let mut t = TestGame::new(2);
    double_stroke_naming_skeletal_scrying(&mut t);
    for _ in 0..4 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.lands(P0, "Swamp", 4);
    let scrying = t.hand(P0, "Skeletal Scrying");
    t.cast(P0, scrying).x(2).go();
    t.answer_yes(P0, true);
    t.settle();
    // Resolve Double Stroke's trigger: the copy is on the stack with X = 2.
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(t.obj(copy).stack.as_ref().unwrap().x, Some(2));
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
}

#[test]
fn a_copied_command_keeps_its_modes_but_can_get_new_targets() {
    cr!("707.10", "700.2g", "707.10c");
    ruling!(
        "Silumgar's Command",
        "If a Command is copied, the effect that creates the copy will usually allow you to choose new targets for the copy, but you can’t choose new modes."
    );
    supported("Silumgar's Command");
    supported("Twincast");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    let ring = t.battlefield(P1, "Sol Ring");
    t.lands(P0, "Underground Sea", 7);
    let cmd = t.hand(P0, "Silumgar's Command");
    // Return target permanent (Sol Ring) + target creature gets -3/-3 (Grizzly Bears).
    let cmd = t
        .cast(P0, cmd)
        .modes(&[1, 2])
        .target(ring)
        .target(bears)
        .go();
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(cmd).go();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 3]));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(modes_on_stack(&t, copy), vec![1, 2]);
    assert_eq!(
        targets_on_stack(&t, copy),
        vec![Entity::Object(giant), Entity::Object(elves)]
    );
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant") && t.in_hand(P1, "Sol Ring"));
    assert!(!t.g.is_live(elves) && !t.g.is_live(bears));
}

#[test]
fn a_copied_confluence_keeps_its_modes_but_can_get_new_targets() {
    cr!("707.10", "700.2g", "700.2d");
    ruling!(
        "Wretched Confluence",
        "If a Confluence is copied, the effect that creates the copy will usually allow you to choose new targets, but you can’t choose new modes."
    );
    supported("Wretched Confluence");
    supported("Twincast");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Underground Sea", 7);
    let conf = t.hand(P0, "Wretched Confluence");
    // Target creature gets -2/-2 three times (the same mode more than once).
    let conf = t
        .cast(P0, conf)
        .modes(&[1, 1, 1])
        .target(bears)
        .target(bears)
        .target(bears)
        .go();
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(conf).go();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 0, 0]));
    t.answer_yes(P0, true);
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Object(giant)]);
    }
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(modes_on_stack(&t, copy), vec![1, 1, 1]);
    assert_eq!(targets_on_stack(&t, copy), vec![Entity::Object(giant); 3]);
    t.resolve_all();
    assert!(!t.g.is_live(giant) && !t.g.is_live(bears));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn hero_of_the_games_doesnt_trigger_for_a_copy_or_changed_targets() {
    cr!("707.10", "603.2", "115.7");
    ruling!(
        "Hero of the Games",
        "This ability doesn't trigger if you copy a spell that targets it, or if a spell's targets are changed to target it."
    );
    supported("Hero of the Games");
    supported("Twincast");
    supported("Insidious Will");
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Hero of the Games");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 6);
    // Giant Growth on the Bears, copied onto the Hero: no trigger.
    let growth = t.hand(P0, "Giant Growth");
    let growth = t.cast(P0, growth).target(bears).go();
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(growth).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(hero)]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.pt(hero), (6, 5));
    assert_eq!(t.pt(bears), (5, 5));
    // Another Giant Growth on the Bears, changed to target the Hero: no trigger.
    let growth = t.hand(P0, "Giant Growth");
    let growth = t.cast(P0, growth).target(bears).go();
    let will = t.hand(P0, "Insidious Will");
    t.cast(P0, will).modes(&[1]).target(growth).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(hero)]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(hero), (9, 8));
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn a_clone_cast_from_hand_as_a_copy_of_scion_gets_its_cast_trigger() {
    cr!("707.5", "603.4", "702.157a");
    ruling!(
        "Scion of Vitu-Ghazi",
        "If a creature (such as Clone) enters the battlefield as a copy of this creature, the copy's \"enters-the-battlefield\" ability will still trigger as long as you cast that creature spell from your hand."
    );
    supported("Scion of Vitu-Ghazi");
    let mut t = TestGame::new(2);
    let scion = t.battlefield(P0, "Scion of Vitu-Ghazi");
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_choose(P0, &[Entity::Object(scion)]);
    let before = t.g.battlefield.clone();
    t.resolve_all();
    // A Bird, then a copy of it (populate).
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    assert!(toks.iter().all(|x| t.obj_now(*x).chars.name == "Bird Token"));
    // A Clone put onto the battlefield without being cast doesn't get it.
    t.answer_choose(P0, &[Entity::Object(scion)]);
    let before = t.g.battlefield.clone();
    t.enter(P0, "Clone");
    t.resolve_all();
    assert!(new_tokens(&t, P0, &before).is_empty());
}

#[test]
fn a_copy_of_palace_siege_makes_its_own_choice() {
    cr!("707.6", "607.2a", "614.12");
    ruling!(
        "Palace Siege",
        "If a permanent enters the battlefield as a copy of one of the Sieges, its controller will make a new choice for that Siege."
    );
    supported("Palace Siege");
    supported("Copy Enchantment");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    // The original chooses Khans; the copy chooses Dragons.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let siege = t.enter(P0, "Palace Siege");
    t.settle();
    t.answer_choose(P0, &[Entity::Object(siege)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.enter(P0, "Copy Enchantment");
    t.settle();
    let sieges = t.named_on_battlefield("Palace Siege");
    assert_eq!(sieges.len(), 2);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.answer_targets(P0, &[]);
    t.resolve_all();
    // Both modes happened: the Bears came back, and P1 was drained.
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn mask_of_the_mimic_finds_a_card_named_like_what_a_copy_is_copying() {
    cr!("707.2", "201.2");
    ruling!(
        "Mask of the Mimic",
        "If a copy card is targeted by this effect, you get to look for another copy of the card it is copying."
    );
    supported("Mask of the Mimic");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_choose(P1, &[Entity::Object(angel)]);
    let clone = t.enter(P1, "Clone");
    t.settle();
    let clone = t.g.current(clone);
    t.library_top(P0, "Serra Angel");
    t.library_top(P0, "Clone");
    let fodder = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    let mask = t.hand(P0, "Mask of the Mimic");
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.cast(P0, mask).target(clone).go();
    t.resolve_all();
    let mine: Vec<ObjectId> = t
        .named_on_battlefield("Serra Angel")
        .into_iter()
        .filter(|a| t.obj_now(*a).controller == P0)
        .collect();
    assert_eq!(mine.len(), 1);
    assert!(t.named_on_battlefield("Clone").is_empty());
}
