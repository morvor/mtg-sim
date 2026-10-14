//! Rulings batch S18 — adventurer cards (CR 715), casting: the Adventure's alternative
//! characteristics decide whether and how it can be cast, "has an Adventure", copies of
//! Adventure spells, and effects that let a spell be cast without paying its mana cost.

use crate::r_s01_common::*;
use crate::r_s08_common::legal_cast_methods;
use crate::r_s18_common::*;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, ObjKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

/// The options of the choices asked since decision `from`.
fn options_asked(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    asked_since(t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------------------
// Casting an Adventure without paying its mana cost.
// ---------------------------------------------------------------------------------------

/// P0 casts Bloodbraid Elf, whose cascade exiles `name` (its creature's mana value is less
/// than 4); P0 casts it as its Adventure without paying its mana cost, with `targets`.
fn cascade_into_adventure(t: &mut TestGame, name: &str, targets: &[Entity]) {
    stack_library(t, P0, &[name]);
    give_mana_for(t, P0, "Bloodbraid Elf");
    let elf = t.hand(P0, "Bloodbraid Elf");
    let from = t.asked().len();
    t.cast(P0, elf).go();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    t.resolve();
    let inset = &card(name).faces[1].chars.name;
    let offered = options_asked(t, from);
    assert_eq!(offered.len(), 1, "{offered:?}");
    assert!(offered[0][1].contains(inset.as_str()), "{offered:?}");
    // The Adventure is on the stack; no mana was spent (only the Elf's lands are tapped).
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, *inset);
    assert_eq!(tapped_lands(t, P0), 4);
}

#[test]
fn an_adventure_can_be_cast_without_paying_its_mana_cost() {
    cr!("715.3", "715.3a", "702.85a", "118.9a");
    ruling!(
        "Bonecrusher Giant // Stomp",
        "Casting a card as an Adventure isn't casting it for an alternative cost. Effects that allow you to cast a spell for an alternative cost or without paying its mana cost may allow you to apply those to the Adventure."
    );
    ruling!(
        "Callous Sell-Sword // Burn Together",
        "Casting a card as an Adventure isn’t casting it for an alternative cost. Effects that allow you to cast a spell for an alternative cost or without paying its mana cost may allow you to apply those to the Adventure."
    );
    supported("Bloodbraid Elf");
    // Stomp ({1}{R}: "Damage can't be prevented this turn. Stomp deals 2 damage to any
    // target."), cast by cascade.
    let mut t = TestGame::new(2);
    cascade_into_adventure(&mut t, "Bonecrusher Giant // Stomp", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // It went on an adventure: P0 may cast Bonecrusher Giant from exile.
    let giant = exiled_named(&t, "Bonecrusher Giant");
    assert_eq!(giant.len(), 1);
    give_mana_for(&mut t, P0, "Bonecrusher Giant // Stomp");
    assert_eq!(
        legal_cast_methods(&mut t, P0, giant[0]),
        vec![CastMethod::Normal]
    );
    // Burn Together ({R}: "Target creature you control deals damage equal to its power to
    // any other target. Then sacrifice it.").
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cascade_into_adventure(
        &mut t,
        "Callous Sell-Sword // Burn Together",
        &[Entity::Object(bears), Entity::Player(P1)],
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(exiled_named(&t, "Callous Sell-Sword").len(), 1);
}

// ---------------------------------------------------------------------------------------
// "Has an Adventure".
// ---------------------------------------------------------------------------------------

#[test]
fn a_card_has_an_adventure_even_if_it_was_never_cast_as_one() {
    cr!("715.2a");
    ruling!(
        "Edgewall Innkeeper",
        "An effect may refer to a card, spell, or permanent that \"has an Adventure.\" This refers to a card, spell, or permanent that has an adventurer card's set of alternative characteristics, even if they're not being used and even if that card was never cast as an Adventure."
    );
    ruling!(
        "Callous Sell-Sword // Burn Together",
        "An effect may refer to a card, spell, or permanent that “has an Adventure.” This refers to a card, spell, or permanent that has an adventurer card’s set of alternative characteristics, even if they’re not being used and even if that card was never cast as an Adventure."
    );
    supported("Edgewall Innkeeper");
    supported("Garenbrig Squire");
    // Edgewall Innkeeper: "Whenever you cast a creature spell that has an Adventure, draw
    // a card." Bonecrusher Giant cast from hand as a creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgewall Innkeeper");
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    assert!(has_adventure(&mut t, giant));
    give_mana_for(&mut t, P0, "Bonecrusher Giant // Stomp");
    t.cast(P0, giant).go();
    assert!(has_adventure(&mut t, giant));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(has_adventure(&mut t, giant));
    // Garenbrig Squire: "Whenever you cast a creature spell that has an Adventure, this
    // creature gets +1/+1 until end of turn." Callous Sell-Sword cast as a creature.
    let mut t = TestGame::new(2);
    let squire = t.battlefield(P0, "Garenbrig Squire");
    let sword = t.hand(P0, "Callous Sell-Sword // Burn Together");
    give_mana_for(&mut t, P0, "Callous Sell-Sword // Burn Together");
    t.cast(P0, sword).go();
    t.resolve_all();
    assert_eq!(t.pt(squire), (3, 3));
    // In the graveyard too, the card has an Adventure.
    let gy = t.graveyard(P0, "Callous Sell-Sword // Burn Together");
    assert!(has_adventure(&mut t, gy));
}

#[test]
fn an_adventure_spell_doesnt_have_an_adventure() {
    cr!("715.2a", "715.3b");
    ruling!(
        "Edgewall Innkeeper",
        "If an effect refers to a card, spell, or permanent that has an Adventure, it won't find an instant or sorcery spell on the stack that's been cast as an Adventure."
    );
    ruling!(
        "Callous Sell-Sword // Burn Together",
        "If an effect refers to a card, spell, or permanent that has an Adventure, it won’t find an instant or sorcery spell on the stack that’s been cast as an Adventure."
    );
    // Stomp cast with Edgewall Innkeeper out: no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgewall Innkeeper");
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    t.lands(P0, "Mountain", 2);
    let spell = t.cast(P0, giant).method(ADVENTURE).target(P1).go();
    assert!(!has_adventure(&mut t, spell));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P1), 18);
    // Burn Together cast with Garenbrig Squire out: no bonus.
    let mut t = TestGame::new(2);
    let squire = t.battlefield(P0, "Garenbrig Squire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.hand(P0, "Callous Sell-Sword // Burn Together");
    t.lands(P0, "Mountain", 1);
    let spell = t
        .cast(P0, sword)
        .method(ADVENTURE)
        .target(bears)
        .target(P1)
        .go();
    assert!(!has_adventure(&mut t, spell));
    t.resolve_all();
    assert_eq!(t.pt(squire), (2, 2));
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------------------
// Characteristics on the stack.
// ---------------------------------------------------------------------------------------

#[test]
fn an_adventure_spell_has_only_the_adventures_characteristics() {
    cr!("715.3b", "715.4", "202.3");
    ruling!(
        "Two-Headed Hunter // Twice the Rage",
        "When casting a spell as an Adventure, use the alternative characteristics and ignore all of the card’s normal characteristics. The spell’s color, mana cost, mana value, and so on are determined by only those alternative characteristics. If the spell leaves the stack, it immediately resumes using its normal characteristics."
    );
    ruling!(
        "Brazen Borrower // Petty Theft",
        "When playing a card as an Adventure, use the alternative characteristics and ignore all of the card's normal characteristics. The resulting spell's color, mana cost, mana value, and so on are determined by only those alternative characteristics. If the spell leaves the stack, it immediately resumes using its normal characteristics."
    );
    supported("Tempest Hart // Scan the Clouds");
    // Tempest Hart: "Whenever you cast a spell with mana value 5 or greater, put a +1/+1
    // counter on this creature." Twice the Rage ({1}{R}) doesn't trigger it; Two-Headed
    // Hunter ({4}{R}) does.
    let mut t = TestGame::new(2);
    let hart = t.battlefield(P0, "Tempest Hart // Scan the Clouds");
    rainbow_lands(&mut t, P0, 3);
    let hunter = t.hand(P0, "Two-Headed Hunter // Twice the Rage");
    let spell = t.cast(P0, hunter).method(ADVENTURE).target(hart).go();
    let c = t.obj(spell).chars.clone();
    assert_eq!(c.name, "Twice the Rage");
    assert!(c.is(CardType::Instant) && !c.is_creature());
    assert_eq!(t.g.mana_value_of(spell), 2);
    t.resolve_all();
    assert_eq!(t.counters(hart, counters::PLUS1), 0);
    // Exiled, it's Two-Headed Hunter again: a creature card with mana value 5.
    let exiled = exiled_named(&t, "Two-Headed Hunter")[0];
    assert!(t.obj(exiled).chars.is_creature());
    assert_eq!(t.g.mana_value_of(exiled), 5);
    t.cast(P0, exiled).go();
    t.resolve_all();
    assert_eq!(t.counters(hart, counters::PLUS1), 1);
    // Petty Theft is a blue instant with mana value 2 — no flying or flash — and the card
    // is Brazen Borrower again once it left the stack.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let borrower = t.hand(P0, "Brazen Borrower // Petty Theft");
    let spell = t.cast(P0, borrower).method(ADVENTURE).target(bears).go();
    let c = t.obj(spell).chars.clone();
    assert_eq!(c.name, "Petty Theft");
    assert!(c.is(CardType::Instant) && !c.is_creature());
    assert!(!c.has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert!(!c.has_keyword(mtg_engine::keywords::KeywordKind::Flash));
    assert_eq!(t.g.mana_value_of(spell), 2);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    let exiled = exiled_named(&t, "Brazen Borrower")[0];
    assert!(t.obj(exiled).chars.is_creature());
    assert_eq!(t.g.mana_value_of(exiled), 3);
}

// ---------------------------------------------------------------------------------------
// Copies of Adventure spells.
// ---------------------------------------------------------------------------------------

/// P0 casts `name` as its Adventure with `targets` (from `setup`) and copies it with
/// Twincast: the copy resolves, is exiled and ceases to exist; only the card itself goes
/// on an adventure and can be cast later.
fn copy_of_an_adventure(name: &str, setup: fn(&mut TestGame) -> Vec<Entity>) {
    supported(name);
    supported("Twincast");
    let mut t = TestGame::new(2);
    let targets = setup(&mut t);
    rainbow_lands(&mut t, P0, 3);
    let c = t.hand(P0, name);
    let spell = t.cast(P0, c).method(ADVENTURE).targets(&targets).go();
    let tw = t.hand(P0, "Twincast");
    t.cast(P0, tw).target(spell).go();
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(copy).kind, ObjKind::SpellCopy);
    assert_eq!(t.g.obj(copy).chars.name, t.g.obj(spell).chars.name);
    t.resolve();
    // The copy is gone: nothing is in exile yet.
    assert!(!t.g.is_live(copy) || t.g.obj(copy).zone != mtg_engine::object::Zone::Stack);
    assert!(t.g.exile.is_empty(), "{:?}", t.g.exile);
    t.resolve_all();
    let creature = &card(name).faces[0].chars.name;
    assert_eq!(t.g.exile.len(), 1);
    let exiled = t.g.exile[0];
    assert_eq!(t.obj(exiled).chars.name, *creature);
    rainbow_lands(&mut t, P0, 3);
    assert_eq!(
        legal_cast_methods(&mut t, P0, exiled),
        vec![CastMethod::Normal],
        "{name}"
    );
}

#[test]
fn a_copy_of_an_adventure_spell_ceases_to_exist() {
    cr!("715.3c", "715.3d", "704.5e", "707.10");
    ruling!(
        "Two-Headed Hunter // Twice the Rage",
        "If an effect copies an Adventure spell, that copy is exiled as it resolves. It ceases to exist as a state-based action; it’s not possible to cast the copy as a permanent."
    );
    ruling!(
        "Young Red Dragon // Bathe in Gold",
        "If an effect copies an Adventure spell, that copy is exiled as it resolves. It ceases to exist as a state-based action; it’s not possible to cast the copy from exile."
    );
    ruling!(
        "Murderous Rider // Swift End",
        "If an effect copies an Adventure spell, that copy is exiled as it resolves. It ceases to exist as a state-based action; it's not possible to play the copy as a permanent."
    );
    ruling!(
        "Karvanista, Loyal Lupari // Lupari Shield",
        "If an effect copies an Adventure spell, that copy is exiled as it resolves. It ceases to exist as a state-based action; it's not possible to cast the copy from exile."
    );
    copy_of_an_adventure("Two-Headed Hunter // Twice the Rage", |t| {
        vec![Entity::Object(t.battlefield(P0, "Grizzly Bears"))]
    });
    copy_of_an_adventure("Young Red Dragon // Bathe in Gold", |_| vec![]);
    // (Swift End destroys an indestructible creature: it stays a legal target for the
    // original spell.)
    copy_of_an_adventure("Murderous Rider // Swift End", |t| {
        vec![Entity::Object(t.battlefield(P1, "Darksteel Myr"))]
    });
    copy_of_an_adventure("Karvanista, Loyal Lupari // Lupari Shield", |_| vec![]);
}

// ---------------------------------------------------------------------------------------
// Whether it can be cast is decided by the Adventure's characteristics.
// ---------------------------------------------------------------------------------------

/// Checks that only the Adventure's characteristics decide whether `name` can be cast as
/// an Adventure: with Garruk's Horde ("You may cast creature spells from the top of your
/// library."), the card on top of the library can be cast as the creature but not as the
/// Adventure; during an opponent's turn, it can be cast as the Adventure only if that's
/// an instant.
fn legality_by_the_adventure(name: &str) {
    supported(name);
    supported("Garruk's Horde");
    let def = card(name);
    let instant = def.faces[1].chars.card_types.contains(CardType::Instant);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Horde");
    rainbow_lands(&mut t, P0, 3);
    let top = t.library_top(P0, name);
    assert_eq!(legal_cast_methods(&mut t, P0, top), vec![CastMethod::Normal]);
    // From the hand: both in P0's main phase; during P1's turn, only an instant
    // Adventure (the creature doesn't have flash).
    let c = t.hand(P0, name);
    assert_eq!(
        legal_cast_methods(&mut t, P0, c),
        vec![CastMethod::Normal, ADVENTURE]
    );
    t.set_step(P1, Step::PrecombatMain);
    let expected = if instant { vec![ADVENTURE] } else { vec![] };
    assert_eq!(legal_cast_methods(&mut t, P0, c), expected, "{name}");
}

#[test]
fn only_the_adventures_characteristics_decide_if_it_can_be_cast() {
    cr!("715.3a", "601.3e", "304.1", "307.1");
    ruling!(
        "Two-Headed Hunter // Twice the Rage",
        "If you cast an adventurer card as an Adventure, use only its alternative characteristics to determine whether it’s legal to cast that spell. For example, if you control Johann, Apprentice Sorcerer (“Once each turn, you may cast an instant or sorcery spell from the top of your library.”) and Questing Druid is on top of your library, you can cast Seek the Beast, but not Questing Druid."
    );
    ruling!(
        "Young Red Dragon // Bathe in Gold",
        "If you cast an adventurer card as an Adventure, use only its alternative characteristics to determine whether it’s legal to cast that spell."
    );
    ruling!(
        "Twining Twins // Swift Spiral",
        "If you cast an adventurer card as an Adventure, use only its alternative characteristics to determine whether it's legal to cast that spell. For example, if you control Johann, Apprentice Sorcerer (\"Once each turn, you may cast an instant or sorcery spell from the top of your library.\") and Questing Druid is on top of your library, you can cast Seek the Beast, but not Questing Druid."
    );
    ruling!(
        "Bilbo Baggins, Burglar // Take a Glance",
        "If you cast an adventurer card as an Adventure, use only its alternative characteristics to determine whether it's legal to cast that spell. For example, if you control Gandalf, Party Guest"
    );
    ruling!(
        "Murderous Rider // Swift End",
        "If you cast an adventurer card as an Adventure, use only its alternative characteristics to determine whether it's legal to cast that spell. For example, if you control Traveling Chocobo"
    );
    ruling!(
        "Karvanista, Loyal Lupari // Lupari Shield",
        "If you cast an adventurer card as an Adventure, use only its alternative characteristics to determine whether it's legal to cast that spell."
    );
    for name in [
        "Two-Headed Hunter // Twice the Rage",
        "Young Red Dragon // Bathe in Gold",
        "Twining Twins // Swift Spiral",
        "Bilbo Baggins, Burglar // Take a Glance",
        "Murderous Rider // Swift End",
        "Karvanista, Loyal Lupari // Lupari Shield",
    ] {
        legality_by_the_adventure(name);
    }
}

#[test]
fn a_cascade_hit_can_be_cast_as_its_adventure_only_if_that_costs_less() {
    cr!("715.3a", "702.85a");
    ruling!(
        "Giant Killer // Chop Down",
        "If you cast an adventurer card as an Adventure, use only its alternative characteristics to determine whether it's legal to cast that spell. For example, if Giant Killer is exiled with the last ability of Vivien, Champion of the Wilds, you can't cast it as Chop Down."
    );
    supported("Giant Killer // Chop Down");
    supported("Shardless Agent");
    legality_by_the_adventure("Giant Killer // Chop Down");
    // Shardless Agent (mana value 3) cascades into Giant Killer (1): Chop Down (3) isn't
    // less than 3, so only Giant Killer can be cast.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Giant Killer // Chop Down"]);
    give_mana_for(&mut t, P0, "Shardless Agent");
    let agent = t.hand(P0, "Shardless Agent");
    let from = t.asked().len();
    t.cast(P0, agent).go();
    t.answer_yes(P0, true);
    t.resolve();
    assert!(options_asked(&t, from).is_empty());
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Giant Killer");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Giant Killer").len(), 1);
    // Bloodbraid Elf (4) could cast either.
    let mut t = TestGame::new(2);
    let serra = t.battlefield(P1, "Serra Angel");
    cascade_into_adventure(&mut t, "Giant Killer // Chop Down", &[Entity::Object(serra)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Serra Angel"));
}
