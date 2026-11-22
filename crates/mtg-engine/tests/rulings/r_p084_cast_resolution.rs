//! Rulings batch P084 — casting exiled graveyard cards, or copies of them, while an
//! effect resolves (CR 608.2g, 707.12): X is 0 (CR 107.3b), Blue Mage's Cane's {3}
//! (CR 118.9), timing permissions and restrictions (CR 601.3), cards that can't be cast,
//! where the card goes afterwards, copies that aren't cast cease to exist (CR 707.12),
//! the resolving spell or ability is still on the stack, and the order of several copies.

use crate::r_p084_common::*;
use crate::r_s01_common::{attack_with, tapped_lands, watch};
use crate::r_s06_common::attach_new;
use mtg_engine::ability::ZoneKind;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::{Game, GameConfig};
use mtg_engine::kw::cipher::encoded_on;
use mtg_engine::object::{CastMethod, FaceState, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const SOUNDWAVE: &str = "Soundwave, Sonic Spy // Soundwave, Superior Captain";

/// The spells named `name` cast so far this turn, with the zone each was cast from.
fn casts(t: &TestGame, name: &str) -> Vec<(ObjectId, Option<ZoneKind>)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::SpellCast { spell, from, .. } if t.g.obj(*spell).chars.name == name => {
                Some((*spell, *from))
            }
            _ => None,
        })
        .collect()
}

/// The names of the objects on the stack, bottom first.
fn stack_names(g: &Game) -> Vec<String> {
    g.stack
        .iter()
        .map(|id| g.obj(*id).chars.name.to_string())
        .collect()
}

fn is_yes_no(d: &Decision) -> bool {
    matches!(d, Decision::YesNo { .. })
}

fn is_targets(d: &Decision) -> bool {
    matches!(d, Decision::ChooseTargets { .. })
}

/// Whether `p` could cast any spell now.
fn can_cast_anything(t: &mut TestGame, p: PlayerId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { .. }))
}

/// Makes the creature a token (a creature token you control, for Soundwave).
fn tokenize(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].kind = ObjKind::Token;
}

#[test]
fn x_is_zero_for_free_copies_and_blue_mages_canes_three() {
    cr!("107.3b", "118.9", "707.12", "601.2b");
    ruling!(
        "Soundwave, Sonic Spy // Soundwave, Superior Captain",
        "If a spell you cast without paying its mana cost has an {X} in its mana cost, you must choose 0 as the value of X."
    );
    ruling!(
        "Mnemonic Deluge",
        "If the card you copy has {X} in its mana cost, you must choose 0 as the value of X when casting the copy without paying its mana cost."
    );
    ruling!(
        "Blue Mage's Cane",
        "If the spell has {X} in its mana cost, you must choose 0 as the value of X when casting it by paying {3} rather than paying its mana cost."
    );
    // Blaze ({X}{R}: "Blaze deals X damage to any target."), with X = 3 asked for, and
    // three untapped Mountains that could pay for an X of 3 if one could be chosen.
    let queue = |t: &mut TestGame| {
        t.answer(P0, DecisionKind::X, Answer::Number(3));
        t.answer_targets(P0, &[pl(P1)]);
    };
    // Soundwave: a 1/1 creature token deals 1 combat damage; Blaze has mana value 1.
    supported(SOUNDWAVE);
    let mut t = TestGame::new(2);
    let sw = t.battlefield(P0, SOUNDWAVE);
    let tok = t.battlefield(P0, "Llanowar Elves");
    tokenize(&mut t, tok);
    let blaze = t.graveyard(P1, "Blaze");
    t.lands(P0, "Mountain", 3);
    t.answer_targets(P0, &[obj(blaze)]);
    t.answer_yes(P0, true);
    queue(&mut t);
    t.attack(&[(tok, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(casts(&t, "Blaze").len(), 1, "{}", t.dump_log());
    assert_eq!(t.life(P1), 19);
    assert_eq!(tapped_lands(&t, P0), 0);
    assert_eq!(t.obj_now(sw).face, FaceState::Back);
    // Mnemonic Deluge: one of the three copies is cast.
    let mut t = TestGame::new(2);
    let blaze = t.graveyard(P1, "Blaze");
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    queue(&mut t);
    cast_modes(&mut t, P0, "Mnemonic Deluge", "6UUU", &[], &[&[obj(blaze)]]);
    t.lands(P0, "Mountain", 3);
    t.resolve_all();
    assert_eq!(casts(&t, "Blaze").len(), 1, "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
    assert_eq!(tapped_lands(&t, P0), 0);
    // Blue Mage's Cane: {3} is paid, not {3} plus X = 3.
    supported("Blue Mage's Cane");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Blue Mage's Cane", bears);
    let blaze = t.graveyard(P1, "Blaze");
    t.lands(P0, "Mountain", 6);
    t.answer_targets(P0, &[obj(blaze)]);
    t.answer_yes(P0, true);
    queue(&mut t);
    t.attack(&[(bears, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(casts(&t, "Blaze").len(), 1, "{}", t.dump_log());
    assert_eq!(tapped_lands(&t, P0), 3);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn blue_mages_cane_three_is_an_alternative_cost() {
    cr!("118.9", "118.9a", "601.2b", "601.2f");
    ruling!(
        "Blue Mage's Cane",
        "If you cast a spell by paying {3} rather than paying its mana cost, you can't choose to cast it for any other alternative cost. You can, however, pay additional costs. If the spell has any mandatory additional costs, those must be paid to cast the card."
    );
    supported("Blue Mage's Cane");
    /// Grizzly Bears with the Cane attacks P1, whose graveyard has `spell`; P0 casts the
    /// copy with seven Mountains available.
    fn cane(spell: &str, queue: impl FnOnce(&mut TestGame, ObjectId)) -> (TestGame, ObjectId) {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, "Blue Mage's Cane", bears);
        let elves = t.battlefield(P0, "Llanowar Elves");
        let card = t.graveyard(P1, spell);
        t.lands(P0, "Mountain", 7);
        t.answer_targets(P0, &[obj(card)]);
        t.answer_yes(P0, true);
        queue(&mut t, elves);
        t.attack(&[(bears, pl(P1))], &[]);
        t.resolve_all();
        (t, elves)
    }
    // Burst Lightning kicked: {3} plus kicker {4}.
    let (t, _) = cane("Burst Lightning", |t, _| {
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[pl(P1)]);
    });
    assert_eq!(tapped_lands(&t, P0), 7, "{}", t.dump_log());
    assert_eq!(t.life(P1), 20 - 4 - 2);
    // Village Rites: {3} and a sacrificed creature.
    let (t, _) = cane("Village Rites", |t, elves| {
        t.answer_choose(P0, &[obj(elves)]);
    });
    assert_eq!(tapped_lands(&t, P0), 3, "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    // Invigorate: no choice of its own alternative cost; {3} is paid.
    let mut from = 0;
    let (t, elves) = cane("Invigorate", |t, elves| {
        t.battlefield(P0, "Forest");
        t.answer_targets(P0, &[obj(elves)]);
        from = t.asked().len();
    });
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseCastingMethod { .. })));
    assert_eq!(tapped_lands(&t, P0), 3, "{}", t.dump_log());
    assert_eq!(t.pt(elves), (5, 5));
    assert_eq!(t.life(P1), 18);
}

/// Memory Plunder targets `card`'s card in P1's graveyard and P0 chooses to cast it.
fn memory_plunder(t: &mut TestGame, card: ObjectId) {
    t.answer_yes(P0, true);
    cast_modes(t, P0, "Memory Plunder", "UUUU", &[], &[&[obj(card)]]);
    t.resolve_all();
}

/// Wrexial attacks P1 alone; its trigger targets `card` in P1's graveyard and P0 chooses
/// to cast it.
fn wrexial(t: &mut TestGame, card: ObjectId) {
    supported("Wrexial, the Risen Deep");
    let w = t.battlefield(P0, "Wrexial, the Risen Deep");
    t.answer_targets(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    t.attack(&[(w, pl(P1))], &[]);
    t.resolve_all();
}

#[test]
fn a_card_that_cant_be_cast_stays_in_its_owners_graveyard() {
    cr!("608.2g", "601.2c");
    ruling!(
        "Memory Plunder",
        "If you are unable to cast the card (there are no legal targets for the spell, for example), nothing happens when Memory Plunder resolves, and the card remains in its owner's graveyard."
    );
    ruling!(
        "Wrexial, the Risen Deep",
        "If you are unable to cast the targeted card (there are no legal targets for the spell, for example), nothing happens when the ability resolves, and the card remains in its owner's graveyard."
    );
    // Shatter with no artifact on the battlefield (with P1's Bone Saw there, it's cast).
    for wrex in [false, true] {
        for artifact in [false, true] {
            let mut t = TestGame::new(2);
            let shatter = t.graveyard(P1, "Shatter");
            let saw = artifact.then(|| t.battlefield(P1, "Bone Saw"));
            if wrex {
                wrexial(&mut t, shatter);
            } else {
                memory_plunder(&mut t, shatter);
            }
            let what = format!("wrexial={wrex} artifact={artifact}");
            assert_eq!(casts(&t, "Shatter").len(), usize::from(artifact), "{what}");
            match saw {
                Some(saw) => {
                    assert_eq!(t.zone(saw), Zone::Graveyard(P1), "{what}");
                    let gone_to = if wrex {
                        Zone::Exile
                    } else {
                        Zone::Graveyard(P1)
                    };
                    assert_eq!(t.zone(shatter), gone_to, "{what}");
                }
                None => assert_eq!(t.zone(shatter), Zone::Graveyard(P1), "{what}"),
            }
        }
    }
}

#[test]
fn casting_during_resolution_ignores_type_timing_but_not_other_restrictions() {
    cr!("608.2g", "307.1", "601.3");
    ruling!(
        "Memory Plunder",
        "If you want to cast the card, you cast it as part of the resolution of Memory Plunder. Timing restrictions based on the card's type are ignored if it's a sorcery. Other casting restrictions are not (such as “Cast [this card] only during combat”)."
    );
    ruling!(
        "Wrexial, the Risen Deep",
        "If you want to cast the targeted card, you cast it as part of the resolution of Wrexial's triggered ability. Timing restrictions based on the card's type are ignored. Other restrictions are not (such as \"Cast [this card] only during your end step\")."
    );
    // Memory Plunder in P1's main phase: Divination (a sorcery) is cast; Gorilla War Cry
    // ("Cast this spell only during combat before blockers are declared.") isn't.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let div = t.graveyard(P1, "Divination");
    let hand = t.hand_size(P0);
    memory_plunder(&mut t, div);
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.zone(div), Zone::Graveyard(P1));
    assert_eq!(casts(&t, "Divination").len(), 1);
    let mut t = TestGame::new(2);
    let cry = t.graveyard(P1, "Gorilla War Cry");
    memory_plunder(&mut t, cry);
    assert_eq!(t.zone(cry), Zone::Graveyard(P1));
    assert!(casts(&t, "Gorilla War Cry").is_empty());
    // (In P0's beginning of combat step, before blockers, it is cast.)
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let cry = t.graveyard(P1, "Gorilla War Cry");
    memory_plunder(&mut t, cry);
    assert_eq!(casts(&t, "Gorilla War Cry").len(), 1, "{}", t.dump_log());
    // Wrexial in the combat damage step: Divination is cast (and exiled); Gorilla War Cry
    // isn't, blockers having been declared.
    let mut t = TestGame::new(2);
    let div = t.graveyard(P1, "Divination");
    let hand = t.hand_size(P0);
    wrexial(&mut t, div);
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.zone(div), Zone::Exile);
    let mut t = TestGame::new(2);
    let cry = t.graveyard(P1, "Gorilla War Cry");
    wrexial(&mut t, cry);
    assert_eq!(t.zone(cry), Zone::Graveyard(P1));
}

#[test]
fn wrexial_exiles_the_card_whether_it_resolves_or_is_countered() {
    cr!("614.1a", "608.2n", "701.6a");
    ruling!(
        "Wrexial, the Risen Deep",
        "If you cast the targeted card and it would be put into its owner's graveyard from the stack for any reason (either because it resolves or because it's countered), that card is exiled instead."
    );
    for countered in [false, true] {
        let mut t = TestGame::new(2);
        let w = t.battlefield(P0, "Wrexial, the Risen Deep");
        let bolt = t.graveyard(P1, "Lightning Bolt");
        t.answer_targets(P0, &[obj(bolt)]);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[pl(P1)]);
        attack_with(&mut t, &[(w, pl(P1))]);
        t.advance_to(P0, Step::CombatDamage);
        // Wrexial's trigger resolves: Lightning Bolt is cast and on the stack.
        t.resolve();
        assert_eq!(stack_names(&t.g), vec!["Lightning Bolt".to_string()]);
        if countered {
            let cs = t.hand(P1, "Counterspell");
            mana(&mut t, P1, "UU");
            let spell = *t.g.stack.last().unwrap();
            t.answer_targets(P1, &[obj(spell)]);
            t.g.turn.priority = Some(P1);
            t.g.cast_spell(P1, cs, CastMethod::Normal).unwrap();
        }
        t.resolve_all();
        assert_eq!(t.zone(bolt), Zone::Exile, "countered={countered}");
        assert_eq!(t.life(P1), if countered { 15 } else { 12 });
    }
}

#[test]
fn wildfire_devils_does_nothing_if_the_random_player_has_no_instant_or_sorcery() {
    cr!("609.3", "608.2c");
    ruling!(
        "Wildfire Devils",
        "If the chosen player has no instant or sorcery cards in their graveyard, Wildfire Devils’s ability does nothing. You don’t pick a different player."
    );
    supported("Wildfire Devils");
    // Only P1 has one: when P0 is chosen at random, nothing happens.
    let mut outcomes = std::collections::BTreeSet::new();
    for seed in 0..24 {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                seed,
                ..Default::default()
            },
        );
        let div = t.graveyard(P1, "Divination");
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.enter(P0, "Wildfire Devils");
        t.resolve_all();
        let cast = t.zone(div) == Zone::Exile;
        assert_eq!(
            t.hand_size(P0),
            if cast { hand + 2 } else { hand },
            "seed {seed}"
        );
        if !cast {
            assert_eq!(t.zone(div), Zone::Graveyard(P1), "seed {seed}");
        }
        outcomes.insert(cast);
    }
    assert_eq!(outcomes.len(), 2, "both players were chosen across seeds");
}

#[test]
fn wildfire_devils_copy_is_cast_on_resolution_or_never() {
    cr!("608.2g", "707.12", "307.1");
    ruling!(
        "Wildfire Devils",
        "You can cast the copy during the resolution of the triggered ability if it’s a sorcery, no matter whose turn it is or which phase it is."
    );
    ruling!(
        "Wildfire Devils",
        "You can cast the copy only as the triggered ability resolves. If you don’t want to cast it at that time (or you can’t cast it, perhaps because there are no legal targets available), the copy ceases to exist. You can’t cast it later."
    );
    for yes in [true, false] {
        // In P1's upkeep (as if it had flash); both graveyards have a Divination, so
        // whoever is chosen exiles one.
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::Upkeep);
        t.graveyard(P0, "Divination");
        t.graveyard(P1, "Divination");
        let hand = t.hand_size(P0);
        let seen = watch(&mut t, P0, is_yes_no, |g| stack_names(g));
        t.answer_yes(P0, yes);
        t.enter(P0, "Wildfire Devils");
        t.resolve_all();
        assert_eq!(
            t.hand_size(P0),
            if yes { hand + 2 } else { hand },
            "yes={yes}"
        );
        // Asked while the trigger was still on the stack.
        let seen = seen.lock().unwrap().clone();
        assert_eq!(
            seen,
            vec![vec!["Wildfire Devils ability".to_string()]],
            "yes={yes}"
        );
        // The exiled card stays exiled; no copy is left to cast later.
        assert!(t.in_exile("Divination"));
        assert_eq!(t.g.exile.len(), 1, "only the exiled card: yes={yes}");
        assert!(t.stack.is_empty());
        assert!(!can_cast_anything(&mut t, P0), "yes={yes}");
    }
}

#[test]
fn reenact_the_crime_copy_is_cast_while_it_is_on_the_stack() {
    cr!("608.2g", "707.12");
    ruling!(
        "Reenact the Crime",
        "You cast the copy while Reenact the Crime is resolving and still on the stack. You can't wait to cast it later in the turn."
    );
    supported("Reenact the Crime");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let div = t.hand(P1, "Divination");
        let div = crate::r_s05_common::move_to(&mut t, div, Zone::Graveyard(P1)).unwrap();
        let hand = t.hand_size(P0);
        let seen = watch(&mut t, P0, is_yes_no, |g| stack_names(g));
        t.answer_yes(P0, yes);
        cast_modes(&mut t, P0, "Reenact the Crime", "1UUU", &[], &[&[obj(div)]]);
        t.resolve_all();
        let seen = seen.lock().unwrap().clone();
        assert_eq!(seen, vec![vec!["Reenact the Crime".to_string()]]);
        assert_eq!(t.hand_size(P0), if yes { hand + 2 } else { hand });
        assert_eq!(t.zone(div), Zone::Exile);
        assert_eq!(t.g.exile.len(), 1, "no copy is left: yes={yes}");
        assert!(!can_cast_anything(&mut t, P0));
    }
}

#[test]
fn soundwave_targets_only_the_exact_mana_value() {
    cr!("115.1", "202.3", "603.3d");
    ruling!(
        "Soundwave, Sonic Spy // Soundwave, Superior Captain",
        "Soundwave, Sonic Spy's triggered ability can target only an instant or sorcery card with mana value exactly equal to the amount of damage dealt to the player by creature tokens you control."
    );
    supported(SOUNDWAVE);
    let mut t = TestGame::new(2);
    t.battlefield(P0, SOUNDWAVE);
    let tok = t.battlefield(P0, "Grizzly Bears");
    tokenize(&mut t, tok);
    t.graveyard(P1, "Lightning Bolt");
    let cs = t.graveyard(P1, "Counterspell");
    t.graveyard(P1, "Divination");
    t.graveyard(P0, "Counterspell");
    t.graveyard(P1, "Hill Giant");
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.attack(&[(tok, pl(P1))], &[]);
    let c = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(c, vec![vec![obj(cs)]]);
    // Two 1/1 tokens and a nontoken Hill Giant: the tokens dealt 2 together (not 1 each,
    // and not the 5 all the attackers dealt).
    let mut t = TestGame::new(2);
    t.battlefield(P0, SOUNDWAVE);
    let a = t.battlefield(P0, "Llanowar Elves");
    let b = t.battlefield(P0, "Llanowar Elves");
    tokenize(&mut t, a);
    tokenize(&mut t, b);
    let giant = t.battlefield(P0, "Hill Giant");
    t.graveyard(P1, "Lightning Bolt");
    let cs = t.graveyard(P1, "Counterspell");
    t.graveyard(P1, "Divination");
    t.graveyard(P1, "Plow Under");
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.attack(&[(a, pl(P1)), (b, pl(P1)), (giant, pl(P1))], &[]);
    assert_eq!(t.life(P1), 15);
    let c = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(c, vec![vec![obj(cs)]]);
}

#[test]
fn soundwave_copy_not_cast_ceases_to_exist_and_the_card_stays_exiled() {
    cr!("707.12", "704.5e", "608.2g");
    ruling!(
        "Soundwave, Sonic Spy // Soundwave, Superior Captain",
        "If you don't cast the copy, it ceases to exist the next time state-based actions are checked. The exiled card stays exiled whether you cast the copy or not."
    );
    let mut t = TestGame::new(2);
    let sw = t.battlefield(P0, SOUNDWAVE);
    let tok = t.battlefield(P0, "Grizzly Bears");
    tokenize(&mut t, tok);
    let cs = t.graveyard(P1, "Counterspell");
    t.answer_targets(P0, &[obj(cs)]);
    t.answer_yes(P0, false);
    t.attack(&[(tok, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.zone(cs), Zone::Exile);
    assert_eq!(t.g.exile.len(), 1, "no copy left in exile");
    assert!(t.stack.is_empty());
    assert!(!can_cast_anything(&mut t, P0));
    // Not cast: Soundwave doesn't convert.
    assert_eq!(t.obj_now(sw).face, FaceState::Front);
}

#[test]
fn superior_captain_uses_the_chosen_x_for_mana_value() {
    cr!("107.3", "202.3e", "712.8");
    ruling!(
        "Soundwave, Sonic Spy // Soundwave, Superior Captain",
        "For Soundwave, Superior Captain's abilities, if a spell you are casting has an {X} in its mana cost, use the value chosen for X to determine its total mana value."
    );
    // Blaze with X = 2 has mana value 3 (odd: Ravage); with X = 1, 2 (even: Laserbeak).
    for (x, token) in [(2, "Ravage"), (1, "Laserbeak")] {
        let mut t = TestGame::new(2);
        let sw = t.battlefield(P0, SOUNDWAVE);
        assert!(mtg_engine::dfc::transform(&mut t.g, sw));
        t.g.recompute();
        assert_eq!(t.obj_now(sw).face, FaceState::Back);
        let blaze = t.hand(P0, "Blaze");
        mana(&mut t, P0, &format!("{x}R"));
        t.cast(P0, blaze).x(x).target(pl(P1)).go();
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield(token).len(),
            1,
            "x={x}: {}",
            t.dump_log()
        );
        assert_eq!(t.life(P1), 20 - x as i32);
    }
}

#[test]
fn spelltwine_casts_both_copies_from_exile_in_either_order() {
    cr!("707.12", "608.2g", "601.2a");
    ruling!(
        "Spelltwine",
        "The copies are created in exile and cast from exile. The cards remain exiled."
    );
    ruling!("Spelltwine", "You can cast the copies in either order.");
    ruling!(
        "Spelltwine",
        "The copy you cast first will already be on the stack when you cast the other copy. If that spell targets a spell, you may choose the first copy as a legal target."
    );
    supported("Spelltwine");
    for order in [vec![0usize, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let div = t.graveyard(P0, "Divination");
        let cancel = t.graveyard(P1, "Cancel");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        let seen = watch(&mut t, P0, is_targets, |g| {
            g.stack
                .iter()
                .map(|id| (g.obj(*id).chars.name.to_string(), g.obj(*id).kind))
                .collect::<Vec<_>>()
        });
        cast_modes(
            &mut t,
            P0,
            "Spelltwine",
            "5U",
            &[],
            &[&[obj(div)], &[obj(cancel)]],
        );
        let from = t.asked().len();
        t.resolve_all();
        let div_cast = casts(&t, "Divination");
        let cancel_cast = casts(&t, "Cancel");
        assert_eq!(div_cast.len(), 1, "{order:?}: {}", t.dump_log());
        assert_eq!(cancel_cast.len(), 1, "{order:?}");
        // Cast from exile; the cards stay there.
        assert_eq!(div_cast[0].1, Some(ZoneKind::Exile));
        assert_eq!(cancel_cast[0].1, Some(ZoneKind::Exile));
        assert_eq!(t.zone(div), Zone::Exile);
        assert_eq!(t.zone(cancel), Zone::Exile);
        // The order chosen is the order they were cast in.
        let first_is_div = div_cast[0].0 .0 < cancel_cast[0].0 .0;
        assert_eq!(first_is_div, order == vec![0, 1], "{order:?}");
        if first_is_div {
            // Cancel's target choice saw the Divination copy on the stack, and it was a
            // legal target.
            let seen = seen.lock().unwrap().clone();
            let last = seen.last().unwrap();
            assert!(
                last.contains(&("Divination".to_string(), ObjKind::CardCopy)),
                "{last:?}"
            );
            let candidates = crate::r_s02_common::target_candidates(&t, P0, from);
            let cancels = candidates.last().expect("Cancel's targets");
            assert!(cancels.contains(&obj(div_cast[0].0)), "{cancels:?}");
        }
    }
}

/// Spellweaver Helix enters under P0's control and exiles `a` and `b` from P1's
/// graveyard. Returns the Helix.
fn helix(t: &mut TestGame, a: &str, b: &str) -> ObjectId {
    supported("Spellweaver Helix");
    let a = t.graveyard(P1, a);
    let b = t.graveyard(P1, b);
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer_yes(P0, true);
    let h = t.enter(P0, "Spellweaver Helix");
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    h
}

/// P0 casts a Divination from hand (the same name as one of the Helix cards), wanting to
/// copy the other card and cast the copy; the triggered ability goes on the stack.
fn cast_divination(t: &mut TestGame) {
    let d = t.hand(P0, "Divination");
    mana(t, P0, "2U");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, d).go();
    t.settle();
}

#[test]
fn spellweaver_helix_copy_is_cast_with_both_still_on_the_stack() {
    cr!("608.2g", "707.12", "603.4");
    ruling!(
        "Spellweaver Helix",
        "You cast the copy while this ability is resolving, and still on the stack. Normally, you’re not allowed to cast spells or activate abilities at this time. Spellweaver Helix’s ability breaks this rule. (The card that triggered this ability is also still on the stack.)"
    );
    let mut t = TestGame::new(2);
    helix(&mut t, "Divination", "Lava Spike");
    let seen = watch(&mut t, P0, is_targets, |g| stack_names(g));
    cast_divination(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    let seen = seen.lock().unwrap().clone();
    // Lava Spike's target is chosen with Divination and the Helix trigger on the stack.
    let last = seen.last().unwrap();
    assert_eq!(last[0], "Divination", "{seen:?}");
    assert!(
        last.contains(&"Spellweaver Helix ability".to_string()),
        "{seen:?}"
    );
}

#[test]
fn spellweaver_helix_copies_even_after_it_leaves_the_battlefield() {
    cr!("608.2h", "113.7a", "607.2a");
    ruling!(
        "Spellweaver Helix",
        "If this card leaves the battlefield while the ability to make a copy is on the stack, the ability will still make a copy using the last-known-information rule."
    );
    let mut t = TestGame::new(2);
    let h = helix(&mut t, "Divination", "Lava Spike");
    cast_divination(&mut t);
    assert_eq!(t.stack_len(), 2);
    t.g.destroy(h, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Spellweaver Helix"));
    assert_eq!(casts(&t, "Lava Spike").len(), 1, "{}", t.dump_log());
    assert_eq!(t.life(P1), 17);
}

#[test]
fn spellweaver_helix_copy_needs_targets_and_permission_to_cast() {
    cr!("601.2c", "601.3", "707.12", "101.2");
    ruling!(
        "Spellweaver Helix",
        "You can’t cast the copy unless all of its targets can be chosen."
    );
    ruling!(
        "Spellweaver Helix",
        "You can’t cast the copy if an effect prevents you from casting sorceries or from casting that particular sorcery."
    );
    // Flame Slash ("Flame Slash deals 4 damage to target creature.") with no creature
    // (with P1's Grizzly Bears there, the copy is cast).
    for creature in [false, true] {
        let mut t = TestGame::new(2);
        helix(&mut t, "Divination", "Flame Slash");
        let bears = creature.then(|| t.battlefield(P1, "Grizzly Bears"));
        let d = t.hand(P0, "Divination");
        mana(&mut t, P0, "2U");
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        if let Some(b) = bears {
            t.answer_targets(P0, &[obj(b)]);
        }
        t.cast(P0, d).go();
        t.settle();
        t.resolve_all();
        assert_eq!(
            casts(&t, "Flame Slash").len(),
            usize::from(creature),
            "creature={creature}"
        );
        assert!(t.in_exile("Flame Slash"));
        assert_eq!(t.g.exile.len(), 2, "no copy left: creature={creature}");
        if let Some(b) = bears {
            assert_eq!(t.zone(b), Zone::Graveyard(P1));
        }
    }
    // Meddling Mage naming Lava Spike ("Spells with the chosen name can't be cast.").
    let mut t = TestGame::new(2);
    helix(&mut t, "Divination", "Lava Spike");
    t.answer(P1, DecisionKind::Name, Answer::Text("Lava Spike".into()));
    t.enter(P1, "Meddling Mage");
    t.resolve_all();
    cast_divination(&mut t);
    t.resolve_all();
    assert!(casts(&t, "Lava Spike").is_empty(), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
    assert!(casts(&t, "Divination").len() == 1);
    // Abeyance on P0 ("Until end of turn, target player can't cast instant or sorcery
    // spells, ..."): P1's Divination triggers P0's Helix, but P0 can't cast the copy
    // (without Abeyance, P0 does).
    for abeyance in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        helix(&mut t, "Divination", "Lava Spike");
        if abeyance {
            cast_modes(&mut t, P1, "Abeyance", "1W", &[], &[&[pl(P0)]]);
            t.resolve_all();
        }
        let d = t.hand(P1, "Divination");
        mana(&mut t, P1, "2U");
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[pl(P1)]);
        t.cast(P1, d).go();
        t.settle();
        assert_eq!(
            crate::r_s01_common::triggers_on_stack(&t, "copy the other"),
            1
        );
        t.resolve_all();
        assert_eq!(
            casts(&t, "Lava Spike").len(),
            usize::from(!abeyance),
            "abeyance={abeyance}: {}",
            t.dump_log()
        );
        assert_eq!(t.life(P1), if abeyance { 20 } else { 17 });
        assert_eq!(casts(&t, "Divination").len(), 1);
    }
}

#[test]
fn diluvian_primordial_cipher_spell_is_exiled_encoded_or_not() {
    cr!("702.99a", "614.1a", "608.2n");
    ruling!(
        "Diluvian Primordial",
        "If you cast an instant or sorcery spell with cipher this way, you may exile the card encoded on a creature you control. If you can't, or if you choose not to, the card will end up exiled but not encoded on a creature."
    );
    supported("Diluvian Primordial");
    for encode in [true, false] {
        let mut t = TestGame::new(2);
        let slice = t.graveyard(P1, "Shadow Slice");
        t.answer_targets(P0, &[obj(slice)]);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[pl(P1)]);
        let dp = t.enter(P0, "Diluvian Primordial");
        let choice: Vec<Entity> = if encode { vec![obj(dp)] } else { vec![] };
        t.answer_choose(P0, &choice);
        t.resolve_all();
        assert_eq!(t.life(P1), 17, "encode={encode}: {}", t.dump_log());
        assert_eq!(t.zone(slice), Zone::Exile, "encode={encode}");
        let card = t.g.current(slice);
        let on = encoded_on(&t.g, card);
        assert_eq!(on, encode.then_some(dp), "encode={encode}");
    }
}
