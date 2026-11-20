//! Item `rulings-residue-shapechange-typal` — interaction rulings about tokens that copy a
//! card with exceptions (Leonardo da Vinci, Anikthea, Hand of Erebos): the copiable values
//! are the card's printed ones plus the exceptions (CR 707.2, 707.9b), "enters" abilities of
//! the copied card work (CR 707.5, 614.1c), and a value locked in as an ability resolves
//! (CR 608.2h).

use crate::r_residue_shapechange_typal_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Leonardo da Vinci draws and then discards `discard` (in P0's hand) with its second
/// ability, which resolves. Returns the token created, if any.
fn leonardo_loots(t: &mut TestGame, leo: ObjectId, discard: ObjectId) -> Option<ObjectId> {
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let before: Vec<ObjectId> = crate::r_s01_common::tokens(t, P0);
    t.answer_choose(P0, &[obj(discard)]);
    t.activate(P0, leo, 1, &[]).unwrap();
    t.resolve_all();
    crate::r_s01_common::tokens(t, P0)
        .into_iter()
        .find(|id| !before.contains(id))
}

#[test]
fn leonardo_token_copies_the_printed_card_and_its_enters_abilities_work() {
    cr!("707.2", "707.9b", "707.5", "614.1c", "603.6a");
    ruling!(
        "Leonardo da Vinci",
        "Any enters-the-battlefield abilities of the copied card will trigger when the token enters the battlefield. Any “as [this artifact]” enters the battlefield or “[this artifact] enters the battlefield with” abilities of the copied card will also work."
    );
    ruling!(
        "Leonardo da Vinci",
        "Except for the listed exceptions, the token copies exactly what was printed on the discarded card and nothing else."
    );
    supported("Leonardo da Vinci");
    supported("Costume Closet");
    supported("Prophetic Prism");
    supported("Metallic Mimic");
    let mut t = main_phase(2);
    let leo = t.battlefield(P0, "Leonardo da Vinci");
    // "This artifact enters with two +1/+1 counters on it."
    let closet = t.hand(P0, "Costume Closet");
    let tok = leonardo_loots(&mut t, leo, closet).expect("a token");
    assert_eq!(name_now(&t, tok), "Costume Closet");
    assert_eq!(plus1(&t, tok), 2);
    assert_eq!(pt_now(&mut t, tok), (2, 4));
    assert!(has_color(&mut t, tok, Color::White));
    assert_eq!(t.obj_now(tok).chars.mana_value(), 2);
    assert!(crate::r_residue_type_change_common::subtype(
        &mut t, tok, "Thopter"
    ));
    assert!(is_type(&mut t, tok, CardType::Artifact));
    assert!(is_type(&mut t, tok, CardType::Creature));
    assert!(t.in_exile("Costume Closet"));
    // "When this artifact enters, draw a card."
    t.g.objects[leo.0 as usize].tapped = false;
    let prism = t.hand(P0, "Prophetic Prism");
    let hand = t.hand_size(P0);
    let tok = leonardo_loots(&mut t, leo, prism).expect("a token");
    assert_eq!(name_now(&t, tok), "Prophetic Prism");
    // Drew one, discarded the Prism, drew one from the token's trigger.
    assert_eq!(t.hand_size(P0), hand + 1);
    // "As this creature enters, choose a creature type. ... Each other creature you
    // control of the chosen type enters with an additional +1/+1 counter on it."
    t.g.objects[leo.0 as usize].tapped = false;
    let mimic = t.hand(P0, "Metallic Mimic");
    choose_creature_type(&mut t, P0, "Bear");
    let tok = leonardo_loots(&mut t, leo, mimic).expect("a token");
    assert_eq!(name_now(&t, tok), "Metallic Mimic");
    assert!(crate::r_residue_type_change_common::subtype(
        &mut t, tok, "Bear"
    ));
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(plus1(&t, bears), 1);
}

#[test]
fn leonardo_token_exceptions_are_copiable_values() {
    cr!("707.9b", "707.2", "707.3");
    ruling!(
        "Leonardo da Vinci",
        "The token is a Thopter artifact creature in addition to its other types. Its power and toughness are 0/2, and it has flying. These are copiable values of the token that other effects may copy."
    );
    supported("Clone");
    let mut t = main_phase(2);
    let leo = t.battlefield(P0, "Leonardo da Vinci");
    let closet = t.hand(P0, "Costume Closet");
    let tok = leonardo_loots(&mut t, leo, closet).expect("a token");
    assert!(crate::r_p119_common::has_kw(
        &mut t,
        tok,
        mtg_engine::keywords::KeywordKind::Flying
    ));
    // A Clone copying the token is a 0/2 Thopter artifact creature with flying too (and
    // the copied "enters with" counters apply to it).
    t.answer_choose(P0, &[obj(tok)]);
    let clone = cast_new(&mut t, P0, "Clone", &[]);
    t.resolve_all();
    let clone = t.g.current(clone);
    assert_eq!(name_now(&t, clone), "Costume Closet");
    assert!(crate::r_residue_type_change_common::subtype(
        &mut t, clone, "Thopter"
    ));
    assert!(is_type(&mut t, clone, CardType::Artifact));
    assert!(crate::r_p119_common::has_kw(
        &mut t,
        clone,
        mtg_engine::keywords::KeywordKind::Flying
    ));
    assert_eq!(plus1(&t, clone), 2);
    assert_eq!(pt_now(&mut t, clone), (2, 4));
}

#[test]
fn leonardo_x_is_locked_in_as_the_ability_resolves() {
    cr!("608.2h", "611.2d", "613.4b");
    ruling!(
        "Leonardo da Vinci",
        "The value of X is determined only as Leonardo’s first ability resolves. Once that happens, the value of X won’t change later in the turn even if the number of cards in your hand changes."
    );
    supported("Ornithopter");
    let mut t = main_phase(2);
    let leo = t.battlefield(P0, "Leonardo da Vinci");
    let thopter = t.battlefield(P0, "Ornithopter");
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 3);
    t.activate(P0, leo, 0, &[]).unwrap();
    // A card drawn in response counts: X is 4 as it resolves.
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(pt_now(&mut t, thopter), (4, 4));
    t.g.draw_cards(P0, 2);
    assert_eq!(pt_now(&mut t, thopter), (4, 4));
    let gone: Vec<ObjectId> = t.g.players[0].hand.clone();
    for c in gone {
        t.g.discard(P0, c, None);
    }
    t.settle();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(pt_now(&mut t, thopter), (4, 4));
}

/// P0's Anikthea enters, targeting `card` in P0's graveyard with its trigger, which
/// resolves (with `then` the targets of the token's own trigger, if any). Returns the
/// token created.
fn anikthea_copies_then(t: &mut TestGame, card: ObjectId, then: &[Entity]) -> ObjectId {
    supported("Anikthea, Hand of Erebos");
    let before: Vec<ObjectId> = crate::r_s01_common::tokens(t, P0);
    t.answer_targets(P0, &[obj(card)]);
    for e in then {
        t.answer_targets(P0, &[*e]);
    }
    t.enter(P0, "Anikthea, Hand of Erebos");
    t.resolve_all();
    crate::r_s01_common::tokens(t, P0)
        .into_iter()
        .find(|id| !before.contains(id))
        .expect("a token")
}

fn anikthea_copies(t: &mut TestGame, card: ObjectId) -> ObjectId {
    anikthea_copies_then(t, card, &[])
}

#[test]
fn anikthea_token_copies_the_card_not_the_permanent_it_was() {
    cr!("707.2", "400.7", "707.9b");
    ruling!(
        "Anikthea, Hand of Erebos",
        "Except for the listed exceptions, the token copies exactly what was printed on the original card and nothing else. It doesn't copy any information about the object the card was before it was put into your graveyard."
    );
    supported("Skola Grovedancer");
    let mut t = main_phase(2);
    // A green 2/2 enchantment creature with two +1/+1 counters and Giant Growth dies.
    let dancer = t.battlefield(P0, "Skola Grovedancer");
    give_plus1(&mut t, dancer, 2);
    giant_growth(&mut t, dancer);
    assert_eq!(pt_now(&mut t, dancer), (7, 7));
    destroy(&mut t, dancer);
    let card = t.g.current(dancer);
    let tok = anikthea_copies(&mut t, card);
    assert_eq!(name_now(&t, tok), "Skola Grovedancer");
    assert_eq!(plus1(&t, tok), 0);
    assert_eq!(pt_now(&mut t, tok), (3, 3));
    assert!(has_color(&mut t, tok, Color::Black));
    assert!(!has_color(&mut t, tok, Color::Green));
}

#[test]
fn anikthea_token_exceptions_are_copiable_values() {
    cr!("707.9b", "707.2", "707.3", "105.3");
    ruling!(
        "Anikthea, Hand of Erebos",
        "The token is a Zombie creature in addition to its other types and is black instead of its other colors. Its base power and toughness are 3/3. These are copiable values of the tokens that other effects may copy."
    );
    supported("Sigil of the Empty Throne");
    let mut t = main_phase(2);
    let sigil = t.graveyard(P0, "Sigil of the Empty Throne");
    let tok = anikthea_copies(&mut t, sigil);
    assert_eq!(pt_now(&mut t, tok), (3, 3));
    assert!(is_type(&mut t, tok, CardType::Enchantment));
    assert!(is_type(&mut t, tok, CardType::Creature));
    assert_eq!(creature_types(&mut t, tok), vec!["Zombie".to_string()]);
    assert!(has_color(&mut t, tok, Color::Black));
    assert!(!has_color(&mut t, tok, Color::White));
    // A Clone copying it is the same 3/3 black Zombie enchantment creature.
    t.answer_choose(P0, &[obj(tok)]);
    let clone = cast_new(&mut t, P0, "Clone", &[]);
    t.resolve_all();
    let clone = t.g.current(clone);
    assert_eq!(name_now(&t, clone), "Sigil of the Empty Throne");
    assert_eq!(pt_now(&mut t, clone), (3, 3));
    assert!(is_type(&mut t, clone, CardType::Enchantment));
    assert_eq!(creature_types(&mut t, clone), vec!["Zombie".to_string()]);
    assert!(has_color(&mut t, clone, Color::Black));
    assert!(!has_color(&mut t, clone, Color::White));
    assert!(!has_color(&mut t, clone, Color::Blue));
}

#[test]
fn anikthea_token_has_the_copied_enters_abilities() {
    cr!("707.5", "614.1c", "614.12", "603.6a");
    ruling!(
        "Anikthea, Hand of Erebos",
        "If a card copied by the token had any \"when [this permanent] enters the battlefield\" abilities, the token also has those abilities and will trigger them when it's created. Similarly, any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities that the token has copied will also work."
    );
    supported("Oblivion Ring");
    supported("Divine Intervention");
    supported("Kindred Discovery");
    // "When this enchantment enters, exile another target nonland permanent."
    let mut t = main_phase(2);
    let ring = t.graveyard(P0, "Oblivion Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    anikthea_copies_then(&mut t, ring, &[obj(bears)]);
    assert!(t.in_exile("Grizzly Bears"));
    // "This enchantment enters with two intervention counters on it."
    let mut t = main_phase(2);
    let di = t.graveyard(P0, "Divine Intervention");
    let tok = anikthea_copies(&mut t, di);
    assert_eq!(t.counters(tok, "intervention"), 2);
    // "As this enchantment enters, choose a creature type."
    let mut t = main_phase(2);
    let kd = t.graveyard(P0, "Kindred Discovery");
    choose_creature_type(&mut t, P0, "Elf");
    let tok = anikthea_copies(&mut t, kd);
    let hand = t.hand_size(P0);
    t.enter(P0, "Llanowar Elves");
    t.resolve_all();
    // Kindred Discovery's token draws for the entering Elf.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(name_now(&t, tok), "Kindred Discovery");
}
