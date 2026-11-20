//! CR 121.8 and "Whenever you reveal a [card] this way" after "(You may) reveal the first
//! card you draw each turn (as you draw it)": "If a spell or ability causes a card to be
//! drawn while another spell is being cast, the drawn card is kept face down until that
//! spell becomes cast (see rule 601.2i) ... If an effect allows or instructs a player to
//! reveal the card as it's being drawn, it's revealed after the spell becomes cast or the
//! ability becomes activated. ..." The reveal after the cast is the reveal "this way", so
//! the triggered ability triggers then.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// "Whenever you tap a land for mana, add {G} and draw a card." — a triggered mana
/// ability (CR 605.1b), so it draws while a spell is being cast.
fn drawing_well() -> CardDef {
    compile_def(
        "Drawing Well",
        "Enchantment",
        "{2}{G}",
        "Whenever you tap a land for mana, add {G} and draw a card.",
    )
}

/// A {G}{G} instant: "You gain 1 life."
fn green_gift() -> CardDef {
    CB::new("Green Gift")
        .instant()
        .cost("{G}{G}")
        .spell(Body::effect(gain(1)))
        .build()
}

/// P0 controls `source`, Drawing Well and a Forest, their library is `library` (the last
/// one named on top), and they have drawn no card this turn. Returns the game and Green
/// Gift in P0's hand: casting it taps the Forest, and Drawing Well draws the top card
/// while it's being cast.
fn casting_draws(source: &str, library: &[&str]) -> (TestGame, ObjectId) {
    supported(source);
    let mut t = TestGame::new(2);
    t.battlefield(P0, source);
    t.custom(P0, drawing_well(), Zone::Battlefield);
    t.lands(P0, "Forest", 1);
    t.g.players[0].library.clear();
    for n in library {
        t.library_top(P0, n);
    }
    let gift = t.custom(P0, green_gift(), Zone::Hand(P0));
    (t, gift)
}

/// The names of the sources of the triggered abilities on the stack, bottom first.
fn triggers_on_stack(t: &TestGame) -> Vec<String> {
    t.g.stack
        .iter()
        .filter_map(|s| match t.g.obj(*s).stack.as_deref().map(|si| &si.kind) {
            Some(StackKind::Triggered { source, .. }) => {
                Some(t.g.obj(*source).chars.name.to_string())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn inquisitor_eisenhorn_triggers_for_a_card_revealed_after_the_spell_becomes_cast() {
    cr!("121.8", "601.2i", "603.2", "701.20a");
    ruling!(
        "Inquisitor Eisenhorn",
        "You don't have to reveal a drawn card if you don't wish to create a Cherubael token."
    );
    // The reveal question comes once the spell is cast (it's on the stack), and the
    // triggered ability triggers then.
    for reveal in [true, false] {
        let (mut t, gift) = casting_draws("Inquisitor Eisenhorn", &["Shock"]);
        let log = spy(&mut t, P0, |g, _p, d| match d {
            Decision::YesNo { prompt, .. } if prompt.contains("Reveal") => Some(format!(
                "stack={} spell_cast={}",
                g.stack.len(),
                g.history.spells_cast.len()
            )),
            _ => None,
        });
        t.answer_yes(P0, reveal);
        t.cast(P0, gift).go();
        assert_eq!(probe_lines(&log), vec!["stack=1 spell_cast=1".to_string()]);
        t.settle();
        if reveal {
            // Above the spell: it triggered after the spell became cast.
            assert_eq!(triggers_on_stack(&t), vec!["Inquisitor Eisenhorn".to_string()]);
            assert_eq!(t.g.obj(*t.g.stack.last().unwrap()).kind, ObjKind::StackAbility);
        } else {
            assert!(triggers_on_stack(&t).is_empty());
        }
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Cherubael").len(),
            usize::from(reveal),
            "{reveal}"
        );
        assert!(t.in_hand(P0, "Shock"));
        assert_eq!(t.life(P0), 21);
    }
    // A card that isn't an instant or sorcery, revealed: no trigger.
    let (mut t, gift) = casting_draws("Inquisitor Eisenhorn", &["Grizzly Bears"]);
    t.answer_yes(P0, true);
    t.cast(P0, gift).go();
    t.settle();
    assert!(triggers_on_stack(&t).is_empty());
}

#[test]
fn god_eternal_kefnet_copies_a_card_revealed_after_the_spell_becomes_cast() {
    cr!("121.8", "603.2", "707.12");
    // Divination, drawn while Green Gift is being cast and revealed afterward: Kefnet's
    // ability copies it, and P0 casts the copy for {U} (two less), drawing two cards.
    let (mut t, gift) = casting_draws("God-Eternal Kefnet", &["Island", "Island", "Divination"]);
    let island = t.lands(P0, "Island", 1)[0];
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    t.cast(P0, gift).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t), vec!["God-Eternal Kefnet".to_string()]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_hand(P0, "Divination"));
    assert!(t.obj_now(island).tapped);
}

#[test]
fn primitive_etchings_triggers_for_a_card_revealed_after_the_spell_becomes_cast() {
    cr!("121.8", "603.2", "701.20a");
    // "Reveal the first card you draw each turn. Whenever you reveal a creature card this
    // way, draw a card.": the Grizzly Bears drawn while Green Gift is being cast is
    // revealed afterward, and P0 draws a card.
    let (mut t, gift) = casting_draws("Primitive Etchings", &["Island", "Grizzly Bears"]);
    t.cast(P0, gift).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t), vec!["Primitive Etchings".to_string()]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Island"));
}

#[test]
fn each_may_reveal_ability_is_chosen_for_separately() {
    // 607.2h: "If an object has both a static ability and one or more triggered abilities
    // printed on it in the same paragraph, each of those triggered abilities is linked to
    // the static ability. Each triggered ability refers only to actions taken as a result
    // of the static ability."
    cr!("603.2", "607.2h", "121.9");
    ruling!(
        "God-Eternal Kefnet",
        "If you somehow control more than one God-Eternal Kefnet (perhaps because one is a Spark Double), you may reveal a card you draw for any number of their abilities."
    );
    // God-Eternal Kefnet and a nonlegendary creature with its ability (a Spark Double
    // copying it would be one): P0 reveals the drawn Shock for some of the two abilities,
    // and only those trigger.
    supported("God-Eternal Kefnet");
    let echo = || {
        compile_def(
            "Kefnet's Echo",
            "Creature — Zombie Bird God",
            "{2}{U}{U}",
            "You may reveal the first card you draw each turn as you draw it. Whenever you reveal an instant or sorcery card this way, copy that card and you may cast the copy. That copy costs {2} less to cast.",
        )
    };
    for (first, second) in [(true, false), (false, true), (true, true), (false, false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "God-Eternal Kefnet");
        t.custom(P0, echo(), Zone::Battlefield);
        t.library_top(P0, "Shock");
        t.answer_yes(P0, first);
        t.answer_yes(P0, second);
        t.g.draw_cards(P0, 1);
        t.settle();
        let expected: Vec<String> = [(first, "God-Eternal Kefnet"), (second, "Kefnet's Echo")]
            .into_iter()
            .filter(|(yes, _)| *yes)
            .map(|(_, n)| n.to_string())
            .collect();
        let mut on_stack = triggers_on_stack(&t);
        on_stack.sort();
        assert_eq!(on_stack, expected, "{first} {second}");
    }
}
