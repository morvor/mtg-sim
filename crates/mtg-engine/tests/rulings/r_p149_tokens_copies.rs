//! Rulings batch P149 — tokens and copies: copies of permanent spells becoming tokens that
//! aren't "created" (Archmage of Echoes, Storm of Saruman), Adrix and Nev's extra tokens,
//! casting a copy with X (Nashi), and countering an uncounterable spell (Koma).

use crate::r_p149_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The +1/+1 counters Rosie Cotton's "Whenever you create a token, put a +1/+1 counter on
/// target creature you control other than Rosie Cotton." has put on `bears`.
fn rosie_counters(t: &TestGame, bears: ObjectId) -> u32 {
    t.counters(bears, counters::PLUS1)
}

#[test]
fn archmage_of_echoes_copy_becomes_a_token_that_isnt_created() {
    cr!("608.3f", "111.1", "707.10f");
    ruling!(
        "Archmage of Echoes",
        "A resolving copy of a permanent spell becomes a token, so the token isn't \"created.\" Effects that care about a token being created won't interact with a token that enters the battlefield from Archmage of Echoes's ability."
    );
    supported("Archmage of Echoes");
    supported("Adrix and Nev, Twincasters");
    supported("Rosie Cotton of South Lane");
    supported("Faerie Seer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archmage of Echoes");
    t.battlefield(P0, "Adrix and Nev, Twincasters");
    t.battlefield(P0, "Rosie Cotton of South Lane");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    cast_at(&mut t, P0, "Faerie Seer", &[]);
    t.resolve_all();
    // One Faerie Seer token (not doubled by Adrix and Nev), and no Rosie trigger.
    let seers: Vec<_> = t
        .named_on_battlefield("Faerie Seer")
        .into_iter()
        .filter(|id| t.g.obj(*id).is_token())
        .collect();
    assert_eq!(seers.len(), 1);
    assert_eq!(t.named_on_battlefield("Faerie Seer").len(), 2);
    assert_eq!(rosie_counters(&t, bears), 0);
}

#[test]
fn storm_of_saruman_copy_token_doesnt_trigger_rosie_cotton() {
    cr!("608.3f", "111.1", "707.10f");
    ruling!(
        "Storm of Saruman",
        "The token that a resolving copy of a permanent spell becomes isn't said to have been \"created\" and won't interact with abilities that care about tokens being created, such as that of Rosie Cotton of South Lane."
    );
    supported("Storm of Saruman");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Storm of Saruman");
    t.battlefield(P0, "Rosie Cotton of South Lane");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    cast_at(&mut t, P0, "Hill Giant", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
    assert_eq!(rosie_counters(&t, bears), 0);
    // Tokens that are created do trigger it (once for each of Raise the Alarm's two).
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(bears)]);
    cast_at(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    assert_eq!(rosie_counters(&t, bears), 2);
}

#[test]
fn adrix_and_nev_extra_tokens_are_tapped_attacking_and_get_the_counters() {
    cr!("614.1a", "111.1", "508.4");
    ruling!(
        "Adrix and Nev, Twincasters",
        "Everything that is specified by the effect creating the original token or tokens will also be true about the additional token or tokens created by Adrix and Nev's replacement effect. For example, if an effect tells you to create a token \"tapped and attacking\" the additional tokens will also be tapped and attacking. Similarly, if an effect creates a token and puts counters on it (such as a Fractal token) the additional token will also get those counters."
    );
    supported("Hero of Bladehold");
    supported("Fractal Summoning");
    // Hero of Bladehold: "create two 1/1 white Soldier creature tokens that are tapped and
    // attacking": four of them.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Adrix and Nev, Twincasters");
    let hero = t.battlefield(P0, "Hero of Bladehold");
    attack_with(&mut t, &[(hero, pl(P1))]);
    t.resolve_all();
    let soldiers = crate::r_s01_common::with_subtype(&t, P0, "Soldier")
        .into_iter()
        .filter(|id| t.g.obj(*id).is_token())
        .collect::<Vec<_>>();
    assert_eq!(soldiers.len(), 4);
    for s in &soldiers {
        assert!(t.obj_now(*s).tapped);
        assert!(t.g.is_attacking(*s));
    }
    // Fractal Summoning with X = 2: two Fractals, each with two +1/+1 counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Adrix and Nev, Twincasters");
    lots(&mut t, P0);
    let spell = t.hand(P0, "Fractal Summoning");
    t.cast(P0, spell).x(2).go();
    t.resolve_all();
    let fractals = crate::r_s01_common::with_subtype(&t, P0, "Fractal");
    assert_eq!(fractals.len(), 2);
    for f in &fractals {
        assert_eq!(t.counters(*f, counters::PLUS1), 2);
        assert_eq!(t.pt(*f), (2, 2));
    }
}

#[test]
fn nashi_copy_with_x_chooses_x_as_normal() {
    cr!("707.12", "601.2b", "107.3");
    ruling!(
        "Nashi, Moon's Legacy",
        "Because you're paying the spell's costs, if the spell has {X} in its mana cost, you may choose its value as normal."
    );
    supported("Nashi, Moon's Legacy");
    supported("Verazol, the Split Current");
    // Verazol ({X}{G}{U}: "enters with a +1/+1 counter on it for each mana spent to cast
    // it") copied and cast with X = 3: five mana spent.
    let mut t = TestGame::new(2);
    let nashi = t.battlefield(P0, "Nashi, Moon's Legacy");
    let verazol = t.graveyard(P0, "Verazol, the Split Current");
    t.answer_targets(P0, &[obj(verazol)]);
    t.answer_yes(P0, true);
    crate::r_p153_common::answer_x(&mut t, P0, 3);
    attack_with(&mut t, &[(nashi, pl(P1))]);
    lots(&mut t, P0);
    t.resolve_all();
    let copies = t.named_on_battlefield("Verazol, the Split Current");
    assert_eq!(copies.len(), 1);
    assert!(t.g.obj(copies[0]).is_token());
    assert_eq!(t.counters(copies[0], counters::PLUS1), 5);
}

#[test]
fn koma_can_be_targeted_by_a_counterspell_whose_other_effects_happen() {
    cr!("101.2", "608.2b", "115.1");
    ruling!(
        "Koma, World-Eater",
        "A spell or ability that counters spells can still target Koma. When that spell or ability resolves, Koma won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    supported("Koma, World-Eater");
    supported("Exclude");
    // Exclude: "Counter target creature spell. Draw a card."
    let mut t = TestGame::new(2);
    let koma = cast_at(&mut t, P0, "Koma, World-Eater", &[]);
    t.g.turn.priority = Some(P1);
    let hand = t.hand_size(P1);
    cast_at(&mut t, P1, "Exclude", &[obj(koma)]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Koma, World-Eater").len(), 1);
    assert_eq!(t.hand_size(P1), hand + 1);
}
