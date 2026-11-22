//! Rulings batch P046 — cards that destroy or exile artifacts and enchantments: tokens'
//! mana values (CR 202.3a, 107.3b), simultaneous enters triggers (CR 603.2), lords leaving
//! (CR 704.5g), lands put onto the battlefield (CR 305.4), linked choices remembered by an
//! object (CR 400.7, 607), copies that weren't cast (CR 707.10), and Urza's Constructs.

use crate::r_p046_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn treasure(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let spec = mtg_engine::tokens::predefined("Treasure").expect("Treasure");
    crate::r_p023_common::token_of(t, p, spec)
}

#[test]
fn rampaging_yao_guai_with_x_0_can_destroy_tokens() {
    cr!("202.3a");
    ruling!(
        "Rampaging Yao Guai",
        "A token that's not a copy of something else normally won't have a mana cost. Anything without a mana cost normally has a mana value of 0."
    );
    supported("Rampaging Yao Guai");
    let mut t = TestGame::new(2);
    let a = treasure(&mut t, P1);
    let b = treasure(&mut t, P1);
    let s = t.hand(P0, "Rampaging Yao Guai");
    t.g.players[0].mana_pool.add_type(ManaType::G, 3);
    t.cast(P0, s).x(0).go();
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

#[test]
fn natural_state_sees_x_as_0_on_the_battlefield() {
    cr!("202.3e");
    ruling!(
        "Natural State",
        "If an artifact or enchantment on the battlefield has {X} in its mana cost, X is 0."
    );
    supported("Natural State");
    supported("Astral Cornucopia");
    // Astral Cornucopia ({X}{X}{X}) cast with X = 2: mana value 0 on the battlefield.
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Astral Cornucopia");
    t.g.players[0].mana_pool.add_type(ManaType::C, 6);
    t.cast(P0, c).x(2).go();
    t.resolve_all();
    let c = t.g.current(c);
    assert!(t.on_battlefield(c));
    assert_eq!(t.g.mana_value_of(c), 0);
    let s = t.hand(P0, "Natural State");
    t.g.players[0].mana_pool.add_type(ManaType::G, 1);
    t.cast(P0, s).target(c).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Astral Cornucopia"));
}

#[test]
fn feline_sovereign_leaving_can_make_damage_lethal() {
    cr!("704.5g", "613.4c", "120.6");
    ruling!(
        "Feline Sovereign",
        "nonlethal damage dealt to a Cat you control may become lethal if Feline Sovereign leaves the battlefield during that turn."
    );
    supported("Feline Sovereign");
    let mut t = TestGame::new(2);
    let sov = t.battlefield(P0, "Feline Sovereign");
    // Savannah Lions (a 2/1 Cat): 3/2, dealt 1 damage.
    let lions = t.battlefield(P0, "Savannah Lions");
    assert_eq!(t.pt(lions), (3, 2));
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Object(lions), 1, false);
    t.settle();
    assert!(t.on_battlefield(lions));
    bounce(&mut t, sov);
    t.settle();
    assert!(t.in_graveyard(P0, "Savannah Lions"));
}

#[test]
fn putting_a_land_onto_the_battlefield_isnt_playing_one() {
    cr!("305.2a", "305.4");
    ruling!(
        "Broken Bond",
        "Broken Bond’s effect doesn’t count as playing a land. It can put a land card onto the battlefield even if you’ve already played your land for the turn."
    );
    supported("Broken Bond");
    let mut t = TestGame::new(2);
    let first = t.hand(P0, "Forest");
    t.play_land(P0, first).unwrap();
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let forest = t.hand(P0, "Forest");
    let s = t.hand(P0, "Broken Bond");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    assert!(cost_to_cast(&mut t, P0, s, &[Entity::Object(anthem)]).is_some());
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1);
    ruling!(
        "Insidious Fungus",
        "Putting a land card onto the battlefield with the last mode of Insidious Fungus's ability doesn't count as playing a land."
    );
    supported("Insidious Fungus");
    let mut t = TestGame::new(2);
    let first = t.hand(P0, "Forest");
    t.play_land(P0, first).unwrap();
    let fungus = t.battlefield(P0, "Insidious Fungus");
    let forest = t.hand(P0, "Forest");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.g.players[0].mana_pool.add_type(ManaType::C, 2);
    t.activate(P0, fungus, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert!(t.obj_now(forest).tapped);
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1);
}

// ---------------------------------------------------------------------------------------
// Garth One-Eye: "{T}: Choose a card name that hasn't been chosen from among Disenchant,
// Braingeyser, Terror, Shivan Dragon, Regrowth, and Black Lotus. Create a copy of the
// card with the chosen name. You may cast the copy."
// ---------------------------------------------------------------------------------------

/// Activates `garth` for its controller, choosing the first name offered and not casting
/// the copy; returns the number of names it could choose from (0 if it made no copy).
fn garth_names(t: &mut TestGame, garth: ObjectId) -> usize {
    let garth = t.g.current(garth);
    let p = t.g.obj(garth).controller;
    t.g.untap(garth);
    t.g.objects[garth.0 as usize].activations_this_turn.clear();
    t.g.objects[garth.0 as usize].summoning_sick = false;
    t.answer(p, DecisionKind::Option, Answer::Index(0));
    t.answer_yes(p, false);
    let copies = |t: &TestGame| {
        t.g.objects
            .iter()
            .filter(|o| o.kind == mtg_engine::object::ObjKind::CardCopy)
            .count()
    };
    let before = copies(t);
    let from = t.asked().len();
    t.activate(p, garth, 0, &[]).unwrap();
    t.resolve_all();
    t.clear_answers();
    if copies(t) == before {
        return 0;
    }
    // A single name left is chosen without asking.
    asked_since(t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt == "Choose a card name" => Some(options.len()),
            _ => None,
        })
        .unwrap_or(1)
}

#[test]
fn each_garth_one_eye_remembers_its_own_choices() {
    cr!("400.7", "707.13");
    ruling!(
        "Garth One-Eye",
        "Each Garth One-Eye keeps track of card names chosen for itself, but not for other Garth One-Eyes."
    );
    supported("Garth One-Eye");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Garth One-Eye");
    assert_eq!(garth_names(&mut t, a), 6);
    assert_eq!(garth_names(&mut t, a), 5);
    // Another Garth (as if it were a second copy, without the legend rule: P1's).
    let b = t.battlefield(P1, "Garth One-Eye");
    t.set_step(P1, Step::PrecombatMain);
    assert_eq!(garth_names(&mut t, b), 6);
    // Another player gains control of the first: it still remembers.
    t.set_step(P1, Step::PrecombatMain);
    let treason = t.hand(P1, "Act of Treason");
    t.g.players[1].mana_pool.add_type(ManaType::R, 3);
    t.cast(P1, treason).target(a).go();
    t.resolve_all();
    assert_eq!(t.obj_now(a).controller, P1);
    assert_eq!(garth_names(&mut t, a), 4);
    // It leaves the battlefield and returns: a new object that remembers nothing.
    let a = t.g.current(a);
    let gone = t
        .g
        .move_object(a, Zone::Exile, mtg_engine::events::MoveCause::Effect, None)
        .unwrap();
    let back = t
        .g
        .move_object(
            gone,
            Zone::Battlefield,
            mtg_engine::events::MoveCause::Effect,
            Some(P0),
        )
        .unwrap();
    t.g.recompute();
    t.set_step(t.g.obj(back).controller, Step::PrecombatMain);
    assert_eq!(garth_names(&mut t, back), 6);
}

#[test]
fn garth_one_eye_does_nothing_once_every_name_was_chosen() {
    cr!("707.13", "608.2c");
    ruling!(
        "Garth One-Eye",
        "If all choices have been named before, Garth's ability won't do anything when it resolves."
    );
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Garth One-Eye");
    for n in (1..=6).rev() {
        assert_eq!(garth_names(&mut t, g), n);
    }
    // No copy is created.
    assert_eq!(garth_names(&mut t, g), 0);
}

// ---------------------------------------------------------------------------------------
// Act of Authority: "When this enchantment enters, you may exile target artifact or
// enchantment. At the beginning of your upkeep, you may exile target artifact or
// enchantment. If you do, its controller gains control of this enchantment."
// ---------------------------------------------------------------------------------------

#[test]
fn act_of_authority_must_target_itself_but_may_not_exile() {
    cr!("603.3d", "603.5");
    ruling!(
        "Act of Authority",
        "For either ability, if you are the only player who controls an artifact or enchantment, you must choose one of them as the target. However, you may choose to not exile it when the ability resolves."
    );
    supported("Act of Authority");
    let mut t = TestGame::new(2);
    let act = t.enter(P0, "Act of Authority");
    t.settle();
    // The trigger is on the stack targeting Act of Authority itself.
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    let targets = &t.g.obj(top).stack.as_ref().unwrap().chosen[0].targets;
    assert_eq!(targets, &vec![vec![Entity::Object(act)]]);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.on_battlefield(act));
}

#[test]
fn act_of_authority_stays_yours_if_you_dont_exile() {
    cr!("603.5", "608.2c");
    ruling!(
        "Act of Authority",
        "If you choose to not exile a permanent with the second ability, you'll retain control of Act of Authority."
    );
    let upkeep = |exile: bool| {
        let mut t = TestGame::new(2);
        let act = t.battlefield(P0, "Act of Authority");
        let anthem = t.battlefield(P1, "Glorious Anthem");
        t.answer_targets(P0, &[Entity::Object(anthem)]);
        t.answer_yes(P0, exile);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.on_battlefield(anthem), !exile);
        t.obj_now(act).controller
    };
    assert_eq!(upkeep(false), P0);
    assert_eq!(upkeep(true), P1);
}

#[test]
fn leonin_relic_warder_leaving_first_exiles_for_good() {
    cr!("603.6c", "607.2a");
    ruling!(
        "Leonin Relic-Warder",
        "If Leonin Relic-Warder leaves the battlefield before its first ability has resolved, its second ability will trigger and when it resolves, do nothing. Then its first ability will resolve and exile the targeted artifact or enchantment with no way to return it to the battlefield."
    );
    supported("Leonin Relic-Warder");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_targets(P0, &[Entity::Object(anthem)]);
    let warder = t.enter(P0, "Leonin Relic-Warder");
    t.settle();
    bounce(&mut t, warder);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_exile("Glorious Anthem"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn qasali_slingers_triggers_for_each_cat_entering_with_it() {
    cr!("603.2", "603.6a");
    ruling!(
        "Qasali Slingers",
        "If Qasali Slingers enters the battlefield at the same time as other Cats you control, its ability will trigger for each of those Cats."
    );
    supported("Qasali Slingers");
    supported("Living Death");
    // Living Death returns Qasali Slingers and two Savannah Lions at the same time.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Qasali Slingers");
    t.graveyard(P0, "Savannah Lions");
    t.graveyard(P0, "Savannah Lions");
    for _ in 0..3 {
        t.battlefield(P1, "Glorious Anthem");
    }
    let s = t.hand(P0, "Living Death");
    give_mana_for(&mut t, P0, "Living Death");
    t.cast(P0, s).go();
    t.resolve();
    t.settle();
    assert_eq!(creatures(&t, P0).len(), 3);
    assert_eq!(triggers_on_stack(&t, "destroy target artifact or enchantment"), 3);
}

#[test]
fn sunder_shaman_with_menace_cant_be_blocked() {
    cr!("702.111b", "509.1b");
    ruling!("Sunder Shaman", "If Sunder Shaman gains menace, it can’t be blocked at all.");
    supported("Sunder Shaman");
    supported("Madcap Skills");
    // One blocker isn't enough for menace (without menace, it could block alone), and two
    // are too many for its own restriction.
    for n in [1, 2] {
        let mut t = TestGame::new(2);
        let shaman = t.battlefield(P0, "Sunder Shaman");
        let skills = t.hand(P0, "Madcap Skills");
        give_mana_for(&mut t, P0, "Madcap Skills");
        t.cast(P0, skills).target(shaman).go();
        t.resolve_all();
        let blockers = bears(&mut t, P1, 2);
        attack_with(&mut t, &[(shaman, Entity::Player(P1))]);
        let blocks: Vec<_> = blockers[..n].iter().map(|b| (*b, shaman)).collect();
        block_and_finish(&mut t, P1, &blocks);
        // Unblocked: 8 damage.
        assert_eq!(t.life(P1), 12, "{n} blockers");
        assert!(blockers.iter().all(|b| t.on_battlefield(*b)));
    }
}

#[test]
fn a_copy_of_return_to_dust_exiles_only_the_first_target() {
    cr!("707.10", "601.2");
    ruling!(
        "Return to Dust",
        "If a spell or ability copies Return to Dust, the copy exiles only the first target artifact or enchantment. This is because the copy wasn't cast at all."
    );
    supported("Return to Dust");
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Darksteel Relic");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let s = t.hand(P0, "Return to Dust");
    fill_pool(&mut t, P0, 4);
    t.cast(P0, s).target(relic).target(anthem).go();
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(s).go();
    t.answer_yes(P0, false);
    t.resolve();
    // The copy is on the stack above Return to Dust: it resolves first.
    t.resolve();
    assert!(t.in_exile("Darksteel Relic"));
    assert!(t.on_battlefield(anthem));
    t.resolve_all();
    assert!(t.in_exile("Glorious Anthem"));
}

#[test]
fn urzas_constructs_count_themselves() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Urza's Command",
        "The ability of that Construct token does count the Construct itself, so it'll be at least 1/1."
    );
    supported("Urza's Command");
    let construct = |t: &TestGame| {
        let c = with_subtype(t, P0, "Construct");
        assert_eq!(c.len(), 1);
        t.pt(c[0])
    };
    // Construct and "scry 1, then draw": the only artifact.
    let mut t = TestGame::new(2);
    let s = t.hand(P0, "Urza's Command");
    fill_pool(&mut t, P0, 4);
    t.cast(P0, s).modes(&[2, 3]).go();
    t.resolve_all();
    assert_eq!(construct(&t), (1, 1));
    // With the Powerstone too: 2/2.
    let mut t = TestGame::new(2);
    let s = t.hand(P0, "Urza's Command");
    fill_pool(&mut t, P0, 4);
    t.cast(P0, s).modes(&[1, 2]).go();
    t.resolve_all();
    assert_eq!(construct(&t), (2, 2));
    ruling!(
        "Urza, Chief Artificer",
        "The ability of that Construct token does count the Construct itself, so it'll be at least 1/1."
    );
    supported("Urza, Chief Artificer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Urza, Chief Artificer");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(construct(&t), (1, 1));
}
