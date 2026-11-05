//! Rulings batch P025 — the exceptions a copy effect makes are part of the copy's copiable
//! values, so a copy of the copy has them too (CR 707.9b); choosing what to copy as a
//! permanent enters (or as a spell resolves) doesn't target (CR 115.1, 707.2); the legend
//! rule and the effects that turn it off (CR 704.5j).

use crate::r_p017_common::choice_candidates;
use crate::r_p025_common::*;
use mtg_engine::card::card;
use mtg_engine::deck::DeckProblem;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::partner::check_commander_deck;
use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
}

fn tokens_named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    named(t, name)
        .into_iter()
        .filter(|o| t.obj_now(*o).kind == ObjKind::Token)
        .collect()
}

// --- Exceptions are copiable -------------------------------------------------------------

#[test]
fn a_clone_of_a_copy_has_the_abilities_the_copy_gained() {
    cr!("707.9a", "707.3");
    ruling!(
        "Gigantoplasm",
        "The activated ability Gigantoplasm gives itself becomes part of its copiable values. Unless the ability is overwritten by another copy effect, a creature that's a copy of Gigantoplasm will have that ability."
    );
    ruling!(
        "Mercurial Pretender",
        "The activated ability Mercurial Pretender gives itself becomes part of its copiable values. Unless the ability is overwritten by another copy effect, a creature that’s a copy of Mercurial Pretender will have that ability."
    );
    ruling!(
        "Evil Twin",
        "The activated ability that Evil Twin gains as part of its copy effect is a copiable value that other effects may copy."
    );
    for (copier, ability) in [
        ("Gigantoplasm", "base power and toughness X/X"),
        ("Mercurial Pretender", "Return"),
        ("Evil Twin", "Destroy target creature with the same name"),
    ] {
        supported(copier);
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let c = enter_copying(&mut t, P0, copier, giant);
        let clone = clone_of(&mut t, P1, c);
        assert_eq!(name_of(&t, clone), "Hill Giant", "{copier}");
        assert_eq!(abilities_with(&t, clone, ability), 1, "{copier}");
        assert_eq!(t.pt(clone), (3, 3), "{copier}");
    }
}

#[test]
fn a_clone_of_an_embalmed_vizier_is_a_white_zombie_with_no_mana_cost() {
    cr!("707.9b", "702.128a");
    ruling!(
        "Vizier of Many Faces",
        "The characteristics Vizier of Many Faces gains as part of its copy effect are copiable values that other effects may copy."
    );
    supported("Vizier of Many Faces");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let giant = t.battlefield(P1, "Hill Giant");
    let vizier = t.graveyard(P0, "Vizier of Many Faces");
    t.answer_choose(P0, &[obj(giant)]);
    let before = t.g.battlefield.clone();
    t.activate(P0, vizier, 0, &[]).unwrap();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let clone = clone_of(&mut t, P1, toks[0]);
    let o = t.obj_now(clone);
    assert_eq!(o.chars.name, "Hill Giant");
    assert_eq!(o.chars.colors, ColorSet::single(Color::White));
    assert!(has_subtype(&t, clone, "Zombie") && has_subtype(&t, clone, "Giant"));
    assert!(o.chars.mana_cost.is_none());
    assert!(!o.is_token());
}

#[test]
fn a_clone_of_a_mirror_of_the_forebears_copy_is_an_artifact() {
    cr!("707.9b", "707.3");
    ruling!(
        "Mirror of the Forebears",
        "Mirror of the Forebears becomes an artifact in addition to its other types. This is a copiable value that other effects may copy."
    );
    supported("Mirror of the Forebears");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Giant");
    let mirror = t.enter(P0, "Mirror of the Forebears");
    t.settle();
    lands_for(&mut t, P0, "{1}");
    t.activate(P0, mirror, 0, &[obj(giant)]).expect("activate");
    t.resolve_all();
    let clone = clone_of(&mut t, P1, mirror);
    let o = t.obj_now(clone);
    assert_eq!(o.chars.name, "Hill Giant");
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
}

#[test]
fn spark_doubles_nonlegendary_exception_is_copiable() {
    cr!("707.9b", "707.3", "704.5j");
    ruling!(
        "Spark Double",
        "Spark Double isn’t legendary if it copies a legendary permanent, and this exception is copiable. If something else copies Spark Double later, that copy also won’t be legendary. If you control two or more permanents with the same name but only one is legendary, the “legend rule” doesn’t apply."
    );
    supported("Spark Double");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let double = enter_copying(&mut t, P0, "Spark Double", isamaru);
    let clone = clone_of(&mut t, P0, double);
    // Three Isamarus, only one of them legendary: all stay.
    for id in [isamaru, double, clone] {
        assert!(t.on_battlefield(id));
        assert_eq!(name_of(&t, id), "Isamaru, Hound of Konda");
    }
    assert!(legendary(&t, isamaru));
    assert!(!legendary(&t, double) && !legendary(&t, clone));
    // The Clone doesn't get Spark Double's +1/+1 counter (that's an "enters with").
    assert_eq!(t.pt(double), (3, 3));
    assert_eq!(t.pt(clone), (2, 2));
}

#[test]
fn rikus_token_has_the_copiable_values_of_the_creature_it_copies() {
    cr!("707.2", "707.9b", "707.3");
    ruling!(
        "Riku of Two Reflections",
        "The copiable values of the token's characteristics are the same as the copiable values of the characteristics of the creature it's copying."
    );
    supported("Riku of Two Reflections");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riku of Two Reflections");
    let angel = t.battlefield(P1, "Serra Angel");
    lands_for(&mut t, P0, "{G}{U}");
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    // A Phantasmal Image enters as a copy of the Angel; Riku copies the Image.
    let image = enter_copying(&mut t, P0, "Phantasmal Image", angel);
    t.resolve_all();
    let toks: Vec<_> = new_tokens(&t, P0, &before)
        .into_iter()
        .filter(|x| *x != image)
        .collect();
    assert_eq!(toks.len(), 1);
    assert_eq!(name_of(&t, toks[0]), "Serra Angel");
    assert!(has_subtype(&t, toks[0], "Illusion") && has_subtype(&t, toks[0], "Angel"));
    assert_eq!(abilities_with(&t, toks[0], "sacrifice it"), 1);
    assert_eq!(t.pt(toks[0]), (4, 4));
}

#[test]
fn irenicuss_vile_duplication_token_has_flying_whether_or_not_it_was_legendary() {
    cr!("707.9a", "707.9b");
    ruling!(
        "Irenicus's Vile Duplication",
        "Irenicus's Vile Duplication has received an update to its Oracle text to clarify that the token still has flying even if the creature it's copying isn't legendary."
    );
    for (what, pt) in [("Hill Giant", (3, 3)), ("Isamaru, Hound of Konda", (2, 2))] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, what);
        let toks = cast_for_tokens(
            &mut t,
            P0,
            "Irenicus's Vile Duplication",
            "{3}{U}",
            &[obj(c)],
        );
        assert_eq!(toks.len(), 1, "{what}");
        assert_eq!(name_of(&t, toks[0]), what);
        assert!(has_kw(&t, toks[0], KeywordKind::Flying), "{what}");
        assert!(!legendary(&t, toks[0]), "{what}");
        assert_eq!(t.pt(toks[0]), pt);
        assert!(t.on_battlefield(c));
    }
}

// --- Not targeted --------------------------------------------------------------------------

#[test]
fn entering_as_a_copy_doesnt_target() {
    cr!("707.2", "614.1c", "702.18a", "702.11b");
    ruling!(
        "Mercurial Pretender",
        "Mercurial Pretender’s ability doesn’t target the chosen creature."
    );
    ruling!(
        "Pirated Copy",
        "Pirated Copy's ability doesn't target the creature."
    );
    ruling!(
        "Dack's Duplicate",
        "The ability of Dack's Duplicate doesn't target the creature."
    );
    ruling!(
        "Machine God's Effigy",
        "The ability of Machine God’s Effigy doesn’t target the creature."
    );
    // Mercurial Pretender copies a creature you control: one with shroud. The others copy
    // an opponent's creature with hexproof.
    for (copier, what, owner) in [
        ("Mercurial Pretender", "Blastoderm", P0),
        ("Pirated Copy", "Gladecover Scout", P1),
        ("Dack's Duplicate", "Gladecover Scout", P1),
        ("Machine God's Effigy", "Gladecover Scout", P1),
    ] {
        supported(copier);
        let mut t = TestGame::new(2);
        let c = t.battlefield(owner, what);
        let copy = enter_copying(&mut t, P0, copier, c);
        assert_eq!(name_of(&t, copy), what, "{copier}");
    }
}

#[test]
fn renegade_doppelganger_can_copy_a_creature_with_shroud_or_protection() {
    cr!("707.2", "702.18a", "702.16b");
    ruling!(
        "Renegade Doppelganger",
        "Renegade Doppelganger's ability isn't targeted. It can copy a creature with shroud or protection."
    );
    supported("Renegade Doppelganger");
    // Blastoderm has shroud, Yavimaya Barbarian protection from blue (the Doppelganger is
    // blue), Beloved Chaplain protection from creatures.
    for what in ["Blastoderm", "Yavimaya Barbarian", "Beloved Chaplain"] {
        let mut t = TestGame::new(2);
        let dopp = t.battlefield(P0, "Renegade Doppelganger");
        t.answer_yes(P0, true);
        t.enter(P0, what);
        t.settle();
        t.resolve_all();
        assert_eq!(name_of(&t, dopp), what);
    }
}

#[test]
fn cytoshape_chooses_what_to_copy_as_it_resolves() {
    cr!("707.2", "608.2d", "115.1");
    ruling!(
        "Cytoshape",
        "Only the creature that’s becoming a copy is targeted. The creature that it will copy isn’t chosen until Cytoshape resolves."
    );
    supported("Cytoshape");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{1}{G}{U}");
    let spell = t.hand(P0, "Cytoshape");
    let from = t.asked().len();
    t.cast(P0, spell).target(obj(bears)).go();
    assert!(choice_candidates(&t, from).is_empty());
    // In response, P1's Gladecover Scout (hexproof) enters; it's chosen on resolution.
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.answer_choose(P0, &[obj(scout)]);
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Gladecover Scout");
    assert_eq!(t.pt(bears), (1, 1));
}

// --- The legend rule -----------------------------------------------------------------------

#[test]
fn clone_legion_copying_your_legendary_creature_and_the_legend_rule() {
    cr!("704.5j", "707.2");
    ruling!(
        "Clone Legion",
        "Remember that if you control more than one legendary permanent with the same name, you'll choose one to remain on the battlefield and put the rest into their owners' graveyards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.battlefield(P0, "Hill Giant");
    cast_for_tokens(
        &mut t,
        P0,
        "Clone Legion",
        "{7}{U}{U}",
        &[Entity::Player(P0)],
    );
    assert_eq!(named(&t, "Isamaru, Hound of Konda").len(), 1);
    assert_eq!(named(&t, "Hill Giant").len(), 2);
}

#[test]
fn cadric_the_legend_rule_doesnt_apply_to_tokens() {
    cr!("704.5j");
    ruling!(
        "Cadric, Soul Kindler",
        "If, while you control Cadric, Soul Kindler, you also control a token legendary permanent and a single nontoken legendary permanent with the same name, the legend rule will not cause you to put either one into your graveyard."
    );
    ruling!(
        "Cadric, Soul Kindler",
        "If, while you control Cadric, Soul Kindler, you also control more than one nontoken legendary permanent with the same name and at least one token legendary permanent with that name, the legend rule applies to only the nontoken permanents."
    );
    supported("Cadric, Soul Kindler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cadric, Soul Kindler");
    // Isamaru enters; Cadric's ability makes a token copy of it.
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.enter(P0, "Isamaru, Hound of Konda");
    t.settle();
    t.resolve_all();
    let name = "Isamaru, Hound of Konda";
    assert_eq!(named(&t, name).len(), 2);
    assert_eq!(tokens_named(&t, name).len(), 1);
    // A second nontoken Isamaru: one of the two nontoken ones goes, the token stays.
    t.battlefield(P0, name);
    t.settle();
    assert_eq!(named(&t, name).len(), 2);
    assert_eq!(tokens_named(&t, name).len(), 1);
    assert!(t.in_graveyard(P0, name));
}

#[test]
fn sakashima_of_a_thousand_faces_turns_off_the_legend_rule_for_you_only() {
    cr!("704.5j");
    ruling!(
        "Sakashima of a Thousand Faces",
        "The \"legend rule\" is the rule that states that if a player controls two or more legendary permanents with the same name, that player chooses one of them, and the rest are put into their owners' graveyards."
    );
    supported("Sakashima of a Thousand Faces");
    let mut t = TestGame::new(2);
    let name = "Isamaru, Hound of Konda";
    let isamaru = t.battlefield(P0, name);
    let saka = enter_copying(&mut t, P0, "Sakashima of a Thousand Faces", isamaru);
    assert_eq!(name_of(&t, saka), name);
    assert!(t.on_battlefield(isamaru) && t.on_battlefield(saka));
    // P1 controls two Isamarus and no Sakashima.
    t.battlefield(P1, name);
    t.battlefield(P1, name);
    t.settle();
    assert_eq!(named(&t, name).len(), 3);
    assert!(t.in_graveyard(P1, name));
}

#[test]
fn sakashima_of_a_thousand_faces_partner_color_identity() {
    cr!("702.124c", "903.4");
    ruling!(
        "Sakashima of a Thousand Faces",
        "If your Commander deck has two commanders, you can only include cards whose own color identities are also found in your commanders' combined color identities."
    );
    // Sakashima is blue, Tymna white-black.
    let pair = vec![
        card("Sakashima of a Thousand Faces"),
        card("Tymna the Weaver"),
    ];
    let mut d = pair.clone();
    d.extend((0..95).map(|_| card("Island")));
    d.extend(["Plains", "Swamp", "Island"].map(card));
    assert!(check_commander_deck(&d, &pair, &[], false).is_empty());
    d.pop();
    d.push(card("Mountain"));
    assert!(check_commander_deck(&d, &pair, &[], false).contains(
        &DeckProblem::OutsideColorIdentity {
            name: "Mountain".into()
        }
    ));
}

#[test]
fn machine_gods_effigy_copies_legendary_but_not_creature_types() {
    cr!("707.9b", "704.5j", "205.3d");
    ruling!(
        "Machine God's Effigy",
        "Machine God’s Effigy will not copy any types (such as creature or enchantment) or non-artifact subtypes (such as Phyrexian or Elf) that creature had, but it will copy any supertypes, such as legendary, and artifact subtypes (such as Treasure or Vehicle). It also has the same name, so if you copy a legendary creature you control, you will need to choose one of them to put in your graveyard due to the legend rule."
    );
    supported("Machine God's Effigy");
    // Lizard Blades: an Equipment Lizard artifact creature.
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P1, "Lizard Blades");
    let e = enter_copying(&mut t, P0, "Machine God's Effigy", blades);
    assert!(has_subtype(&t, e, "Equipment") && !has_subtype(&t, e, "Lizard"));
    assert!(!is_creature(&t, e));
    // A legendary creature you control: the Effigy is a legendary artifact named Isamaru,
    // and the legend rule leaves one of them.
    let mut t = TestGame::new(2);
    let name = "Isamaru, Hound of Konda";
    let isamaru = t.battlefield(P0, name);
    t.answer_choose(P0, &[obj(isamaru)]);
    let e = t.enter(P0, "Machine God's Effigy");
    let e_now = t.g.current(e);
    assert!(legendary(&t, e_now) && !has_subtype(&t, e_now, "Dog"));
    t.settle();
    assert_eq!(named(&t, name).len(), 1);
    assert!(t.in_graveyard(P0, name) || t.in_graveyard(P0, "Machine God's Effigy"));
}

#[test]
fn sakashima_the_impostor_cant_copy_itself_and_gets_its_ability_only_as_a_copy() {
    cr!("707.9a", "614.12");
    ruling!(
        "Sakashima the Impostor",
        "Sakashima doesn't get the return to hand ability unless a creature is copied as Sakashima enters. Sakashima can't copy itself because the choice is made as part of the event that puts it onto the battlefield, and whatever will be copied must already be on the battlefield."
    );
    supported("Sakashima the Impostor");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(giant)]);
    let s = t.enter(P0, "Sakashima the Impostor");
    t.settle();
    let offered = choice_candidates(&t, from);
    assert!(!offered.is_empty());
    assert!(offered.iter().all(|c| !c.contains(&obj(s))));
    let s = t.g.current(s);
    assert_eq!(name_of(&t, s), "Sakashima the Impostor");
    assert_eq!(activated_with(&t, s, "Return"), 1);
    // Entering as itself (nothing chosen): no return ability.
    let mut t = TestGame::new(2);
    t.answer_choose(P0, &[]);
    let s = t.enter(P0, "Sakashima the Impostor");
    t.settle();
    let s = t.g.current(s);
    assert_eq!(name_of(&t, s), "Sakashima the Impostor");
    assert_eq!(activated_with(&t, s, "Return"), 0);
}

#[test]
fn shameless_charlatan_commander_that_becomes_a_copy_is_still_your_commander() {
    cr!("903.3", "707.2", "613.1a", "613.1f");
    ruling!(
        "Shameless Charlatan",
        "If your commander becomes a copy of something else while you control Shameless Charlatan, it will still be your commander and it will still have the ability granted to it."
    );
    supported("Shameless Charlatan");
    let mut t = crate::r_s13_common::commander_game();
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Shameless Charlatan");
    let cmdr = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.players[0]
        .commander_names
        .push("Isamaru, Hound of Konda".into());
    t.g.recompute();
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{2}{U}");
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, cmdr, "becomes a copy").expect("activation");
    t.resolve_all();
    let o = t.obj_now(cmdr);
    assert_eq!(o.chars.name, "Hill Giant");
    assert!(o.is_commander);
    assert_eq!(
        abilities_with(&t, cmdr, "becomes a copy of another target creature"),
        1
    );
}
