//! Rulings batch P024 — copying a token: the copy gets the original characteristics the
//! effect that created the token gave it (CR 707.2, 111.3), with the copy effect's own
//! exceptions (CR 707.9b), and a permanent that becomes a copy of a token doesn't become a
//! token (CR 111.1, 707.2).

use crate::r_p024_common::*;
use crate::r_s26_common::dress_up;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s green 2/2 creature token named and typed `name`, then changed in every way a
/// copy doesn't copy (tapped then untapped, two +1/+1 counters, +3/+3 and green-blue until
/// end of turn): a 7/7.
fn dressed_token(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let tok = typed_token(t, p, name, Color::Green, 2);
    dress_up(t, tok);
    let tok = t.g.current(tok);
    t.g.untap(tok);
    t.g.recompute();
    assert_eq!(t.pt(tok), (7, 7));
    tok
}

/// Asserts `id` has the original characteristics of a [`dressed_token`] named `name`: a
/// green (not blue) `name` creature with no counters, a token when `token`, and `pt`.
fn assert_original(t: &TestGame, id: ObjectId, name: &str, token: bool, pt: (i32, i32)) {
    let o = t.obj_now(id);
    assert_eq!(o.chars.name, name, "{:?}", o.chars);
    assert!(o.chars.colors.contains(Color::Green) && !o.chars.colors.contains(Color::Blue));
    assert!(o.is(CardType::Creature) && has_subtype(t, id, name));
    assert!(o.counters.values().all(|n| *n == 0), "it copied the counters");
    assert_eq!(o.is_token(), token);
    assert_eq!(t.pt(id), pt);
}

// --- Spells that create a token copy -----------------------------------------------------

#[test]
fn spells_copying_a_token_copy_its_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!("Flash Photography", "If the copied permanent is a token, the new token that's created copies the original characteristics of that token as stated by the effect that created the token.");
    ruling!("Saheeli's Artistry", "If the copied permanent is a token, the token created with Saheeli's Artistry copies the original characteristics of that token as stated by the effect that that created that token.");
    ruling!("Echocasting Symposium", "If the copied permanent is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token.");
    ruling!("Stolen Identity", "If the copied permanent is a token, the token that's created copies the original characteristics of that token as stated by the effect that put the token onto the battlefield.");
    ruling!("Three Steps Ahead", "If the copied permanent is a token, the token that’s created copies the original characteristics of that token as stated by the effect that created that token.");
    ruling!("Ashling's Command", "If the copied permanent is a token, the token that's created copies the original characteristics of that token as stated by the effect that created that token.");
    // (spell, cost, token type, modes, extra targets after the token, artifact exception)
    let cases: &[(&str, &str, &str, Option<&[usize]>, &[Entity], bool)] = &[
        ("Flash Photography", "{2}{U}{U}", "Wolf", None, &[], false),
        ("Relm's Sketching", "{2}{U}{U}", "Wolf", None, &[], false),
        ("Saheeli's Artistry", "{4}{U}{U}", "Wolf", Some(&[1]), &[], true),
        ("Stolen Identity", "{4}{U}{U}", "Wolf", None, &[], false),
        ("Three Steps Ahead", "{3}{U}", "Wolf", Some(&[1]), &[], false),
        ("Ashling's Command", "{3}{U}{R}", "Elemental", Some(&[0, 1]), &[Entity::Player(P0)], false),
        ("Brigid's Command", "{1}{G}{W}", "Kithkin", Some(&[0, 2]), &[], false),
        ("Grub's Command", "{3}{B}{R}", "Goblin", Some(&[0, 3]), &[Entity::Player(P1)], false),
        ("Sygg's Command", "{1}{W}{U}", "Merfolk", Some(&[0, 2]), &[Entity::Player(P0)], false),
        ("Trystan's Command", "{4}{B}{G}", "Elf", Some(&[0, 3]), &[Entity::Player(P1)], false),
    ];
    for (spell, cost, ty, modes, extra, artifact) in cases {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let tok = dressed_token(&mut t, P0, ty);
        let mut targets = vec![obj(tok)];
        if *spell == "Brigid's Command" {
            // "Target creature you control gets +3/+3": the original, after the copy.
            targets.push(obj(tok));
        }
        targets.extend_from_slice(extra);
        let toks = cast_spell(&mut t, P0, spell, cost, &targets, *modes, None);
        assert_eq!(toks.len(), 1, "{spell}: {}", t.dump_log());
        assert_original(&t, toks[0], ty, true, (2, 2));
        assert_eq!(is_artifact(&t, toks[0]), *artifact, "{spell}");
    }
}

#[test]
fn echocasting_symposium_copies_a_tokens_original_characteristics_for_the_target_player() {
    cr!("707.2", "111.3", "111.2");
    // "Target player creates a token that's a copy of target creature you control."
    let mut t = TestGame::new(2);
    let wolf = dressed_token(&mut t, P0, "Wolf");
    let before = t.g.battlefield.clone();
    cast_spell(
        &mut t,
        P0,
        "Echocasting Symposium",
        "{4}{U}{U}",
        &[Entity::Player(P1), obj(wolf)],
        None,
        None,
    );
    let toks = new_tokens(&t, P1, &before);
    assert_eq!(toks.len(), 1);
    assert_original(&t, toks[0], "Wolf", true, (2, 2));
}

#[test]
fn molten_duplication_copies_a_tokens_original_characteristics_with_its_exceptions() {
    cr!("707.2", "707.9b", "111.3");
    ruling!("Molten Duplication", "If the copied permanent is a token, the token that’s created copies the original characteristics of that token as stated by the effect that created that token, with the stated exceptions.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wolf = dressed_token(&mut t, P0, "Wolf");
    let toks = cast_spell(&mut t, P0, "Molten Duplication", "{1}{R}", &[obj(wolf)], None, None);
    assert_eq!(toks.len(), 1);
    assert_original(&t, toks[0], "Wolf", true, (2, 2));
    assert!(is_artifact(&t, toks[0]) && has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn devastating_onslaught_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "107.3a");
    ruling!("Devastating Onslaught", "If the copied permanent is a token, the tokens that are created copy the original characteristics of that token as stated by the effect that created the token.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wolf = dressed_token(&mut t, P0, "Wolf");
    let toks = cast_spell(&mut t, P0, "Devastating Onslaught", "{4}{R}", &[obj(wolf)], None, Some(2));
    assert_eq!(toks.len(), 2);
    for c in toks {
        assert_original(&t, c, "Wolf", true, (2, 2));
        assert!(has_kw(&t, c, KeywordKind::Haste));
    }
}

#[test]
fn season_of_weaving_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "700.2i");
    ruling!("Season of Weaving", "If the copied permanent is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token.");
    // "{P}{P} — Choose an artifact or creature you control. Create a token that's a copy of
    // it." Chosen once (and "{P} — Draw a card" for the rest isn't chosen).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wolf = dressed_token(&mut t, P0, "Wolf");
    t.answer_choose(P0, &[obj(wolf)]);
    let toks = cast_spell(&mut t, P0, "Season of Weaving", "{4}{U}{U}", &[], Some(&[1]), None);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_original(&t, toks[0], "Wolf", true, (2, 2));
}

#[test]
fn saw_in_half_copies_a_tokens_original_characteristics_except_half_its_power() {
    cr!("707.2", "707.9b", "111.3", "608.2h");
    ruling!("Saw in Half", "If the copied creature was a token, the new tokens copy the original characteristics of that token as stated by the effect that created the token, with the listed exceptions.");
    let mut t = TestGame::new(2);
    let wolf = dressed_token(&mut t, P1, "Wolf");
    let before = t.g.battlefield.clone();
    cast_spell(&mut t, P0, "Saw in Half", "{2}{B}", &[obj(wolf)], None, None);
    let toks = new_tokens(&t, P1, &before);
    assert_eq!(toks.len(), 2);
    for c in toks {
        // Half of the 7/7 it was, rounded up: 4/4 (not half of the 2/2 Wolf).
        assert_original(&t, c, "Wolf", true, (4, 4));
    }
}

// --- Abilities that create a token copy ----------------------------------------------------

#[test]
fn saheeli_the_suns_brilliance_copies_a_tokens_original_characteristics() {
    cr!("707.2", "707.9b", "111.3");
    ruling!("Saheeli, the Sun's Brilliance", "If the copied permanent is a token, the new token that's created copies the original characteristics of that token as stated by the effect that created the token (with the listed exceptions).");
    supported("Saheeli, the Sun's Brilliance");
    let mut t = TestGame::new(2);
    let saheeli = t.battlefield(P0, "Saheeli, the Sun's Brilliance");
    let wolf = dressed_token(&mut t, P0, "Wolf");
    lands_for(&mut t, P0, "{U}{R}");
    let before = t.g.battlefield.clone();
    t.activate(P0, saheeli, 0, &[obj(wolf)]).expect("activation");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_original(&t, toks[0], "Wolf", true, (2, 2));
    assert!(is_artifact(&t, toks[0]) && has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn saheeli_rai_copies_a_tokens_original_characteristics() {
    cr!("707.2", "707.9b", "111.3");
    ruling!("Saheeli Rai", "If the copied permanent is a token, the token created with Saheeli copies the original characteristics of that token as stated by the effect that created that token.");
    supported("Saheeli Rai");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let saheeli = t.battlefield(P0, "Saheeli Rai");
    let wolf = dressed_token(&mut t, P0, "Wolf");
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(wolf)]);
    activate_containing(&mut t, P0, saheeli, "Create a token").expect("loyalty ability");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_original(&t, toks[0], "Wolf", true, (2, 2));
    assert!(is_artifact(&t, toks[0]) && has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn oko_the_ringleader_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    // "−5: For each other nonland permanent you control, create a token that's a copy of
    // that permanent." (The ruling is shared with the Commands above; Oko words it
    // differently.)
    supported("Oko, the Ringleader");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let oko = t.battlefield(P0, "Oko, the Ringleader");
    t.g.add_counters(obj(oko), counters::LOYALTY, 5, None);
    dressed_token(&mut t, P0, "Wolf");
    t.lands(P0, "Forest", 1);
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, oko, "For each other nonland permanent").expect("−5");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_original(&t, toks[0], "Wolf", true, (2, 2));
}

// --- Permanents that become a copy of a token ------------------------------------------

#[test]
fn mindlink_mech_copying_a_token_doesnt_become_a_token() {
    cr!("707.2", "707.9b", "111.3", "702.122a");
    ruling!("Mindlink Mech", "If the creature is a token, Mindlink Mech copies the original characteristics of that token as stated by the effect that put the token onto the battlefield. It doesn't become a token.");
    supported("Mindlink Mech");
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mindlink Mech");
    let wolf = dressed_token(&mut t, P0, "Wolf");
    assert!(crate::r_s04_common::crew(&mut t, P0, mech, &[wolf]));
    t.resolve_all();
    // "except it's 4/3, it's a Vehicle artifact in addition to its other types, and it
    // has flying."
    assert_original(&t, mech, "Wolf", false, (4, 3));
    assert!(is_artifact(&t, mech) && has_kw(&t, mech, KeywordKind::Flying));
}

#[test]
fn volrath_copying_a_token_doesnt_become_a_token() {
    cr!("707.2", "707.9b", "111.3");
    ruling!("Volrath, the Shapestealer", "If the target creature is a token, Volrath copies the original characteristics of that token as stated by the effect that created the token. Volrath doesn't become a token in this case.");
    supported("Volrath, the Shapestealer");
    let mut t = TestGame::new(2);
    let volrath = t.battlefield(P0, "Volrath, the Shapestealer");
    // "target creature with a counter on it": the dressed-up Wolf has two.
    dressed_token(&mut t, P1, "Wolf");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, volrath, "becomes a copy")
        .map(|_| ())
        .unwrap_or_else(|e| panic!("{e:?}"));
    t.resolve_all();
    // "except it's 7/5 and it has this ability."
    assert_original(&t, volrath, "Wolf", false, (7, 5));
    assert_eq!(abilities_with(&t, volrath, "becomes a copy of target creature with a counter"), 1);
}

#[test]
fn mass_copy_effects_of_a_token_dont_make_tokens() {
    cr!("707.2", "707.9b", "111.3", "613.1a");
    ruling!("Nanogene Conversion", "If the targeted creature is a token, each other creature copies the original characteristics of that token as stated by the effect that created the token. Those creatures don't become tokens.");
    ruling!("Mirrorweave", "If the targeted creature is a token, each other creature copies the original characteristics of that token as stated by the effect that put it onto the battlefield. Those creatures don’t become tokens.");
    for (spell, cost) in [("Nanogene Conversion", "{3}{U}"), ("Mirrorweave", "{2}{W}{U}")] {
        let mut t = TestGame::new(2);
        let wolf = dressed_token(&mut t, P0, "Wolf");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        let toks = cast_spell(&mut t, P0, spell, cost, &[obj(wolf)], None, None);
        assert!(toks.is_empty());
        for c in [bears, giant] {
            assert_original(&t, c, "Wolf", false, (2, 2));
        }
        // The Wolf itself keeps its changes.
        assert_eq!(t.pt(wolf), (7, 7));
    }
}
