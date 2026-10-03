//! CR 712: meld cards and melded permanents (CR 712.4, 712.5, 712.8b, 712.8g, 712.14c,
//! 712.21).

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDb, Layout};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::merge;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Melds Graf Rats and Midnight Scavengers (owned by `p`) into Chittering Host.
fn chittering_host(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let rats = t.exile(p, "Graf Rats");
    let scav = t.exile(p, "Midnight Scavengers");
    let host = merge::meld(&mut t.g, rats, scav, card("Chittering Host"), p).expect("meld");
    t.g.recompute();
    host
}

fn cards_named(t: &TestGame, zone: Zone, name: &str) -> usize {
    t.g.find_in_zone(zone, name).len()
}

#[test]
fn a_meld_cards_back_face_is_half_of_the_melded_permanent() {
    cr!("712.4", "712.4b");
    ruling!(
        "Fang, Fearless l'Cie",
        "While a meld card is in any zone other than the battlefield, it has only the characteristics of its front face"
    );
    // A meld card has one Magic card face; its back is half of the combined back face.
    let rats = card("Graf Rats");
    assert_eq!(rats.layout, Layout::Meld);
    assert_eq!(rats.faces.len(), 1);
    assert!(rats
        .related
        .iter()
        .any(|(k, n)| k == "meld_result" && n == "Chittering Host"));
    // Alone, on the battlefield or elsewhere, it has only its front face.
    let mut t = TestGame::new(2);
    let hand = t.hand(P0, "Graf Rats");
    let perm = t.battlefield(P0, "Graf Rats");
    t.g.recompute();
    for id in [hand, perm] {
        assert_eq!(t.obj(id).chars.name, "Graf Rats");
        assert_eq!(t.obj(id).face, FaceState::Front);
    }
    assert_eq!(t.pt(perm), (2, 1));
    // The combined back face is used only by the melded permanent.
    let host = chittering_host(&mut t, P0);
    assert_eq!(t.obj(host).chars.name, "Chittering Host");
    assert_eq!(t.pt(host), (5, 6));
}

#[test]
fn the_seven_meld_pairs() {
    cr!("712.5", "712.5a", "712.5b", "712.5c", "712.5d", "712.5e", "712.5f", "712.5g");
    let pairs = [
        ("Midnight Scavengers", "Graf Rats", "Chittering Host"),
        (
            "Hanweir Garrison",
            "Hanweir Battlements",
            "Hanweir, the Writhing Township",
        ),
        (
            "Bruna, the Fading Light",
            "Gisela, the Broken Blade",
            "Brisela, Voice of Nightmares",
        ),
        (
            "Phyrexian Dragon Engine",
            "Mishra, Claimed by Gix",
            "Mishra, Lost to Phyrexia",
        ),
        (
            "The Mightstone and Weakstone",
            "Urza, Lord Protector",
            "Urza, Planeswalker",
        ),
        (
            "Argoth, Sanctum of Nature",
            "Titania, Voice of Gaea",
            "Titania, Gaea Incarnate",
        ),
        (
            "Fang, Fearless l'Cie",
            "Vanille, Cheerful l'Cie",
            "Ragnarok, Divine Deliverance",
        ),
    ];
    for (a, b, result) in pairs {
        let mut t = TestGame::new(2);
        let ca = t.exile(P0, a);
        let cb = t.exile(P0, b);
        let r = CardDb::global().get(result).expect(result);
        let melded = merge::meld(&mut t.g, ca, cb, r.clone(), P0)
            .unwrap_or_else(|| panic!("{a} and {b} meld"));
        t.g.recompute();
        assert_eq!(t.obj(melded).chars.name, result);
        assert_eq!(merge::physical_components(&t.g, melded).len(), 2);
        // A card of another pair isn't its counterpart.
        let mut t = TestGame::new(2);
        let ca = t.exile(P0, a);
        let other = t.exile(
            P0,
            if result == "Chittering Host" {
                "Gisela, the Broken Blade"
            } else {
                "Graf Rats"
            },
        );
        assert!(merge::meld(&mut t.g, ca, other, r, P0).is_none());
    }
}

#[test]
fn a_meld_card_on_the_stack_has_its_front_face() {
    cr!("712.8b");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 2);
    let rats = t.hand(P0, "Graf Rats");
    let spell = t.cast(P0, rats).go();
    let c = &t.obj(spell).chars;
    assert_eq!(c.name, "Graf Rats");
    assert_eq!((c.power, c.toughness), (Some(2), Some(1)));
    assert_eq!(mv(&mut t, spell), 2);
}

#[test]
fn a_melded_permanent_has_the_combined_back_face_and_its_front_faces_mana_value() {
    cr!("712.8", "712.8g", "712.14c");
    ruling!(
        "Mishra, Claimed by Gix",
        "While a melded permanent is on the battlefield, it has only the characteristics of its combined back face"
    );
    ruling!(
        "Titania, Voice of Gaea",
        "The mana value of a melded permanent is the sum of the mana values of its front faces"
    );
    let mut t = TestGame::new(2);
    let host = chittering_host(&mut t, P0);
    // One permanent with its back faces up, represented by the two cards.
    assert_eq!(t.g.battlefield, vec![host]);
    assert_eq!(t.obj(host).face, FaceState::Melded);
    let c = t.obj(host).chars.clone();
    assert_eq!(c.name, "Chittering Host");
    assert!(c.has_subtype("Eldrazi") && c.has_subtype("Horror"));
    assert!(!c.has_subtype("Rat"));
    assert_eq!(c.colors, ColorSet::NONE);
    assert!(c.mana_cost.is_none());
    // {1}{B} + {4}{B}.
    assert_eq!(mv(&mut t, host), 7);
    // A copy of it has mana value 0.
    let bears = t.battlefield(P1, "Grizzly Bears");
    run_effect(
        &mut t,
        P1,
        None,
        &[Entity::Object(bears)],
        Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::All(Filter::Objects(vec![host])),
            duration: Duration::Permanent,
        },
    );
    assert_eq!(t.obj(bears).chars.name, "Chittering Host");
    assert_eq!(mv(&mut t, bears), 0);
}

#[test]
fn a_melded_permanent_leaving_is_one_permanent_and_two_cards() {
    cr!("712.21", "712.21e");
    let mut t = TestGame::new(2);
    // Blood Artist: "Whenever Blood Artist or another creature dies, target player loses 1
    // life and you gain 1 life." Planar Void: "Whenever another card is put into a
    // graveyard from anywhere, exile that card."
    t.battlefield(P1, "Blood Artist");
    t.battlefield(P1, "Planar Void");
    let host = chittering_host(&mut t, P0);
    t.g.move_object(host, Zone::Graveyard(P0), MoveCause::Destroy, None);
    // One creature died; two cards were put into a graveyard.
    assert_eq!(t.graveyard_size(P0), 2);
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert_eq!(t.g.eval_value(&Value::CreaturesDiedThisTurn, &ctx), 1);
    t.settle();
    let artist =
        t.g.stack
            .iter()
            .filter(|s| t.g.describe(**s).contains("Blood Artist"))
            .count();
    let void =
        t.g.stack
            .iter()
            .filter(|s| t.g.describe(**s).contains("Planar Void"))
            .count();
    assert_eq!((artist, void), (1, 2));
}

fn order_answer(t: &mut TestGame, p: PlayerId, v: Vec<usize>) {
    t.answer(p, DecisionKind::Order, Answer::Indices(v));
}

#[test]
fn the_owner_arranges_the_two_cards_in_a_graveyard_or_library() {
    cr!("712.21a");
    ruling!(
        "Mishra, Claimed by Gix",
        "If the cards are put on the top or bottom of your library, you choose their relative order"
    );
    for reversed in [false, true] {
        let mut t = TestGame::new(2);
        let host = chittering_host(&mut t, P0);
        // Components: Graf Rats then Midnight Scavengers. The order asked is bottom to top.
        order_answer(&mut t, P0, if reversed { vec![1, 0] } else { vec![0, 1] });
        t.g.move_object(host, Zone::Graveyard(P0), MoveCause::Destroy, None);
        let gy: Vec<String> =
            t.g.player(P0)
                .graveyard
                .iter()
                .map(|id| t.obj(*id).chars.name.to_string())
                .collect();
        let expected = if reversed {
            vec!["Midnight Scavengers", "Graf Rats"]
        } else {
            vec!["Graf Rats", "Midnight Scavengers"]
        };
        assert_eq!(gy, expected);
        // The owner was asked.
        assert!(t
            .asked()
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
    }
    // On top of the library: the chosen top card is drawn first.
    let mut t = TestGame::new(2);
    let host = chittering_host(&mut t, P0);
    order_answer(&mut t, P0, vec![1, 0]);
    run_effect(
        &mut t,
        P1,
        None,
        &[Entity::Object(host)],
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::library_top(),
        },
    );
    let lib = &t.g.player(P0).library;
    let names: Vec<&str> = lib[lib.len() - 2..]
        .iter()
        .map(|id| t.obj(*id).chars.name.as_str())
        .collect();
    // Bottom to top: Midnight Scavengers, then Graf Rats on top.
    assert_eq!(names, vec!["Midnight Scavengers", "Graf Rats"]);
    let top = t.g.library_top(P0).unwrap();
    assert_eq!(t.obj(top).chars.name, "Graf Rats");
}

#[test]
fn the_player_exiling_it_determines_the_cards_timestamp_order() {
    cr!("712.21b");
    for (order, later) in [
        (vec![0, 1], "Midnight Scavengers"),
        (vec![1, 0], "Graf Rats"),
    ] {
        let mut t = TestGame::new(2);
        let host = chittering_host(&mut t, P0);
        // P1 exiles it and chooses the order.
        order_answer(&mut t, P1, order);
        run_effect(
            &mut t,
            P1,
            None,
            &[Entity::Object(host)],
            Effect::Exile {
                what: Sel::Target(0),
                face_down: false,
                link: false,
            },
        );
        let mut exiled: Vec<ObjectId> = t.g.exile.clone();
        assert_eq!(exiled.len(), 2);
        exiled.sort_by_key(|id| t.obj(*id).timestamp);
        assert_eq!(t.obj(exiled[1]).chars.name, later);
        assert!(t
            .asked()
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::Order { .. })));
    }
}

#[test]
fn an_effect_that_finds_the_new_object_finds_both_cards() {
    cr!("712.21c");
    ruling!(
        "Fang, Fearless l'Cie",
        "If an effect moves a melded permanent to a new zone and then affects \"that card,\" it affects both cards"
    );
    supported("Cloudshift");
    // Cloudshift: "Exile target creature you control, then return that card to the
    // battlefield under your control." Both cards return.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let host = chittering_host(&mut t, P0);
    t.lands(P0, "Plains", 1);
    let cs = t.hand(P0, "Cloudshift");
    t.cast(P0, cs).target(host).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Chittering Host").is_empty());
    assert_eq!(t.named_on_battlefield("Graf Rats").len(), 1);
    assert_eq!(t.named_on_battlefield("Midnight Scavengers").len(), 1);
    assert!(t.g.exile.is_empty());
    // False Demise: "When enchanted creature dies, return that card to the battlefield
    // under your control." Both cards return.
    supported("False Demise");
    let mut t = TestGame::new(2);
    let host = chittering_host(&mut t, P0);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 3);
    let fd = t.hand(P0, "False Demise");
    t.cast(P0, fd).target(host).go();
    t.resolve_all();
    t.g.move_object(host, Zone::Graveyard(P0), MoveCause::Destroy, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Graf Rats").len(), 1);
    assert_eq!(t.named_on_battlefield("Midnight Scavengers").len(), 1);
}

#[test]
fn a_replacement_effect_applied_to_one_card_applies_to_both() {
    cr!("712.21d");
    // Leyline of the Void: "If a card would be put into an opponent's graveyard from
    // anywhere, exile it instead."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of the Void");
    let host = chittering_host(&mut t, P0);
    t.g.move_object(host, Zone::Graveyard(P0), MoveCause::Destroy, None);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(cards_named(&t, Zone::Exile, "Graf Rats"), 1);
    assert_eq!(cards_named(&t, Zone::Exile, "Midnight Scavengers"), 1);
    // With two replacement effects that could apply (Leyline of the Void, and "If a card
    // would be put into a graveyard from anywhere, put it on the bottom of its owner's
    // library instead"), the owner chooses one once and it applies to both cards.
    let mut outcomes = Vec::new();
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Leyline of the Void");
        let wheel = CB::new("Wheel Lite")
            .enchantment()
            .ability(stat(StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::ZoneChange {
                    filter: Filter::Any,
                    from: None,
                    to: Some(ZoneKind::Graveyard),
                },
                action: ReplacementAction::MoveInstead(Destination::library_bottom()),
                self_replacement: false,
                optional: false,
            })))
            .build();
        t.custom(P0, wheel, Zone::Battlefield);
        let host = chittering_host(&mut t, P0);
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        let lib_before = t.library_size(P0);
        t.g.move_object(host, Zone::Graveyard(P0), MoveCause::Destroy, None);
        let replacement_choices = t
            .asked()
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseReplacement { .. }))
            .count();
        assert_eq!(replacement_choices, 1, "one choice for both cards");
        let exiled = cards_named(&t, Zone::Exile, "Graf Rats")
            + cards_named(&t, Zone::Exile, "Midnight Scavengers");
        let bottom = t.library_size(P0) - lib_before;
        assert_eq!(t.graveyard_size(P0), 0);
        outcomes.push((exiled, bottom));
    }
    // Each choice sent both cards to the same zone, and the two choices differ.
    outcomes.sort();
    assert_eq!(outcomes, vec![(0, 2), (2, 0)]);
}

#[test]
fn its_last_known_information_is_the_melded_permanents_not_its_cards() {
    cr!("712.21", "712.21c");
    // "Whenever a creature you control dies, you gain life equal to its power." "Its
    // power" is the melded permanent's last known power (Chittering Host, 5), not that of
    // the two cards it became (Graf Rats 2 and Midnight Scavengers 3).
    let mut t = TestGame::new(2);
    let watcher = oracle_card(
        "Mourning Bell",
        "Enchantment",
        "{0}",
        None,
        "Whenever a creature you control dies, you gain life equal to its power.",
    );
    assert!(
        watcher.is_fully_supported(),
        "{:?}",
        watcher.unsupported_text()
    );
    t.custom(P0, watcher, Zone::Battlefield);
    let host = chittering_host(&mut t, P0);
    assert_eq!(t.pt(host), (5, 6));
    t.g.move_object(host, Zone::Graveyard(P0), MoveCause::Destroy, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}
