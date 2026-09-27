//! Oracle hardening: what "it", "that creature" and "that player" refer to (patterns in
//! `src/oracle/patterns/oracle_hardening_referents.rs`).
//!
//! A spell's pronouns never mean the spell itself and "that player" never means "you":
//! without a tracked antecedent the text is unsupported instead of compiling into an
//! effect on the wrong object. The antecedents the compiler does track are checked in
//! play: a player target, the spell as the subject of an earlier sentence, cards found by
//! a search, tokens just created, the permanent an Aura enchants, the sacrificed creature,
//! the amassed Army, "its owner"/"its controller", and a trigger's player.

use mtg_engine::card::{CardDef, Layout};
use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

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
    compile_with_keywords(type_line, &[], text)
}

/// [`compile`] for a card whose Scryfall keywords are `keywords`.
fn compile_with_keywords(
    type_line: &str,
    keywords: &[String],
    text: &str,
) -> Result<(), Vec<String>> {
    let tl = TypeLine::parse(type_line);
    let ctx = CompileContext {
        card_name: "Test Card",
        full_name: "Test Card",
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords,
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

/// A card compiled from oracle text with the real compiler (which must understand all of
/// it).
fn custom_card(type_line: &str, cost: &str, text: &str) -> CardDef {
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
    assert!(c.unsupported.is_empty(), "unsupported: {:?}", c.unsupported);
    let m = mtg_engine::mana::ManaCost::parse(cost);
    CardDef::custom(Characteristics {
        name: SmolStr::new("Test Card"),
        colors: m.as_ref().map_or(ColorSet::NONE, |m| m.colors()),
        mana_cost: m,
        card_types: tl.card_types,
        abilities: c.abilities,
        rules_text: Arc::from(text),
        ..Default::default()
    })
}

/// The creatures `p` controls, except `but`.
fn creatures_of(t: &TestGame, p: PlayerId, but: &[ObjectId]) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|&o| {
            !but.contains(&o) && t.obj_now(o).controller == p && t.obj_now(o).is(CardType::Creature)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// No antecedent: unsupported
// ---------------------------------------------------------------------------

#[test]
fn a_spells_pronoun_without_an_antecedent_is_unsupported() {
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
fn a_cleaved_texts_pronoun_needs_an_antecedent_too() {
    // Cast for its cleave cost, the spell loses the bracketed words and is compiled again
    // from what's left, where "its power" would have no antecedent.
    let cleave = ["Cleave".to_string()];
    let full = "Exile target creature. Until end of turn, creatures you control get +X/+0, \
                where X is its power.";
    assert_eq!(compile("Sorcery", full), Ok(()));
    let text = "Cleave {2}\n[Exile target creature. ]Until end of turn, creatures you control \
                get +X/+0, where X is its power.";
    let err = compile_with_keywords("Sorcery", &cleave, text).unwrap_err();
    assert!(err.iter().any(|u| u.contains("where X is its power")), "{err:?}");
    // Real cards whose cleaved text keeps its antecedents are understood.
    assert_supported(&["Alchemist's Retrieval", "Dread Fugue", "Dig Up"]);
}

#[test]
fn an_empty_effect_is_not_a_noop() {
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
    assert_supported(&["Recoil", "Frightful Delusion"]);
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
    for pump in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 1);
        t.lands(P0, "Forest", 1);
        let giant = t.battlefield(P1, "Hill Giant");
        let aura = t.hand(P0, "Bind the Monster");
        t.cast(P0, aura).target(giant).go();
        // The Aura resolves; its enters trigger goes on the stack.
        t.resolve();
        assert_eq!(t.stack_len(), 1);
        if pump {
            // In response, the Giant gets +3/+3.
            let growth = t.hand(P0, "Giant Growth");
            t.cast(P0, growth).target(giant).go();
        }
        t.resolve_all();
        assert!(t.obj_now(giant).tapped);
        // The Giant (not the Aura) deals damage equal to its (not the Aura's) power as
        // the ability resolves.
        assert_eq!(t.life(P0), if pump { 14 } else { 17 }, "pump: {pump}");
        assert_eq!(t.life(P1), 20);
    }
}

#[test]
fn enchanted_creature_is_the_one_it_last_enchanted() {
    cr!("608.2h");
    ruling!(
        "Bind the Monster",
        "If Bind the Monster is no longer on the battlefield as the enters-the-battlefield ability resolves, use the power of the creature it was last enchanting"
    );
    ruling!(
        "Bind the Monster",
        "Damage is dealt even if that creature is also not on the battlefield at that time."
    );
    // In response to the enters trigger, the Aura is destroyed (Disenchant) or the Giant
    // returns to its owner's hand (Unsummon).
    for (response, cost) in [("Disenchant", "Plains"), ("Unsummon", "Island")] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 1);
        t.lands(P0, cost, 2);
        let giant = t.battlefield(P1, "Hill Giant");
        let aura = t.hand(P0, "Bind the Monster");
        t.cast(P0, aura).target(giant).go();
        t.resolve();
        assert_eq!(t.stack_len(), 1);
        let target = if response == "Disenchant" {
            t.named_on_battlefield("Bind the Monster")[0]
        } else {
            giant
        };
        let spell = t.hand(P0, response);
        t.cast(P0, spell).target(target).go();
        t.resolve_all();
        assert!(t.named_on_battlefield("Bind the Monster").is_empty());
        // The Giant it last enchanted deals damage equal to its last known power, 3.
        assert_eq!(t.life(P0), 17, "{response}: {}", t.dump_log());
        assert_eq!(t.life(P1), 20);
        if response == "Disenchant" {
            assert!(t.obj_now(giant).tapped);
        } else {
            assert!(t.in_hand(P1, "Hill Giant"));
        }
    }
}

#[test]
fn an_already_tapped_creature_still_deals_the_damage() {
    cr!("603.2");
    ruling!(
        "Bind the Monster",
        "The enters-the-battlefield ability triggers even if the enchanted creature is already tapped. That creature will still deal damage to you."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    let aura = t.hand(P0, "Bind the Monster");
    t.cast(P0, aura).target(giant).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
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
    // A 2/5 on the battlefield; the card in the graveyard is a 1/4.
    t.g.add_counters(Entity::Object(turtle), "+1/+1", 1, None);
    let spell = t.hand(P0, "Momentous Fall");
    let hand = t.hand_size(P0);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(turtle));
    // Draw 2 (its power), gain 5 (its toughness); the spell left the hand.
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert_eq!(t.life(P0), 25);
}

#[test]
fn x_is_the_amassed_armys_power() {
    cr!("701.47a", "701.47c", "603.12");
    ruling!(
        "Foray of Orcs",
        "That means the Army creature you chose to receive counters"
    );
    assert_supported(&["Foray of Orcs"]);
    // No Army: a new 0/0 Orc Army gets two counters.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Foray of Orcs");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2, "{}", t.dump_log());

    // A 1/1 changeling is an Army: it's the amassed Army, and the spell deals damage equal
    // to its power with the counters, 3.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let outcast = t.battlefield(P0, "Changeling Outcast");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Foray of Orcs");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.counters(outcast, "+1/+1"), 2);
    assert!(t.named_on_battlefield("Orc Army").is_empty());
    assert!(!t.on_battlefield(giant), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn equipped_creature_is_it_after_the_equipment_is_sacrificed() {
    cr!("608.2c", "603.7c", "701.21a");
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

    // Attached to a creature P0 doesn't control: it can't be blocked, but P0 can't
    // sacrifice it.
    let mut t = TestGame::new(2);
    let wings = t.battlefield(P0, "Wings of Hubris");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P0, "Wall of Wood");
    t.advance_to(P1, Step::Upkeep);
    assert!(t.g.attach(wings, Entity::Object(bears)));
    t.activate(P0, wings, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(wings));
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.advance_to(P1, Step::DeclareAttackers);
    let opts = block_options(&t.g, &[P0]);
    assert!(!block_declaration_legal(&t.g, &opts, &[(wall, bears)]));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears), "{}", t.dump_log());
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn a_creature_controlled_until_end_of_turn_is_sacrificed() {
    cr!("514.2", "701.21a");
    ruling!(
        "Wings of Hubris",
        "If you gain control of a creature \"until end of turn,\" you control it during your end step."
    );
    // P0 gains control of P1's Bears until end of turn, equips them, and sacrifices the
    // Equipment: P0 still controls the Bears as the delayed ability resolves in the end
    // step (control ends only in the cleanup step), so P0 sacrifices them.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wings = t.battlefield(P0, "Wings of Hubris");
    let treason = t.hand(P0, "Act of Treason");
    t.cast(P0, treason).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    // Equip {1}, then "Sacrifice this Equipment: ...".
    t.activate(P0, wings, 1, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(wings).attached_to, Some(Entity::Object(bears)));
    t.activate(P0, wings, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "{}", t.dump_log());
    // Sacrificed: put into its owner's graveyard.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn that_player_is_the_owner_of_the_returned_permanent() {
    cr!("608.2c", "701.9a");
    ruling!(
        "Dinrova Horror",
        "If a player has no cards in their hand and Dinrova Horror returns a card to that player’s hand, the player must discard that card."
    );
    assert_supported(&["Dinrova Horror"]);
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_targets(P0, &[Entity::Object(angel)]);
    t.enter(P0, "Dinrova Horror");
    t.resolve_all();
    // P1 (the owner) discards the returned Angel; P0 keeps its card.
    assert!(t.in_graveyard(P1, "Serra Angel"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn a_returned_token_cant_be_discarded() {
    cr!("111.6", "111.7", "701.9a");
    ruling!(
        "Dinrova Horror",
        "A token permanent returned to a player’s hand isn’t a card and can’t be discarded."
    );
    // P1 controls two Soldier tokens and has a Hill Giant card in hand. One token returns
    // to P1's hand; P1 must discard the Giant (the token isn't a card), then the token
    // ceases to exist.
    let mut t = TestGame::new(2);
    t.lands(P1, "Plains", 2);
    let alarm = t.hand(P1, "Raise the Alarm");
    t.cast(P1, alarm).go();
    t.resolve_all();
    let soldiers = creatures_of(&t, P1, &[]);
    assert_eq!(soldiers.len(), 2);
    let giant = t.hand(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(soldiers[0])]);
    t.enter(P0, "Dinrova Horror");
    t.resolve_all();
    // The token in P1's hand wasn't offered as a card to discard.
    let offered: Vec<Entity> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if p == P1 && prompt.contains("discard") => Some(candidates),
            _ => None,
        })
        .flatten()
        .collect();
    assert_eq!(offered, vec![Entity::Object(giant)], "{}", t.dump_log());
    assert_eq!(t.zone(giant), Zone::Graveyard(P1));
    assert_eq!(t.hand_size(P1), 0);
    assert!(!t.on_battlefield(soldiers[0]));
    assert!(t.on_battlefield(soldiers[1]));
}

#[test]
fn no_one_discards_if_the_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Dinrova Horror",
        "If the target permanent is an illegal target when Dinrova Horror’s ability tries to resolve, the ability won’t resolve and none of its effects will happen. No player will discard a card."
    );
    // In response, P0 returns the targeted Angel to P1's hand itself (Unsummon).
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.hand(P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_targets(P0, &[Entity::Object(angel)]);
    t.enter(P0, "Dinrova Horror");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let unsummon = t.hand(P0, "Unsummon");
    t.cast(P0, unsummon).target(angel).go();
    t.resolve_all();
    // The Angel stays in P1's hand, and P0 keeps its Bears.
    assert!(t.in_hand(P1, "Serra Angel"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P1), 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.graveyard_size(P1), 0);
}

#[test]
fn that_player_controls_the_targeting_spell() {
    cr!("603.2", "608.2c");
    assert_supported(&["Black Bolt, Inhuman King"]);
    let mut t = TestGame::new(2);
    let bolt = t.battlefield(P0, "Black Bolt, Inhuman King");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(bolt).go();
    // The trigger's target must be a nonland permanent P1 (who cast Shock) controls: P0's
    // own Bears aren't a legal choice, and the only legal one is P1's Bears.
    t.answer_targets(P0, &[Entity::Object(mine)]);
    t.resolve_all();
    assert!(!t.on_battlefield(theirs), "{}", t.dump_log());
    assert!(t.on_battlefield(mine));
    assert!(t.on_battlefield(bolt));
}

#[test]
fn that_lands_controller_after_it_was_destroyed() {
    cr!("608.2c", "608.2h");
    assert_supported(&["Orcish Mine"]);
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P1, "Mountain");
    let mine = t.battlefield(P0, "Orcish Mine");
    assert!(t.g.attach(mine, Entity::Object(mountain)));
    // One ore counter left.
    let n = t.counters(mine, "ore");
    if n == 0 {
        t.g.add_counters(Entity::Object(mine), "ore", 1, None);
    } else {
        t.g.remove_counters(Entity::Object(mine), "ore", n - 1);
    }
    assert_eq!(t.counters(mine, "ore"), 1);
    // The land becomes tapped: the last counter is removed, the land is destroyed, and
    // its controller (not the Aura's) is dealt 2 damage.
    t.g.tap(mountain);
    t.resolve_all();
    assert!(!t.on_battlefield(mountain), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Mountain"));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn those_creatures_are_the_tokens_just_created() {
    cr!("608.2c", "701.30d");
    assert_supported(&["Gilt-Leaf Ambush"]);
    for win in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 3);
        let bears = t.battlefield(P0, "Grizzly Bears");
        // Mana values 6 and 0 (or the other way around).
        let (mine, theirs) = if win {
            ("Colossal Dreadmaw", "Forest")
        } else {
            ("Forest", "Colossal Dreadmaw")
        };
        t.library_top(P0, mine);
        t.library_top(P1, theirs);
        let spell = t.hand(P0, "Gilt-Leaf Ambush");
        t.cast(P0, spell).go();
        t.resolve_all();
        let elves: Vec<ObjectId> =
            t.g.battlefield
                .iter()
                .copied()
                .filter(|&o| o != bears && t.obj_now(o).is(CardType::Creature))
                .collect();
        assert_eq!(elves.len(), 2, "{}", t.dump_log());
        for e in elves {
            assert_eq!(
                t.obj_now(e).has_keyword(KeywordKind::Deathtouch),
                win,
                "win: {win}"
            );
        }
        assert!(!t.obj_now(bears).has_keyword(KeywordKind::Deathtouch));
    }
}

#[test]
fn that_land_is_still_the_found_land_after_beholding() {
    cr!("608.2c", "701.4a", "701.23a");
    assert_supported(&["Elven Passage"]);
    for behold in [true, false] {
        let mut t = TestGame::new(2);
        let elves = t.battlefield(P0, "Llanowar Elves");
        t.g.tap(elves);
        let found = t.library_top(P0, "Forest");
        let passage = t.battlefield(P0, "Elven Passage");
        // The search finds the Forest; P0 beholds the tapped Elf permanent (or not).
        t.answer_choose(P0, &[Entity::Object(found)]);
        t.answer_yes(P0, behold);
        t.answer_choose(P0, &[Entity::Object(elves)]);
        t.activate(P0, passage, 0, &[]).unwrap();
        t.resolve_all();
        let forest = t.named_on_battlefield("Forest");
        assert_eq!(forest.len(), 1, "{}", t.dump_log());
        // "That land" is the Forest, not the beheld Elf.
        assert_eq!(t.obj_now(forest[0]).tapped, !behold, "behold: {behold}");
        assert!(t.obj_now(elves).tapped, "behold: {behold}");
        assert_eq!(t.life(P0), 19);
    }
}

#[test]
fn it_is_what_the_latest_instruction_brought_into_play() {
    cr!("608.2c", "701.23a", "111.1");
    // A token, then a card found by a search: "it" is the found card.
    let spell = custom_card(
        "Sorcery",
        "{G}",
        "Create a 1/1 white Spirit creature token. Search your library for a creature card, \
         put it onto the battlefield, then shuffle. It gains haste until end of turn.",
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let found = t.library_top(P0, "Grizzly Bears");
    let s = t.custom(P0, spell, Zone::Hand(P0));
    t.answer_choose(P0, &[Entity::Object(found)]);
    t.cast(P0, s).go();
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1, "{}", t.dump_log());
    assert!(t.obj_now(bears[0]).has_keyword(KeywordKind::Haste));
    let spirits = creatures_of(&t, P0, &bears);
    assert_eq!(spirits.len(), 1);
    assert!(!t.obj_now(spirits[0]).has_keyword(KeywordKind::Haste));

    // A found card, then a token: "it" is the token.
    let spell = custom_card(
        "Sorcery",
        "{G}",
        "Search your library for a basic land card, put it onto the battlefield tapped, \
         then shuffle. Create a 1/1 white Spirit creature token. Put a +1/+1 counter on it.",
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let found = t.library_top(P0, "Plains");
    let s = t.custom(P0, spell, Zone::Hand(P0));
    t.answer_choose(P0, &[Entity::Object(found)]);
    t.cast(P0, s).go();
    t.resolve_all();
    let plains = t.named_on_battlefield("Plains");
    assert_eq!(plains.len(), 1, "{}", t.dump_log());
    assert_eq!(t.counters(plains[0], "+1/+1"), 0);
    let spirits = creatures_of(&t, P0, &[]);
    assert_eq!(spirits.len(), 1);
    assert_eq!(t.counters(spirits[0], "+1/+1"), 1);
    assert_eq!(t.pt(spirits[0]), (2, 2));
}

// ---------------------------------------------------------------------------
// Mechanics the engine doesn't implement
// ---------------------------------------------------------------------------

#[test]
fn augment_and_host_cards_are_unsupported() {
    // Unstable's augment and host aren't in the Comprehensive Rules or the engine; "Host"
    // isn't one of the supertypes (CR 205.4a).
    assert_unsupported("Adorable Kitten", "Host");
    assert_unsupported("Mer Man", "Host");
    assert_unsupported("Humming-", "Augment");
    // Old type lines with words that aren't types ("Summon Dragon").
    assert_unsupported("Prismatic Dragon", "Summon");
    // Tokens and ordinary cards are unaffected.
    assert_supported(&["Grizzly Bears"]);
}
