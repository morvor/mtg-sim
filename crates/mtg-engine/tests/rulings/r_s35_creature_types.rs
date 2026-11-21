//! Rulings batch S35 — creature types and other subtypes (CR 205.3): Time Lord is one
//! two-word creature type; choosing a creature type means one existing creature type
//! (not "outlaw", not a card type, not two types); outlaws (CR 700.12); a "[subtype]
//! spell" or "[subtype] card" has that subtype; Gates and Lessons have no rules of their
//! own.

use crate::r_s01_common::{supported, with_subtype};
use crate::r_s02_common::can_cast;
use crate::r_s03_common::choice_candidates;
use crate::r_s04_common::spell_targets;
use crate::r_s05_common::enter;
use crate::r_s24_common::choose_creature_type;
use crate::r_s35_common::creature_type_options;
use mtg_engine::card::card;
use mtg_engine::deck::{self, DeckProblem};
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{is_creature_type, CardType};
use mtg_engine::*;

/// P0's Kindred Discovery ("As this enchantment enters, choose a creature type. Whenever
/// a creature you control of the chosen type enters or attacks, draw a card.") enters
/// with `ty` chosen; returns the options P0 was offered.
fn kindred_discovery(t: &mut TestGame, ty: &str) -> Vec<String> {
    let from = t.asked().len();
    choose_creature_type(t, P0, ty);
    enter(t, P0, "Kindred Discovery");
    t.resolve_all();
    let mut opts = creature_type_options(t, P0, from);
    assert_eq!(opts.len(), 1);
    opts.remove(0)
}

#[test]
fn time_lord_is_one_two_word_creature_type() {
    cr!("205.3m", "205.3e");
    ruling!(
        "The Thirteenth Doctor",
        "Unlike other creature types in Magic that are each only one word, the two words \"Time Lord\" represent a single creature subtype. Time Lord is the only two-word creature type."
    );
    ruling!(
        "The Thirteenth Doctor",
        "Neither \"Time\" nor \"Lord\" are creature types. Some older cards were printed with the subtype \"Lord,\" but all of those cards have updated Oracle card text that removed that type."
    );
    supported("The Thirteenth Doctor");
    supported("Goblin King");
    // "Legendary Creature — Time Lord Doctor": two creature types.
    let doctor = card("The Thirteenth Doctor");
    let subtypes: Vec<&str> = doctor.faces[0]
        .chars
        .subtypes
        .iter()
        .map(|s| s.as_str())
        .collect();
    assert_eq!(subtypes, vec!["Time Lord", "Doctor"]);
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "The Thirteenth Doctor");
    let c = &t.obj_now(d).chars;
    assert!(c.has_subtype("Time Lord") && c.has_subtype("Doctor"));
    assert!(!c.has_subtype("Time") && !c.has_subtype("Lord"));
    assert!(is_creature_type("Time Lord"));
    assert!(!is_creature_type("Time") && !is_creature_type("Lord"));
    // Goblin King was printed as a "Lord"; its Oracle type line is "Creature — Goblin".
    let king = card("Goblin King");
    let king: Vec<&str> = king.faces[0]
        .chars
        .subtypes
        .iter()
        .map(|s| s.as_str())
        .collect();
    assert_eq!(king, vec!["Goblin"]);
    // The only multi-word creature type.
    assert_eq!(
        mtg_engine::types::subtype_lists()
            .creature
            .iter()
            .filter(|c| c.contains(' '))
            .collect::<Vec<_>>(),
        vec!["Time Lord"]
    );
    // Neither word alone can be chosen as a creature type.
    let mut t = TestGame::new(2);
    let opts = kindred_discovery(&mut t, "Doctor");
    assert!(!opts.iter().any(|o| o == "Time" || o == "Lord"));
}

#[test]
fn you_may_choose_time_lord() {
    cr!("205.3m", "205.3e");
    ruling!(
        "The Thirteenth Doctor",
        "If an effect instructs you to choose a creature type, you may choose Time Lord."
    );
    supported("Kindred Discovery");
    let mut t = TestGame::new(2);
    let opts = kindred_discovery(&mut t, "Time Lord");
    assert!(opts.iter().any(|o| o == "Time Lord"));
    // The Thirteenth Doctor, a Time Lord, entering: draw a card.
    let hand = t.hand_size(P0);
    enter(&mut t, P0, "The Thirteenth Doctor");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn outlaw_isnt_a_creature_type_that_can_be_chosen() {
    cr!("205.3e", "700.12");
    ruling!(
        "Shoot the Sheriff",
        "Outlaw is not a creature type. If an effect asks you to choose a creature type, you can’t choose outlaw."
    );
    ruling!(
        "Mine Raider",
        "Outlaw is not a creature type. If an effect asks you to choose a creature type, you can't choose outlaw."
    );
    assert!(!is_creature_type("Outlaw"));
    let mut t = TestGame::new(2);
    let opts = kindred_discovery(&mut t, "Rogue");
    assert!(!opts.iter().any(|o| o.eq_ignore_ascii_case("outlaw")));
    for ty in ["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"] {
        assert!(opts.iter().any(|o| o == ty), "{ty}");
    }
}

#[test]
fn a_permanent_with_two_outlaw_types_is_one_outlaw() {
    cr!("700.12");
    ruling!(
        "Double Down",
        "A card, spell, or permanent is an outlaw if it has the Assassin, Mercenary, Pirate, Rogue, or Warlock creature type. It doesn't matter if it has more than one of those creature types; as long as it has at least one, it's an outlaw."
    );
    supported("Double Down");
    supported("Rathi Assassin");
    supported("Mine Raider");
    // Double Down: "Whenever you cast an outlaw spell, copy that spell." Rathi Assassin
    // (a Phyrexian Zombie Mercenary Assassin) is an outlaw spell: one copy.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Double Down");
    t.lands(P0, "Swamp", 4);
    let assassin = t.hand(P0, "Rathi Assassin");
    t.cast(P0, assassin).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Rathi Assassin").len(), 2);
    // Mine Raider: "When this creature enters, if you control another outlaw, create a
    // Treasure token." Shoot the Sheriff: "Destroy target non-outlaw creature."
    let mut t = TestGame::new(2);
    let assassin = t.battlefield(P0, "Rathi Assassin");
    enter(&mut t, P0, "Mine Raider");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    let targets = spell_targets(&mut t, P1, "Shoot the Sheriff");
    assert!(!targets.contains(&Entity::Object(assassin)));
}

#[test]
fn a_cat_warrior_is_both_a_cat_and_a_warrior() {
    cr!("205.3e", "205.3m");
    ruling!(
        "Kindred Discovery",
        "You can't choose multiple creature types, such as \"Cat Warrior.\" A Cat Warrior is both a Cat and a Warrior. It's affected by anything that affects either type and unaffected by things that affect non-Cat or non-Warrior creatures."
    );
    supported("Kindred Discovery");
    supported("Kindred Dominance");
    supported("Oreskos Swiftclaw");
    // Kindred Discovery naming Warrior: Oreskos Swiftclaw (a Cat Warrior) entering draws.
    let mut t = TestGame::new(2);
    let opts = kindred_discovery(&mut t, "Warrior");
    assert!(!opts.iter().any(|o| o == "Cat Warrior"));
    assert!(opts.iter().all(|o| !o.contains(' ') || o == "Time Lord"));
    let hand = t.hand_size(P0);
    enter(&mut t, P0, "Oreskos Swiftclaw");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Kindred Dominance: "Choose a creature type. Destroy all creatures that aren't of the
    // chosen type." Naming Cat: the Cat Warrior survives; a Lizard Warrior doesn't.
    let mut t = TestGame::new(2);
    let cat_warrior = t.battlefield(P0, "Oreskos Swiftclaw");
    let lizard = t.battlefield(P1, "Lizard Warrior");
    t.lands(P0, "Swamp", 7);
    let dominance = t.hand(P0, "Kindred Dominance");
    choose_creature_type(&mut t, P0, "Cat");
    t.cast(P0, dominance).go();
    t.resolve_all();
    assert!(t.on_battlefield(cat_warrior));
    assert!(!t.on_battlefield(lizard));
}

#[test]
fn you_must_choose_an_existing_creature_type_not_a_card_type() {
    cr!("205.3e");
    ruling!(
        "Kindred Dominance",
        "You must choose an existing creature type, such as Vampire or Cat. Card types such as \"artifact\" can't be chosen."
    );
    supported("Kindred Dominance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 7);
    let dominance = t.hand(P0, "Kindred Dominance");
    let from = t.asked().len();
    // An answer that isn't one of the creature types offered.
    t.answer(P0, DecisionKind::Option, Answer::Index(100_000));
    t.cast(P0, dominance).go();
    t.resolve_all();
    let opts = creature_type_options(&t, P0, from);
    assert_eq!(opts.len(), 1);
    for ok in ["Vampire", "Cat", "Elemental"] {
        assert!(opts[0].iter().any(|o| o == ok), "{ok}");
    }
    for bad in ["Artifact", "Creature", "Enchantment", "Kindred", "Equipment", "Forest"] {
        assert!(!opts[0].iter().any(|o| o.eq_ignore_ascii_case(bad)), "{bad}");
    }
    // Some creature type was chosen nonetheless: the Bears (not of it) were destroyed.
    assert!(!t.on_battlefield(bears));
    assert!(!t.obj_now(bears).chars.card_types.is_empty());
}

#[test]
fn a_pirate_card_is_a_card_with_the_pirate_subtype() {
    cr!("205.3", "701.23a");
    ruling!(
        "Forerunner of the Coalition",
        "If an effect refers to a “[subtype] spell” or “[subtype] card,” it refers only to a spell or card that has that subtype. For example, March of the Drowned is a card that benefits Pirates and features Pirates in its illustration, but it isn’t a Pirate card."
    );
    supported("Forerunner of the Coalition");
    // "When this creature enters, you may search your library for a Pirate card, reveal
    // it, then shuffle and put that card on top." March of the Drowned isn't a Pirate
    // card; another Forerunner (a Human Pirate) is.
    let mut t = TestGame::new(2);
    let march = t.library_top(P0, "March of the Drowned");
    let pirate = t.library_top(P0, "Forerunner of the Coalition");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    enter(&mut t, P0, "Forerunner of the Coalition");
    t.resolve_all();
    let offered: Vec<Entity> = choice_candidates(&t, from, "")
        .into_iter()
        .flatten()
        .collect();
    assert!(offered.contains(&Entity::Object(pirate)), "{offered:?}");
    assert!(!offered.contains(&Entity::Object(march)));
}

#[test]
fn a_merfolk_spell_is_a_spell_with_the_merfolk_subtype() {
    cr!("205.3", "601.2i");
    ruling!(
        "Deeproot Waters",
        "If an effect refers to a \"[subtype] spell\" or \"[subtype] card,\" it refers only to a spell or card that has that subtype. For example, March of the Drowned is a card that benefits Pirates and features Pirates in its illustration, but it isn't a Pirate card."
    );
    supported("Deeproot Waters");
    supported("Coral Merfolk");
    // "Whenever you cast a Merfolk spell, create a 1/1 blue Merfolk creature token with
    // hexproof." Another Deeproot Waters benefits Merfolk, but isn't a Merfolk spell.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deeproot Waters");
    t.lands(P0, "Island", 5);
    let waters = t.hand(P0, "Deeproot Waters");
    t.cast(P0, waters).go();
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Merfolk").is_empty());
    let merfolk = t.hand(P0, "Coral Merfolk");
    t.cast(P0, merfolk).go();
    t.resolve_all();
    // Coral Merfolk is: each Deeproot Waters (both on the battlefield now) creates a
    // token.
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Merfolk"))
            .count(),
        2
    );
}

#[test]
fn gate_has_no_rules_of_its_own_but_is_referred_to() {
    cr!("205.3i");
    ruling!(
        "Simic Guildgate",
        "The subtype Gate has no special rules significance, but other spells and abilities may refer to it."
    );
    supported("Simic Guildgate");
    supported("Gates Ablaze");
    // A Gate is a land like any other: it enters tapped (its own ability) and taps for
    // mana.
    let mut t = TestGame::new(2);
    let gates = t.lands(P0, "Simic Guildgate", 2);
    assert!(t.obj_now(gates[0]).chars.has_subtype("Gate"));
    assert!(t.obj_now(gates[0]).chars.card_types.contains(CardType::Land));
    // Gates Ablaze: "deals X damage to each creature, where X is the number of Gates you
    // control." The Guildgates pay for it ({2}) along with a Mountain.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let ablaze = t.hand(P0, "Gates Ablaze");
    t.cast(P0, ablaze).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn a_lesson_is_cast_like_any_other_sorcery() {
    cr!("205.3k", "307.1");
    ruling!(
        "Pest Summoning",
        "Lesson is a spell subtype found on some instant and sorcery cards in the Strixhaven set. The Lesson subtype has no special rules associated with it."
    );
    supported("Pest Summoning");
    // "Sorcery — Lesson": "Create two 1/1 black and green Pest creature tokens ..."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let lesson = t.hand(P0, "Pest Summoning");
    assert!(t.obj_now(lesson).chars.has_subtype("Lesson"));
    // Only when a sorcery could be cast.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, lesson, CastMethod::Normal));
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, lesson, CastMethod::Normal));
    t.cast(P0, lesson).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Pest").len(), 2);
    assert!(t.in_graveyard(P0, "Pest Summoning"));
}

#[test]
fn lessons_can_be_in_the_main_deck() {
    cr!("100.2a", "205.3k");
    ruling!(
        "Introduction to Prophecy",
        "Although you may want to include Lessons in your sideboard if you’re playing with cards that instruct you to learn, Lesson cards can be included in your main deck like other instant or sorcery cards."
    );
    supported("Introduction to Prophecy");
    // Four copies in a 60-card deck, like any other card.
    let mut deck = vec![card("Island"); 56];
    deck.extend(vec![card("Introduction to Prophecy"); 4]);
    assert!(deck::check_constructed(&deck).is_empty());
    deck.push(card("Introduction to Prophecy"));
    assert!(matches!(
        &deck::check_constructed(&deck)[..],
        [DeckProblem::TooManyCopies { have: 5, max: 4, .. }]
    ));
    // Drawn from the library, it's cast like any sorcery: "Scry 2, then draw a card."
    let mut t = TestGame::new(2);
    let lesson = t.library_top(P0, "Introduction to Prophecy");
    t.g.draw_cards(P0, 1);
    assert!(t.in_hand(P0, "Introduction to Prophecy"));
    t.lands(P0, "Island", 3);
    let hand = t.hand_size(P0);
    t.cast(P0, t.g.current(lesson)).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Introduction to Prophecy"));
}
