//! Spell copies (`src/oracle/patterns/copy_spell_grammar.rs`): "copy it [N times] [, except
//! the copy ...] [if ...]", "you may copy ~ and may choose a new target for the copy",
//! another player copying the spell, and "You may choose new targets for that copy".

use mtg_engine::object::ObjKind;
use mtg_engine::types::CardType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn spell_copy_wordings_compile() {
    assert_compiles(&[
        // "When you cast this spell, copy it [N times / for each ... / if ...]."
        "Brass Knuckles",
        "Photon Blast Barrage",
        "Storm of Forms",
        "Mentor's Guidance",
        "Lumaret's Favor",
        "Bygone Marvels",
        "Malicious Affliction",
        // "Whenever you cast ..., [you may] copy it, except the copy ..."
        "The Sixth Doctor",
        "Storm of Saruman",
        "Iron Man, Bleeding Edge",
        "Tawnos, the Toymaker",
        "Donal, Herald of Wings",
        "Tomb of Horrors Adventurer",
        // Target spells, counts, exceptions, "instead".
        "Myojin of Cryptic Dreams",
        "Double Major",
        "Increasing Vengeance",
        "Narset's Reversal",
        "Mischievous Quanar",
        "Mirror Sheen",
        // The spell's own text; other players copying it.
        "Sevinne's Reclamation",
        "Chain of Acid",
        "Chain of Smog",
    ]);
}

fn tokens_named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|o| t.obj_now(*o).kind == ObjKind::Token)
        .collect()
}

#[test]
fn brass_knuckles_cast_trigger_copies_the_permanent_spell_into_a_token() {
    cr!("707.10", "707.10f", "608.3f");
    ruling!(
        "Brass Knuckles",
        "A copy of a permanent spell enters the battlefield as a token."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    let k = t.hand(P0, "Brass Knuckles");
    t.cast(P0, k).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Brass Knuckles").len(), 2);
    assert_eq!(tokens_named(&t, "Brass Knuckles").len(), 1);
}

#[test]
fn the_sixth_doctor_copies_a_historic_spell_as_a_nonlegendary_token() {
    cr!("707.9b", "707.10", "608.3f");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Sixth Doctor");
    t.lands(P0, "Plains", 1);
    let isamaru = t.hand(P0, "Isamaru, Hound of Konda");
    t.cast(P0, isamaru).go();
    t.resolve_all();
    let all = t.named_on_battlefield("Isamaru, Hound of Konda");
    assert_eq!(all.len(), 2, "{}", t.dump_log());
    let token = tokens_named(&t, "Isamaru, Hound of Konda");
    assert_eq!(token.len(), 1);
    assert!(!t.obj_now(token[0]).chars.is_legendary());
    let card = all.iter().find(|o| !token.contains(o)).copied().unwrap();
    assert!(t.obj_now(card).chars.is_legendary());
}

#[test]
fn tawnos_the_toymaker_copy_is_an_artifact_in_addition_to_its_other_types() {
    cr!("707.9b", "707.10", "608.3f");
    ruling!(
        "Tawnos, the Toymaker",
        "A resolving copy of a permanent spell becomes a token"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tawnos, the Toymaker");
    t.lands(P0, "Forest", 4);
    let baloth = t.hand(P0, "Ravenous Baloth");
    t.answer_yes(P0, true);
    t.cast(P0, baloth).go();
    t.resolve_all();
    let token = tokens_named(&t, "Ravenous Baloth");
    assert_eq!(token.len(), 1, "{}", t.dump_log());
    let c = &t.obj_now(token[0]).chars;
    assert!(c.is(CardType::Artifact) && c.is(CardType::Creature));
    assert!(c.has_subtype("Beast"));
}

#[test]
fn double_major_copy_of_a_legendary_creature_spell_isnt_legendary() {
    cr!("707.9b", "707.10", "608.3f");
    ruling!(
        "Double Major",
        "If a creature spell is copied, it’s put onto the battlefield as a token"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    let isamaru = t.hand(P0, "Isamaru, Hound of Konda");
    let spell = t.cast(P0, isamaru).go();
    let dm = t.hand(P0, "Double Major");
    t.cast(P0, dm).target(spell).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 2);
    let token = tokens_named(&t, "Isamaru, Hound of Konda");
    assert_eq!(token.len(), 1);
    assert!(!t.obj_now(token[0]).chars.is_legendary());
}

#[test]
fn chain_of_smog_lets_the_discarding_player_copy_it_and_retarget() {
    cr!("707.10", "707.10c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
        t.hand(P1, "Grizzly Bears");
    }
    let chain = t.hand(P0, "Chain of Smog");
    // P1 copies it and targets P0 with the copy; P0 then declines to copy the copy.
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.answer_yes(P0, false);
    t.cast(P0, chain).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 1, "{}", t.dump_log());
}

#[test]
fn narsets_reversal_copies_the_spell_then_returns_it_to_its_owners_hand() {
    cr!("707.10", "707.10c");
    ruling!(
        "Narset's Reversal",
        "If you copy a spell, you control the copy."
    );
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    t.lands(P0, "Island", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    let nr = t.hand(P0, "Narset's Reversal");
    t.cast(P0, nr).target(spell).go();
    // New target for the copy: P1.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Lightning Bolt"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn mentors_guidance_copies_itself_only_if_you_control_a_listed_permanent() {
    cr!("707.10");
    ruling!(
        "Mentor's Guidance",
        "As that ability tries to resolve, it will check"
    );
    for cleric in [false, true] {
        let mut t = TestGame::new(2);
        if cleric {
            t.battlefield(P0, "Soul Warden");
        }
        t.lands(P0, "Island", 3);
        let mg = t.hand(P0, "Mentor's Guidance");
        let before = t.hand_size(P0);
        t.cast(P0, mg).go();
        t.resolve_all();
        let drawn = t.hand_size(P0) + 1 - before;
        assert_eq!(drawn, if cleric { 2 } else { 1 }, "{}", t.dump_log());
    }
}
