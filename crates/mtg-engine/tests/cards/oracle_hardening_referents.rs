//! Oracle hardening: what "it", "that creature" and "that player" refer to (patterns in
//! `src/oracle/patterns/oracle_hardening_referents.rs`).
//!
//! A spell's pronouns never mean the spell itself and "that player" never means "you":
//! without a tracked antecedent the text is unsupported instead of compiling into an
//! effect on the wrong object. The antecedents the compiler does track are checked in
//! play: a player target, the spell as the subject of an earlier sentence, cards found by
//! a search, tokens just created, the permanent an Aura enchants, the sacrificed creature,
//! the amassed Army, "its owner"/"its controller", and a trigger's player.

use mtg_engine::card::Layout;
use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn assert_unsupported(name: &str, text: &str) {
    let u = card(name).unsupported_text().join(" | ");
    assert!(
        u.contains(text),
        "{name} should leave {text:?} unsupported, has: {u:?}"
    );
}

/// Compiles oracle text for a made-up card; `Err` holds the text not understood.
fn compile(type_line: &str, text: &str) -> Result<(), Vec<String>> {
    let tl = TypeLine::parse(type_line);
    let ctx = CompileContext {
        card_name: "Test Card",
        full_name: "Test Card",
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let c = oracle::compile(text, &ctx);
    if c.unsupported.is_empty() {
        Ok(())
    } else {
        Err(c.unsupported)
    }
}

// ---------------------------------------------------------------------------
// No antecedent: unsupported
// ---------------------------------------------------------------------------

#[test]
fn a_spells_pronoun_without_an_antecedent_is_unsupported() {
    cr!("113.3a", "608.2c");
    // "It" in a spell never means the spell; "that player" never means you.
    assert!(compile("Sorcery", "It gains flying until end of turn.").is_err());
    assert!(compile(
        "Instant",
        "Destroy target creature. That player loses 2 life."
    )
    .is_err());
    assert!(compile(
        "Creature — Human",
        "{T}: Destroy target creature. That player loses 2 life."
    )
    .is_err());
    // "That creature" is never the source (oracle text says "~" for it).
    assert!(compile("Creature — Human", "{1}: Untap that creature.").is_err());
    // A permanent's "it" is still the permanent.
    assert!(compile(
        "Creature — Human",
        "{1}: ~ gets +1/+1 until end of turn. It gains flying until end of turn."
    )
    .is_ok());
    // Real cards: each creature's own controller, and the player an earlier ability
    // targeted, aren't tracked (they used to compile as the spell's controller / you).
    assert_unsupported(
        "Rakdos Charm",
        "Each creature deals 1 damage to its controller",
    );
    assert_unsupported("Laquatus's Champion", "that player gains 6 life");
}

#[test]
fn an_empty_effect_is_not_a_noop() {
    cr!("113.3a");
    // "Whenever you attack with two or more creatures," with nothing after it.
    assert!(compile(
        "Creature — Bird",
        "Whenever you attack with two or more creatures,"
    )
    .is_err());
    assert!(compile("Artifact", "{5}:").is_err());
}

// ---------------------------------------------------------------------------
// Tracked antecedents, in play
// ---------------------------------------------------------------------------

#[test]
fn that_player_is_the_player_target() {
    cr!("608.2c", "701.9a");
    assert_supported(&["Ozai's Cruelty"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    t.hand(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Ozai's Cruelty");
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P1), 1);
    // Not the caster.
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn it_is_the_spell_that_was_the_subject() {
    cr!("608.2c", "702.33d");
    assert_supported(&["Breath of Darigaaz"]);
    for (kicked, dmg) in [(false, 1), (true, 4)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 4);
        let giant = t.battlefield(P1, "Hill Giant");
        let angel = t.battlefield(P1, "Serra Angel");
        let spell = t.hand(P0, "Breath of Darigaaz");
        t.cast(P0, spell).kicked(kicked).go();
        t.resolve_all();
        assert_eq!(t.life(P0), 20 - dmg, "kicked: {kicked}");
        assert_eq!(t.life(P1), 20 - dmg);
        if kicked {
            assert!(!t.on_battlefield(giant));
        } else {
            assert_eq!(t.obj_now(giant).damage, 1);
        }
        assert_eq!(t.obj_now(angel).damage, 0);
    }
}

#[test]
fn it_is_the_card_the_search_found() {
    cr!("608.2c", "701.23a");
    assert_supported(&["Savage Order", "Fabled Passage"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    t.battlefield(P0, "Craw Wurm");
    t.library_top(P0, "Colossal Dreadmaw");
    let spell = t.hand(P0, "Savage Order");
    t.cast(P0, spell).go();
    t.resolve_all();
    let dreadmaw = t.named_on_battlefield("Colossal Dreadmaw");
    assert_eq!(dreadmaw.len(), 1, "{}", t.dump_log());
    // The Dinosaur gains indestructible, not the spell.
    assert!(t
        .obj_now(dreadmaw[0])
        .has_keyword(KeywordKind::Indestructible));
}

#[test]
fn that_land_is_the_land_the_search_put_onto_the_battlefield() {
    cr!("608.2c", "701.23a");
    ruling!(
        "Fabled Passage",
        "The land that you put onto the battlefield will be counted when determining whether you control four or more lands, but Fabled Passage will not."
    );
    for (others, untapped) in [(3, true), (2, false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", others);
        let found = t.library_top(P0, "Forest");
        let passage = t.battlefield(P0, "Fabled Passage");
        t.activate(P0, passage, 0, &[]).unwrap();
        t.resolve_all();
        let forest = t.named_on_battlefield("Forest");
        assert_eq!(forest.len(), 1);
        assert_ne!(t.zone(found), Zone::Library(P0));
        assert_eq!(!t.obj_now(forest[0]).tapped, untapped, "others: {others}");
    }
}

#[test]
fn it_is_the_token_just_created() {
    cr!("608.2c", "111.1");
    assert_supported(&["Match the Odds"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Match the Odds");
    t.cast(P0, spell).go();
    t.resolve_all();
    // P0's only creature is the Ally token.
    let ally: Vec<ObjectId> =
        t.g.battlefield
            .iter()
            .copied()
            .filter(|&o| t.obj_now(o).controller == P0 && t.obj_now(o).is(CardType::Creature))
            .collect();
    assert_eq!(ally.len(), 1);
    assert_eq!(t.counters(ally[0], "+1/+1"), 2);
    assert_eq!(t.pt(ally[0]), (3, 3));
}

#[test]
fn that_player_is_its_owner_or_controller() {
    cr!("608.2c", "701.9a");
    ruling!(
        "Frightful Delusion",
        "The player discards a card even if they pay {1}."
    );
    ruling!(
        "Dinrova Horror",
        "If a player has no cards in their hand and Dinrova Horror returns a card to that player’s hand, the player must discard that card."
    );
    assert_supported(&["Recoil", "Frightful Delusion", "Dinrova Horror"]);
    // Recoil: P1's permanent goes to P1's hand, and P1 (with no other card) discards it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 1);
    t.hand(P0, "Grizzly Bears");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Recoil");
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.hand_size(P0), 1);

    // Frightful Delusion: the countered spell's controller discards, paid or not.
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 3);
        t.lands(P1, "Mountain", 2);
        t.hand(P1, "Hill Giant");
        t.hand(P0, "Hill Giant");
        let shock = t.hand(P1, "Shock");
        let on_stack = t.cast(P1, shock).target(P0).go();
        let spell = t.hand(P0, "Frightful Delusion");
        t.answer_yes(P1, pay);
        t.cast(P0, spell).target(on_stack).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P1), 0, "paid: {pay}");
        assert_eq!(t.hand_size(P0), 1);
        // Shock resolved only if the {1} was paid.
        assert_eq!(t.life(P0), if pay { 18 } else { 20 });
    }
}

#[test]
fn that_player_is_the_one_whose_spell_targeted_it() {
    cr!("603.2", "608.2c");
    assert_supported(&["Ashenmoor Liege"]);
    let mut t = TestGame::new(2);
    let liege = t.battlefield(P0, "Ashenmoor Liege");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(liege).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn it_is_the_enchanted_creature() {
    cr!("608.2c", "303.4b");
    ruling!(
        "Bind the Monster",
        "Use the power of the enchanted creature as the enters-the-battlefield ability resolves to determine how much damage is dealt."
    );
    assert_supported(&["Bind the Monster"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let aura = t.hand(P0, "Bind the Monster");
    t.cast(P0, aura).target(giant).go();
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    // The Giant (not the Aura) deals damage equal to its (not the Aura's) power.
    assert_eq!(t.life(P0), 17);
}

#[test]
fn its_toughness_is_the_sacrificed_creatures() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Momentous Fall",
        "The sacrificed creature’s last known existence on the battlefield is checked to determine its power and its toughness."
    );
    assert_supported(&["Momentous Fall"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let turtle = t.battlefield(P0, "Horned Turtle");
    let spell = t.hand(P0, "Momentous Fall");
    let hand = t.hand_size(P0);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(turtle));
    // Draw 1 (its power), gain 4 (its toughness); the spell left the hand.
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn x_is_the_amassed_armys_power() {
    cr!("701.47a", "701.47c", "603.12");
    ruling!(
        "Foray of Orcs",
        "That means the Army creature you chose to receive counters"
    );
    assert_supported(&["Foray of Orcs"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Foray of Orcs");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2, "{}", t.dump_log());
}

#[test]
fn equipped_creature_is_it_after_the_equipment_is_sacrificed() {
    cr!("608.2c", "603.7c");
    ruling!(
        "Wings of Hubris",
        "you won't be able to sacrifice the creature at the beginning of the next end step"
    );
    assert_supported(&["Wings of Hubris"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wings = t.battlefield(P0, "Wings of Hubris");
    assert!(t.g.attach(wings, Entity::Object(bears)));
    let wall = t.battlefield(P1, "Wall of Wood");
    t.activate(P0, wings, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(wings));
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    let opts = block_options(&t.g, &[P1]);
    assert!(!block_declaration_legal(&t.g, &opts, &[(wall, bears)]));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------
// Mechanics the engine doesn't implement
// ---------------------------------------------------------------------------

#[test]
fn augment_and_host_cards_are_unsupported() {
    cr!("205.4a");
    // Unstable's augment and host aren't in the Comprehensive Rules or the engine; a host
    // creature's supertype "Host" isn't a supertype the engine knows.
    assert_unsupported("Adorable Kitten", "Host");
    assert_unsupported("Mer Man", "Host");
    assert_unsupported("Humming-", "Augment");
    // Old type lines with words that aren't types ("Summon Dragon").
    assert_unsupported("Prismatic Dragon", "Summon");
    // Tokens and ordinary cards are unaffected.
    assert_supported(&["Grizzly Bears"]);
}
