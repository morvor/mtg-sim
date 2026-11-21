//! Rulings batch P175 — Exchange of Words (exchanging text boxes, CR 612.1), Fiery
//! Inscription's spell trigger, Frodo Baggins as a Ring-bearer that must be blocked
//! (CR 509.1c), choosing the same Ring-bearer again (CR 701.54a), and Nazgûl's "a deck can
//! have up to nine cards named Nazgûl" (CR 100.2a, 113.6n).

use crate::r_p175_common::*;
use crate::r_s21_common::legal_blocks;
use mtg_engine::ability::{AbilityDef, AbilityKind, Duration, Effect, Modification, Sel};
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::kwa::ring::{ring_bearer, tempt};
use mtg_engine::stickers::{put_sticker, StickerKind};
use mtg_engine::testing::*;
use mtg_engine::*;
use std::sync::Arc;

fn kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

/// P0's Exchange of Words enters targeting `a` and `b`; the trigger is left on the stack.
fn exchange_of_words(t: &mut TestGame, a: ObjectId, b: ObjectId) -> ObjectId {
    supported("Exchange of Words");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    let e = t.enter(P0, "Exchange of Words");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    e
}

#[test]
fn exchange_of_words_exchanges_rules_text() {
    cr!("612.1", "612.5", "613.1c", "707.2", "208.1");
    // The ruling's second sentence (flavor text and watermarks in Un- games) is exempt
    // (docs/rulings-exemptions/rulings-P175.tsv); its first and last are tested here.
    ruling!(
        "Exchange of Words",
        "Exchanging two creatures’ text boxes means that each one loses its rules text and replaces it with the rules text the other one had."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let e = exchange_of_words(&mut t, angel, bears);
    t.resolve_all();
    assert!(kw(&t, bears, KeywordKind::Flying) && kw(&t, bears, KeywordKind::Vigilance));
    assert!(!kw(&t, angel, KeywordKind::Flying) && !kw(&t, angel, KeywordKind::Vigilance));
    // Power and toughness aren't in the text box.
    assert_eq!(t.pt(angel), (4, 4));
    assert_eq!(t.pt(bears), (2, 2));
    // "In both cases, this is a text-changing effect" (layer 3), not a copy effect: the
    // exchanged text isn't among the copiable values, so a Clone entering as a copy of
    // the Bears is a plain Grizzly Bears.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let clone = t.enter(P0, "Clone");
    t.resolve_all();
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Grizzly Bears");
    assert!(!kw(&t, clone, KeywordKind::Flying) && !kw(&t, clone, KeywordKind::Vigilance));
    // It ends once Exchange of Words leaves the battlefield.
    exile(&mut t, e);
    assert!(kw(&t, angel, KeywordKind::Flying) && !kw(&t, bears, KeywordKind::Flying));
}

#[test]
fn exchange_of_words_lasts_while_it_remains_even_without_abilities() {
    cr!("611.2b", "612.1");
    ruling!(
        "Exchange of Words",
        "If Exchange of Words has lost its own abilities (or perhaps its text box), the effect will still only end once it is no longer on the battlefield."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let e = exchange_of_words(&mut t, angel, bears);
    t.resolve_all();
    // Exchange of Words loses all abilities until end of turn.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![obj(e)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert!(t.obj_now(e).chars.abilities.is_empty());
    assert!(kw(&t, bears, KeywordKind::Flying), "still exchanged");
    exile(&mut t, e);
    assert!(!kw(&t, bears, KeywordKind::Flying));
}

#[test]
fn exchange_of_words_that_leaves_or_loses_a_target_exchanges_nothing() {
    cr!("608.2b", "611.2b", "702.11b");
    ruling!(
        "Exchange of Words",
        "If Exchange of Words leaves the battlefield before its enters-the-battlefield ability resolves, the text boxes are never exchanged. Neither creature is ever affected. The same is true if either of the two creatures becomes an illegal target before the enters-the-battlefield ability resolved."
    );
    for leaves in ["Exchange of Words", "Grizzly Bears"] {
        let mut t = TestGame::new(2);
        let angel = t.battlefield(P1, "Serra Angel");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let e = exchange_of_words(&mut t, angel, bears);
        if leaves == "Grizzly Bears" {
            bounce(&mut t, bears);
        } else {
            exile(&mut t, e);
        }
        t.resolve_all();
        assert!(kw(&t, angel, KeywordKind::Flying), "{leaves}");
        if leaves != "Grizzly Bears" {
            assert!(!kw(&t, bears, KeywordKind::Flying), "{leaves}");
        }
    }
    // The Bears become an illegal target but stay on the battlefield (hexproof against
    // P0's ability): neither creature is affected.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    exchange_of_words(&mut t, angel, bears);
    crate::r_s26_common::modify_until_eot(
        &mut t,
        bears,
        vec![Modification::AddKeyword(Keyword::new(
            KeywordKind::Hexproof,
        ))],
    );
    t.resolve_all();
    assert!(kw(&t, angel, KeywordKind::Flying) && kw(&t, angel, KeywordKind::Vigilance));
    assert!(!kw(&t, bears, KeywordKind::Flying) && !kw(&t, bears, KeywordKind::Vigilance));
}

#[test]
fn exchange_of_words_doesnt_exchange_granted_abilities_or_stickers() {
    cr!("612.3", "612.5", "123.7", "613.1c", "613.1f");
    ruling!(
        "Exchange of Words",
        "Notably, ability stickers (and other granted abilities) aren’t part of the rules text that get exchanged. Any ability stickers on either of the two permanents continue to apply to the permanent they are on."
    );
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P1, "Raging Goblin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // The Bears wear Flight ("Enchanted creature has flying") and have a trample sticker.
    crate::r_s06_common::attach_new(&mut t, P0, "Flight", bears);
    assert!(put_sticker(
        &mut t.g,
        P1,
        bears,
        StickerKind::Ability(vec![AbilityDef::new(
            AbilityKind::Keyword(Keyword::new(KeywordKind::Trample)),
            "Trample",
        )]),
    ));
    t.g.recompute();
    exchange_of_words(&mut t, goblin, bears);
    t.resolve_all();
    assert!(kw(&t, bears, KeywordKind::Haste), "the Goblin's text");
    assert!(kw(&t, bears, KeywordKind::Flying) && kw(&t, bears, KeywordKind::Trample));
    assert!(!kw(&t, goblin, KeywordKind::Haste));
    assert!(!kw(&t, goblin, KeywordKind::Flying) && !kw(&t, goblin, KeywordKind::Trample));
}

#[test]
fn fiery_inscription_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5", "601.2i");
    ruling!(
        "Fiery Inscription",
        "Fiery Inscription's second triggered ability will resolve before the spell that caused the ability to trigger."
    );
    supported("Fiery Inscription");
    let mut t = TestGame::new(2);
    let insc = t.battlefield(P0, "Fiery Inscription");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_at(&mut t, P0, "Lightning Bolt", &[obj(bears)]);
    t.settle();
    assert!(top_is_trigger_of(&t, insc));
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(bears), "the Bolt hasn't resolved yet");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

/// P0's Frodo Baggins enters and becomes P0's Ring-bearer (the Ring has tempted P0 once:
/// "Your Ring-bearer is legendary and can't be blocked by creatures with greater power.").
fn frodo(t: &mut TestGame) -> ObjectId {
    supported("Frodo Baggins");
    let f = t.enter(P0, "Frodo Baggins");
    t.answer_choose(P0, &[obj(f)]);
    t.resolve_all();
    assert_eq!(ring_bearer(&t.g, P0), Some(f));
    t.g.obj_mut(f).summoning_sick = false;
    f
}

#[test]
fn frodo_ring_bearer_needs_only_one_blocker() {
    cr!("509.1c");
    ruling!(
        "Frodo Baggins",
        "If Frodo Baggins is your Ring-bearer, only one creature is required to block it. Other creatures may also block it and are free to block other creatures or not block at all."
    );
    let mut t = TestGame::new(2);
    let f = frodo(&mut t);
    let other = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let orni = t.battlefield(P1, "Ornithopter");
    attack_with(&mut t, &[(f, pl(P1)), (other, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(elves, other)]));
    assert!(legal_blocks(&mut t, P1, &[(elves, f)]));
    assert!(legal_blocks(&mut t, P1, &[(elves, f), (orni, f)]));
    assert!(legal_blocks(&mut t, P1, &[(elves, f), (orni, other)]));
}

#[test]
fn frodo_ring_bearer_unblockable_or_costly_to_block_isnt_blocked() {
    cr!("509.1b", "509.1c", "701.54c");
    ruling!(
        "Frodo Baggins",
        "If Frodo Baggins is your Ring-bearer but each creature the defending player controls can't block for any reason (such as by having greater power than Frodo along with the first ability of the Ring emblem), then Frodo Baggins isn't blocked. If there's a cost associated with blocking Frodo Baggins, the defending player isn't forced to pay that cost, so it doesn't have to be blocked in that case either."
    );
    // Only creatures with greater power (Hill Giant): no block is required.
    let mut t = TestGame::new(2);
    let f = frodo(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(f, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(giant, f)]));
    assert!(legal_blocks(&mut t, P1, &[]));
    // War Cadence (blocking costs {X} per blocker): the Elves needn't block.
    supported("War Cadence");
    let mut t = TestGame::new(2);
    let f = frodo(&mut t);
    let cadence = t.battlefield(P0, "War Cadence");
    t.battlefield(P1, "Llanowar Elves");
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(1));
    act(&mut t, P0, cadence, 0, &[]);
    attack_with(&mut t, &[(f, pl(P1))]);
    assert!(legal_blocks(&mut t, P1, &[]));
}

#[test]
fn choosing_the_same_ring_bearer_again_still_counts() {
    cr!("701.54a", "603.2");
    ruling!(
        "Call of the Ring",
        "If the creature you choose as your Ring-bearer was already your Ring-bearer, that still counts as choosing that creature as your Ring-bearer for the purpose of abilities that trigger \"whenever you choose a creature as your Ring-bearer\" or abilities that care about which creature was chosen as your Ring-bearer."
    );
    supported("Call of the Ring");
    // Call of the Ring: "Whenever you choose a creature as your Ring-bearer, you may pay 2
    // life. If you do, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Call of the Ring");
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        t.answer_choose(P0, &[obj(bears)]);
        t.answer_yes(P0, true);
        tempt(&mut t.g, P0, None);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(ring_bearer(&t.g, P0), Some(bears));
    }
    assert_eq!(t.life(P0), 16);
    assert_eq!(drawn(&t, P0), 2);
}

#[test]
fn choosing_the_same_ring_bearer_again_counts_for_other_than_conditions() {
    cr!("701.54a", "603.4");
    // The same ruling, for the cards whose abilities care about which creature was
    // chosen: "if you chose a creature other than ~ as your Ring-bearer".
    ruling!(
        "Faramir, Field Commander",
        "If the creature you choose as your Ring-bearer was already your Ring-bearer, that still counts as choosing that creature as your Ring-bearer for the purpose of abilities that trigger \"whenever you choose a creature as your Ring-bearer\" or abilities that care about which creature was chosen as your Ring-bearer."
    );
    supported("Faramir, Field Commander");
    // Faramir: "Whenever the Ring tempts you, if you chose a creature other than Faramir as
    // your Ring-bearer, create a 1/1 white Human Soldier creature token."
    let mut t = TestGame::new(2);
    let faramir = t.battlefield(P0, "Faramir, Field Commander");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // The Bears twice (the second time already the Ring-bearer), then Faramir.
    for (choice, made) in [(bears, 1), (bears, 2), (faramir, 2)] {
        t.answer_choose(P0, &[obj(choice)]);
        tempt(&mut t.g, P0, None);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(ring_bearer(&t.g, P0), Some(choice));
        assert_eq!(tokens(&t, P0).len(), made, "{choice:?}");
    }
}

fn deck(cards: &[(&str, usize)]) -> Vec<Arc<CardDef>> {
    let mut d = Vec::new();
    for (n, k) in cards {
        d.extend(std::iter::repeat_n(card(n), *k));
    }
    d
}

fn too_many(problems: &[DeckProblem]) -> bool {
    problems
        .iter()
        .any(|p| matches!(p, DeckProblem::TooManyCopies { name, .. } if name == "Nazgûl"))
}

#[test]
fn nazgul_deck_limits() {
    cr!("100.2a", "100.2b", "113.6n", "903.5b", "100.6");
    ruling!(
        "Nazgûl",
        "If you're fortunate enough to open or draft ten or more Nazgûl during a Limited event, you can include only nine of them in your deck. The rest remain in your sideboard."
    );
    ruling!(
        "Nazgûl",
        "The last ability of Nazgûl also lets you ignore the \"singleton\" rule in formats like Commander."
    );
    ruling!(
        "Nazgûl",
        "The last ability of Nazgûl lets you ignore the \"four-of\" rule. It doesn't let you ignore format legality."
    );
    supported("Nazgûl");
    // Constructed: nine, not ten.
    assert!(check_constructed(&deck(&[("Nazgûl", 9), ("Swamp", 51)])).is_empty());
    assert!(too_many(&check_constructed(&deck(&[
        ("Nazgûl", 10),
        ("Swamp", 50)
    ]))));
    // Limited with ten in the pool: nine in the deck, the tenth in the sideboard.
    let pool = deck(&[("Nazgûl", 10), ("Grizzly Bears", 40)]);
    let d = deck(&[("Nazgûl", 9), ("Grizzly Bears", 31)]);
    assert!(check_limited(&d, &pool).is_empty());
    let side = limited_sideboard(&pool, &d);
    assert_eq!(side.iter().filter(|c| c.name == "Nazgûl").count(), 1);
    let d = deck(&[("Nazgûl", 10), ("Grizzly Bears", 30)]);
    assert!(too_many(&check_limited(&d, &pool)));
    // Limited: no Nazgûl from outside the pool.
    let pool = deck(&[("Nazgûl", 2), ("Grizzly Bears", 40)]);
    let d = deck(&[("Nazgûl", 3), ("Grizzly Bears", 37)]);
    assert!(check_limited(&d, &pool)
        .iter()
        .any(|p| matches!(p, DeckProblem::NotInPool { .. })));
    // Commander: nine Nazgûl with the Witch-king.
    let wk = card("Witch-king of Angmar");
    let d = deck(&[("Nazgûl", 9), ("Swamp", 90)]);
    assert!(!too_many(&check_commander(&d, &wk, &[], false)));
    let d = deck(&[("Nazgûl", 10), ("Swamp", 89)]);
    assert!(too_many(&check_commander(&d, &wk, &[], false)));
    // Format legality still applies.
    let legal = card("Nazgûl").is_legal_in("standard");
    let problems = check_format_legality(&deck(&[("Nazgûl", 9), ("Swamp", 51)]), &[], "standard");
    assert_eq!(problems.is_empty(), legal);
    assert!(!legal, "Nazgûl isn't Standard-legal");
}
