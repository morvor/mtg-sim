//! CR 702.29 Cycling and typecycling.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use crate::common_k702_027_037::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The candidates offered by the last search decision.
fn search_candidates(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap_or_default()
}

/// Whether `id` has a cycling ability that `p` could activate now.
fn can_cycle(t: &mut TestGame, p: PlayerId, id: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let id = t.g.current(id);
    t.g.activatable_abilities(p)
        .iter()
        .any(|(s, a)| *s == id && a.text == "Cycling")
}

#[test]
fn cycling_discards_the_card_to_draw_a_card() {
    cr!("702.29", "702.29a");
    assert_supported("Street Wraith");
    let mut t = TestGame::new(2);
    let wraith = t.hand(P0, "Street Wraith");
    t.library_top(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cycle(&mut t, P0, wraith, 0).unwrap();
    // The cost is paid as it's activated: 2 life, and the card is discarded.
    assert_eq!(t.life(P0), 18);
    assert!(t.in_graveyard(P0, "Street Wraith"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn cycling_functions_only_while_the_card_is_in_a_hand() {
    cr!("702.29a", "702.29b");
    ruling!(
        "Yawgmoth's Will",
        "Thus, Cycling abilities of cards in the graveyard can’t be activated."
    );
    assert_supported("Forgotten Cave");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let in_hand = t.hand(P0, "Forgotten Cave");
    let in_play = t.battlefield(P0, "Forgotten Cave");
    let in_yard = t.graveyard(P0, "Forgotten Cave");
    // The ability exists in every zone...
    for id in [in_hand, in_play, in_yard] {
        assert!(t.obj_now(id).chars.abilities.iter().any(|a| {
            a.text == "Cycling" && matches!(a.kind, AbilityKind::Activated(_))
        }));
    }
    // ...but can be activated only from the hand.
    assert!(can_cycle(&mut t, P0, in_hand));
    assert!(!can_cycle(&mut t, P0, in_play));
    assert!(!can_cycle(&mut t, P0, in_yard));
    assert!(cycle(&mut t, P0, in_play, 0).is_err());
}

#[test]
fn a_permanent_with_cycling_has_an_activated_ability_that_isnt_a_mana_ability() {
    cr!("702.29b");
    assert_supported("Tsabo's Web");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Tsabo's Web");
    let cave = t.battlefield(P0, "Forgotten Cave");
    let mountain = t.battlefield(P0, "Mountain");
    t.g.tap(cave);
    t.g.tap(mountain);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    // Forgotten Cave's cycling ability makes it a land with an activated ability that
    // isn't a mana ability.
    assert!(t.obj_now(cave).tapped);
    assert!(!t.obj_now(mountain).tapped);
}

#[test]
fn abilities_that_trigger_when_you_cycle_this_card_resolve_first() {
    cr!("702.29c");
    ruling!(
        "Renewed Faith",
        "When you cycle this card, first the cycling ability goes on the stack, then the triggered ability goes on the stack on top of it. The triggered ability will resolve before you draw a card from the cycling ability."
    );
    assert_supported("Renewed Faith");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let faith = t.hand(P0, "Renewed Faith");
    t.library_top(P0, "Grizzly Bears");
    cycle(&mut t, P0, faith, 0).unwrap();
    t.settle();
    // It triggered from the graveyard, where the card went as it was discarded.
    assert!(t.in_graveyard(P0, "Renewed Faith"));
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.life(P0), 22);
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn cycle_or_discard_abilities_trigger_only_once_when_a_card_is_cycled() {
    cr!("702.29d");
    ruling!(
        "Drake Haven",
        "An ability that triggers whenever you “cycle or discard” a card triggers only once if you cycle a card."
    );
    assert_supported("Drake Haven");
    let mut t = TestGame::new(2);
    let haven = t.battlefield(P0, "Drake Haven");
    t.lands(P0, "Island", 1);
    let wraith = t.hand(P0, "Street Wraith");
    cycle(&mut t, P0, wraith, 0).unwrap();
    t.settle();
    let haven_triggers = t
        .g
        .stack
        .iter()
        .filter(|id| {
            t.g.obj(**id)
                .stack
                .as_ref()
                .is_some_and(|si| matches!(si.kind, object::StackKind::Triggered { source, .. } if source == haven))
        })
        .count();
    assert_eq!(haven_triggers, 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(tokens(&t, P0), 1);
}

#[test]
fn typecycling_searches_for_a_card_of_that_type_instead_of_drawing() {
    cr!("702.29e");
    ruling!(
        "Twisted Abomination",
        "Unlike the normal cycling ability, Swampcycling doesn't allow you to draw a card. Instead, it lets you search your library for a Swamp card."
    );
    ruling!(
        "Twisted Abomination",
        "You can choose to find any card with the Swamp land type, including nonbasic lands."
    );
    assert_supported("Twisted Abomination");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let grave = t.library_top(P0, "Watery Grave");
    let swamp = t.library_top(P0, "Swamp");
    t.library_top(P0, "Island");
    let hand = t.hand_size(P0);
    let abom = t.hand(P0, "Twisted Abomination");
    cycle(&mut t, P0, abom, 0).unwrap();
    assert!(t.in_graveyard(P0, "Twisted Abomination"));
    t.answer_choose(P0, &[Entity::Object(grave)]);
    t.resolve();
    let offered = search_candidates(&t);
    assert!(offered.contains(&Entity::Object(grave)));
    assert!(offered.contains(&Entity::Object(swamp)));
    assert_eq!(offered.len(), 2);
    // The Swamp card was put into the hand; no card was drawn.
    assert!(t.in_hand(P0, "Watery Grave"));
    assert!(!t.in_hand(P0, "Island"));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn typecycling_can_name_any_type_or_combination_of_types() {
    cr!("702.29e");
    ruling!(
        "Sojourner's Companion",
        "For example, a card with basic landcycling lets you search for a basic land card"
    );
    assert_supported("Ash Barrens");
    assert_supported("Sojourner's Companion");
    // Basic landcycling: a basic land card (a supertype and a card type).
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let forest = t.library_top(P0, "Forest");
    let grave = t.library_top(P0, "Watery Grave");
    let barrens = t.hand(P0, "Ash Barrens");
    cycle(&mut t, P0, barrens, 0).unwrap();
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.resolve();
    let offered = search_candidates(&t);
    assert_eq!(offered, vec![Entity::Object(forest)]);
    assert!(!offered.contains(&Entity::Object(grave)));
    assert!(t.in_hand(P0, "Forest"));
    // Artifact landcycling: an artifact land card (two card types).
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let seat = t.library_top(P0, "Seat of the Synod");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Ornithopter");
    let comp = t.hand(P0, "Sojourner's Companion");
    cycle(&mut t, P0, comp, 0).unwrap();
    t.answer_choose(P0, &[Entity::Object(seat)]);
    t.resolve();
    assert_eq!(search_candidates(&t), vec![Entity::Object(seat)]);
    assert!(t.in_hand(P0, "Seat of the Synod"));
}

#[test]
fn only_one_of_several_landcycling_abilities_can_be_activated() {
    cr!("702.29e", "702.29f");
    ruling!(
        "Jhessian Zombies",
        "You may only activate one landcycling ability at a time. You must specify which landcycling ability you are activating as you cycle this card"
    );
    assert_supported("Jhessian Zombies");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let zombies = t.hand(P0, "Jhessian Zombies");
    assert_eq!(keyword_count(&t, zombies, KeywordKind::Cycling), 2);
    let island = t.library_top(P0, "Island");
    t.library_top(P0, "Swamp");
    // Islandcycling.
    cycle(&mut t, P0, zombies, 0).unwrap();
    // The card was discarded: the swampcycling ability can't be activated any more.
    assert!(!can_cycle(&mut t, P0, zombies));
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.resolve();
    assert_eq!(search_candidates(&t), vec![Entity::Object(island)]);
    assert!(t.in_hand(P0, "Island"));
    assert_eq!(untapped_lands(&t, P0), 2);
}

#[test]
fn typecycling_is_cycling_for_triggers_and_prohibitions() {
    cr!("702.29f");
    ruling!(
        "Twisted Abomination",
        "Swampcycling is a form of cycling. Any ability that triggers on a card being cycled also triggers on Swampcycling this card. Any ability that stops a cycling ability from being activated also stops Swampcycling from being activated."
    );
    ruling!("Stabilizer", "This affects both Cycling and Landcycling.");
    assert_supported("Lightning Rift");
    assert_supported("Stabilizer");
    // Lightning Rift triggers on swampcycling.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lightning Rift");
    t.lands(P0, "Swamp", 3);
    let abom = t.hand(P0, "Twisted Abomination");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cycle(&mut t, P0, abom, 0).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Stabilizer stops cycling and swampcycling alike.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Stabilizer");
    t.lands(P0, "Swamp", 3);
    let abom = t.hand(P0, "Twisted Abomination");
    let wraith = t.hand(P0, "Street Wraith");
    assert!(!can_cycle(&mut t, P0, abom));
    assert!(!can_cycle(&mut t, P0, wraith));
    assert!(cycle(&mut t, P0, abom, 0).is_err());
    assert!(t.in_hand(P0, "Twisted Abomination"));
}

#[test]
fn cycling_cost_changes_apply_to_typecycling_costs() {
    cr!("702.29f");
    assert_supported("Fluctuator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fluctuator");
    // Swampcycling {2} costs {0}.
    let abom = t.hand(P0, "Twisted Abomination");
    t.library_top(P0, "Swamp");
    assert!(can_cycle(&mut t, P0, abom));
    cycle(&mut t, P0, abom, 0).unwrap();
    assert!(t.in_graveyard(P0, "Twisted Abomination"));
    // An opponent's Fluctuator doesn't reduce your costs ("you activate").
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Fluctuator");
    let abom = t.hand(P0, "Twisted Abomination");
    assert!(!can_cycle(&mut t, P0, abom));
}

#[test]
fn effects_looking_for_cards_with_cycling_find_typecycling_cards() {
    cr!("702.29f");
    ruling!(
        "Vile Manifestation",
        "Certain older cards have variants of cycling, such as basic landcycling or Wizardcycling. Vile Manifestation’s effect counts these cards."
    );
    assert_supported("Vile Manifestation");
    let mut t = TestGame::new(2);
    let vile = t.battlefield(P0, "Vile Manifestation");
    let base = t.pt(vile).0;
    t.graveyard(P0, "Twisted Abomination");
    t.graveyard(P0, "Ash Barrens");
    t.graveyard(P0, "Street Wraith");
    t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(vile).0, base + 3);
}

#[test]
fn cycling_granted_to_cards_in_a_hand_functions_there() {
    cr!("702.29a", "702.29e");
    ruling!(
        "Homing Sliver",
        "Unlike a normal cycling ability, Slivercycling doesn't have you draw a card. Instead, it lets you search your library for a Sliver card."
    );
    assert_supported("Tectonic Reformation");
    assert_supported("Homing Sliver");
    // "Each land card in your hand has cycling {R}."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tectonic Reformation");
    t.lands(P0, "Mountain", 1);
    let forest = t.hand(P0, "Forest");
    let yard_forest = t.graveyard(P0, "Forest");
    let bears = t.hand(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    assert!(can_cycle(&mut t, P0, forest));
    assert!(!can_cycle(&mut t, P0, yard_forest));
    assert!(!can_cycle(&mut t, P0, bears));
    // The opponent's land cards don't have it.
    let theirs = t.hand(P1, "Forest");
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::Cycling));
    cycle(&mut t, P0, forest, 0).unwrap();
    t.resolve();
    assert!(t.in_hand(P0, "Island"));
    assert!(t.in_graveyard(P0, "Forest"));
    // "Each Sliver card in each player's hand has slivercycling {3}": typecycling, for
    // every player.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Homing Sliver");
    t.lands(P1, "Plains", 3);
    let sliver = t.hand(P1, "Muscle Sliver");
    let found = t.library_top(P1, "Metallic Sliver");
    t.library_top(P1, "Grizzly Bears");
    assert!(can_cycle(&mut t, P1, sliver));
    cycle(&mut t, P1, sliver, 0).unwrap();
    t.answer_choose(P1, &[Entity::Object(found)]);
    t.resolve();
    assert_eq!(search_candidates(&t), vec![Entity::Object(found)]);
    assert!(t.in_hand(P1, "Metallic Sliver"));
}
