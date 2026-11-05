//! Rulings batch P026 — which parts of a copy effect are copiable values that a later copy
//! copies (CR 707.9a, 707.9b: the exceptions of the copy effect) and which aren't (effects
//! that come with the copy but aren't part of the copy effect: "It gains haste", delayed
//! triggers, kicker). The later copy is a Clone entering as a copy of the token.

use crate::r_p026_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Advances from P0's main phase to P0's end step and resolves its triggers.
fn to_end_step(t: &mut TestGame) {
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
}

#[test]
fn minion_reflector_haste_and_sacrifice_trigger_are_copiable() {
    cr!("707.9a", "707.3", "603.7");
    ruling!(
        "Minion Reflector",
        "That means if something becomes a copy of the token, it will also have haste and it will also have to be sacrificed at the end of the turn."
    );
    supported("Minion Reflector");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Minion Reflector");
    lands_for(&mut t, P0, "{2}");
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    let giant = t.enter(P0, "Hill Giant");
    t.settle();
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert!(has_kw(&t, tok, KeywordKind::Haste));
    // A Clone entering as a copy of the token has haste and the sacrifice trigger too.
    let c = clone_of(&mut t, P0, tok);
    assert!(has_kw(&t, c, KeywordKind::Haste));
    assert_eq!(abilities_with(&t, c, "sacrifice"), 1);
    assert!(!has_kw(&t, giant, KeywordKind::Haste));
    to_end_step(&mut t);
    assert!(!t.on_battlefield(tok) && !t.on_battlefield(c));
    assert!(t.in_graveyard(P0, "Clone"));
    assert!(t.on_battlefield(giant));
}

#[test]
fn saheeli_sublime_artificer_artifact_exception_is_copiable() {
    cr!("707.9b", "707.3", "606.3");
    ruling!(
        "Saheeli, Sublime Artificer",
        "The target artifact is an artifact in addition to whatever types the second target has, and this exception is copiable. If something else copies the artifact later in the turn, that copy also will be an artifact."
    );
    supported("Saheeli, Sublime Artificer");
    let mut t = TestGame::new(2);
    let saheeli = t.battlefield(P0, "Saheeli, Sublime Artificer");
    let thopter = t.battlefield(P0, "Ornithopter");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(thopter)]);
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, saheeli, "becomes a copy").expect("activate");
    t.resolve_all();
    assert_eq!(name_of(&t, thopter), "Hill Giant");
    assert!(t.obj_now(thopter).is(CardType::Artifact));
    let c = clone_of(&mut t, P0, thopter);
    assert_eq!(name_of(&t, c), "Hill Giant");
    assert!(t.obj_now(c).is(CardType::Artifact) && is_creature(&t, c));
    // The original Giant isn't an artifact.
    assert!(!t.obj_now(giant).is(CardType::Artifact));
}

/// P0's Isamaru (a legendary 2/2) and a token copy of it, which isn't legendary, made by
/// `make`. Then a Clone copies the token. Returns (Isamaru, token, Clone).
fn nonlegendary_token_then_clone(
    t: &mut TestGame,
    make: fn(&mut TestGame, ObjectId) -> ObjectId,
) -> (ObjectId, ObjectId, ObjectId) {
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let tok = make(t, isamaru);
    assert_eq!(name_of(t, tok), "Isamaru, Hound of Konda");
    assert!(!legendary(t, tok) && legendary(t, isamaru));
    let c = clone_of(t, P0, tok);
    assert_eq!(name_of(t, c), "Isamaru, Hound of Konda");
    assert!(!legendary(t, c));
    // Only one of the three is legendary: the legend rule doesn't apply.
    t.settle();
    assert!(t.on_battlefield(isamaru) && t.on_battlefield(tok) && t.on_battlefield(c));
    (isamaru, tok, c)
}

fn helm_token(t: &mut TestGame, isamaru: ObjectId) -> ObjectId {
    attach_new(t, P0, "Helm of the Host", isamaru);
    t.set_step(P0, Step::PrecombatMain);
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    one(new_tokens(t, P0, &before))
}

#[test]
fn helm_of_the_host_nonlegendary_is_copiable_and_haste_isnt() {
    cr!("707.9b", "707.3", "704.5j");
    ruling!(
        "Helm of the Host",
        "The token isn't legendary, and this exception is copiable. If something else copies the token later, that copy also won't be legendary. If you control two or more permanents with the same name but only one is legendary, the \"legend rule\" doesn't apply."
    );
    ruling!(
        "Helm of the Host",
        "The token gains haste indefinitely, and this effect isn't copiable. If something else copies the token later, that copy won't have haste."
    );
    supported("Helm of the Host");
    let mut t = TestGame::new(2);
    let (_, tok, c) = nonlegendary_token_then_clone(&mut t, helm_token);
    assert!(has_kw(&t, tok, KeywordKind::Haste));
    assert!(!has_kw(&t, c, KeywordKind::Haste));
    // "Indefinitely": the token still has haste on a later turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(has_kw(&t, tok, KeywordKind::Haste));
}

#[test]
fn vesuvan_duplimancy_nonlegendary_is_copiable() {
    cr!("707.9b", "707.3", "704.5j");
    ruling!(
        "Vesuvan Duplimancy",
        "The token isn’t legendary, and this exception is copiable. If something else copies the token later, that copy also won’t be legendary. If you control two or more permanents with the same name but only one is legendary, the “legend rule” doesn’t apply."
    );
    supported("Vesuvan Duplimancy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vesuvan Duplimancy");
    nonlegendary_token_then_clone(&mut t, |t, isamaru| {
        let before = t.g.battlefield.clone();
        crate::r_p021_common::giant_growth(t, P0, isamaru);
        one(new_tokens(t, P0, &before))
    });
}

#[test]
fn ardyn_token_demon_exception_is_copiable() {
    cr!("707.9b", "707.3");
    ruling!(
        "Ardyn, the Usurper",
        "The token is a Demon creature instead of its other types and is black instead of its other colors. Its base power and toughness are 5/5. These are copiable values of the token that other effects may copy."
    );
    // Starscourge — "At the beginning of combat on your turn, exile up to one target
    // creature card from a graveyard. If you exiled a card this way, create a token that's a
    // copy of that card, except it's a 5/5 black Demon."
    supported("Ardyn, the Usurper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ardyn, the Usurper");
    let card = t.graveyard(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(card)]);
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    let c = clone_of(&mut t, P0, tok);
    for id in [tok, c] {
        assert_eq!(name_of(&t, id), "Hill Giant");
        assert_eq!(t.obj_now(id).chars.colors, ColorSet::single(Color::Black));
        assert!(has_subtype(&t, id, "Demon") && !has_subtype(&t, id, "Giant"));
        assert_eq!(t.pt(id), (5, 5));
    }
}

#[test]
fn abyssal_harvester_nightmare_exception_is_copiable() {
    cr!("707.9b", "707.3");
    ruling!(
        "Abyssal Harvester",
        "The token is a Nightmare in addition to its other creature types. This is a copiable value of the token that other effects may copy."
    );
    // "{T}: Exile target creature card from a graveyard that was put there this turn.
    // Create a token that's a copy of it, except it's a Nightmare in addition to its other
    // types. Then exile all other Nightmare tokens you control."
    supported("Abyssal Harvester");
    let mut t = TestGame::new(2);
    let harvester = t.battlefield(P0, "Abyssal Harvester");
    let giant = t.battlefield(P1, "Hill Giant");
    kill(&mut t, giant);
    let card = t.g.current(giant);
    let tok = one(activate_for_tokens(
        &mut t,
        P0,
        harvester,
        "Exile target",
        &[obj(card)],
    ));
    let c = clone_of(&mut t, P0, tok);
    for id in [tok, c] {
        assert_eq!(name_of(&t, id), "Hill Giant");
        assert!(has_subtype(&t, id, "Nightmare") && has_subtype(&t, id, "Giant"));
    }
}

#[test]
fn sauron_wraith_exception_is_copiable() {
    cr!("707.9b", "707.3", "508.4");
    ruling!(
        "Sauron, the Necromancer",
        "The token is a Wraith instead of its other creature types (and other subtypes if it has any) and is black instead of its other colors. Its base power and toughness are 3/3. These are copiable values of the token that other effects may copy."
    );
    // "Whenever Sauron attacks, exile target creature card from your graveyard. Create a
    // tapped and attacking token that's a copy of that card, except it's a 3/3 black Wraith
    // with menace." Serra Angel: a white 4/4 Angel.
    supported("Sauron, the Necromancer");
    let mut t = TestGame::new(2);
    let sauron = t.battlefield(P0, "Sauron, the Necromancer");
    let card = t.graveyard(P0, "Serra Angel");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(card)]);
    let before = t.g.battlefield.clone();
    t.attack(&[(sauron, Entity::Player(P1))], &[]);
    let tok = one(new_tokens(&t, P0, &before));
    let c = clone_of(&mut t, P0, tok);
    for id in [tok, c] {
        assert_eq!(name_of(&t, id), "Serra Angel");
        assert_eq!(t.obj_now(id).chars.colors, ColorSet::single(Color::Black));
        assert!(has_subtype(&t, id, "Wraith") && !has_subtype(&t, id, "Angel"));
        assert!(has_kw(&t, id, KeywordKind::Menace) && has_kw(&t, id, KeywordKind::Flying));
        assert_eq!(t.pt(id), (3, 3));
    }
}

#[test]
fn saheeli_rai_artifact_is_copiable_but_haste_and_exile_arent() {
    cr!("707.9b", "707.3", "603.7");
    ruling!(
        "Saheeli Rai",
        "The token is an artifact in addition to its other types. This is a copiable value of the token that other effects may copy. Haste and the ability's delayed triggered ability aren't copiable."
    );
    supported("Saheeli Rai");
    let mut t = TestGame::new(2);
    let saheeli = t.battlefield(P0, "Saheeli Rai");
    let giant = t.battlefield(P0, "Hill Giant");
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, saheeli, "Create a token").expect("activate");
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    let c = clone_of(&mut t, P0, tok);
    assert!(has_kw(&t, tok, KeywordKind::Haste) && !has_kw(&t, c, KeywordKind::Haste));
    for id in [tok, c] {
        assert_eq!(name_of(&t, id), "Hill Giant");
        assert!(t.obj_now(id).is(CardType::Artifact) && is_creature(&t, id));
    }
    // The delayed trigger exiles only the token.
    to_end_step(&mut t);
    assert!(!t.on_battlefield(tok));
    assert!(t.on_battlefield(c));
}

#[test]
fn grist_insect_exception_is_copiable() {
    cr!("707.9b", "707.3", "606.3");
    ruling!(
        "Grist, Voracious Larva // Grist, the Plague Swarm",
        "The tokens are Insects instead of their other creature types, and they're black and green instead of their other colors. These are copiable values of the tokens that other effects may copy."
    );
    // "−6: For each creature card in your graveyard, create a token that's a copy of it,
    // except it's a 1/1 black and green Insect."
    supported("Grist, Voracious Larva // Grist, the Plague Swarm");
    let mut t = TestGame::new(2);
    let grist = t.battlefield(P0, "Grist, Voracious Larva // Grist, the Plague Swarm");
    mtg_engine::dfc::transform(&mut t.g, grist);
    let grist = t.g.current(grist);
    // Seven loyalty: the −6 doesn't put Grist (a creature card) into the graveyard.
    t.g.add_counters(Entity::Object(grist), counters::LOYALTY, 7, None);
    t.g.recompute();
    t.graveyard(P0, "Serra Angel");
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, grist, "For each creature card").expect("activate");
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    let c = clone_of(&mut t, P0, tok);
    let bg = ColorSet::single(Color::Black).union(ColorSet::single(Color::Green));
    for id in [tok, c] {
        assert_eq!(name_of(&t, id), "Serra Angel");
        assert_eq!(t.obj_now(id).chars.colors, bg);
        assert!(has_subtype(&t, id, "Insect") && !has_subtype(&t, id, "Angel"));
        assert_eq!(t.pt(id), (1, 1));
    }
}

#[test]
fn offsprings_revenge_one_one_is_copiable_and_haste_isnt() {
    cr!("707.9b", "707.9d", "707.3", "604.3");
    ruling!(
        "Offspring's Revenge",
        "The token’s power and toughness are 1/1. These are copiable values of the token that other effects may copy. If the copied card has an ability that defines its power or toughness, that ability isn’t copied. On the other hand, gaining haste after the token has been created is not a copiable value."
    );
    // "At the beginning of combat on your turn, exile target red, white, or black creature
    // card from your graveyard. Create a token that's a copy of that card, except it's 1/1.
    // It gains haste until your next turn." Nightmare: a black */* flier whose power and
    // toughness are the number of Swamps you control.
    supported("Offspring's Revenge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Offspring's Revenge");
    t.lands(P0, "Swamp", 4);
    let card = t.graveyard(P0, "Nightmare");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(card)]);
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert_eq!(name_of(&t, tok), "Nightmare");
    assert!(has_kw(&t, tok, KeywordKind::Haste) && has_kw(&t, tok, KeywordKind::Flying));
    let c = clone_of(&mut t, P0, tok);
    assert!(!has_kw(&t, c, KeywordKind::Haste));
    for id in [tok, c] {
        assert_eq!(t.pt(id), (1, 1));
    }
    // A Nightmare that's not a copy is a 4/4.
    let n = t.battlefield(P0, "Nightmare");
    assert_eq!(t.pt(n), (4, 4));
}

#[test]
fn callidus_assassin_granted_trigger_is_copiable() {
    cr!("707.9a", "707.3", "707.5", "603.6a");
    ruling!(
        "Callidus Assassin",
        "The triggered ability that Callidus Assassin has is part of its copy effect and is a copiable value that other effects may copy."
    );
    // Polymorphine — "...except it has "When this creature enters, destroy up to one other
    // target creature with the same name as this creature.""
    supported("Callidus Assassin");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[]);
    let assassin = t.enter(P0, "Callidus Assassin");
    t.settle();
    t.resolve_all();
    let assassin = t.g.current(assassin);
    assert_eq!(name_of(&t, assassin), "Hill Giant");
    assert!(t.on_battlefield(giant));
    // A Clone copying the Assassin has its "enters" trigger and destroys the Giant.
    t.answer_targets(P0, &[obj(giant)]);
    let c = clone_of(&mut t, P0, assassin);
    assert_eq!(
        abilities_with(&t, c, "destroy up to one other target creature"),
        1
    );
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(c) && t.on_battlefield(assassin));
}

#[test]
fn soul_foundry_token_is_an_exact_copy_except_its_a_token() {
    cr!("707.2", "111.1", "607.2a");
    ruling!(
        "Soul Foundry",
        "The token is an exact copy in every way, except that it's a token, not a card."
    );
    supported("Soul Foundry");
    let mut t = TestGame::new(2);
    let angel = t.hand(P0, "Serra Angel");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(angel)]);
    let foundry = t.enter(P0, "Soul Foundry");
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Serra Angel"));
    lands_for(&mut t, P0, "{5}");
    let before = t.g.battlefield.clone();
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(5));
    activate_containing(&mut t, P0, foundry, "Create a token").expect("activate");
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    let printed = t.battlefield(P1, "Serra Angel");
    let printed = t.obj_now(printed).chars.clone();
    let o = t.obj_now(tok);
    assert!(o.is_token());
    assert_eq!(o.chars.name, printed.name);
    assert_eq!(o.chars.mana_cost, printed.mana_cost);
    assert_eq!(o.chars.colors, printed.colors);
    assert_eq!(o.chars.card_types, printed.card_types);
    assert_eq!(o.chars.subtypes, printed.subtypes);
    assert_eq!(t.pt(tok), (4, 4));
    assert!(has_kw(&t, tok, KeywordKind::Flying) && has_kw(&t, tok, KeywordKind::Vigilance));
}

#[test]
fn dance_of_many_token_copies_the_mana_cost() {
    cr!("707.2", "202.3");
    ruling!("Dance of Many", "The mana cost is copied.");
    // "When this enchantment enters, create a token that's a copy of target nontoken
    // creature." Hill Giant costs {3}{R}.
    supported("Dance of Many");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    let before = t.g.battlefield.clone();
    t.enter(P0, "Dance of Many");
    t.settle();
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert_eq!(
        t.obj_now(tok).chars.mana_cost,
        t.obj_now(giant).chars.mana_cost
    );
    assert_eq!(crate::r_s26_common::mv(&mut t, tok), 4);
    assert_eq!(t.obj_now(tok).chars.colors, red());
}

/// P0 casts Kavu Titan kicked ("If this creature was kicked, it enters with three +1/+1
/// counters on it and with trample."). Returns it.
fn kicked_kavu_titan(t: &mut TestGame) -> ObjectId {
    supported("Kavu Titan");
    lands_for(t, P0, "{1}{G}{2}{G}");
    let kavu = t.hand(P0, "Kavu Titan");
    t.cast(P0, kavu).kicked(true).go();
    t.resolve_all();
    let kavu = t
        .named_on_battlefield("Kavu Titan")
        .into_iter()
        .find(|id| !t.obj(*id).is_token())
        .unwrap();
    assert_eq!(t.counters(kavu, counters::PLUS1), 3);
    kavu
}

#[test]
fn token_copies_of_kicked_creatures_arent_kicked() {
    cr!("707.2", "702.33d", "614.1c");
    ruling!(
        "Dual Nature",
        "The token is not considered to have been kicked, even if the creature it's a copy of was kicked."
    );
    ruling!(
        "Splinter Twin",
        "The token will not be kicked, even if the creature it's copying was."
    );
    // Dual Nature: "Whenever a nontoken creature enters, its controller creates a token
    // that's a copy of that creature."
    supported("Dual Nature");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dual Nature");
    kicked_kavu_titan(&mut t);
    let tok = one(tokens_named(&t, "Kavu Titan"));
    assert_eq!(t.counters(tok, counters::PLUS1), 0);
    assert_eq!(t.pt(tok), (2, 2));
    assert!(!has_kw(&t, tok, KeywordKind::Trample));
    // Splinter Twin: "Enchanted creature has "{T}: Create a token that's a copy of this
    // creature, except it has haste. ...""
    supported("Splinter Twin");
    let mut t = TestGame::new(2);
    let kavu = kicked_kavu_titan(&mut t);
    unsick(&mut t, kavu);
    attach_new(&mut t, P0, "Splinter Twin", kavu);
    let tok = one(activate_for_tokens(&mut t, P0, kavu, "Create a token", &[]));
    assert_eq!(name_of(&t, tok), "Kavu Titan");
    assert_eq!(t.counters(tok, counters::PLUS1), 0);
    assert_eq!(t.pt(tok), (2, 2));
    assert!(!has_kw(&t, tok, KeywordKind::Trample));
}

#[test]
fn abyssal_harvester_exiles_the_other_nightmare_tokens_not_the_new_one() {
    cr!("608.2c", "707.9b");
    ruling!(
        "Abyssal Harvester",
        "Abyssal Harvester's ability will exile all other Nightmare tokens you control, not just Nightmare tokens created with Abyssal Harvester's ability."
    );
    // "...Create a token that's a copy of it, except it's a Nightmare in addition to its
    // other types. Then exile all other Nightmare tokens you control." P0 already controls
    // a Nightmare token made by another effect; the opponent controls one too.
    supported("Abyssal Harvester");
    let mut t = TestGame::new(2);
    let harvester = t.battlefield(P0, "Abyssal Harvester");
    let mine = crate::r_p023_common::token_of(
        &mut t,
        P0,
        crate::r_p025_common::creature_spec("Nightmare", Color::Black, 2),
    );
    let theirs = crate::r_p023_common::token_of(
        &mut t,
        P1,
        crate::r_p025_common::creature_spec("Nightmare", Color::Black, 2),
    );
    let giant = t.battlefield(P1, "Hill Giant");
    kill(&mut t, giant);
    let card = t.g.current(giant);
    let tok = one(activate_for_tokens(
        &mut t,
        P0,
        harvester,
        "Exile target",
        &[obj(card)],
    ));
    assert_eq!(name_of(&t, tok), "Hill Giant");
    assert!(t.on_battlefield(tok));
    assert!(!t.on_battlefield(mine));
    assert!(t.on_battlefield(theirs));
}
