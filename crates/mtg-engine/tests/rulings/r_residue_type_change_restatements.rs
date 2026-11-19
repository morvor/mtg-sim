//! Item `rulings-residue-type-change` — rulings that restate what the card's own text
//! says (no interaction with other rules or cards beyond the minimal setup): one table,
//! each case with its card, a minimal setup and the asserted outcome, citing its rules and
//! its ruling.

use crate::r_residue_type_change_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A restatement ruling: its card and the check (setup, outcome, citations).
type Case = (&'static str, fn());

/// The target candidates of the first target decision `p` was asked since `from`.
fn candidates_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<Entity> {
    asked_since(t, from)
        .into_iter()
        .find_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .expect("asked for targets")
}

/// The targets of the top object of the stack.
fn top_targets(t: &TestGame) -> Vec<Entity> {
    crate::r_s25_common::targets_of(t, *t.g.stack.last().expect("stack is empty"))
}

/// Queues P0's choice of `ty` among the creature types other than Wall.
fn choose_type_but_wall(t: &mut TestGame, ty: &str) {
    let i = subtype_lists()
        .creature
        .iter()
        .filter(|c| c.as_str() != "Wall")
        .position(|c| c == ty)
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
}

/// Activates the `index`th activated ability of P0's `source` with all of P0's lands
/// available (Wastes added for the generic part given).
fn activate_with(t: &mut TestGame, source: ObjectId, index: usize, targets: &[Entity]) {
    t.g.turn.priority = Some(P0);
    t.activate(P0, source, index, targets).unwrap();
}

#[test]
fn restatement_rulings() {
    let cases: &[Case] = &[
        // viridian corrupter with no artifacts just enters
        ("Viridian Corrupter", || {
            cr!("603.3d", "115.1");
            ruling!(
                "Viridian Corrupter",
                "If there are no artifacts on the battlefield when Viridian Corrupter enters the battlefield, its enters-the-battlefield does nothing. Viridian Corrupter will simply enter the battlefield."
            );
            supported("Viridian Corrupter");
            let mut t = TestGame::new(2);
            let v = t.enter(P0, "Viridian Corrupter");
            t.settle();
            assert_eq!(t.stack_len(), 0);
            assert!(t.on_battlefield(v));
        }),
        // viridian corrupter must target your own artifact
        ("Viridian Corrupter", || {
            cr!("603.3d", "115.1");
            ruling!(
                "Viridian Corrupter",
                "The enters-the-battlefield ability is not optional. You must target an artifact you control if there are no other artifacts on the battlefield."
            );
            let mut t = TestGame::new(2);
            let stone = t.battlefield(P0, "Mind Stone");
            t.enter(P0, "Viridian Corrupter");
            t.settle();
            assert_eq!(top_targets(&t), vec![obj(stone)]);
            t.resolve_all();
            assert!(t.in_graveyard(P0, "Mind Stone"));
        }),
        // gore vassal regeneration shield lasts until used or end of turn
        ("Gore Vassal", || {
            cr!("701.19a", "514.2");
            ruling!(
                "Gore Vassal",
                "The regeneration shield will last until the turn ends or until it’s used."
            );
            supported("Gore Vassal");
            // Used once: the second destruction kills it.
            let mut t = TestGame::new(2);
            let gv = t.battlefield(P0, "Gore Vassal");
            let giant = t.battlefield(P0, "Hill Giant");
            activate_with(&mut t, gv, 0, &[obj(giant)]);
            t.resolve_all();
            assert_eq!(pt_now(&mut t, giant), (2, 2));
            destroy(&mut t, giant);
            assert!(t.on_battlefield(giant) && tapped(&t, giant));
            destroy(&mut t, giant);
            assert!(t.in_graveyard(P0, "Hill Giant"));
            // Not used this turn: gone at the turn's end.
            let mut t = TestGame::new(2);
            let gv = t.battlefield(P0, "Gore Vassal");
            let giant = t.battlefield(P0, "Hill Giant");
            activate_with(&mut t, gv, 0, &[obj(giant)]);
            t.resolve_all();
            t.advance_to(P1, Step::Upkeep);
            destroy(&mut t, giant);
            assert!(t.in_graveyard(P0, "Hill Giant"));
        }),
        // plaguemaw beast with nothing to proliferate
        ("Plaguemaw Beast", || {
            cr!("701.34a", "602.2");
            ruling!(
                "Plaguemaw Beast",
                "You can activate the ability even if there are no players or permanents with counters on them."
            );
            supported("Plaguemaw Beast");
            let mut t = TestGame::new(2);
            let beast = t.battlefield(P0, "Plaguemaw Beast");
            let bears = t.battlefield(P0, "Grizzly Bears");
            t.answer_choose(P0, &[obj(bears)]);
            activate_with(&mut t, beast, 0, &[]);
            t.resolve_all();
            assert!(t.in_graveyard(P0, "Grizzly Bears"));
            assert!(tapped(&t, beast));
        }),
        // plaguemaw beast sacrifices itself for its own cost
        ("Plaguemaw Beast", || {
            cr!("602.2b", "601.2h", "113.7a", "701.34a");
            ruling!(
                "Plaguemaw Beast",
                "You can sacrifice Plaguemaw Beast to pay the cost of its own activated ability."
            );
            let mut t = TestGame::new(2);
            let beast = t.battlefield(P0, "Plaguemaw Beast");
            let bears = t.battlefield(P0, "Grizzly Bears");
            give_plus1(&mut t, bears, 1);
            t.answer_choose(P0, &[obj(beast)]);
            t.answer_choose(P0, &[obj(bears)]);
            activate_with(&mut t, beast, 0, &[]);
            assert!(t.in_graveyard(P0, "Plaguemaw Beast"));
            t.resolve_all();
            assert_eq!(plus1(&t, bears), 2);
        }),
        // mild mannered librarian becomes a werewolf and isnt a human
        ("Mild-Mannered Librarian", || {
            cr!("205.1a", "613.1d");
            ruling!(
                "Mild-Mannered Librarian",
                "As Mild-Mannered Librarian's last ability resolves, it loses its other creature types and becomes a Werewolf. It's no longer a Human."
            );
            supported("Mild-Mannered Librarian");
            let mut t = TestGame::new(2);
            let lib = t.battlefield(P0, "Mild-Mannered Librarian");
            t.library_top(P0, "Island");
            t.lands(P0, "Forest", 4);
            let hand = t.hand_size(P0);
            activate_with(&mut t, lib, 0, &[]);
            t.resolve_all();
            assert_eq!(creature_types(&mut t, lib), vec!["Werewolf"]);
            assert_eq!(pt_now(&mut t, lib), (3, 3));
            assert_eq!(t.hand_size(P0), hand + 1);
        }),
        // wild shape target and mode chosen on announcement
        ("Wild Shape", || {
            cr!("601.2b", "601.2c", "700.2a");
            ruling!(
                "Wild Shape",
                "Both the target creature and the mode are chosen on announcement. You can't choose a different form if something changes in response to the spell."
            );
            supported("Wild Shape");
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            lands_for_cost(&mut t, P0, "Wild Shape");
            let ws = t.hand(P0, "Wild Shape");
            let spell = t.cast(P0, ws).modes(&[1]).target(bears).go();
            assert_eq!(crate::r_s25_common::targets_of(&t, spell), vec![obj(bears)]);
            let from = t.asked().len();
            t.resolve_all();
            // Nothing is chosen as it resolves.
            assert!(!asked_since(&t, from).iter().any(|(_, d)| matches!(
                d,
                Decision::ChooseModes { .. } | Decision::ChooseOption { .. }
            )));
            assert_eq!(pt_now(&mut t, bears), (1, 5));
            assert_eq!(creature_types(&mut t, bears), vec!["Spider"]);
            assert!(has_kw(&mut t, bears, KeywordKind::Reach));
        }),
        // coercive recruiter can target itself or your own untapped creature
        ("Coercive Recruiter", || {
            cr!("115.1", "603.3d");
            ruling!(
                "Coercive Recruiter",
                "Coercive Recruiter's ability can target any creature, even one that's untapped or one you already control. It can even target itself or the creature that caused the ability to trigger."
            );
            supported("Coercive Recruiter");
            // Itself.
            let mut t = TestGame::new(2);
            t.answer_targets(P0, &[]);
            let from = t.asked().len();
            let r = t.enter(P0, "Coercive Recruiter");
            t.settle();
            let c = candidates_since(&t, P0, from);
            assert!(c.contains(&obj(r)));
            t.clear_answers();
            // An untapped creature P0 already controls: it becomes a Pirate with haste.
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            t.answer_targets(P0, &[obj(bears)]);
            t.enter(P0, "Coercive Recruiter");
            t.resolve_all();
            assert_eq!(controller(&mut t, bears), P0);
            assert_eq!(creature_types(&mut t, bears), vec!["Bear", "Pirate"]);
            assert!(has_kw(&mut t, bears, KeywordKind::Haste));
        }),
        // thermal flux adds or removes only snow
        ("Thermal Flux", || {
            cr!("205.4a", "205.4g", "613.1d");
            ruling!(
                "Thermal Flux",
                "Thermal Flux adds or removes the supertype snow from a permanent. Adding or removing a supertype doesn’t affect any other types, supertypes, or subtypes that permanent has."
            );
            supported("Thermal Flux");
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            let island = t.battlefield(P1, "Snow-Covered Island");
            for (mode, target) in [(0, bears), (1, island)] {
                lands_for_cost(&mut t, P0, "Thermal Flux");
                let tf = t.hand(P0, "Thermal Flux");
                t.cast(P0, tf).modes(&[mode]).target(target).go();
                t.resolve_all();
            }
            t.g.recompute();
            let b = &t.obj_now(bears).chars;
            assert!(b.supertypes.contains(Supertype::Snow));
            assert!(b.card_types.contains(CardType::Creature) && b.has_subtype("Bear"));
            let i = &t.obj_now(island).chars;
            assert!(!i.supertypes.contains(Supertype::Snow));
            assert!(i.supertypes.contains(Supertype::Basic));
            assert!(i.card_types.contains(CardType::Land) && i.has_subtype("Island"));
        }),
        // mistform sliver adds a creature type
        ("Mistform Sliver", || {
            cr!("205.1b", "613.1d");
            ruling!(
                "Mistform Sliver",
                "Unlike most type changing effects, this one adds a creature type instead of replacing existing creature types."
            );
            supported("Mistform Sliver");
            let mut t = TestGame::new(2);
            let ms = t.battlefield(P0, "Mistform Sliver");
            t.lands(P0, "Wastes", 1);
            choose_creature_type(&mut t, P0, "Goblin");
            activate_with(&mut t, ms, 0, &[]);
            t.resolve_all();
            assert_eq!(
                creature_types(&mut t, ms),
                vec!["Goblin", "Illusion", "Sliver"]
            );
        }),
        // wishful merfolk human isnt a merfolk
        ("Wishful Merfolk", || {
            cr!("205.1a", "613.1d", "613.1f");
            ruling!(
                "Wishful Merfolk",
                "While Wishful Merfolk is a Human, it's no longer a Merfolk."
            );
            supported("Wishful Merfolk");
            let mut t = TestGame::new(2);
            let wm = t.battlefield(P0, "Wishful Merfolk");
            t.lands(P0, "Island", 2);
            activate_with(&mut t, wm, 0, &[]);
            t.resolve_all();
            assert_eq!(creature_types(&mut t, wm), vec!["Human"]);
            assert!(!has_kw(&mut t, wm, KeywordKind::Defender));
        }),
        // unnatural selection offers only existing creature types
        ("Unnatural Selection", || {
            cr!("205.3m", "205.1a");
            ruling!(
                "Unnatural Selection",
                "You must choose a creature type that already exists in Magic."
            );
            supported("Unnatural Selection");
            let mut t = TestGame::new(2);
            let us = t.battlefield(P0, "Unnatural Selection");
            let bears = t.battlefield(P1, "Grizzly Bears");
            t.lands(P0, "Wastes", 1);
            choose_type_but_wall(&mut t, "Goblin");
            let from = t.asked().len();
            activate_with(&mut t, us, 0, &[obj(bears)]);
            t.resolve_all();
            let options = asked_since(&t, from)
                .into_iter()
                .find_map(|(_, d)| match d {
                    Decision::ChooseOption { options, .. } => Some(options),
                    _ => None,
                })
                .expect("asked for a creature type");
            let all = &subtype_lists().creature;
            assert!(options.iter().all(|o| all.contains(o)));
            assert!(!options.iter().any(|o| o == "Wall"));
            assert_eq!(creature_types(&mut t, bears), vec!["Goblin"]);
        }),
        // goblin test pilot can target itself
        ("Goblin Test Pilot", || {
            cr!("115.1");
            ruling!(
                "Goblin Test Pilot",
                "Goblin Test Pilot is a legal target of its own ability."
            );
            supported("Goblin Test Pilot");
            assert!(pilot_targets(40).contains(&PilotTarget::Pilot));
        }),
        // goblin test pilot target chosen at random as it is activated
        ("Goblin Test Pilot", || {
            cr!("601.2c", "602.2b", "117.3c");
            ruling!(
                "Goblin Test Pilot",
                "The target is chosen at random as part of activating the ability. Players can respond to the ability knowing what the target of the ability is."
            );
            let mut t = TestGame::new(2);
            let pilot = t.battlefield(P0, "Goblin Test Pilot");
            t.battlefield(P1, "Grizzly Bears");
            activate_with(&mut t, pilot, 0, &[]);
            // On the stack with its target chosen, and P0 has priority (then P1, to respond).
            assert_eq!(t.stack_len(), 1);
            assert_eq!(top_targets(&t).len(), 1);
            assert_eq!(t.g.turn.priority, Some(P0));
        }),
        // goblin test pilot every legal target equally likely
        ("Goblin Test Pilot", || {
            cr!("115.1", "601.2c");
            ruling!(
                "Goblin Test Pilot",
                "To choose a target at random, all possible legal targets (including creatures and players) must have an equal chance of being chosen."
            );
            let seen = pilot_targets(200);
            for k in [
                PilotTarget::P0,
                PilotTarget::P1,
                PilotTarget::Pilot,
                PilotTarget::Bears,
            ] {
                let n = seen.iter().filter(|x| **x == k).count();
                // Expected 50 of 200 each (a target twice as likely as another is caught).
                assert!((35..=65).contains(&n), "{k:?}: {n}");
            }
        }),
        // cyclopean mummy not exiled from elsewhere
        ("Cyclopean Mummy", || {
            cr!("700.4", "603.6c");
            ruling!(
                "Cyclopean Mummy",
                "It does not get exiled if it goes to the graveyard from someplace other than the battlefield. This includes discarding from your hand or if the spell is countered."
            );
            supported("Cyclopean Mummy");
            // Discarded.
            let mut t = TestGame::new(2);
            let m = t.hand(P0, "Cyclopean Mummy");
            t.g.discard(P0, m, None);
            t.resolve_all();
            assert!(t.in_graveyard(P0, "Cyclopean Mummy"));
            // Countered.
            let mut t = TestGame::new(2);
            let spell = cast_new(&mut t, P0, "Cyclopean Mummy", &[]);
            cast_new(&mut t, P1, "Cancel", &[obj(spell)]);
            t.resolve_all();
            assert!(t.in_graveyard(P0, "Cyclopean Mummy"));
            assert!(!t.in_exile("Cyclopean Mummy"));
        }),
        // ferropede can target a permanent with no counters
        ("Ferropede", || {
            cr!("115.1", "603.3d");
            ruling!(
                "Ferropede",
                "Any permanent is a legal target for Ferropede’s last ability, not just one with a counter on it."
            );
            supported("Ferropede");
            let mut t = TestGame::new(2);
            let f = t.battlefield(P0, "Ferropede");
            let island = t.battlefield(P1, "Island");
            t.answer_yes(P0, true);
            t.answer_targets(P0, &[obj(island)]);
            let from = t.asked().len();
            attack_with(&mut t, &[(f, Entity::Player(P1))]);
            block_and_finish(&mut t, P1, &[]);
            t.resolve_all();
            assert_eq!(t.life(P1), 19);
            let c = candidates_since(&t, P0, from);
            assert!(c.contains(&obj(island)) && c.contains(&obj(f)));
        }),
    ];
    for (card, check) in cases {
        eprintln!("case: {card}");
        check();
    }
}

/// What Goblin Test Pilot's ability targeted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PilotTarget {
    P0,
    P1,
    Pilot,
    Bears,
}

/// Goblin Test Pilot's ability activated in `n` games with different seeds (P0's Pilot,
/// P1's Grizzly Bears): what each targeted.
fn pilot_targets(n: u64) -> Vec<PilotTarget> {
    (0..n)
        .map(|seed| {
            let mut t = TestGame::with_config(
                2,
                mtg_engine::game::GameConfig {
                    seed,
                    ..Default::default()
                },
            );
            let pilot = t.battlefield(P0, "Goblin Test Pilot");
            let bears = t.battlefield(P1, "Grizzly Bears");
            activate_with(&mut t, pilot, 0, &[]);
            match top_targets(&t)[..] {
                [Entity::Player(P0)] => PilotTarget::P0,
                [Entity::Player(P1)] => PilotTarget::P1,
                [Entity::Object(o)] if o == pilot => PilotTarget::Pilot,
                [Entity::Object(o)] if o == bears => PilotTarget::Bears,
                ref other => panic!("unexpected target {other:?}"),
            }
        })
        .collect()
}
