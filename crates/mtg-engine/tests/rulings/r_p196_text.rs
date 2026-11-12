//! Rulings batch P196 — text-changing effects (CR 612) with no duration: which words they
//! change (color words in the text box, including "non-" words, but not names, mana
//! symbols, keywords or words granted by other effects), how long they last, and what a
//! later text change can't reach.

use crate::r_p196_common::*;
use crate::r_s21_common::legal_blocks;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::Color;
use mtg_engine::*;

/// The protection filters of the permanent now, as debug strings.
fn protections(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Protection)
        .map(|k| format!("{:?}", k.filter))
        .collect()
}

#[test]
fn sleight_of_mind_can_target_a_card_without_color_words() {
    cr!("612.2", "115.1");
    ruling!("Sleight of Mind", "Can target a card with no color words on it.");
    supported("Sleight of Mind");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    choose_words(&mut t, P0, &["black", "red"]);
    let s = cast_at(&mut t, P0, "Sleight of Mind", &[obj(bears)]);
    assert_eq!(crate::r_s25_common::targets_of(&t, s), vec![obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sleight of Mind"));
    assert_eq!(t.pt(bears), (2, 2));
    assert!(t.obj_now(bears).chars.has_subtype("Bear"));
}

#[test]
fn sleight_of_mind_must_choose_a_different_color_word() {
    cr!("612.2");
    ruling!(
        "Sleight of Mind",
        "Can’t change a color word to the same color word. It must be a different word."
    );
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let seen = choose_words(&mut t, P0, &["black", "black", "red"]);
    cast_at(&mut t, P0, "Sleight of Mind", &[obj(knight)]);
    t.resolve_all();
    let lists = seen.lock().unwrap().clone();
    assert_eq!(lists.len(), 2);
    assert!(lists[0].iter().any(|w| w == "black"));
    assert!(!lists[1].iter().any(|w| w == "black"));
    assert_eq!(protections(&t, knight), vec!["Some(Color(Red))".to_string()]);
}

#[test]
fn glamerdye_changes_color_words_but_not_names_or_mana_symbols() {
    cr!("612.2", "201.2", "107.4");
    ruling!(
        "Glamerdye",
        "Glamerdye affects each instance of the chosen word in the text box of the spell or permanent, as long as it's being used as a color word. Glamerdye won't change such words in card names, such as White Knight. It also won't change mana symbols."
    );
    supported("Glamerdye");
    supported("Paragon of Gathering Mists");
    // White Knight (protection from black): white -> blue changes nothing in its name.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    choose_words(&mut t, P0, &["white", "blue"]);
    cast_at(&mut t, P0, "Glamerdye", &[obj(knight)]);
    t.resolve_all();
    assert_eq!(t.obj_now(knight).chars.name, "White Knight");
    assert_eq!(protections(&t, knight), vec!["Some(Color(Black))".to_string()]);
    // Paragon of Gathering Mists: "Other blue creatures you control get +1/+1. {U}, {T}:
    // Another target blue creature you control gains flying until end of turn."
    // blue -> red: red creatures get +1/+1, and the ability still costs {U}.
    let mut t = TestGame::new(2);
    let paragon = t.battlefield(P0, "Paragon of Gathering Mists");
    let blue = t.battlefield(P0, "Coral Merfolk");
    let red = t.battlefield(P0, "Hill Giant");
    assert_eq!((t.pt(blue), t.pt(red)), ((3, 2), (3, 3)));
    choose_words(&mut t, P0, &["blue", "red"]);
    cast_at(&mut t, P0, "Glamerdye", &[obj(paragon)]);
    t.resolve_all();
    assert_eq!((t.pt(blue), t.pt(red)), ((2, 1), (4, 4)));
    // Activating it takes blue mana and targets the red creature.
    t.g.players[0].mana_pool.empty();
    mana(&mut t, P0, mtg_engine::mana::ManaType::R, 2);
    let r = crate::r_s06_common::activate_containing(&mut t, P0, paragon, "{T}");
    assert!(r.is_err(), "{{U}} can't be paid with red mana");
    mana(&mut t, P0, mtg_engine::mana::ManaType::U, 1);
    t.answer_targets(P0, &[obj(red)]);
    crate::r_s06_common::activate_containing(&mut t, P0, paragon, "{T}").unwrap();
    t.resolve_all();
    assert!(t.obj_now(red).has_keyword(KeywordKind::Flying));
}

#[test]
fn glamerdye_changes_non_color_words() {
    cr!("612.2", "105.2");
    ruling!(
        "Glamerdye",
        "Glamerdye can change color words preceded by \"non.\" For example, if you have it change \"red\" to \"green,\" it will also change \"nonred\" to \"nongreen.\""
    );
    supported("Soot Imp");
    // Soot Imp: "Whenever a player casts a nonblack spell, that player loses 1 life."
    // black -> green: "nongreen".
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Soot Imp");
    choose_words(&mut t, P0, &["black", "green"]);
    cast_at(&mut t, P0, "Glamerdye", &[obj(imp)]);
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    // A green spell: no trigger.
    cast_at(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // A black spell now triggers.
    cast_at(&mut t, P1, "Disfigure", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn glamerdye_cant_affect_fear() {
    cr!("612.2", "702.36b");
    ruling!("Glamerdye", "Glamerdye can't affect the keyword fear.");
    supported("Severed Legion");
    // Severed Legion has fear; black -> white leaves it alone.
    let mut t = TestGame::new(2);
    let legion = t.battlefield(P1, "Severed Legion");
    choose_words(&mut t, P0, &["black", "white"]);
    cast_at(&mut t, P0, "Glamerdye", &[obj(legion)]);
    t.resolve_all();
    assert!(t.obj_now(legion).has_keyword(KeywordKind::Fear));
    let white = t.battlefield(P0, "Savannah Lions");
    let corpse = t.battlefield(P0, "Walking Corpse");
    t.set_step(P1, mtg_engine::turn::Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(legion, pl(P0))]);
    // Only the black creature may block it.
    assert!(!legal_blocks(&mut t, P0, &[(white, legion)]));
    assert!(legal_blocks(&mut t, P0, &[(corpse, legion)]));
}

#[test]
fn glamerdye_changes_only_printed_and_copied_words() {
    cr!("612.1", "612.2", "707.2");
    ruling!(
        "Glamerdye",
        "Glamerdye changes only words actually printed on the spell or permanent (if it's a card), words that are part of the spell or permanent's original characteristics as stated by the effect that created it (if it's a token or a copy of a spell), or words that were granted to the spell or permanent as part of a copy effect. It won't change words that are granted to the spell or permanent by any other effects. For example, if an Aura grants a creature protection from blue, you can change that to protection from red by targeting the Aura, but not by targeting the creature. Glamerdye also can't change what color was chosen for a permanent (such as Painter's Servant) as it entered."
    );
    supported("Red Ward");
    supported("Painter's Servant");
    // An Aura grants the Bears protection from a color; targeting the creature changes
    // nothing (CR 612.3). (Red Ward rather than Blue Ward: Glamerdye, a blue spell, can't
    // target a creature with protection from blue.)
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ward = attach_new(&mut t, P1, "Red Ward", bears);
    choose_words(&mut t, P0, &["red", "green"]);
    let s = cast_at(&mut t, P0, "Glamerdye", &[obj(bears)]);
    assert_eq!(crate::r_s25_common::targets_of(&t, s), vec![obj(bears)]);
    t.resolve_all();
    assert_eq!(protections(&t, bears), vec!["Some(Color(Red))".to_string()]);
    // Targeting the Aura does.
    choose_words(&mut t, P0, &["red", "green"]);
    let s = cast_at(&mut t, P0, "Glamerdye", &[obj(ward)]);
    assert_eq!(crate::r_s25_common::targets_of(&t, s), vec![obj(ward)]);
    t.resolve_all();
    assert_eq!(protections(&t, bears), vec!["Some(Color(Green))".to_string()]);
    // A Clone copying White Knight: the copied words change.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    t.answer_choose(P1, &[obj(knight)]);
    t.answer_yes(P1, true);
    let clone = t.enter(P1, "Clone");
    t.settle();
    assert_eq!(protections(&t, clone), vec!["Some(Color(Black))".to_string()]);
    choose_words(&mut t, P0, &["black", "green"]);
    cast_at(&mut t, P0, "Glamerdye", &[obj(clone)]);
    t.resolve_all();
    assert_eq!(protections(&t, clone), vec!["Some(Color(Green))".to_string()]);
    assert_eq!(protections(&t, knight), vec!["Some(Color(Black))".to_string()]);
    // Painter's Servant with blue chosen keeps blue whatever words change.
    let mut t = TestGame::new(2);
    choose_color(&mut t, P1, Color::Blue);
    let servant = t.enter(P1, "Painter's Servant");
    t.settle();
    let card = t.hand(P1, "Grizzly Bears");
    assert!(t.obj_now(card).chars.colors.contains(Color::Blue));
    choose_words(&mut t, P0, &["blue", "red"]);
    cast_at(&mut t, P0, "Glamerdye", &[obj(servant)]);
    t.resolve_all();
    t.g.recompute();
    let c = t.obj_now(card).chars.colors;
    assert!(c.contains(Color::Blue) && !c.contains(Color::Red));
}

#[test]
fn glamerdye_lasts_and_follows_a_permanent_spell_onto_the_battlefield() {
    cr!("611.2a", "612.2", "400.7");
    ruling!(
        "Glamerdye",
        "Glamerdye's effect has no duration. It will last until the game ends or the affected spell or permanent leaves the relevant zone, with the following exception: If Glamerdye affects an artifact, creature, enchantment, or planeswalker spell, it will also affect the permanent that spell becomes when it resolves."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let spell = cast_at(&mut t, P1, "White Knight", &[]);
    choose_words(&mut t, P0, &["black", "red"]);
    cast_at(&mut t, P0, "Glamerdye", &[obj(spell)]);
    t.resolve_all();
    let knight = t.named_on_battlefield("White Knight")[0];
    assert_eq!(protections(&t, knight), vec!["Some(Color(Red))".to_string()]);
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert_eq!(protections(&t, knight), vec!["Some(Color(Red))".to_string()]);
    // It ends when the permanent leaves the battlefield.
    crate::r_p189_common::bounce(&mut t, knight);
    let card = t.g.find_in_zone(mtg_engine::object::Zone::Hand(P1), "White Knight")[0];
    assert!(has_ability(&t, card, "black"));
}

#[test]
fn artificial_evolution_cant_reach_a_type_granted_by_an_effect() {
    cr!("612.2", "613.1c", "613.1d");
    ruling!(
        "Artificial Evolution",
        "If a creature's creature type has been changed by a spell or ability, changing the card's text won't affect or alter the new creature type."
    );
    // Olivia Voldaren makes the Hill Giant a Vampire; Vampire -> Elf on the Giant doesn't
    // touch that.
    let mut t = TestGame::new(2);
    let olivia = t.battlefield(P0, "Olivia Voldaren");
    let giant = t.battlefield(P1, "Hill Giant");
    act(&mut t, P0, olivia, 0, &[obj(giant)]);
    assert!(t.obj_now(giant).chars.has_subtype("Vampire"));
    choose_words(&mut t, P0, &["Vampire", "Elf"]);
    cast_at(&mut t, P0, "Artificial Evolution", &[obj(giant)]);
    t.resolve_all();
    let c = &t.obj_now(giant).chars;
    assert!(c.has_subtype("Vampire") && c.has_subtype("Giant"));
    assert!(!c.has_subtype("Elf"));
    // A printed type changes: Giant -> Elf.
    choose_words(&mut t, P0, &["Giant", "Elf"]);
    cast_at(&mut t, P0, "Artificial Evolution", &[obj(giant)]);
    t.resolve_all();
    let c = &t.obj_now(giant).chars;
    assert!(c.has_subtype("Vampire") && c.has_subtype("Elf"));
    assert!(!c.has_subtype("Giant"));
}
