//! Rulings batch P208 — enchant: Auras that return the enchanted card to the battlefield
//! when it dies (or is exiled): Kaya's Ghostform and Unholy Indenture.

use crate::r_p209_common::cast_spell;
use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s06_common::*;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const GHOSTFORM: &str = "Kaya's Ghostform";

#[test]
fn unholy_indenture_returns_a_nontoken_with_a_counter_but_not_a_token() {
    cr!("111.7", "111.8", "122.6", "603.10a");
    ruling!(
        "Unholy Indenture",
        "If Unholy Indenture enchants a token, it won’t be returned to the battlefield when it dies."
    );
    supported("Unholy Indenture");
    let mut t = TestGame::new(2);
    let tok = create_token(&mut t, P1, "Soldier");
    attach_new(&mut t, P0, "Unholy Indenture", tok);
    destroy(&mut t, tok);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert!(tokens(&t, P1).is_empty());
    // A nontoken creature returns under your control with a +1/+1 counter.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Unholy Indenture", bears);
    destroy(&mut t, bears);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P0);
    assert_eq!(t.counters(back[0], counters::PLUS1), 1);
    assert_eq!(t.pt(back[0]), (3, 3));
}

#[test]
fn kayas_ghostform_returns_a_creature_that_dies_or_is_exiled() {
    cr!("603.10a", "603.1b", "400.7");
    supported(GHOSTFORM);
    supported("Swords to Plowshares");
    // Dies.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, GHOSTFORM, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, GHOSTFORM, bears);
    cast_spell(&mut t, P1, "Swords to Plowshares", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn kayas_ghostform_leaving_with_its_permanent_still_returns_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Kaya's Ghostform",
        "If Kaya’s Ghostform and the enchanted permanent are both put into graveyards and/or exiled at the same time, the enchanted permanent will be returned to the battlefield."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let g = attach_new(&mut t, P0, GHOSTFORM, bears);
    t.g.destroy_all(vec![bears, g], None, false);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_graveyard(P0, GHOSTFORM));
}

#[test]
fn kayas_ghostform_returns_a_permanent_you_dont_own_under_your_control() {
    cr!("303.4e", "800.4a", "110.2a");
    ruling!(
        "Kaya's Ghostform",
        "If Kaya’s Ghostform enchants a permanent you control but don’t own, that permanent will return to the battlefield under your control when it dies or is exiled."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_control(&mut t, bears, P0);
    attach_new(&mut t, P0, GHOSTFORM, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P0);
    assert_eq!(t.g.obj(back[0]).owner, P1);
    // Multiplayer: if you leave the game, it's exiled.
    let mut t = TestGame::with_config(3, GameConfig::default());
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_control(&mut t, bears, P0);
    attach_new(&mut t, P0, GHOSTFORM, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    t.g.lose_game(P0);
    t.settle();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn kayas_ghostform_and_flicker_or_temporary_exile() {
    cr!("400.7", "610.3c", "603.10a");
    ruling!(
        "Kaya's Ghostform",
        "If an effect exiles the enchanted permanent and immediately returns it to the battlefield, the last ability of Kaya’s Ghostform triggers but will have no effect."
    );
    supported("Cloudshift");
    supported("Banisher Priest");
    // Cloudshift: the trigger does nothing (the card is already a new permanent).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, GHOSTFORM, bears);
    cast_spell(&mut t, P0, "Cloudshift", &[Entity::Object(bears)]);
    t.g.resolve_top();
    t.settle();
    assert_eq!(t.stack_len(), 1, "Kaya's Ghostform triggered");
    let flickered = t.g.current(bears);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears"), vec![flickered]);
    assert!(t.in_graveyard(P0, GHOSTFORM));

    // Banisher Priest: Kaya's Ghostform returns the card; it isn't returned again later.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, GHOSTFORM, bears);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let priest_card = crate::r_s03_common::in_hand_with_mana(&mut t, P1, "Banisher Priest");
    t.g.turn.priority = Some(P1);
    t.cast(P1, priest_card).go();
    t.answer_targets(P1, &[Entity::Object(bears)]);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], bears, "the Bears was exiled and returned");
    assert!(t.in_graveyard(P0, GHOSTFORM));
    assert!(!t.in_exile("Grizzly Bears"));
    let priest = t.named_on_battlefield("Banisher Priest")[0];
    destroy(&mut t, priest);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears"), back);
}

#[test]
fn kayas_ghostform_falls_off_when_another_player_gains_control_of_the_permanent() {
    cr!("303.4e", "704.5m");
    ruling!(
        "Kaya's Ghostform",
        "If another player gains control of the enchanted permanent, Kaya’s Ghostform will be put into your graveyard."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, GHOSTFORM, bears);
    give_control(&mut t, bears, P1);
    assert!(t.in_graveyard(P0, GHOSTFORM));
}

#[test]
fn athreos_returns_a_creature_with_a_coin_counter_that_dies_or_is_exiled() {
    cr!("603.10a", "603.6c", "110.2a");
    // "Whenever another creature with a coin counter on it dies or is put into exile,
    // return that card to the battlefield under your control." (Kaya's Ghostform's
    // trigger, with a non-Aura subject).
    supported("Athreos, Shroud-Veiled");
    supported("Swords to Plowshares");
    for exile in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Athreos, Shroud-Veiled");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let plain = t.battlefield(P1, "Hill Giant");
        t.g.add_counters(Entity::Object(bears), "coin", 1, None);
        for c in [bears, plain] {
            if exile {
                cast_spell(&mut t, P0, "Swords to Plowshares", &[Entity::Object(c)]);
            } else {
                destroy(&mut t, c);
            }
            t.resolve_all();
        }
        let back = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(back.len(), 1, "exile: {exile}");
        assert_eq!(t.g.obj(back[0]).controller, P0);
        assert_eq!(t.g.obj(back[0]).owner, P1);
        // No coin counter: the Hill Giant stays where it went.
        assert!(t.named_on_battlefield("Hill Giant").is_empty(), "exile: {exile}");
        if exile {
            assert!(t.in_exile("Hill Giant"));
        } else {
            assert!(t.in_graveyard(P1, "Hill Giant"));
        }
    }
}
