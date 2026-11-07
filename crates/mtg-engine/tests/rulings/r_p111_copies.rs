//! Rulings batch P111 — copies of spells and abilities (CR 707.10): a copy has the same
//! mode, value of X, division of damage and paid alternative costs as the original
//! (CR 707.10, 707.10c, 601.2b); "copy that spell" copies spells with or without targets;
//! Agrus Kos copies an ability once per other creature it could target (CR 707.10d).

use crate::r_p109_common::{divide, division_of};
use crate::r_s01_common::{custom_card, supported};
use crate::r_s25_common::{cast_new, keep_copy_targets, spell_copies};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn mana(t: &mut TestGame, p: PlayerId, tys: &[ManaType]) {
    for ty in tys {
        t.g.players[p.idx()].mana_pool.add_type(*ty, 1);
    }
}

/// Agrus Kos and other creatures for P0; P0 will pay for the copies ({1}{R/W} from the
/// pool) and the opponent P1 has priority in its main phase.
fn agrus(t: &mut TestGame) -> ObjectId {
    supported("Agrus Kos, Eternal Soldier");
    let a = t.battlefield(P0, "Agrus Kos, Eternal Soldier");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Plains", 2);
    t.answer_yes(P0, true);
    a
}

#[test]
fn agrus_kos_ignores_creatures_the_ability_couldnt_target() {
    cr!("707.10d", "115.1", "702.18a");
    ruling!(
        "Agrus Kos, Eternal Soldier",
        "Any creature you control that can't be targeted by the copied ability (due to protection abilities, targeting restrictions, or any other reason) is just ignored. No copy is created for that creature."
    );
    supported("Rod of Ruin");
    // Rod of Ruin: "{3}, {T}: Rod of Ruin deals 1 damage to any target." P0's Wall of
    // Denial has shroud.
    let mut t = TestGame::new(2);
    let ag = agrus(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P0, "Wall of Denial");
    let rod = t.battlefield(P1, "Rod of Ruin");
    mana(&mut t, P1, &[ManaType::C, ManaType::C, ManaType::C]);
    t.activate(P1, rod, 0, &[obj(ag)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(ag).damage, 1);
    assert_eq!(t.obj_now(bears).damage, 1);
    assert_eq!(t.obj_now(wall).damage, 0);
}

#[test]
fn agrus_kos_copies_an_ability_whose_targets_are_all_it() {
    cr!("707.10d", "115.3");
    ruling!(
        "Agrus Kos, Eternal Soldier",
        "If an ability has multiple targets, but it's targeting Agrus Kos with all of them, the last ability triggers. In that case, each of the copies will also target only one of those creatures."
    );
    // An ability with two targets: "{T}: Target creature gets +1/+0 until end of turn.
    // Target creature gains flying until end of turn."
    let def = custom_card(
        "Twin Rod",
        "Artifact",
        "{1}",
        None,
        "{T}: Target creature gets +1/+0 until end of turn. Target creature gains flying until end of turn.",
    );
    let mut t = TestGame::new(2);
    let ag = agrus(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P0, "Hill Giant");
    let rod = t.custom(P1, def, Zone::Battlefield);
    t.activate(P1, rod, 0, &[obj(ag), obj(ag)]).unwrap();
    t.resolve_all();
    for (id, pt) in [(ag, (4, 4)), (bears, (3, 2)), (ogre, (4, 3))] {
        assert_eq!(t.pt(id), pt);
        assert!(t.obj_now(id).has_keyword(KeywordKind::Flying));
    }
}

#[test]
fn agrus_kos_copies_keep_x_and_the_mode() {
    cr!("707.10", "700.2", "107.3");
    ruling!(
        "Agrus Kos, Eternal Soldier",
        "If the ability that's copied has an X whose value was determined as it was put on the stack, the copies have the same value of X."
    );
    ruling!(
        "Agrus Kos, Eternal Soldier",
        "If the ability that's copied is modal (that is, it says \"Choose one —\" or the like), the copies will have the same mode. Their controller can't choose a different one."
    );
    supported("Chamber Sentry");
    supported("Three Bowls of Porridge");
    // Chamber Sentry: "{X}, {T}, Remove X +1/+1 counters from this creature: It deals X
    // damage to any target." X = 2.
    let mut t = TestGame::new(2);
    let ag = agrus(&mut t);
    t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P0, "Wall of Stone");
    let sentry = t.battlefield(P1, "Chamber Sentry");
    t.g.objects[sentry.0 as usize]
        .counters
        .insert("+1/+1".into(), 3);
    mana(&mut t, P1, &[ManaType::C, ManaType::C]);
    t.answer(P1, DecisionKind::X, Answer::Number(2));
    t.activate(P1, sentry, 0, &[obj(ag)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(ag).damage, 2);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.obj_now(wall).damage, 2);
    // Three Bowls of Porridge: "{2}, {T}: Choose one that hasn't been chosen — • This
    // artifact deals 2 damage to target creature. • Tap target creature. • ..." Tapping:
    // the copies tap too and deal no damage.
    let mut t = TestGame::new(2);
    let ag = agrus(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bowls = t.battlefield(P1, "Three Bowls of Porridge");
    mana(&mut t, P1, &[ManaType::C, ManaType::C]);
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.activate(P1, bowls, 0, &[obj(ag)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(ag).tapped);
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.obj_now(bears).damage, 0);
}

/// Mizzix on P0's battlefield; P0 will pay for the copy.
fn mizzix(t: &mut TestGame) {
    supported("Mizzix, Replica Rider");
    t.battlefield(P0, "Mizzix, Replica Rider");
    t.lands(P0, "Island", 2);
    t.answer_yes(P0, true);
}

#[test]
fn mizzix_copies_any_spell_cast_from_elsewhere_with_its_mode() {
    cr!("707.10", "702.34a", "700.2");
    ruling!(
        "Mizzix, Replica Rider",
        "Any spell you cast will be copied if you pay the cost, not just one that requires targets."
    );
    ruling!(
        "Mizzix, Replica Rider",
        "If the copied spell is modal (that is, it says \"Choose one –\" or the like), the copy will have the same mode. You can't choose a different one."
    );
    supported("Think Twice");
    supported("Wreck and Rebuild");
    const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
    // Think Twice: "Draw a card. Flashback {2}{U}", cast from the graveyard.
    let mut t = TestGame::new(2);
    mizzix(&mut t);
    let tt = t.graveyard(P0, "Think Twice");
    t.lands(P0, "Island", 3);
    let library = t.library_size(P0);
    t.cast(P0, tt).method(FLASHBACK).go();
    t.resolve_all();
    assert_eq!(library - t.library_size(P0), 2);
    // Wreck and Rebuild (flashback {3}{R}{G}): "Choose one — • Destroy target artifact
    // or enchantment. • Mill five cards, ..." The copy destroys another artifact and
    // mills nothing.
    let mut t = TestGame::new(2);
    mizzix(&mut t);
    let a = t.battlefield(P1, "Sol Ring");
    let b = t.battlefield(P1, "Ornithopter");
    let wr = t.graveyard(P0, "Wreck and Rebuild");
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Forest", 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[obj(a)]);
    // The copy: new targets, yes: Ornithopter.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(b)]);
    let library = t.library_size(P0);
    t.cast(P0, wr).method(FLASHBACK).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Sol Ring"));
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert_eq!(t.library_size(P0), library, "no mill mode");
}

/// P0's Geistblast in the graveyard copies `spell` ({2}{U} from the pool); returns after
/// its ability resolved (the copy is on the stack).
fn geistblast(t: &mut TestGame, spell: ObjectId) {
    geistblast_then(t, spell, |_| {});
}

/// As [`geistblast`], with `answers` queued after the activation, before it resolves.
fn geistblast_then(t: &mut TestGame, spell: ObjectId, answers: impl FnOnce(&mut TestGame)) {
    supported("Geistblast");
    let g = t.graveyard(P0, "Geistblast");
    mana(t, P0, &[ManaType::U, ManaType::C, ManaType::C]);
    t.activate(P0, g, 0, &[obj(spell)]).unwrap();
    answers(t);
    t.resolve();
    assert_eq!(spell_copies(t).len(), 1, "no copy");
}

#[test]
fn geistblast_copies_spells_without_targets_and_keeps_x() {
    cr!("707.10", "107.3", "707.10c");
    ruling!(
        "Geistblast",
        "Geistblast's ability can copy any instant or sorcery spell, not just one with targets."
    );
    ruling!(
        "Geistblast",
        "If the spell being copied has an X whose value was determined as it was cast (like Avacyn's Judgment's madness cost has), the copy will have the same value of X."
    );
    supported("Divination");
    supported("Blaze");
    let mut t = TestGame::new(2);
    let library = t.library_size(P0);
    let d = cast_new(&mut t, P0, "Divination", &[]);
    geistblast(&mut t, d);
    t.resolve_all();
    assert_eq!(library - t.library_size(P0), 4);
    // Blaze: "Blaze deals X damage to any target." X = 3.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let blaze = t.hand(P0, "Blaze");
    let spell = t.cast(P0, blaze).x(3).target(Entity::Player(P1)).go();
    geistblast(&mut t, spell);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn a_copy_keeps_the_division_of_damage_but_may_change_targets() {
    cr!("707.10c", "601.2d");
    ruling!(
        "Geistblast",
        "If the spell has damage divided as it was cast (also like Avacyn's Judgment), the division can't be changed (although the targets receiving that damage still can)."
    );
    supported("Forked Bolt");
    // Forked Bolt: "Forked Bolt deals 2 damage divided as you choose among one or two
    // targets." 1 to each of two creatures; the copy hits two others, 1 each.
    let mut t = TestGame::new(2);
    let c = [
        t.battlefield(P1, "Wall of Stone"),
        t.battlefield(P1, "Wall of Stone"),
        t.battlefield(P1, "Wall of Stone"),
        t.battlefield(P1, "Wall of Stone"),
    ];
    t.lands(P0, "Mountain", 1);
    divide(&mut t, P0, &[1, 1]);
    let fb = t.hand(P0, "Forked Bolt");
    let spell = t.cast(P0, fb).targets(&[obj(c[0]), obj(c[1])]).go();
    assert_eq!(division_of(&t, spell), vec![1, 1]);
    t.clear_answers();
    geistblast_then(&mut t, spell, |t| {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(c[2])]);
        t.answer_targets(P0, &[obj(c[3])]);
        // A different division, if asked, must be ignored.
        divide(t, P0, &[2, 0]);
    });
    t.resolve_all();
    let marked: Vec<u32> = c.iter().map(|id| t.obj_now(*id).damage).collect();
    assert_eq!(marked, vec![1, 1, 1, 1]);
}

#[test]
fn a_copy_of_a_spell_cast_for_its_surge_cost_was_cast_for_it_too() {
    cr!("707.10", "702.117a", "601.2b");
    ruling!(
        "Geistblast",
        "If an alternative cost was paid for the spell, such as its madness cost or surge cost, the same alternative cost is considered to have been paid for the copy."
    );
    supported("Crush of Tentacles");
    // Crush of Tentacles (surge {3}{U}{U}): "Return all nonland permanents to their
    // owners' hands. If this spell's surge cost was paid, create an 8/8 blue Octopus
    // creature token."
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    t.lands(P0, "Island", 5);
    let crush = t.hand(P0, "Crush of Tentacles");
    let spell = t
        .cast(P0, crush)
        .method(CastMethod::Keyword(KeywordKind::Surge))
        .go();
    geistblast(&mut t, spell);
    // Only the copy has resolved: it made an Octopus.
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    let octopi = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Octopus"))
            .count()
    };
    assert_eq!(octopi(&t), 1);
    t.resolve_all();
    assert_eq!(octopi(&t), 1);
}
