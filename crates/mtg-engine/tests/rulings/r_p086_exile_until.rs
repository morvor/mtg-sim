//! Rulings batch P086 — exile "until this leaves the battlefield" (CR 610.3) and similar:
//! Haytham Kenway, Consulate Crackdown, Circle of Confinement, Faceless Devourer, and
//! Burn Away's delayed trigger.

use crate::r_p086_common::*;
use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's `name` enters, its enters ability targeting `targets`; the ability is left on the
/// stack.
fn enters_targeting(t: &mut TestGame, name: &str, targets: &[Entity]) -> ObjectId {
    if !targets.is_empty() {
        t.answer_targets(P0, targets);
    }
    let id = t.enter(P0, name);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{name}");
    id
}

/// P1's Grizzly Bears with P0's Pacifism, P1's Bonesplitter and a +1/+1 counter.
fn decorated_bears(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = attach_new(t, P0, "Pacifism", obj(bears));
    let eq = attach_new(t, P1, "Bonesplitter", obj(bears));
    put_counters(t, bears, mtg_engine::types::counters::PLUS1, 1);
    (bears, aura, eq)
}

#[test]
fn exiled_until_leaves_auras_equipment_counters_and_a_new_object() {
    cr!("400.7", "610.3", "704.5m", "704.5n");
    ruling!(
        "Haytham Kenway",
        "Auras attached to creatures exiled with Haytham Kenway’s last ability will be put into their owners’ graveyards. Any Equipment attached to them will become unattached and remain on the battlefield. Any counters on the exiled creatures cease to exist. When those creatures return to the battlefield, they will be new objects with no connection to the cards that were exiled."
    );
    ruling!(
        "Circle of Confinement",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    for name in ["Haytham Kenway", "Circle of Confinement"] {
        supported(name);
        let mut t = TestGame::new(2);
        let (bears, _aura, eq) = decorated_bears(&mut t);
        let src = enters_targeting(&mut t, name, &[obj(bears)]);
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"), "{name}");
        assert!(t.in_graveyard(P0, "Pacifism"), "{name}");
        assert!(
            t.on_battlefield(eq) && attached_to(&t, eq).is_none(),
            "{name}"
        );
        let exiled = t.g.current(bears);
        destroy(&mut t, src);
        t.resolve_all();
        let back = t.g.current(exiled);
        assert_ne!(back, exiled, "{name}");
        assert!(t.on_battlefield(back), "{name}");
        assert_eq!(controller(&t, back), P1, "{name}");
        assert_eq!(t.pt(back), (2, 2), "{name}");
        assert!(attached_to(&t, eq).is_none(), "{name}");
    }
}

#[test]
fn consulate_crackdown_exiled_artifacts_lose_auras_and_counters() {
    cr!("400.7", "610.3", "704.5m");
    ruling!(
        "Consulate Crackdown",
        "Auras attached to the exiled artifacts will be put into their owners’ graveyards. Any counters on the exiled artifacts will cease to exist."
    );
    ruling!(
        "Consulate Crackdown",
        "If an artifact token is exiled, it ceases to exist. It won’t be returned to the battlefield."
    );
    supported("Consulate Crackdown");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    attach_new(&mut t, P0, "Pacifism", obj(thopter));
    put_counters(&mut t, thopter, mtg_engine::types::counters::PLUS1, 2);
    let treasure = create_token(&mut t, P1, "Treasure");
    let cd = enters_targeting(&mut t, "Consulate Crackdown", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Pacifism"));
    assert!(t.in_exile("Ornithopter"));
    assert!(!t.g.is_live(treasure) || t.zone(treasure) != mtg_engine::object::Zone::Battlefield);
    destroy(&mut t, cd);
    t.resolve_all();
    let back = t.named_on_battlefield("Ornithopter");
    assert_eq!(back.len(), 1);
    assert_eq!(t.pt(back[0]), (0, 2));
    assert!(t.named_on_battlefield("Treasure").is_empty());
}

#[test]
fn haytham_kenway_exiled_token_ceases_to_exist() {
    cr!("111.7", "704.5d", "610.3");
    ruling!(
        "Haytham Kenway",
        "If a token is exiled with Haytham Kenway’s last ability, it will cease to exist and won’t return to the battlefield."
    );
    let mut t = TestGame::new(2);
    let soldier = create_token(&mut t, P1, "Soldier");
    let h = enters_targeting(&mut t, "Haytham Kenway", &[obj(soldier)]);
    t.resolve_all();
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());
    destroy(&mut t, h);
    t.resolve_all();
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());
}

#[test]
fn exile_until_leaves_does_nothing_if_the_source_left_first() {
    cr!("610.3b");
    ruling!(
        "Haytham Kenway",
        "If Haytham Kenway leaves the battlefield before its last ability resolves, the target creatures won’t be exiled."
    );
    ruling!(
        "Circle of Confinement",
        "If Circle of Confinement leaves the battlefield before its enters-the-battlefield ability resolves, the target creature won't be exiled."
    );
    ruling!(
        "Consulate Crackdown",
        "If Consulate Crackdown leaves the battlefield before its triggered ability resolves, no artifacts will be exiled."
    );
    for name in [
        "Haytham Kenway",
        "Circle of Confinement",
        "Consulate Crackdown",
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let thopter = t.battlefield(P1, "Ornithopter");
        let targets = if name == "Consulate Crackdown" {
            vec![]
        } else {
            vec![obj(bears)]
        };
        let src = enters_targeting(&mut t, name, &targets);
        destroy(&mut t, src);
        t.resolve_all();
        assert!(t.on_battlefield(bears), "{name}");
        assert!(t.on_battlefield(thopter), "{name}");
    }
}

#[test]
fn consulate_crackdown_owner_leaving_returns_the_artifacts() {
    cr!("800.4a", "610.3");
    ruling!(
        "Consulate Crackdown",
        "In a multiplayer game, if Consulate Crackdown’s owner leaves the game, the exiled cards will return to the battlefield. Because the one-shot effect that returns the cards isn’t an ability that goes on the stack, it won’t cease to exist along with the leaving player’s spells and abilities on the stack."
    );
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P2, "Thran Dynamo");
    enters_targeting(&mut t, "Consulate Crackdown", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    t.take_action(P0, Action::Concede);
    t.settle();
    assert!(t.has_lost(P0));
    let back_a = t.named_on_battlefield("Ornithopter");
    let back_b = t.named_on_battlefield("Thran Dynamo");
    assert_eq!(back_a.len(), 1);
    assert_eq!(back_b.len(), 1);
    assert_eq!(controller(&t, back_a[0]), P1);
    assert_eq!(controller(&t, back_b[0]), P2);
}

#[test]
fn faceless_devourer_leaving_first_exiles_the_creature_indefinitely() {
    cr!("607.2a", "603.6c");
    ruling!(
        "Faceless Devourer",
        "If Faceless Devourer leaves the battlefield before its first triggered ability has resolved, its last ability triggers. This ability does nothing when it resolves. Then its first ability resolves and exiles the target creature indefinitely."
    );
    supported("Faceless Devourer");
    let mut t = TestGame::new(2);
    let slayer = t.battlefield(P1, "Dauthi Slayer");
    let dev = enters_targeting(&mut t, "Faceless Devourer", &[obj(slayer)]);
    destroy(&mut t, dev);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(t.on_battlefield(slayer));
    t.resolve_all();
    assert!(t.in_exile("Dauthi Slayer"));
    // Nothing brings it back later.
    next_main(&mut t, P1);
    assert!(t.in_exile("Dauthi Slayer"));
}

#[test]
fn burn_away_delayed_trigger_ends_with_the_turn() {
    cr!("603.7b");
    ruling!(
        "Burn Away",
        "If the creature doesn’t die that turn, the delayed triggered ability will simply cease to exist. It won’t trigger on a future turn."
    );
    supported("Burn Away");
    let mut t = TestGame::new(2);
    let avacyn = t.battlefield(P1, "Avacyn, Angel of Hope");
    t.graveyard(P1, "Shock");
    cast_new(&mut t, P0, "Burn Away", &[obj(avacyn)]);
    t.resolve_all();
    assert!(t.on_battlefield(avacyn));
    next_main(&mut t, P1);
    // It dies on a later turn: nothing is exiled.
    let id = t.g.current(avacyn);
    t.g.sacrifice(id, P1);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Avacyn, Angel of Hope"));
    assert!(t.in_graveyard(P1, "Shock"));
}
