//! CR 613.7h–613.7j: timestamps of the casual-variant cards that function in the command
//! zone (planes, phenomena, schemes, vanguards and conspiracies).

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::game::GameConfig;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::variants;
use mtg_engine::*;

/// "Creatures are [color]." — a layer 5 color-setting effect, so that the latest
/// timestamp wins.
fn creatures_are(name: &str, t: CardType, c: Color) -> CardDef {
    permanent(
        name,
        &[t],
        vec![continuous(
            Filter::Type(CardType::Creature),
            vec![Modification::SetColors(colors(&[c]))],
        )],
    )
}

#[test]
fn plane_phenomenon_and_scheme_get_a_timestamp_when_turned_face_up() {
    // CR 613.7h: a face-up plane, phenomenon or scheme card receives a timestamp at the
    // time it's turned face up — so its effects apply after those of permanents that were
    // already on the battlefield, even though the card existed before them.
    cr!("613.7h");
    for kind in [CardType::Plane, CardType::Phenomenon, CardType::Scheme] {
        let mut t = TestGame::new(2);
        let card = t.custom(
            P0,
            creatures_are("Blue Card", kind, Color::Blue),
            Zone::Command,
        );
        t.g.objects[card.0 as usize].face_down = true;
        t.recompute();
        let bear = t.custom(
            P0,
            creature("Bear", 2, 2, &[Color::Green]),
            Zone::Battlefield,
        );
        t.custom(
            P1,
            creatures_are("Red Enchantment", CardType::Enchantment, Color::Red),
            Zone::Battlefield,
        );
        t.recompute();
        assert_eq!(color_of(&t, bear), colors(&[Color::Red]), "{kind:?}");
        assert!(variants::turn_face_up_in_command(&mut t.g, card));
        t.recompute();
        assert_eq!(color_of(&t, bear), colors(&[Color::Blue]), "{kind:?}");
        // A permanent that enters afterwards has a later timestamp still.
        t.custom(
            P1,
            creatures_are("White Enchantment", CardType::Enchantment, Color::White),
            Zone::Battlefield,
        );
        t.recompute();
        assert_eq!(color_of(&t, bear), colors(&[Color::White]), "{kind:?}");
        // Already face up: nothing happens.
        assert!(!variants::turn_face_up_in_command(&mut t.g, card));
    }
}

fn started_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            starting_player: Some(P0),
            skip_mulligans: true,
            ..Default::default()
        },
    )
}

#[test]
fn vanguard_gets_a_timestamp_at_the_beginning_of_the_game() {
    // CR 613.7i: a face-up vanguard card receives a timestamp at the beginning of the
    // game: later than every object that existed before the game began, and earlier than
    // anything that happens during the game.
    cr!("613.7i");
    let mut t = started_game();
    let vanguard = t.custom(
        P0,
        creatures_are("Red Vanguard", CardType::Vanguard, Color::Red),
        Zone::Command,
    );
    let library_card = t.library_top(P0, "Grizzly Bears");
    let before = t.obj_now(library_card).timestamp;
    assert!(t.obj_now(vanguard).timestamp < before);
    t.g.start();
    let ts = t.obj_now(vanguard).timestamp;
    assert!(ts > before);
    t.set_step(P0, Step::PrecombatMain);
    let bear = t.custom(
        P0,
        creature("Bear", 2, 2, &[Color::Green]),
        Zone::Battlefield,
    );
    t.recompute();
    assert_eq!(color_of(&t, bear), colors(&[Color::Red]));
    let ench = t.custom(
        P1,
        creatures_are("Blue Enchantment", CardType::Enchantment, Color::Blue),
        Zone::Battlefield,
    );
    t.recompute();
    assert!(t.obj_now(ench).timestamp > ts);
    assert_eq!(color_of(&t, bear), colors(&[Color::Blue]));
}

#[test]
fn conspiracy_timestamp_at_game_start_and_again_when_turned_face_up() {
    // CR 613.7j: a conspiracy card receives a timestamp at the beginning of the game; a
    // face-down one (hidden agenda) receives a new timestamp when it's turned face up.
    cr!("613.7j");
    let mut t = started_game();
    let face_up = t.custom(
        P0,
        creatures_are("Red Conspiracy", CardType::Conspiracy, Color::Red),
        Zone::Command,
    );
    let face_down = t.custom(
        P1,
        creatures_are("White Conspiracy", CardType::Conspiracy, Color::White),
        Zone::Command,
    );
    t.g.objects[face_down.0 as usize].face_down = true;
    let library_card = t.library_top(P1, "Grizzly Bears");
    let before = t.obj_now(library_card).timestamp;
    t.g.start();
    assert!(t.obj_now(face_up).timestamp > before);
    assert!(t.obj_now(face_down).timestamp > before);
    t.set_step(P0, Step::PrecombatMain);
    let bear = t.custom(
        P0,
        creature("Bear", 2, 2, &[Color::Green]),
        Zone::Battlefield,
    );
    t.recompute();
    // Only the face-up conspiracy functions.
    assert_eq!(color_of(&t, bear), colors(&[Color::Red]));
    t.custom(
        P0,
        creatures_are("Blue Enchantment", CardType::Enchantment, Color::Blue),
        Zone::Battlefield,
    );
    t.recompute();
    assert_eq!(color_of(&t, bear), colors(&[Color::Blue]));
    // Revealing the hidden agenda gives it a new timestamp, later than the enchantment's.
    assert!(variants::turn_face_up_in_command(&mut t.g, face_down));
    t.recompute();
    assert_eq!(color_of(&t, bear), colors(&[Color::White]));
}
