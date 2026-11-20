//! The question asked for an optional replacement effect names the card the event would move
//! when the deciding player may see it (as Library of Leng's ruling has it: "You can look at
//! a randomly discarded card before deciding where it goes."). A face-down permanent has no
//! name (CR 708.2a): its controller may look at it (CR 708.5), but the question names
//! nothing.

use crate::r614_audit_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::facedown;
use mtg_engine::game::Variant;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// A Commander game: P0's commander `name` (turned face down first if `face_down`) is on the
/// battlefield, and P0 casts Unsummon ("Return target creature to its owner's hand.") on it,
/// then puts it into the command zone instead (CR 903.9b: "If a commander would be put into
/// its owner's hand or library from anywhere, its owner may put it into the command zone
/// instead."). Returns the game, the questions P0 was asked about that replacement effect,
/// and the commander.
fn unsummon_commander(name: &str, face_down: bool) -> (TestGame, Vec<String>, ObjectId) {
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            starting_life: 40,
            ..Default::default()
        },
    );
    let cmdr = t.battlefield(P0, name);
    t.g.objects[cmdr.0 as usize].is_commander = true;
    if face_down {
        assert!(facedown::turn_face_down(&mut t.g, cmdr));
        t.g.recompute();
        assert!(t.obj_now(cmdr).chars.name.is_empty());
    }
    t.answer_yes(P0, true);
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Unsummon", &[obj(cmdr)]);
    let prompts: Vec<String> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::YesNo { prompt, .. }
                if *p == P0 && prompt.starts_with("Apply: Commander") =>
            {
                Some(prompt.clone())
            }
            _ => None,
        })
        .collect();
    (t, prompts, cmdr)
}

#[test]
fn the_question_names_a_face_up_commander_put_into_its_owners_hand() {
    cr!("903.9b");
    supported("Grumgully, the Generous");
    let (t, prompts, cmdr) = unsummon_commander("Grumgully, the Generous", false);
    assert_eq!(
        prompts,
        vec!["Apply: Commander: put into command zone instead? (Grumgully, the Generous)"]
    );
    assert_eq!(t.zone(cmdr), Zone::Command);
}

#[test]
fn the_question_about_a_face_down_commander_names_nothing() {
    cr!("903.9b", "708.2a", "708.5");
    // A morph commander turned face down: a 2/2 creature with no name.
    supported("Akroma, Angel of Fury");
    let (t, prompts, cmdr) = unsummon_commander("Akroma, Angel of Fury", true);
    assert_eq!(
        prompts,
        vec!["Apply: Commander: put into command zone instead?"]
    );
    assert_eq!(t.zone(cmdr), Zone::Command);
}
