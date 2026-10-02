//! Rulings batch P030 — "create a token that's a copy of this creature": if the permanent
//! has left the battlefield, the token uses its copiable values as it last existed there
//! (CR 707.2, 608.2h, 113.7a); the token copies only the copiable values — not counters,
//! tapped status or non-copy effects — but copy effects that applied to the permanent are
//! taken into account (CR 707.2, 707.3, 707.9a).

use crate::r_p030_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::{activate_containing, damage};
use crate::r_s11_common::empty_library;
use crate::r_s25_common::cast_new;
use crate::r_s26_common::{dress_up, fresh};
use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Asserts `p` has exactly one token named `name`, and returns it.
fn one_token(t: &TestGame, p: PlayerId, name: &str) -> ObjectId {
    let toks = tokens_named(t, p, name);
    assert_eq!(toks.len(), 1, "tokens named {name}: {toks:?}");
    toks[0]
}

/// Puts the real card `name` onto the battlefield as `p`'s commander.
fn commander(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.dirty = true;
    t.settle();
    id
}

/// Attacks P1 with `c`, unblocked, and advances to the combat damage step (combat damage
/// triggers on the stack).
fn hit(t: &mut TestGame, c: ObjectId) {
    attack_with(t, &[(c, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
}

/// Whether the permanent has an ability whose text contains `text`.
fn has_ability(t: &TestGame, id: ObjectId, text: &str) -> bool {
    t.obj_now(id).chars.abilities.iter().any(|a| a.text.contains(text))
}

/// Advances to the cleanup of this turn and into the next turn's upkeep (P1's).
fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
}

// --- The token uses the permanent's last known copiable values -----------------------------

#[test]
fn biowaste_blob_s_token_after_it_left() {
    cr!("707.2", "608.2h", "603.4");
    ruling!(
        "Biowaste Blob",
        "If Biowaste Blob leaves the battlefield before its triggered ability resolves, the token will still enter the battlefield as a copy of Biowaste Blob"
    );
    supported("Biowaste Blob");
    // "Oozes you control get +1/+1. At the beginning of your upkeep, if you control a
    // commander, create a token that's a copy of this creature."
    let mut t = TestGame::new(2);
    let blob = t.battlefield(P0, "Biowaste Blob");
    commander(&mut t, P0, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, blob);
    t.resolve_all();
    let tok = one_token(&t, P0, "Biowaste Blob");
    assert_eq!(t.pt(tok), (1, 1));
}

#[test]
fn biowaste_blob_affects_itself() {
    cr!("611.3a", "613.4c");
    ruling!("Biowaste Blob", "Biowaste Blob's first ability affects itself.");
    let mut t = TestGame::new(2);
    let blob = t.battlefield(P0, "Biowaste Blob");
    assert_eq!(t.pt(blob), (1, 1));
}

#[test]
fn biowaste_blob_needs_a_commander_as_upkeep_begins_and_on_resolution() {
    cr!("603.4", "903.3");
    ruling!(
        "Biowaste Blob",
        "If you don't control a commander when the trigger resolves, it won't create a token. These don't have to be the same commander at both times, however, and it doesn't have to be your commander."
    );
    // No commander as the upkeep begins: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Biowaste Blob");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A commander (P1's) as the upkeep begins, gone on resolution: no token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Biowaste Blob");
    let c = commander(&mut t, P1, "Grizzly Bears");
    crate::r_s06_common::give_control(&mut t, c, P0);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, c);
    t.resolve_all();
    assert!(tokens_named(&t, P0, "Biowaste Blob").is_empty());
    // A different commander on resolution: a token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Biowaste Blob");
    let c = commander(&mut t, P0, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    kill(&mut t, c);
    commander(&mut t, P0, "Hill Giant");
    t.resolve_all();
    one_token(&t, P0, "Biowaste Blob");
}

#[test]
fn compy_swarm_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Compy Swarm",
        "If Compy Swarm leaves the battlefield before its triggered ability resolves, the token will still enter the battlefield as a copy of Compy Swarm"
    );
    supported("Compy Swarm");
    // "At the beginning of your end step, if a creature died this turn, create a tapped
    // token that's a copy of this creature."
    let mut t = TestGame::new(2);
    let compy = t.battlefield(P0, "Compy Swarm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, bears);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, compy);
    t.resolve_all();
    let tok = one_token(&t, P0, "Compy Swarm");
    assert!(t.obj_now(tok).tapped);
}

#[test]
fn copy_catchers_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Copy Catchers",
        "If Copy Catchers leaves the battlefield before its triggered ability resolves, the token will still enter the battlefield as a copy of Copy Catchers"
    );
    supported("Copy Catchers");
    // "Whenever you surveil, you may pay {1}{U}. If you do, create a token that's a copy of
    // this creature."
    let mut t = TestGame::new(2);
    let cc = t.battlefield(P0, "Copy Catchers");
    surveil(&mut t, P0, 1);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, cc);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    one_token(&t, P0, "Copy Catchers");
}

#[test]
fn copy_catchers_triggers_after_surveilling_an_empty_library() {
    cr!("701.25d");
    ruling!(
        "Copy Catchers",
        "An ability that triggers “whenever you surveil” triggers after you’re done surveilling, even if you have fewer cards in your library than the number of cards you’re instructed to surveil. It triggers even if you have no cards in your library."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Copy Catchers");
    empty_library(&mut t, P0);
    surveil(&mut t, P0, 2);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn homunculus_horde_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Homunculus Horde",
        "If Homunculus Horde leaves the battlefield before its triggered ability resolves, the token will still enter as a copy of Homunculus Horde"
    );
    supported("Homunculus Horde");
    // "Whenever you draw your second card each turn, create a token that's a copy of this
    // creature."
    let mut t = TestGame::new(2);
    let horde = t.battlefield(P0, "Homunculus Horde");
    draw(&mut t, P0, 2);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, horde);
    t.resolve_all();
    one_token(&t, P0, "Homunculus Horde");
}

#[test]
fn homunculus_horde_needn_t_be_there_for_the_first_draw() {
    cr!("603.2", "603.10");
    ruling!(
        "Homunculus Horde",
        "Homunculus Horde doesn't need to have been under your control when the first card is drawn for its ability to trigger."
    );
    let mut t = TestGame::new(2);
    draw(&mut t, P0, 1);
    t.battlefield(P0, "Homunculus Horde");
    draw(&mut t, P0, 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    one_token(&t, P0, "Homunculus Horde");
}

#[test]
fn improvised_arsenal_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Improvised Arsenal",
        "If Improvised Arsenal leaves the battlefield before its second ability resolves, the token will still enter as a copy of Improvised Arsenal"
    );
    supported("Improvised Arsenal");
    // "{4}{R}: Create a token that's a copy of this Equipment."
    let mut t = TestGame::new(2);
    let arsenal = t.battlefield(P0, "Improvised Arsenal");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    activate_containing(&mut t, P0, arsenal, "Create a token").unwrap();
    kill(&mut t, arsenal);
    t.resolve_all();
    let tok = one_token(&t, P0, "Improvised Arsenal");
    assert!(t.obj_now(tok).chars.subtypes.iter().any(|s| s == "Equipment"));
}

#[test]
fn mirror_sigil_sergeant_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Mirror-Sigil Sergeant",
        "If Mirror-Sigil Sergeant has left the battlefield by the time its triggered ability resolves, you’ll still put a token onto the battlefield. That token has the copiable values of the characteristics of Mirror-Sigil Sergeant as it last existed on the battlefield."
    );
    supported("Mirror-Sigil Sergeant");
    // "At the beginning of your upkeep, if you control a blue permanent, you may create a
    // token that's a copy of this creature."
    let mut t = TestGame::new(2);
    let sgt = t.battlefield(P0, "Mirror-Sigil Sergeant");
    t.battlefield(P0, "Wind Drake");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, sgt);
    t.answer_yes(P0, true);
    t.resolve_all();
    let tok = one_token(&t, P0, "Mirror-Sigil Sergeant");
    assert_eq!(t.pt(tok), (4, 4));
}

#[test]
fn mishra_s_self_replicator_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Mishra's Self-Replicator",
        "If Mishra’s Self-Replicator leaves the battlefield before its triggered ability resolves, the token will still enter the battlefield as a copy of Mishra’s Self-Replicator"
    );
    supported("Mishra's Self-Replicator");
    // "Whenever you cast a historic spell, you may pay {1}. If you do, create a token
    // that's a copy of this creature."
    let mut t = TestGame::new(2);
    let msr = t.battlefield(P0, "Mishra's Self-Replicator");
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    kill(&mut t, msr);
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    one_token(&t, P0, "Mishra's Self-Replicator");
}

#[test]
fn mist_syndicate_naga_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Mist-Syndicate Naga",
        "If Mist-Syndicate Naga leaves the battlefield before its triggered ability resolves, the token will still enter the battlefield as a copy of Mist-Syndicate Naga"
    );
    supported("Mist-Syndicate Naga");
    let mut t = TestGame::new(2);
    let naga = t.battlefield(P0, "Mist-Syndicate Naga");
    hit(&mut t, naga);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, naga);
    t.resolve_all();
    one_token(&t, P0, "Mist-Syndicate Naga");
}

#[test]
fn pack_rat_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Pack Rat",
        "If Pack Rat leaves the battlefield before its activated ability resolves, the token will still enter the battlefield as a copy of Pack Rat"
    );
    supported("Pack Rat");
    // "{2}{B}, Discard a card: Create a token that's a copy of this creature."
    let mut t = TestGame::new(2);
    let rat = t.battlefield(P0, "Pack Rat");
    let card = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(card)]);
    activate_containing(&mut t, P0, rat, "Create a token").unwrap();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    kill(&mut t, rat);
    t.resolve_all();
    let tok = one_token(&t, P0, "Pack Rat");
    assert_eq!(t.pt(tok), (1, 1));
}

#[test]
fn polyraptor_s_token_after_it_died_of_the_damage() {
    cr!("707.2", "608.2h", "704.5g");
    ruling!(
        "Polyraptor",
        "If Polyraptor leaves the battlefield before its triggered ability resolves, most likely because it was dealt lethal damage, the token will still enter the battlefield as a copy of Polyraptor"
    );
    supported("Polyraptor");
    // "Enrage — Whenever this creature is dealt damage, create a token that's a copy of
    // this creature."
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Polyraptor");
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, 5, raptor);
    assert!(t.in_graveyard(P0, "Polyraptor"));
    t.resolve_all();
    let tok = one_token(&t, P0, "Polyraptor");
    assert_eq!(t.pt(tok), (5, 5));
}

#[test]
fn sorcerer_s_broom_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Sorcerer's Broom",
        "If Sorcerer's Broom leaves the battlefield before its triggered ability resolves, the token will still enter the battlefield as a copy of Sorcerer's Broom"
    );
    supported("Sorcerer's Broom");
    // "Whenever you sacrifice another permanent, you may pay {3}. If you do, create a
    // token that's a copy of this creature."
    let mut t = TestGame::new(2);
    let broom = t.battlefield(P0, "Sorcerer's Broom");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.sacrifice(bears, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, broom);
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.resolve_all();
    one_token(&t, P0, "Sorcerer's Broom");
}

#[test]
fn stormsplitter_s_token_after_it_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Stormsplitter",
        "If Stormsplitter splits… er, leaves the battlefield before its triggered ability resolves, the token will still enter as a copy of Stormsplitter"
    );
    supported("Stormsplitter");
    let mut t = TestGame::new(2);
    let ss = t.battlefield(P0, "Stormsplitter");
    cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    kill(&mut t, ss);
    t.resolve();
    one_token(&t, P0, "Stormsplitter");
}

// --- Chronozoa --------------------------------------------------------------------------

#[test]
fn chronozoa_s_tokens_enter_with_time_counters() {
    cr!("707.2", "702.63a", "614.1c");
    ruling!(
        "Chronozoa",
        "Chronozoa's last ability creates two Chronozoa tokens that each enter with three time counters and have all of Chronozoa's abilities."
    );
    supported("Chronozoa");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Chronozoa");
    t.g.objects[c.0 as usize].counters.remove(counters::TIME);
    kill(&mut t, c);
    t.resolve_all();
    let toks = tokens_named(&t, P0, "Chronozoa");
    assert_eq!(toks.len(), 2);
    for tok in toks {
        assert_eq!(t.counters(tok, counters::TIME), 3);
        assert!(has_ability(&t, tok, "if it had no time counters"));
    }
}

#[test]
fn chronozoa_s_trigger_follows_copy_effects() {
    cr!("707.2", "603.10a", "707.4");
    ruling!(
        "Chronozoa",
        "If Chronozoa becomes a copy of another creature and is put into the graveyard from the battlefield, no copies will be created. If another creature becomes a copy of Chronozoa and is put into a graveyard from the battlefield (with no time counters on it), two copies of Chronozoa will be created."
    );
    supported("Cytoshape");
    // Cytoshape: "Choose a nonlegendary creature on the battlefield. Target creature
    // becomes a copy of that creature until end of turn."
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Chronozoa");
    t.g.objects[c.0 as usize].counters.remove(counters::TIME);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    cast_new(&mut t, P0, "Cytoshape", &[obj(c)]);
    t.resolve_all();
    assert_eq!(t.obj_now(c).chars.name, "Hill Giant");
    kill(&mut t, c);
    t.resolve_all();
    assert!(tokens_named(&t, P0, "Chronozoa").is_empty());
    // Grizzly Bears becomes a copy of a Chronozoa, then dies.
    let c2 = t.battlefield(P0, "Chronozoa");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(c2)]);
    cast_new(&mut t, P0, "Cytoshape", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Chronozoa");
    assert_eq!(t.counters(bears, counters::TIME), 0);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Chronozoa").len(), 2);
}

// --- What the token copies ----------------------------------------------------------------

/// Asserts the token copies only the printed values: named `name`, `pt`, untapped, no
/// counters, not green-blue.
fn assert_printed(t: &TestGame, tok: ObjectId, name: &str, pt: (i32, i32)) {
    assert_eq!(t.obj_now(tok).chars.name, name);
    assert_eq!(t.pt(tok), pt);
    assert!(fresh(t, tok));
    assert!(!t.obj_now(tok).chars.colors.contains(Color::Blue) || name == "Copy Catchers");
}

/// Dresses up the permanent (two +1/+1 counters, +3/+3, green and blue) but leaves it
/// untapped.
fn dress(t: &mut TestGame, id: ObjectId) {
    dress_up(t, id);
    t.g.untap(t.g.current(id));
    t.g.recompute();
}

#[test]
fn giant_adephage_s_token_copies_only_printed_values() {
    cr!("707.2");
    ruling!(
        "Giant Adephage",
        "It won't copy counters on the Giant Adephage, nor will it copy other effects that have changed Giant Adephage's power, toughness, types, color, or so on."
    );
    supported("Giant Adephage");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Giant Adephage");
    dress(&mut t, a);
    assert_eq!(t.pt(a), (12, 12));
    hit(&mut t, a);
    t.resolve_all();
    let tok = one_token(&t, P0, "Giant Adephage");
    assert_printed(&t, tok, "Giant Adephage", (7, 7));
}

#[test]
fn spawnwrithe_s_token_copies_only_printed_values() {
    cr!("707.2");
    ruling!(
        "Spawnwrithe",
        "It won’t copy counters on the Spawnwrithe, nor will it copy other effects that have changed Spawnwrithe’s power, toughness, types, color, or so on."
    );
    supported("Spawnwrithe");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Spawnwrithe");
    dress(&mut t, s);
    hit(&mut t, s);
    t.resolve_all();
    let tok = one_token(&t, P0, "Spawnwrithe");
    assert_printed(&t, tok, "Spawnwrithe", (2, 2));
}

#[test]
fn mirror_sigil_sergeant_s_token_copies_only_printed_values() {
    cr!("707.2");
    ruling!(
        "Mirror-Sigil Sergeant",
        "It won’t copy counters on the Mirror-Sigil Sergeant, nor will it copy other effects that have changed Mirror-Sigil Sergeant’s power, toughness, types, color, or so on."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Mirror-Sigil Sergeant");
    t.battlefield(P0, "Wind Drake");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    dress(&mut t, s);
    t.answer_yes(P0, true);
    t.resolve_all();
    let tok = one_token(&t, P0, "Mirror-Sigil Sergeant");
    assert_printed(&t, tok, "Mirror-Sigil Sergeant", (4, 4));
}

#[test]
fn myr_propagator_s_token_copies_only_printed_values() {
    cr!("707.2");
    ruling!(
        "Myr Propagator",
        "It won’t copy counters on the Myr Propagator, nor will it copy other effects that have changed Myr Propagator’s power, toughness, types, color, or so on."
    );
    supported("Myr Propagator");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Myr Propagator");
    dress(&mut t, m);
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, m, "Create a token").unwrap();
    t.resolve_all();
    let tok = one_token(&t, P0, "Myr Propagator");
    assert_printed(&t, tok, "Myr Propagator", (1, 1));
}

#[test]
fn myr_propagator_s_token_copies_what_it_s_copying() {
    cr!("707.2", "707.3", "707.9a");
    ruling!(
        "Myr Propagator",
        "the token will be a copy of whatever creature the Myr Propagator is currently a copy of. After the turn ends, the Cytoshaped Myr Propagator reverts back to what it was, but the token will stay as it is."
    );
    supported("Cemetery Puca");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Myr Propagator");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, m, "Create a token").unwrap();
    // In response, Cytoshape turns Myr Propagator into a Hill Giant.
    t.answer_choose(P0, &[obj(giant)]);
    cast_new(&mut t, P0, "Cytoshape", &[obj(m)]);
    t.resolve_all();
    let tok = one_token(&t, P0, "Hill Giant");
    assert_eq!(t.pt(tok), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.obj_now(m).chars.name, "Myr Propagator");
    assert_eq!(t.obj_now(tok).chars.name, "Hill Giant");
    // Cemetery Puca becomes a copy of a Myr Propagator that dies, keeping its ability; its
    // token is a Myr Propagator with that ability.
    let mut t = TestGame::new(2);
    let puca = t.battlefield(P0, "Cemetery Puca");
    let m = t.battlefield(P0, "Myr Propagator");
    kill(&mut t, m);
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.obj_now(puca).chars.name, "Myr Propagator");
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, puca, "Create a token").unwrap();
    t.resolve_all();
    let tok = one_token(&t, P0, "Myr Propagator");
    assert!(has_ability(&t, tok, "becomes a copy of that creature"));
}

#[test]
fn mirror_sigil_sergeant_s_token_copies_what_it_s_copying() {
    cr!("707.2", "707.3", "707.9a");
    ruling!(
        "Mirror-Sigil Sergeant",
        "the token will be a copy of whatever creature the Mirror-Sigil Sergeant is currently a copy of. After the turn ends, the Mirrorweaved Mirror-Sigil Sergeant reverts back to what it was, but the token will stay as it is."
    );
    supported("Mirrorweave");
    // Mirrorweave: "Each other creature becomes a copy of target nonlegendary creature
    // until end of turn."
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Mirror-Sigil Sergeant");
    // A blue permanent that Mirrorweave doesn't change.
    t.battlefield(P0, "Propaganda");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    cast_new(&mut t, P0, "Mirrorweave", &[obj(giant)]);
    t.resolve();
    assert_eq!(t.obj_now(s).chars.name, "Hill Giant");
    t.answer_yes(P0, true);
    t.resolve_all();
    let tok = one_token(&t, P0, "Hill Giant");
    next_turn(&mut t);
    assert_eq!(t.obj_now(s).chars.name, "Mirror-Sigil Sergeant");
    assert_eq!(t.obj_now(tok).chars.name, "Hill Giant");
    // Cemetery Puca copying a Mirror-Sigil Sergeant makes a Sergeant with Puca's ability.
    let mut t = TestGame::new(2);
    let puca = t.battlefield(P0, "Cemetery Puca");
    t.battlefield(P0, "Wind Drake");
    let s = t.battlefield(P0, "Mirror-Sigil Sergeant");
    kill(&mut t, s);
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.obj_now(puca).chars.name, "Mirror-Sigil Sergeant");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.answer_yes(P0, true);
    t.resolve_all();
    let tok = one_token(&t, P0, "Mirror-Sigil Sergeant");
    assert!(has_ability(&t, tok, "becomes a copy of that creature"));
}

#[test]
fn spawnwrithe_s_token_copies_what_it_s_copying() {
    cr!("707.2", "707.3");
    ruling!(
        "Spawnwrithe",
        "If Spawnwrithe’s ability triggers, then Spawnwrithe becomes a copy of another creature before its ability resolves (due to Mirrorweave, perhaps), the token will be a copy of whatever creature the Spawnwrithe is currently a copy of."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Spawnwrithe");
    let giant = t.battlefield(P1, "Hill Giant");
    hit(&mut t, s);
    assert_eq!(t.stack_len(), 1);
    cast_new(&mut t, P0, "Mirrorweave", &[obj(giant)]);
    t.resolve();
    t.resolve_all();
    let tok = one_token(&t, P0, "Hill Giant");
    next_turn(&mut t);
    assert_eq!(t.obj_now(s).chars.name, "Spawnwrithe");
    assert_eq!(t.obj_now(tok).chars.name, "Hill Giant");
}

#[test]
fn spawnwrithe_copied_by_cemetery_puca_makes_spawnwrithes_with_puca_s_ability() {
    cr!("707.2", "707.9a");
    ruling!(
        "Spawnwrithe",
        "A token created by a Cemetery Puca that’s copying a Spawnwrithe will be a Spawnwrithe with the Cemetery Puca ability."
    );
    let mut t = TestGame::new(2);
    let puca = t.battlefield(P0, "Cemetery Puca");
    let s = t.battlefield(P0, "Spawnwrithe");
    kill(&mut t, s);
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.obj_now(puca).chars.name, "Spawnwrithe");
    hit(&mut t, puca);
    t.resolve_all();
    let tok = one_token(&t, P0, "Spawnwrithe");
    assert!(has_ability(&t, tok, "becomes a copy of that creature"));
    assert!(t
        .obj_now(tok)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Triggered(_)) && a.text.contains("combat damage")));
}

#[test]
fn sprouting_phytohydra_s_token_is_created_even_if_the_damage_destroys_it() {
    cr!("603.10a", "707.2");
    ruling!(
        "Sprouting Phytohydra",
        "A token copy of Sprouting Phytohydra is created even if the damage dealt to Sprouting Phytohydra destroys it."
    );
    supported("Sprouting Phytohydra");
    let mut t = TestGame::new(2);
    let ph = t.battlefield(P0, "Sprouting Phytohydra");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    damage(&mut t, giant, 3, ph);
    assert!(t.in_graveyard(P0, "Sprouting Phytohydra"));
    t.resolve_all();
    one_token(&t, P0, "Sprouting Phytohydra");
}

#[test]
fn a_cytoshaped_phytohydra_s_token_stays_a_phytohydra() {
    cr!("707.2", "707.3", "611.2a");
    ruling!(
        "Sprouting Phytohydra",
        "If Cytoshape turns a creature into a copy of Sprouting Phytohydra, and that creature is dealt damage, the token that is put onto the battlefield is simply a copy of Sprouting Phytohydra."
    );
    let mut t = TestGame::new(2);
    let ph = t.battlefield(P0, "Sprouting Phytohydra");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(ph)]);
    cast_new(&mut t, P0, "Cytoshape", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Sprouting Phytohydra");
    t.answer_yes(P0, true);
    damage(&mut t, giant, 1, bears);
    t.resolve_all();
    let tok = one_token(&t, P0, "Sprouting Phytohydra");
    next_turn(&mut t);
    assert_eq!(t.obj_now(bears).chars.name, "Grizzly Bears");
    assert_eq!(t.obj_now(tok).chars.name, "Sprouting Phytohydra");
}
