//! CR 300: card types in general — the list of card types, objects with several card
//! types, lands that are also other types, and kindred cards.

use crate::r300_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn the_card_types_are_the_fifteen_listed_ones() {
    cr!("300.1");
    // The engine knows exactly the card types the rule lists.
    let rule = mtg_data::comprehensive_rules()
        .get("300.1")
        .expect("300.1")
        .text
        .clone();
    let list = rule
        .split(" are ")
        .nth(1)
        .unwrap()
        .trim_end_matches('.')
        .replace(" and ", " ");
    let words: Vec<&str> = list
        .split(',')
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .collect();
    assert_eq!(words.len(), 15);
    let parsed: Vec<CardType> = words
        .iter()
        .map(|w| CardType::from_word(w).unwrap_or_else(|| panic!("unknown type {w}")))
        .collect();
    let mut all = CardType::ALL.to_vec();
    all.sort();
    let mut got = parsed.clone();
    got.sort();
    assert_eq!(got, all);
    // Real cards of each type have it.
    for (name, ty) in [
        ("Ornithopter", CardType::Artifact),
        ("Invasion of Ixalan", CardType::Battle),
        ("Power Play", CardType::Conspiracy),
        ("Grizzly Bears", CardType::Creature),
        ("Lost Mine of Phandelver", CardType::Dungeon),
        ("Pacifism", CardType::Enchantment),
        ("Lightning Bolt", CardType::Instant),
        ("Bitterblossom", CardType::Kindred),
        ("Forest", CardType::Land),
        ("Mutual Epiphany", CardType::Phenomenon),
        ("Goldmeadow", CardType::Plane),
        ("Jace Beleren", CardType::Planeswalker),
        ("Roots of All Evil", CardType::Scheme),
        ("Divination", CardType::Sorcery),
        ("Titania", CardType::Vanguard),
    ] {
        assert!(types_of(name).contains(ty), "{name} is a {}", ty.word());
    }
}

#[test]
fn an_artifact_creature_is_affected_by_artifact_and_creature_effects() {
    cr!("300.2");
    // Ornithopter is an artifact creature: "destroy target artifact" and "destroy target
    // creature" both apply to it.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Ornithopter");
    assert!(t.obj(a).is(CardType::Artifact) && t.obj(a).is_creature());
    t.lands(P0, "Mountain", 2);
    let shatter = t.hand(P0, "Shatter");
    t.cast(P0, shatter).target(a).go();
    t.resolve();
    assert!(!t.on_battlefield(a));
    t.lands(P0, "Swamp", 3);
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(b).go();
    t.resolve();
    assert!(!t.on_battlefield(b));
    // A land creature is both a land and a creature: a land-destruction spell destroys
    // Dryad Arbor, and it's affected by effects on creatures.
    let arbor = t.battlefield(P1, "Dryad Arbor");
    let _anthem = t.battlefield(P1, "Glorious Anthem");
    assert_eq!(t.pt(arbor), (2, 2));
    t.lands(P0, "Mountain", 3);
    let rain = t.hand(P0, "Stone Rain");
    t.cast(P0, rain).target(arbor).go();
    t.resolve();
    assert!(!t.on_battlefield(arbor));
}

#[test]
fn a_land_that_is_also_another_type_can_only_be_played_as_a_land() {
    cr!("300.2a", "305.9");
    ruling!(
        "Dryad Arbor",
        "Dryad Arbor is played as a land. It doesn't use the stack, it's not a spell"
    );
    let mut t = TestGame::new(2);
    let den = t.hand(P0, "Ancient Den");
    let arbor = t.hand(P0, "Dryad Arbor");
    t.lands(P0, "Plains", 3);
    // Neither can be cast as a spell, even with mana available.
    assert!(!can_cast(&mut t, P0, den));
    assert!(!can_cast(&mut t, P0, arbor));
    assert!(t.cast(P0, den).try_go().is_err());
    assert!(t.cast(P0, arbor).try_go().is_err());
    assert!(t.in_hand(P0, "Ancient Den"));
    // Each can be played as a land.
    assert!(can_play_land(&mut t, P0, den));
    assert!(can_play_land(&mut t, P0, arbor));
    t.play_land(P0, arbor).unwrap();
    assert_eq!(t.stack_len(), 0, "it doesn't use the stack");
    let arbor = t.named_on_battlefield("Dryad Arbor")[0];
    assert!(t.obj(arbor).is_creature() && t.obj(arbor).chars.is_land());
    assert_eq!(t.player(P0).lands_played_this_turn, 1);
    // It was this turn's land play: the artifact land can't be played now.
    assert!(!can_play_land(&mut t, P0, den));
    assert!(t.play_land(P0, den).is_err());
}

#[test]
fn a_kindred_instant_is_cast_and_resolves_like_an_instant() {
    cr!("300.2b", "308.1");
    // Crib Swap is a Kindred Instant — Shapeshifter.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let swap = t.hand(P0, "Crib Swap");
    t.lands(P0, "Plains", 3);
    // Any time P0 has priority: here during P1's combat, with a spell on the stack.
    t.set_step(P1, Step::DeclareAttackers);
    hold_stack(&mut t, P1);
    assert!(can_cast(&mut t, P0, swap));
    let spell = t.cast(P0, swap).target(bears).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve();
    // It resolved as an instant: its effect happened and it went to the graveyard.
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Crib Swap"));
    assert!(t.named_on_battlefield("Crib Swap").is_empty());
}

#[test]
fn a_kindred_enchantment_is_cast_and_resolves_like_an_enchantment() {
    cr!("300.2b", "308.1");
    ruling!(
        "Bitterblossom",
        "Kindred is a card type that allows noncreature cards to have creature types"
    );
    // Bitterblossom is a Kindred Enchantment — Faerie: cast at sorcery speed, it
    // resolves onto the battlefield.
    check_sorcery_timing("Bitterblossom", "{1}{B}");
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Bitterblossom");
    t.lands(P0, "Swamp", 2);
    t.cast(P0, c).go();
    t.resolve();
    let bb = t.named_on_battlefield("Bitterblossom");
    assert_eq!(bb.len(), 1);
    let o = t.obj(bb[0]);
    // It's a Faerie, but not a creature.
    assert!(o.is(CardType::Kindred) && o.is(CardType::Enchantment));
    assert!(o.chars.has_subtype("Faerie"));
    assert!(!o.is_creature());
}
