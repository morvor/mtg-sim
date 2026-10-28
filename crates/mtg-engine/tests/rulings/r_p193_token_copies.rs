//! Rulings batch P193 — copies of tokens that entered this turn (Redoubled Stormsinger,
//! Renewed Solidarity, Ocelot Pride): the copies use the copiable values (CR 707.2), their
//! enters abilities work, entering attacking isn't attacking (CR 508.4), and a copy of a
//! permanent spell isn't "created" (CR 111.12, Ellyn Harbreeze, Thalisse).

use crate::r_s01_common::*;
use crate::r_s02_common::create_token;
use crate::r_s05_common::tokens_with_subtype;
use crate::r_s17_common::{become_copy, token_copy};
use crate::r_s24_common::choose_creature_type;
use crate::r_s29_common::put_counters;
use crate::r_p035_common::to_end_step;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn named_tokens(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    tokens(t, p)
        .into_iter()
        .filter(|id| t.obj(*id).chars.name == name)
        .collect()
}

#[test]
fn redoubled_stormsinger_tokens_enter_attacking_without_attack_triggers() {
    cr!("508.4", "603.6a", "707.2");
    ruling!(
        "Redoubled Stormsinger",
        "Although the tokens are attacking, they were never declared as attacking creatures. This means that abilities that trigger whenever a creature attacks won't trigger when these tokens enter attacking."
    );
    ruling!(
        "Redoubled Stormsinger",
        "Any enters-the-battlefield abilities of the copied token will trigger when the token enters the battlefield."
    );
    ruling!(
        "Redoubled Stormsinger",
        "Each of the tokens created by Redoubled Stormsinger's last ability copies the original characteristics of the token it's copying as stated by the effect that created that token."
    );
    supported("Redoubled Stormsinger");
    supported("Hellrider");
    supported("Elvish Visionary");
    let mut t = TestGame::new(2);
    let hellrider = t.battlefield(P0, "Hellrider");
    let singer = t.battlefield(P0, "Redoubled Stormsinger");
    let visionary = t.hand(P0, "Elvish Visionary");
    let tok = token_copy(&mut t, P0, visionary)[0];
    t.resolve_all();
    put_counters(&mut t, tok, counters::PLUS1, 2);
    t.g.tap(tok);
    let hand = t.hand_size(P0);
    crate::r_s03_common::to_blockers(&mut t, &[(singer, Entity::Player(P1))], &[]);
    // Hellrider triggered once (for the Stormsinger), and its damage resolved.
    assert_eq!(crate::r_s11_common::triggered_from(&t, hellrider), 1);
    let copies: Vec<ObjectId> = named_tokens(&t, P0, "Elvish Visionary")
        .into_iter()
        .filter(|id| *id != tok)
        .collect();
    assert_eq!(copies.len(), 1);
    let copy = copies[0];
    assert!(crate::r_s10_common::attacking(&t, copy));
    assert_eq!(t.counters(copy, counters::PLUS1), 0);
    assert_eq!(t.pt(copy), (1, 1));
    // The copy's enters trigger (draw a card) resolved.
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn stormsinger_and_renewed_solidarity_copy_what_a_token_is_copying_and_its_enters_with() {
    cr!("707.2", "707.4", "614.1c");
    ruling!(
        "Redoubled Stormsinger",
        "If a creature token you control that entered this turn is copying something else, then the token enters the battlefield as whatever that creature token copied."
    );
    ruling!(
        "Redoubled Stormsinger",
        "Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the copied token will also work."
    );
    ruling!(
        "Renewed Solidarity",
        "If the last ability creates a copy of a token that is copying something else, then the new token enters as a copy of whatever the original token copied."
    );
    ruling!(
        "Renewed Solidarity",
        "Any abilities of the copied token that trigger when that permanent enters will trigger. Any “as [this permanent] enters” or “[this permanent] enters with” abilities of the copied token will also work."
    );
    supported("Renewed Solidarity");
    supported("Cryptborn Horror");
    // A Soldier token that became a copy of Cryptborn Horror ("enters with X +1/+1
    // counters, where X is the total life lost by your opponents this turn").
    for renewed in [false, true] {
        let mut t = TestGame::new(2);
        t.g.lose_life(P1, 3);
        t.g.flush_events();
        let horror = t.graveyard(P0, "Cryptborn Horror");
        let soldier = create_token(&mut t, P0, "Soldier");
        // A counter first, so the 0/0 copy survives.
        put_counters(&mut t, soldier, counters::PLUS1, 1);
        become_copy(&mut t, soldier, horror);
        if renewed {
            choose_creature_type(&mut t, P0, "Horror");
            t.enter(P0, "Renewed Solidarity");
            to_end_step(&mut t, P0);
            t.resolve_all();
        } else {
            let singer = t.battlefield(P0, "Redoubled Stormsinger");
            crate::r_s03_common::to_blockers(&mut t, &[(singer, Entity::Player(P1))], &[]);
        }
        let copies: Vec<ObjectId> = named_tokens(&t, P0, "Cryptborn Horror")
            .into_iter()
            .filter(|id| *id != t.g.current(soldier))
            .collect();
        assert_eq!(copies.len(), 1, "renewed: {renewed}");
        assert_eq!(t.counters(copies[0], counters::PLUS1), 3, "renewed: {renewed}");
    }
}

#[test]
fn renewed_solidarity_copies_trigger_their_enters_abilities() {
    cr!("707.2", "603.6a");
    ruling!(
        "Renewed Solidarity",
        "Any abilities of the copied token that trigger when that permanent enters will trigger."
    );
    let mut t = TestGame::new(2);
    let visionary = t.hand(P0, "Elvish Visionary");
    token_copy(&mut t, P0, visionary);
    t.resolve_all();
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Renewed Solidarity");
    // A token of another type isn't copied.
    create_token(&mut t, P0, "Soldier");
    let hand = t.hand_size(P0);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(named_tokens(&t, P0, "Elvish Visionary").len(), 2);
    assert_eq!(tokens_with_subtype(&t, P0, "Soldier").len(), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn ocelot_pride_gets_the_citys_blessing_from_its_own_cat_and_copies_original_values() {
    cr!("702.131c", "707.2", "608.2c");
    ruling!(
        "Ocelot Pride",
        "If the creature token created by Ocelot Pride's last ability is your tenth permanent, you'll get the city's blessing before the ability would check to see if you have the city's blessing."
    );
    ruling!(
        "Ocelot Pride",
        "The newly created token doesn't copy whether the original token is tapped or untapped, whether it has any counters on it, whether it has any Auras and/or Equipment attached to it, or any non-copy effects that changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Ocelot Pride",
        "If the original token is copying something else, the token copy you create will use the copiable values of the original token. In most cases, it will be a copy of whatever the original token is copying."
    );
    supported("Ocelot Pride");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ocelot Pride");
    // Seven lands + a token (copying Grizzly Bears, tapped, with a counter) = 9 permanents.
    t.lands(P0, "Plains", 7);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let soldier = create_token(&mut t, P0, "Soldier");
    become_copy(&mut t, soldier, bears);
    put_counters(&mut t, soldier, counters::PLUS1, 1);
    t.g.tap(soldier);
    assert!(!t.g.player(P0).has_citys_blessing);
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert!(t.g.player(P0).has_citys_blessing);
    // The Cat (the tenth permanent) and the Bears token were each copied.
    assert_eq!(tokens_with_subtype(&t, P0, "Cat").len(), 2);
    let bears_tokens = named_tokens(&t, P0, "Grizzly Bears");
    assert_eq!(bears_tokens.len(), 2);
    let copy = *bears_tokens
        .iter()
        .find(|id| **id != t.g.current(soldier))
        .unwrap();
    assert!(!t.obj(copy).tapped);
    assert_eq!(t.counters(copy, counters::PLUS1), 0);
    assert_eq!(t.pt(copy), (2, 2));
}

#[test]
fn a_copy_of_a_permanent_spell_isnt_a_created_token() {
    cr!("111.12", "707.10");
    ruling!(
        "Ellyn Harbreeze, Busybody",
        "If an effect creates a copy of a permanent spell, that spell becomes a token on the battlefield under your control, but that token has not been \"created.\" It won't count for Ellyn's ability."
    );
    ruling!(
        "Thalisse, Reverent Medium",
        "If an effect creates a copy of a permanent spell, that spell becomes a token on the battlefield under your control, but that token has not been \"created.\" It won't count for Thalisse's ability."
    );
    supported("Ellyn Harbreeze, Busybody");
    supported("Thalisse, Reverent Medium");
    supported("Double Major");
    for really_create in [false, true] {
        let mut t = TestGame::new(2);
        let ellyn = t.battlefield(P0, "Ellyn Harbreeze, Busybody");
        t.battlefield(P0, "Thalisse, Reverent Medium");
        t.lands(P0, "Forest", 3);
        t.lands(P0, "Island", 1);
        let bears = t.hand(P0, "Grizzly Bears");
        let spell = t.cast(P0, bears).go();
        let dm = t.hand(P0, "Double Major");
        t.cast(P0, dm).target(spell).go();
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
        if really_create {
            create_token(&mut t, P0, "Soldier");
        }
        let n = usize::from(really_create);
        // Ellyn: look at the top X cards, put one into your hand.
        let hand = t.hand_size(P0);
        t.activate(P0, ellyn, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + n, "created: {really_create}");
        to_end_step(&mut t, P0);
        t.resolve_all();
        assert_eq!(tokens_with_subtype(&t, P0, "Spirit").len(), n);
    }
}
