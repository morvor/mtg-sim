//! Rulings batch P024 — copying something that's copying something else: the copy uses
//! the copiable values of the copied permanent, which are whatever it copied as modified
//! by that copy effect (CR 707.3, 707.9b); a copied {X} is 0 (CR 107.3, 202.3e).

use crate::r_p024_common::*;
use mtg_engine::ability::TokenSpec;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s `name` card on the battlefield and a Clone `p` controls copying it.
fn cloned(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let orig = t.battlefield(p, name);
    clone_of(t, p, orig)
}

/// A Glasspool Mimic (`p`'s) that entered as a copy of `p`'s Grizzly Bears "except it's a
/// Shapeshifter Rogue in addition to its other types": a Grizzly Bears that's a Bear
/// Shapeshifter Rogue.
fn mimic_bears(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let bears = t.battlefield(p, "Grizzly Bears");
    let m = enter_copying(t, p, "Glasspool Mimic // Glasspool Shore", bears);
    assert_modified_bears(t, m);
    m
}

/// Asserts `id` is a Grizzly Bears that's also a Shapeshifter Rogue (see [`mimic_bears`]).
fn assert_modified_bears(t: &TestGame, id: ObjectId) {
    assert_eq!(name_of(t, id), "Grizzly Bears");
    assert!(has_subtype(t, id, "Bear"));
    assert!(has_subtype(t, id, "Shapeshifter") && has_subtype(t, id, "Rogue"));
    assert_eq!(t.pt(id), (2, 2));
}

// --- Token copies --------------------------------------------------------------------------

#[test]
fn token_copies_of_a_clone_are_what_it_copied() {
    cr!("707.3", "707.2");
    ruling!("Stolen Identity", "If the copied permanent is copying something else (for example, if the copied creature is a Clone), then the token enters as whatever that creature copied.");
    ruling!("Devastating Onslaught", "If the copied permanent is copying something else, then the tokens enter as whatever that permanent copied.");
    for (spell, cost, x) in [
        ("Stolen Identity", "{4}{U}{U}", None),
        ("Devastating Onslaught", "{4}{R}", Some(2)),
    ] {
        let mut t = TestGame::new(2);
        let clone = cloned_angel(&mut t, P0);
        let toks = cast_spell(&mut t, P0, spell, cost, &[obj(clone)], None, x);
        assert_eq!(toks.len(), x.unwrap_or(1) as usize, "{spell}");
        for c in toks {
            assert_angel(&t, c, true);
        }
    }
}

#[test]
fn ashlings_command_copying_a_clone_of_an_elemental() {
    cr!("707.3", "707.2");
    ruling!("Ashling's Command", "If the copied permanent is copying something else, then the token enters as whatever that creature copied.");
    let mut t = TestGame::new(2);
    let clone = cloned(&mut t, P0, "Air Elemental");
    let toks = cast_spell(
        &mut t,
        P0,
        "Ashling's Command",
        "{3}{U}{R}",
        &[obj(clone), Entity::Player(P0)],
        Some(&[0, 1]),
        None,
    );
    assert_eq!(toks.len(), 1);
    assert_eq!(name_of(&t, toks[0]), "Air Elemental");
    assert_eq!(t.pt(toks[0]), (4, 4));
    assert!(has_kw(&t, toks[0], KeywordKind::Flying));
}

#[test]
fn saw_in_half_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!("Saw in Half", "If the copied creature was copying something else, then the tokens enter the battlefield as whatever that creature copied, with the listed exceptions.");
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P1);
    let before = t.g.battlefield.clone();
    cast_spell(
        &mut t,
        P0,
        "Saw in Half",
        "{2}{B}",
        &[obj(clone)],
        None,
        None,
    );
    let toks = new_tokens(&t, P1, &before);
    assert_eq!(toks.len(), 2);
    for c in toks {
        assert_angel(&t, c, false);
        assert_eq!(t.pt(c), (2, 2));
    }
}

#[test]
fn kindred_charge_copying_a_clone() {
    cr!("707.3");
    ruling!("Kindred Charge", "If the original creature is copying something else, the token enters as whatever that creature copied.");
    supported("Kindred Charge");
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P0);
    let angel = t.named_on_battlefield("Serra Angel");
    assert_eq!(angel.len(), 2);
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Angel");
    let toks = cast_spell(&mut t, P0, "Kindred Charge", "{4}{R}{R}", &[], None, None);
    // One for the Serra Angel and one for the Clone: both Serra Angels.
    assert_eq!(toks.len(), 2);
    for c in toks {
        assert_angel(&t, c, true);
        assert!(has_kw(&t, c, KeywordKind::Haste));
    }
    let _ = clone;
}

// --- Triggered abilities that create a token copy -------------------------------------------

#[test]
fn inalla_copying_a_clone_of_a_wizard() {
    cr!("707.3", "603.2");
    ruling!("Inalla, Archmage Ritualist", "If the copied creature was copying something else, the token enters the battlefield as whatever that permanent was copying.");
    supported("Inalla, Archmage Ritualist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Inalla, Archmage Ritualist");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(sorcerer)]);
    t.enter(P0, "Clone");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, toks[0]), "Prodigal Sorcerer");
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
    // Only the Clone chose something to copy: the token isn't a Clone.
    let chosen = crate::r_p027_common::choices_since(&t, from);
    assert_eq!(chosen.len(), 1);
}

/// P0's `source` is on the battlefield; P0's Clone copying Serra Angel dies (with `yes`
/// answered to the "may" prompts); returns the new tokens.
fn clone_dies_with(source: &str, life_lands: &str) -> (TestGame, Vec<ObjectId>) {
    supported(source);
    let mut t = TestGame::new(2);
    t.battlefield(P0, source);
    let clone = cloned_angel(&mut t, P0);
    lands_for(&mut t, P0, life_lands);
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    kill(&mut t, clone);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    (t, toks)
}

#[test]
fn kinzu_copying_a_clone_that_died() {
    cr!("707.3", "707.9b", "603.10a");
    ruling!("Kinzu of the Bleak Coven", "If the copied creature was copying something else, then the token enters the battlefield as whatever that creature copied, with the exceptions noted above.");
    let (t, toks) = clone_dies_with("Kinzu of the Bleak Coven", "");
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_angel(&t, toks[0], false);
    assert_eq!(t.pt(toks[0]), (1, 1));
    assert!(has_kw(&t, toks[0], KeywordKind::Toxic));
    assert_eq!(t.life(P0), 18);
    assert!(t.in_exile("Clone"));
}

#[test]
fn brenard_copying_a_clone_that_died() {
    cr!("707.3", "707.9b", "603.10a");
    ruling!("Brenard, Ginger Sculptor", "If the copied creature was copying something else, then the token enters the battlefield as whatever that creature copied, with the stated exceptions.");
    let (t, toks) = clone_dies_with("Brenard, Ginger Sculptor", "");
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    let c = toks[0];
    assert_angel(&t, c, false);
    // A 1/1 Food Golem artifact creature, +2/+2 and trample from Brenard.
    assert!(
        has_subtype(&t, c, "Food") && has_subtype(&t, c, "Golem") && has_subtype(&t, c, "Angel")
    );
    assert!(is_artifact(&t, c) && is_creature(&t, c));
    assert_eq!(t.pt(c), (3, 3));
    assert!(has_kw(&t, c, KeywordKind::Trample));
    assert_eq!(activated_with(&t, c, "You gain 3 life"), 1);
}

fn activated_with(t: &TestGame, id: ObjectId, s: &str) -> usize {
    crate::r_p025_common::activated_with(t, id, s)
}

#[test]
fn ashling_the_limitless_copying_a_sacrificed_clone_of_an_elemental() {
    cr!("707.3", "603.10a");
    ruling!("Ashling, the Limitless", "If the copied permanent was copying something else, then the token enters as whatever that permanent copied.");
    supported("Ashling, the Limitless");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashling, the Limitless");
    let clone = cloned(&mut t, P0, "Air Elemental");
    let before = t.g.battlefield.clone();
    let clone = t.g.current(clone);
    t.g.sacrifice(clone, P0);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, toks[0]), "Air Elemental");
    assert!(has_kw(&t, toks[0], KeywordKind::Flying) && has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn tawnos_copying_an_artifact_token_thats_copying_something_else() {
    cr!("707.3", "707.2", "111.3");
    ruling!("Tawnos, Solemn Survivor", "If the original token is copying something else, the token you create will use the copiable values of the original token. In most cases, it will be a copy of whatever the original token is copying.");
    supported("Tawnos, Solemn Survivor");
    let mut t = TestGame::new(2);
    let tawnos = t.battlefield(P0, "Tawnos, Solemn Survivor");
    // A 1/1 Thopter artifact creature token that became a copy of Ornithopter (a 0/2
    // artifact creature) until end of turn with Cytoshape.
    let thopter = token_of(
        &mut t,
        P0,
        TokenSpec {
            card_types: vec![CardType::Artifact, CardType::Creature],
            colors: ColorSet::NONE,
            ..creature_spec("Thopter", Color::Blue, 1)
        },
    );
    let orni = t.battlefield(P1, "Ornithopter");
    t.answer_choose(P0, &[obj(orni)]);
    cast_spell(
        &mut t,
        P0,
        "Cytoshape",
        "{1}{G}{U}",
        &[obj(thopter)],
        None,
        None,
    );
    assert_eq!(name_of(&t, thopter), "Ornithopter");
    t.lands(P0, "Wastes", 2);
    let before = t.g.battlefield.clone();
    t.activate(P0, tawnos, 0, &[obj(thopter)])
        .expect("activation");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(name_of(&t, toks[0]), "Ornithopter");
    assert_eq!(t.pt(toks[0]), (0, 2));
    // It stays an Ornithopter after the turn (it's a copy of one, not affected by Cytoshape).
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, toks[0]), "Ornithopter");
    assert_eq!(name_of(&t, thopter), "Thopter");
}

// --- Permanents that become copies ---------------------------------------------------------

#[test]
fn becoming_a_copy_of_a_clone_becomes_what_it_copies() {
    cr!("707.3", "707.2");
    ruling!("Cytoshape", "If the creature copies a creature that’s copying a creature, it will become whatever the chosen creature is copying.");
    ruling!("Polymorphous Rush", "If the creatures copy a creature that's copying a creature, they will become whatever the chosen creature is copying.");
    ruling!("True Polymorph", "If the first target permanent copies a permanent that's copying something else, it will become whatever the second target is copying.");
    // Cytoshape and Polymorphous Rush choose the creature to copy on resolution.
    for spell in ["Cytoshape", "Polymorphous Rush"] {
        let mut t = TestGame::new(2);
        let clone = cloned_angel(&mut t, P1);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_choose(P0, &[obj(clone)]);
        let cost = if spell == "Cytoshape" {
            "{1}{G}{U}"
        } else {
            "{2}{U}"
        };
        cast_spell(&mut t, P0, spell, cost, &[obj(bears)], None, None);
        assert_angel(&t, bears, true);
    }
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_spell(
        &mut t,
        P0,
        "True Polymorph",
        "{4}{U}{U}",
        &[obj(bears), obj(clone)],
        None,
        None,
    );
    assert_angel(&t, bears, true);
}

#[test]
fn shapesharer_copying_a_clone() {
    cr!("707.3");
    ruling!("Shapesharer", "If the targeted Shapeshifter copies a creature that’s copying a creature, it will become whatever the chosen creature is copying.");
    supported("Shapesharer");
    let mut t = TestGame::new(2);
    let sharer = t.battlefield(P0, "Shapesharer");
    let clone = cloned_angel(&mut t, P1);
    lands_for(&mut t, P0, "{2}{U}");
    t.activate(P0, sharer, 0, &[obj(sharer), obj(clone)])
        .expect("activation");
    t.resolve_all();
    assert_angel(&t, sharer, true);
}

#[test]
fn saheeli_sublime_artificer_copying_a_clone() {
    cr!("707.3", "707.9b");
    ruling!("Saheeli, Sublime Artificer", "If the target artifact copies a permanent that's copying something else, it will become whatever the target is copying.");
    supported("Saheeli, Sublime Artificer");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let saheeli = t.battlefield(P0, "Saheeli, Sublime Artificer");
    let mirror = t.battlefield(P0, "Mirage Mirror");
    let clone = cloned_angel(&mut t, P0);
    t.answer_targets(P0, &[obj(mirror)]);
    t.answer_targets(P0, &[obj(clone)]);
    activate_containing(&mut t, P0, saheeli, "becomes a copy").expect("−2");
    t.resolve_all();
    assert_angel(&t, mirror, true);
    assert!(is_artifact(&t, mirror));
}

#[test]
fn mass_copy_effects_of_a_modified_copy() {
    cr!("707.3", "707.9b", "613.1a");
    ruling!("Nanogene Conversion", "If the targeted creature is itself copying a creature, each other creature will become whatever it's copying, as modified by that copy effect.");
    ruling!("Mirrorweave", "If the targeted creature is itself copying a creature, each other creature will become whatever it’s copying, as modified by that copy effect. For example, if you target a Cemetery Puca that’s copying a Grizzly Bears (a 2/2 creature with no abilities), each other creature will become a Grizzly Bears with the Cemetery Puca ability.");
    for (spell, cost) in [
        ("Nanogene Conversion", "{3}{U}"),
        ("Mirrorweave", "{2}{W}{U}"),
    ] {
        let mut t = TestGame::new(2);
        let mimic = mimic_bears(&mut t, P0);
        let giant = t.battlefield(P1, "Hill Giant");
        cast_spell(&mut t, P0, spell, cost, &[obj(mimic)], None, None);
        assert_modified_bears(&t, giant);
    }
    // The ruling's own example: a Cemetery Puca copying a Grizzly Bears, "except it has
    // this ability".
    supported("Cemetery Puca");
    let mut t = TestGame::new(2);
    let puca = t.battlefield(P1, "Cemetery Puca");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Wastes", 1);
    t.answer_yes(P1, true);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(name_of(&t, puca), "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_spell(
        &mut t,
        P0,
        "Mirrorweave",
        "{2}{W}{U}",
        &[obj(puca)],
        None,
        None,
    );
    assert_eq!(name_of(&t, giant), "Grizzly Bears");
    assert_eq!(t.pt(giant), (2, 2));
    assert_eq!(abilities_with(&t, giant, "Whenever a creature dies"), 1);
}

#[test]
fn identity_thief_copying_a_modified_copy() {
    cr!("707.3", "707.9b");
    ruling!("Identity Thief", "If the targeted creature is itself copying a permanent, Identity Thief will become whatever it's copying, as modified by that copy effect.");
    supported("Identity Thief");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Identity Thief");
    let mimic = mimic_bears(&mut t, P1);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(mimic)]);
    t.attack(&[(thief, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(mimic));
    assert_modified_bears(&t, thief);
}
