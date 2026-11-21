//! Rulings batch P049 — tokens that are copies (CR 707.2, 111.4): Kaya, Intangible
//! Slayer's and Kambal, Profiteering Mayor's copies have the copied permanent's enters
//! abilities and copy only its copiable values; Ghired, Mirror of the Wilds' copies of
//! tokens; and Jeskai Revelation returning a spell or a copy of a spell to its owner's
//! hand (CR 707.10, 704.5e).

use crate::r_p049_common::*;
use crate::r_s25_common::keep_copy_targets;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The tokens named `name` on the battlefield.
fn tokens_named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// Kaya, Intangible Slayer's −3 on `target`, resolved: the token copy it made.
fn kaya_exiles(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("Kaya, Intangible Slayer");
    let kaya = t.battlefield(P0, "Kaya, Intangible Slayer");
    let name = name_now(t, target);
    t.answer_targets(P0, &[obj(target)]);
    activate_containing(t, P0, kaya, "Exile target").unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(target));
    let toks = tokens_named(t, &name);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    toks[0]
}

#[test]
fn kayas_token_has_the_copied_permanents_enters_abilities() {
    cr!("707.2", "603.6a", "614.1c", "707.9b");
    ruling!(
        "Kaya, Intangible Slayer",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the token enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the chosen permanent will also work."
    );
    // Elvish Visionary: "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    let v = t.battlefield(P1, "Elvish Visionary");
    let hand = t.hand_size(P0);
    let tok = kaya_exiles(&mut t, v);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.obj_now(tok).controller, P0);
    // Spike Feeder: "This creature enters with two +1/+1 counters on it." The copy is a
    // 1/1 Spirit with those counters.
    supported("Spike Feeder");
    let mut t = TestGame::new(2);
    let f = t.enter(P1, "Spike Feeder");
    t.settle();
    assert_eq!(t.counters(f, counters::PLUS1), 2);
    let tok = kaya_exiles(&mut t, f);
    assert_eq!(t.counters(tok, counters::PLUS1), 2);
    assert_eq!(t.pt(tok), (3, 3));
    assert!(t.obj_now(tok).chars.has_subtype("Spirit"));
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Flying));
}

#[test]
fn kayas_token_copies_only_the_copiable_values() {
    cr!("707.2", "707.9b", "111.4");
    ruling!(
        "Kaya, Intangible Slayer",
        "Except for the listed exceptions, the token copies exactly what was printed on the original permanent and nothing else"
    );
    supported("Pacifism");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    // Tapped, with a +1/+1 counter, enchanted, and pumped: none of that is copied.
    t.g.objects[bear.0 as usize].tapped = true;
    put_counters(&mut t, bear, counters::PLUS1, 1);
    let pacifism = attach_new(&mut t, P1, "Pacifism", bear);
    modify_until_eot(
        &mut t,
        bear,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
    assert_eq!(t.pt(bear), (6, 6));
    let tok = kaya_exiles(&mut t, bear);
    let o = t.obj_now(tok);
    assert!(!o.tapped);
    assert_eq!(o.counter(counters::PLUS1), 0);
    assert_eq!(t.pt(tok), (1, 1));
    assert_eq!(name_now(&t, tok), "Grizzly Bears");
    // Pacifism went to the graveyard; nothing is attached to the token.
    assert!(!t.on_battlefield(pacifism));
    assert!(t
        .g
        .permanents()
        .all(|p| p.attached_to != Some(obj(tok))));
}

#[test]
fn ghireds_copy_of_a_token_has_the_tokens_enters_abilities() {
    cr!("707.2", "603.6a", "614.1c");
    ruling!(
        "Ghired, Mirror of the Wilds",
        "Any enters-the-battlefield abilities of the copied token will trigger when the token enters the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the copied token will also work."
    );
    supported("Ghired, Mirror of the Wilds");
    supported("Spike Feeder");
    supported("Elvish Visionary");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ghired, Mirror of the Wilds");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    // A token with "enters with two +1/+1 counters" entered this turn: Grizzly Bears taps
    // to copy it (Ghired grants "{T}: Create a token that's a copy of target token you
    // control that entered this turn.").
    let feeder = token_like(&mut t, P0, "Spike Feeder");
    assert_eq!(t.counters(feeder, counters::PLUS1), 2);
    t.answer_targets(P0, &[obj(feeder)]);
    activate_containing(&mut t, P0, bears, "copy of target token").unwrap();
    t.resolve_all();
    let feeders = tokens_named(&t, "Spike Feeder");
    assert_eq!(feeders.len(), 2);
    for f in feeders {
        assert_eq!(t.counters(f, counters::PLUS1), 2);
    }
    // A token with "when this enters, draw a card": the copy draws too.
    let visionary = token_like(&mut t, P0, "Elvish Visionary");
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[obj(visionary)]);
    activate_containing(&mut t, P0, giant, "copy of target token").unwrap();
    t.resolve_all();
    assert_eq!(tokens_named(&t, "Elvish Visionary").len(), 2);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn kambals_copy_of_an_opponents_token_has_its_enters_abilities() {
    cr!("707.2", "603.6a", "603.2c");
    ruling!(
        "Kambal, Profiteering Mayor",
        "Any enters-the-battlefield abilities of the copied token will trigger when the token enters the battlefield."
    );
    supported("Kambal, Profiteering Mayor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kambal, Profiteering Mayor");
    // P1 creates a token with "when this enters, draw a card": P1 draws for it, and P0
    // draws for the tapped copy Kambal makes (and drains 1 for a token entering).
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    token_like(&mut t, P1, "Elvish Visionary");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), h1 + 1);
    let copies: Vec<ObjectId> = tokens_named(&t, "Elvish Visionary")
        .into_iter()
        .filter(|id| t.obj(*id).controller == P0)
        .collect();
    assert_eq!(copies.len(), 1);
    assert!(tapped(&t, copies[0]));
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn kambals_copy_has_only_the_tokens_copiable_values() {
    cr!("707.2", "707.9b");
    ruling!(
        "Kambal, Profiteering Mayor",
        "Each of the new tokens copies the original characteristics of the appropriate token as stated by the effect that created that token and nothing else"
    );
    supported("Kambal, Profiteering Mayor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kambal, Profiteering Mayor");
    // P1's 1/1 Soldier token enters; before Kambal's trigger resolves it's pumped and gets
    // a +1/+1 counter. The copy is a plain 1/1.
    let soldier = create_token(&mut t, P1, "Soldier");
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "copy"), 1);
    modify_until_eot(
        &mut t,
        soldier,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
    put_counters(&mut t, soldier, counters::PLUS1, 1);
    assert_eq!(t.pt(soldier), (5, 5));
    t.resolve_all();
    let copies: Vec<ObjectId> = tokens_named(&t, "Soldier")
        .into_iter()
        .filter(|id| t.obj(*id).controller == P0)
        .collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(t.pt(copies[0]), (1, 1));
    assert_eq!(t.counters(copies[0], counters::PLUS1), 0);
    assert!(tapped(&t, copies[0]));
}

/// P1 casts `name` (at `target`), then P0 casts Jeskai Revelation targeting the spell (or
/// `spell_target` instead) and P1 for the damage; everything resolves.
fn revelation_returns(t: &mut TestGame, spell_target: ObjectId) {
    supported("Jeskai Revelation");
    let hand = t.hand_size(P0);
    let life = t.life(P0);
    cast_new(
        t,
        P0,
        "Jeskai Revelation",
        &[obj(spell_target), Entity::Player(P1)],
    );
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), life + 4);
    assert_eq!(tokens_of(t, P0, "Monk"), 2);
}

#[test]
fn a_copy_of_a_spell_returned_to_hand_ceases_to_exist() {
    cr!("707.10", "704.5e");
    ruling!(
        "Jeskai Revelation",
        "If a copy of a spell is returned to its owner’s hand, it’s moved there, then it will cease to exist as a state-based action. It can’t be recast."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    cast_new(&mut t, P1, "Twincast", &[obj(bolt)]);
    keep_copy_targets(&mut t, P1);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    let hand = t.hand_size(P1);
    revelation_returns(&mut t, copies[0]);
    // The copy is gone: not on the stack, not in P1's hand.
    assert!(spell_copies(&t).is_empty());
    assert_eq!(t.hand_size(P1), hand);
    assert!(!t.g.player(P1).hand.contains(&t.g.current(copies[0])));
    // The original Lightning Bolt still resolves.
    assert_eq!(stack(&t), vec![bolt]);
    t.resolve_all();
    assert_eq!(t.life(P0), 24 - 3);
}

#[test]
fn a_spell_returned_to_hand_isnt_countered_and_doesnt_resolve() {
    cr!("608.2", "400.7");
    ruling!(
        "Jeskai Revelation",
        "If a spell is returned to its owner’s hand, it’s removed from the stack and thus will not resolve. The spell isn’t countered; it just no longer exists. This works even against a spell that can’t be countered."
    );
    supported("Abrupt Decay");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    // Abrupt Decay: "This spell can't be countered. Destroy target nonland permanent with
    // mana value 3 or less."
    let decay = cast_new(&mut t, P1, "Abrupt Decay", &[obj(bear)]);
    let hand = t.hand_size(P1);
    revelation_returns(&mut t, decay);
    assert_eq!(t.zone(decay), Zone::Hand(P1));
    assert_eq!(t.hand_size(P1), hand + 1);
    assert!(t.in_hand(P1, "Abrupt Decay"));
    assert!(stack(&t).is_empty());
    assert!(t.on_battlefield(bear));
}
