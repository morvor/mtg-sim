//! Rulings batch P004 — tokens and Auras exiled by banishing and flickering effects
//! (CR 111.7, 111.8, 303.4f–g), cards that leave exile before the banisher leaves
//! (CR 610.3, 400.7), and a face-down creature that returns face up (CR 708.2a,
//! 708.9).

use crate::r_p004_common::*;
use crate::r_p023_common::wolf_token;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn no_wolf(t: &TestGame, who: &str) {
    assert!(
        !t.g.battlefield
            .iter()
            .any(|&id| t.g.obj(id).chars.name == "Wolf"),
        "{who}: the token came back"
    );
    assert!(!t.in_exile("Wolf"), "{who}: the token is in exile");
}

#[test]
fn an_exiled_token_ceases_to_exist_and_never_returns() {
    cr!("111.7", "111.8", "704.5d", "610.3");
    ruling!(
        "Silkwrap",
        "If a creature token is exiled, it ceases to exist. It won't be returned to the battlefield."
    );
    ruling!(
        "Stasis Snare",
        "If a creature token is exiled, it ceases to exist. It won't be returned to the battlefield."
    );
    ruling!(
        "Ghostway",
        "If a creature token is exiled, it ceases to exist. It won't be returned to the battlefield."
    );
    ruling!(
        "Banishing Light",
        "If a token is exiled this way, it will cease to exist and won’t return to the battlefield."
    );
    ruling!(
        "Flicker of Fate",
        "If a token is exiled this way, it will cease to exist and won’t return to the battlefield."
    );
    ruling!(
        "Turn to Mist",
        "If a token is exiled this way, it will cease to exist and won’t return to the battlefield."
    );
    ruling!(
        "Borrowed Time",
        "If a token is exiled, it ceases to exist. It won't be returned to the battlefield."
    );
    ruling!(
        "Grasp of Fate",
        "If a token is exiled, it ceases to exist. It won't be returned to the battlefield."
    );
    // Banishers.
    for name in ["Silkwrap", "Stasis Snare", "Banishing Light", "Borrowed Time", "Grasp of Fate"] {
        let mut t = TestGame::new(2);
        let wolf = wolf_token(&mut t, P1);
        let b = banish_trigger(&mut t, name, wolf);
        t.resolve_all();
        assert!(!t.on_battlefield(wolf), "{name}: not exiled");
        destroy_now(&mut t, b);
        t.g.flush_events();
        t.settle();
        no_wolf(&t, name);
    }
    // Flicker of Fate: exiled and returned right away; the token doesn't come back.
    let mut t = TestGame::new(2);
    let wolf = wolf_token(&mut t, P0);
    cast_now(&mut t, P0, "Flicker of Fate", &[obj(wolf)]);
    t.resolve_all();
    no_wolf(&t, "Flicker of Fate");
    // Turn to Mist and Ghostway: nothing returns at the next end step.
    for name in ["Turn to Mist", "Ghostway"] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let wolf = wolf_token(&mut t, P0);
        let targets: &[Entity] = if name == "Ghostway" { &[] } else { &[obj(wolf)] };
        cast_now(&mut t, P0, name, targets);
        t.resolve_all();
        no_wolf(&t, name);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        no_wolf(&t, name);
    }
}

/// P1's Holy Strength on P1's Bears is exiled by P0's banisher `name`; P0 controls
/// Gladecover Scout (hexproof) and Argothian Enchantress (shroud). Returns (banisher,
/// bears, scout, enchantress).
fn exile_aura(t: &mut TestGame, name: &str) -> (ObjectId, ObjectId, ObjectId, ObjectId) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hs = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(hs, obj(bears)));
    let scout = t.battlefield(P0, "Gladecover Scout");
    let ench = t.battlefield(P0, "Argothian Enchantress");
    let b = banish_trigger(t, name, hs);
    t.resolve_all();
    assert!(t.in_exile("Holy Strength"), "{name}: not exiled");
    (b, bears, scout, ench)
}

#[test]
fn an_exiled_aura_returns_attached_to_what_its_owner_chooses_without_targeting() {
    cr!("303.4f", "303.4g", "610.3c", "702.11b", "702.18a");
    ruling!(
        "Prayer of Binding",
        "If an Aura is exiled this way, its owner chooses what it will enchant as it returns to the battlefield. An Aura put onto the battlefield this way doesn't target anything (so it could be attached to a permanent with shroud, for example), but the Aura's enchant ability restricts what it can be attached to. If the Aura can't legally be attached to anything, it remains in exile for the rest of the game."
    );
    ruling!(
        "Banishing Light",
        "If an Aura is exiled this way, its owner chooses what it will enchant as it returns to the battlefield. An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to a permanent an opponent controls with hexproof, for example), but the Aura’s enchant ability restricts what it can be attached to. If the Aura can’t legally be attached to anything, it remains in exile for the rest of the game."
    );
    ruling!(
        "Stormplain Detainment",
        "If an Aura is exiled this way, its owner chooses what it will enchant as it returns to the battlefield. An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to a permanent with shroud, for example), but the Aura’s enchant ability restricts what it can be attached to. If the Aura can’t legally be attached to anything, it remains in exile for the rest of the game."
    );
    ruling!(
        "Seam Rip",
        "If an Aura is exiled this way, its owner chooses what it will enchant as it returns to the battlefield. An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to an opponent’s permanent with hexproof, for example) , but the Aura’s enchant ability restricts what it can be attached to. If the Aura can’t legally be attached to anything, it remains in exile for the rest of the game."
    );
    for (name, host) in [
        ("Prayer of Binding", 1),
        ("Banishing Light", 0),
        ("Stormplain Detainment", 1),
        ("Seam Rip", 0),
    ] {
        let mut t = TestGame::new(2);
        let (b, _bears, scout, ench) = exile_aura(&mut t, name);
        let host = [scout, ench][host];
        t.answer_choose(P1, &[obj(host)]);
        destroy_and_flush(&mut t, b);
        t.settle();
        let hs = t.named_on_battlefield("Holy Strength");
        assert_eq!(hs.len(), 1, "{name}: didn't return");
        assert_eq!(t.obj_now(hs[0]).attached_to, Some(obj(host)), "{name}");
        assert_eq!(t.obj_now(hs[0]).controller, P1, "{name}");
        // The owner was the one choosing.
        assert!(
            t.asked().iter().any(|(p, d)| *p == P1
                && matches!(d, mtg_engine::decision::Decision::ChooseEntities { candidates, .. }
                    if candidates.contains(&obj(scout)) && candidates.contains(&obj(ench)))),
            "{name}: the owner wasn't asked"
        );
    }
    // Nothing it could enchant: it stays in exile.
    for name in ["Prayer of Binding", "Banishing Light"] {
        let mut t = TestGame::new(2);
        let (b, bears, scout, ench) = exile_aura(&mut t, name);
        for c in [bears, scout, ench] {
            destroy_now(&mut t, c);
        }
        t.settle();
        destroy_and_flush(&mut t, b);
        t.settle();
        assert!(t.named_on_battlefield("Holy Strength").is_empty(), "{name}");
        assert!(t.in_exile("Holy Strength"), "{name}: it should remain exiled");
    }
}

#[test]
fn a_card_that_left_exile_doesnt_return_when_the_banisher_leaves() {
    cr!("610.3", "400.7");
    ruling!(
        "Bronzebeak Foragers",
        "If an exiled permanent card leaves exile, perhaps because of Bronzebeak Foragers's last ability, it won't return to the battlefield after Bronzebeak Foragers leaves the battlefield."
    );
    ruling!(
        "Gelatinous Cube",
        "If the engulfed creature card leaves exile, perhaps because of the Dissolve ability, it won't return to the battlefield after Gelatinous Cube leaves the battlefield."
    );
    // Bronzebeak Foragers: {X}{W}: put the exiled card into its owner's graveyard.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let f = t.enter(P0, "Bronzebeak Foragers");
    t.resolve_all();
    let exiled = t.g.exile.iter().copied().find(|&id| t.g.obj(id).chars.name == "Grizzly Bears");
    let exiled = exiled.expect("exiled");
    t.set_step(P0, Step::PrecombatMain);
    mana(&mut t, P0);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, f, 0, &[obj(exiled)]).expect("activate");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 22);
    destroy_and_flush(&mut t, f);
    t.settle();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Gelatinous Cube: Dissolve {X}{B}.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cube = banish_trigger(&mut t, "Gelatinous Cube", bears);
    t.resolve_all();
    let exiled = t.g.exile.iter().copied().find(|&id| t.g.obj(id).chars.name == "Grizzly Bears");
    let exiled = exiled.expect("exiled");
    t.set_step(P0, Step::PrecombatMain);
    mana(&mut t, P0);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, cube, 0, &[obj(exiled)]).expect("activate");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    destroy_and_flush(&mut t, cube);
    t.settle();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn silkwrap_can_exile_a_face_down_creature_which_returns_face_up() {
    cr!("708.2a", "708.9", "202.3a", "610.3");
    ruling!(
        "Silkwrap",
        "Face-down creatures have a mana value of 0. If one is exiled by Silkwrap, it will return to the battlefield face up."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Exalted Angel");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, angel));
    t.g.recompute();
    let angel = t.g.current(angel);
    assert_eq!(t.obj_now(angel).chars.mana_value(), 0);
    let s = banish_trigger(&mut t, "Silkwrap", angel);
    t.resolve_all();
    assert!(t.in_exile("Exalted Angel"));
    destroy_and_flush(&mut t, s);
    t.settle();
    let back = t.named_on_battlefield("Exalted Angel");
    assert_eq!(back.len(), 1, "didn't return face up");
    assert!(!t.obj_now(back[0]).face_down);
    assert_eq!(t.pt(back[0]), (4, 5));
}
