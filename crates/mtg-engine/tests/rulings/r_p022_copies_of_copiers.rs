//! Rulings batch P022 — something becomes or enters as a copy of a permanent that's copying
//! something else, or of a token copy: it copies the copiable values, which include the
//! exceptions of the copy effects (CR 707.3, 707.9b), but not the effect's duration, the
//! abilities or haste another effect granted, or the delayed triggers that exile or
//! sacrifice the original (CR 603.7, 611.2, 707.2).

use crate::r_p022_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_copy_of_artisan_of_forms_copies_what_it_copies_with_heroic() {
    cr!("707.3", "707.9a");
    ruling!("Artisan of Forms", "If another creature becomes a copy of Artisan of Forms, it will become a copy of whatever Artisan of Forms is currently copying (if anything), plus it will have the triggered ability.");
    supported("Artisan of Forms");
    let mut t = TestGame::new(2);
    let artisan = t.battlefield(P0, "Artisan of Forms");
    let giant = t.battlefield(P1, "Hill Giant");
    // P0 casts Giant Growth on it; heroic targets the Giant.
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(obj(artisan)).go();
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(name_of(&t, artisan), "Hill Giant");
    let clone = clone_of(&mut t, P1, artisan);
    assert_eq!(t.pt(clone), (3, 3));
    assert_eq!(abilities_with(&t, clone, "spell that targets"), 1);
}

#[test]
fn a_copy_of_cryptoplasm_copies_what_it_copies_with_its_trigger() {
    cr!("707.3", "707.9a");
    ruling!("Cryptoplasm", "If another creature becomes a copy of Cryptoplasm, it will become a copy of whatever Cryptoplasm is currently copying (if anything), plus it will have Cryptoplasm’s triggered ability.");
    supported("Cryptoplasm");
    let mut t = TestGame::new(2);
    let plasm = t.battlefield(P0, "Cryptoplasm");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, plasm), "Hill Giant");
    let clone = clone_of(&mut t, P1, plasm);
    assert_eq!(t.pt(clone), (3, 3));
    assert_eq!(abilities_with(&t, clone, "beginning of your upkeep"), 1);
}

#[test]
fn copies_of_mimics_have_their_extra_types() {
    cr!("707.3", "707.9b");
    ruling!("Glasspool Mimic // Glasspool Shore", "If another creature becomes a copy of Glasspool Mimic, that creature is also a Shapeshifter Rogue.");
    ruling!(
        "Mirrorhall Mimic // Ghastly Mimicry",
        "If another creature becomes a copy of Mirrorhall Mimic, that creature is also a Spirit."
    );
    for (mimic, types) in [
        (
            "Glasspool Mimic // Glasspool Shore",
            &["Shapeshifter", "Rogue"][..],
        ),
        ("Mirrorhall Mimic // Ghastly Mimicry", &["Spirit"][..]),
    ] {
        supported(mimic);
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let m = enter_copying(&mut t, P0, mimic, giant);
        assert_eq!(name_of(&t, m), "Hill Giant");
        // P1's Clone enters as a copy of the Mimic.
        let clone = clone_of(&mut t, P1, m);
        for ty in types {
            assert!(has_subtype(&t, clone, ty), "{mimic}: {ty}");
        }
        assert!(has_subtype(&t, clone, "Giant"));
    }
}

#[test]
fn a_copy_of_sakashima_has_sakashimas_other_abilities() {
    cr!("707.3", "707.9a");
    ruling!("Sakashima of a Thousand Faces", "If another creature becomes a copy of Sakashima, that creature also has Sakashima's other abilities.");
    supported("Sakashima of a Thousand Faces");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let s = enter_copying(&mut t, P0, "Sakashima of a Thousand Faces", giant);
    assert_eq!(name_of(&t, s), "Hill Giant");
    let clone = clone_of(&mut t, P1, s);
    assert_eq!(abilities_with(&t, clone, "legend rule"), 1);
    assert!(t.obj_now(clone).has_keyword(KeywordKind::Partner));
}

#[test]
fn a_copy_of_the_ever_changing_dane_has_its_ability() {
    cr!("707.3", "707.9a");
    ruling!("The Ever-Changing 'Dane", "If another creature becomes a copy of The Ever-Changing ’Dane, it will become a copy of whatever The Ever-Changing ’Dane is currently copying (if anything), plus it will have the activated ability.");
    supported("The Ever-Changing 'Dane");
    let mut t = TestGame::new(2);
    let dane = t.battlefield(P0, "The Ever-Changing 'Dane");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(giant)]);
    let i = activated_index(&t, dane, "Sacrifice another creature");
    t.activate(P0, dane, i, &[]).expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, dane), "Hill Giant");
    let clone = clone_of(&mut t, P1, dane);
    assert_eq!(t.pt(clone), (3, 3));
    assert_eq!(abilities_with(&t, clone, "Sacrifice another creature"), 1);
}

#[test]
fn a_copy_of_zygon_infiltrator_copies_what_it_copies() {
    cr!("707.3", "707.2");
    ruling!("Zygon Infiltrator", "If another creature becomes a copy of Zygon Infiltrator, it will become a copy of whatever Zygon Infiltrator is currently copying (if anything).");
    supported("Zygon Infiltrator");
    let mut t = TestGame::new(2);
    let zygon = t.battlefield(P0, "Zygon Infiltrator");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{2}{U}");
    let i = activated_index(&t, zygon, "stun counter");
    t.activate(P0, zygon, i, &[obj(giant)]).expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, zygon), "Hill Giant");
    let clone = clone_of(&mut t, P1, zygon);
    assert_eq!(t.pt(clone), (3, 3));
    // Zygon's copy effect has no "except it has this ability".
    assert_eq!(abilities_with(&t, clone, "Body-print"), 0);
}

#[test]
fn a_copy_of_vesuvan_doppelganger_keeps_its_color_and_trigger() {
    cr!("707.3", "707.9a", "707.9c");
    ruling!("Vesuvan Doppelganger", "If another creature copies Vesuvan Doppelganger, the new creature will become a copy of whatever Vesuvan Doppelganger is copying except for its color, will copy Vesuvan Doppelganger's color, and will gain Vesuvan Doppelganger's triggered ability.");
    supported("Vesuvan Doppelganger");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let v = enter_copying(&mut t, P0, "Vesuvan Doppelganger", angel);
    assert_eq!(name_of(&t, v), "Serra Angel");
    assert_eq!(t.obj_now(v).chars.colors, ColorSet::single(Color::Blue));
    let clone = clone_of(&mut t, P1, v);
    assert_eq!(t.obj_now(clone).chars.colors, ColorSet::single(Color::Blue));
    assert!(has_kw(&t, clone, KeywordKind::Flying));
    assert_eq!(abilities_with(&t, clone, "beginning of your upkeep"), 1);
}

#[test]
fn a_copy_of_nascent_metamorph_stays_a_copy_after_the_metamorphs_effect_ends() {
    cr!("707.3", "707.2", "514.2");
    ruling!("Nascent Metamorph", "If another permanent becomes a copy of Nascent Metamorph, it will become whatever Nascent Metamorph is copying. It remains a copy when Nascent Metamorph's effect ends.");
    supported("Nascent Metamorph");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Nascent Metamorph");
    stack_library(&mut t, P1, &["Hill Giant"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(m, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(name_of(&t, m), "Hill Giant");
    let clone = clone_of(&mut t, P1, m);
    assert_eq!(name_of(&t, clone), "Hill Giant");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, m), "Nascent Metamorph");
    assert_eq!(name_of(&t, clone), "Hill Giant");
}

#[test]
fn a_copy_of_cursed_mirror_doesnt_copy_its_duration() {
    cr!("707.2", "707.9a", "611.2a", "514.2");
    ruling!("Cursed Mirror", "If something else becomes a copy of Cursed Mirror while Cursed Mirror is a copy of a creature, the duration of the effect is not copied. The new copy will still be a creature (with haste) after Cursed Mirror stops being a copy at the end of the turn.");
    supported("Cursed Mirror");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let m = t.hand(P0, "Cursed Mirror");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    t.cast(P0, m).go();
    t.resolve_all();
    let mirror = t.g.current(m);
    assert_eq!(name_of(&t, mirror), "Hill Giant");
    assert!(has_kw(&t, mirror, KeywordKind::Haste));
    // P1's Clone (not an artifact) enters as a copy of it.
    let clone = clone_of(&mut t, P1, mirror);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, mirror), "Cursed Mirror");
    assert!(!is_creature(&t, mirror));
    assert_eq!(name_of(&t, clone), "Hill Giant");
    assert!(is_creature(&t, clone));
    assert!(has_kw(&t, clone, KeywordKind::Haste));
}

#[test]
fn a_copy_of_valki_stays_a_copy_after_valki_leaves() {
    cr!("707.3", "707.2");
    ruling!("Valki, God of Lies // Tibalt, Cosmic Impostor", "If another object becomes a copy of Valki, it will become whatever Valki is copying. That object remains a copy even if Valki leaves the battlefield.");
    let (mut t, valki) = crate::r_p022_valki::valki_copying_giant();
    let clone = clone_of(&mut t, P0, valki);
    assert_eq!(name_of(&t, clone), "Hill Giant");
    kill(&mut t, valki);
    assert!(!t.on_battlefield(valki));
    assert_eq!(name_of(&t, clone), "Hill Giant");
    assert_eq!(t.pt(clone), (3, 3));
}

// --- Token copies with exceptions ---------------------------------------------------------

/// P0's Clone enters as a copy of `token`.
fn clone_token(t: &mut TestGame, token: ObjectId) -> ObjectId {
    let c = clone_of(t, P0, token);
    unsick(t, c);
    c
}

#[test]
fn a_copy_of_a_nightmare_shepherd_token_is_a_1_1_nightmare() {
    cr!("707.9b", "707.3");
    ruling!(
        "Nightmare Shepherd",
        "If something becomes a copy of the token, the copy is also a 1/1 and a Nightmare."
    );
    supported("Nightmare Shepherd");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nightmare Shepherd");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    kill(&mut t, giant);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    let clone = clone_token(&mut t, tok);
    assert_eq!(name_of(&t, clone), "Hill Giant");
    assert_eq!(t.pt(clone), (1, 1));
    assert!(has_subtype(&t, clone, "Nightmare"));
}

#[test]
fn a_copy_of_an_apprentices_folly_token_is_a_hasty_nonlegendary_reflection() {
    cr!("707.9a", "707.9b", "707.3");
    ruling!("The Apprentice's Folly", "If something becomes a copy of the token, the copy is also a Reflection in addition to its other types, has haste, and isn't legendary.");
    supported("The Apprentice's Folly");
    supported("Isamaru, Hound of Konda");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.answer_targets(P0, &[obj(isamaru)]);
    let before = t.g.battlefield.clone();
    t.enter(P0, "The Apprentice's Folly");
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    // P1's Clone enters as a copy of the token.
    let clone = clone_of(&mut t, P1, tok);
    assert!(has_subtype(&t, clone, "Reflection"));
    assert!(has_kw(&t, clone, KeywordKind::Haste));
    assert!(!legendary(&t, clone));
}

/// `p` gets a token copy of P0's Hill Giant from `make`; P1's Clone then enters as a copy
/// of the token. Returns the game, the token and the Clone.
fn clone_of_token(
    make: impl FnOnce(&mut TestGame, ObjectId) -> ObjectId,
) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let tok = make(&mut t, giant);
    assert_eq!(name_of(&t, tok), "Hill Giant");
    let clone = clone_of(&mut t, P1, tok);
    unsick(&mut t, clone);
    (t, tok, clone)
}

/// Advances to P1's upkeep; asserts P0's token is gone (exiled or sacrificed) and P1's
/// Clone is still there.
fn token_gone_clone_stays(t: &mut TestGame, tok: ObjectId, clone: ObjectId) {
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.on_battlefield(tok));
    assert!(t.on_battlefield(clone));
}

#[test]
fn a_copy_of_a_twinflame_token_has_haste_but_isnt_exiled() {
    cr!("707.9a", "707.3", "603.7c");
    ruling!("Twinflame", "If another creature becomes or enters the battlefield as a copy of the token, that creature will have haste, but you won't exile it.");
    let (mut t, tok, clone) = clone_of_token(|t, giant| {
        one(cast_spell(
            t,
            P0,
            "Twinflame",
            "{1}{R}",
            &[obj(giant)],
            None,
            None,
        ))
    });
    assert!(has_kw(&t, clone, KeywordKind::Haste));
    token_gone_clone_stays(&mut t, tok, clone);
}

#[test]
fn copies_of_tokens_that_gained_haste_have_no_haste_and_stay() {
    cr!("707.2", "707.3", "603.7c");
    ruling!("Flameshadow Conjuring", "If another creature becomes or enters the battlefield as a copy of the token, that creature won't have haste and it won't be exiled.");
    ruling!("Mordor on the March", "If another creature becomes a copy of, or enters the battlefield as, a copy of the token, that creature will copy the creature card the token is copying. However, the new copy won't gain haste and you won't exile it at the beginning of the next end step.");
    supported("Flameshadow Conjuring");
    // Flameshadow Conjuring: "Whenever a nontoken creature you control enters, you may pay
    // {R}. If you do, create a token that's a copy of that creature. That token gains
    // haste. Exile it at the beginning of the next end step."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Flameshadow Conjuring");
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert!(has_kw(&t, tok, KeywordKind::Haste));
    let clone = clone_of(&mut t, P1, tok);
    assert!(!has_kw(&t, clone, KeywordKind::Haste));
    token_gone_clone_stays(&mut t, tok, clone);
    // Mordor on the March: "Exile a creature card from your graveyard. Create a token
    // that's a copy of it. It gains haste until end of turn. Exile it at the beginning of
    // the next end step." A copy of the token is a Hill Giant without haste that stays.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    let toks = cast_spell(
        &mut t,
        P0,
        "Mordor on the March",
        "{3}{B}{R}",
        &[],
        None,
        None,
    );
    let tok = one(toks);
    assert!(has_kw(&t, tok, KeywordKind::Haste));
    let clone = clone_of(&mut t, P1, tok);
    assert_eq!(name_of(&t, clone), "Hill Giant");
    assert!(!has_kw(&t, clone, KeywordKind::Haste));
    token_gone_clone_stays(&mut t, tok, clone);
}

#[test]
fn a_copy_of_a_felhide_spiritbinder_token_is_an_enchantment_without_haste() {
    cr!("707.9b", "707.3", "603.7c");
    ruling!("Felhide Spiritbinder", "If another creature becomes or enters the battlefield as a copy of the token, that creature won’t have haste and you won’t exile it.");
    supported("Felhide Spiritbinder");
    let (mut t, tok, clone) = clone_of_token(|t, giant| {
        let s = t.battlefield(P0, "Felhide Spiritbinder");
        t.g.tap(s);
        lands_for(t, P0, "{1}{R}");
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(giant)]);
        let before = t.g.battlefield.clone();
        t.g.untap(s);
        t.g.flush_events();
        t.resolve_all();
        one(new_tokens(t, P0, &before))
    });
    assert!(has_kw(&t, tok, KeywordKind::Haste));
    assert!(t.obj_now(clone).is(CardType::Enchantment));
    assert!(!has_kw(&t, clone, KeywordKind::Haste));
    token_gone_clone_stays(&mut t, tok, clone);
}

#[test]
fn a_copy_of_a_flamerush_rider_token_isnt_exiled_at_end_of_combat() {
    cr!("707.2", "603.7c");
    ruling!("Flamerush Rider", "If another creature becomes or enters the battlefield as a copy of the token, you won't exile that creature at end of combat.");
    supported("Flamerush Rider");
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Flamerush Rider");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (rider, Entity::Player(P1)),
            (giant, Entity::Player(P1)),
        ]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    let before = t.g.battlefield.clone();
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    let clone = clone_of(&mut t, P1, tok);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!t.on_battlefield(tok));
    assert!(t.on_battlefield(clone));
}

#[test]
fn copies_of_tokens_from_activated_abilities_that_grant_haste() {
    cr!("707.2", "707.9b", "603.7c");
    ruling!("Feldon of the Third Path", "If another creature becomes a copy of, or enters the battlefield as a copy of the token, that creature will copy the creature card the token is copying, except it will also be an artifact. This is true even if the creature card that was copied by Feldon of the Third Path is no longer in the graveyard at that time. However, the new copy won't gain haste and you won't sacrifice it at the beginning of the next end step.");
    ruling!("Jaxis, the Troublemaker", "If another creature becomes a copy of, or enters the battlefield as a copy of, the token, that creature will copy the creature card the token is copying. However, it won't have either haste or \"When this creature dies, draw a card,\" and you won't sacrifice the new copy at the beginning of the next end step.");
    supported("Feldon of the Third Path");
    supported("Jaxis, the Troublemaker");
    // Feldon: the Hill Giant card is exiled from the graveyard before the Clone copies the
    // token.
    let mut t = TestGame::new(2);
    let feldon = t.battlefield(P0, "Feldon of the Third Path");
    let card = t.graveyard(P0, "Hill Giant");
    lands_for(&mut t, P0, "{2}{R}");
    let toks = activate_for_tokens(&mut t, P0, feldon, "Create a token", &[obj(card)]);
    let tok = one(toks);
    assert!(is_artifact(&t, tok) && has_kw(&t, tok, KeywordKind::Haste));
    let card = t.g.current(card);
    t.g.move_object(
        card,
        Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    let clone = clone_of(&mut t, P1, tok);
    assert_eq!(name_of(&t, clone), "Hill Giant");
    assert!(is_artifact(&t, clone));
    assert!(!has_kw(&t, clone, KeywordKind::Haste));
    token_gone_clone_stays(&mut t, tok, clone);
    // Jaxis: no haste, no "When this creature dies, draw a card".
    let mut t = TestGame::new(2);
    let jaxis = t.battlefield(P0, "Jaxis, the Troublemaker");
    let giant = t.battlefield(P0, "Hill Giant");
    t.hand(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{R}");
    let toks = activate_for_tokens(&mut t, P0, jaxis, "Create a token", &[obj(giant)]);
    let tok = one(toks);
    assert!(has_kw(&t, tok, KeywordKind::Haste));
    assert_eq!(abilities_with(&t, tok, "draw a card"), 1);
    let clone = clone_of(&mut t, P1, tok);
    assert!(!has_kw(&t, clone, KeywordKind::Haste));
    assert_eq!(abilities_with(&t, clone, "draw a card"), 0);
    token_gone_clone_stays(&mut t, tok, clone);
}

#[test]
fn a_copy_of_a_hofri_spirit_isnt_linked_to_hofri() {
    cr!("707.9a", "607.1d", "707.3");
    ruling!("Hofri Ghostforge", "If another creature enters the battlefield as a copy of the Spirit token or another effect creates a copy of the Spirit token, it will have the triggered ability granted by Hofri, but will not be linked to Hofri's ability and will not return the card to the graveyard when it leaves the battlefield.");
    supported("Hofri Ghostforge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hofri Ghostforge");
    let giant = t.battlefield(P0, "Hill Giant");
    let before = t.g.battlefield.clone();
    kill(&mut t, giant);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert!(t.in_exile("Hill Giant"));
    // P1's Clone enters as a copy of the token.
    let clone = clone_of(&mut t, P1, tok);
    assert!(has_subtype(&t, clone, "Spirit"));
    assert_eq!(abilities_with(&t, clone, "return the exiled card"), 1);
    // The Clone leaves: its trigger returns nothing.
    kill(&mut t, clone);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(!t.in_graveyard(P0, "Hill Giant"));
    // The token leaves: the card returns to the graveyard.
    kill(&mut t, tok);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}
