//! Rulings batch P179 — "toughness matters": life gain, tokens and counters equal to a
//! creature's toughness (as it last existed on the battlefield, or on resolution),
//! "creature with toughness 4 or greater" triggers, and friends.

use crate::r_p179_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::ability::Value;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Gives the permanent +p/+t until end of turn (as a resolving spell would).
fn pump(t: &mut TestGame, id: ObjectId, p: i32, tt: i32) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![Modification::ModifyPT(Value::c(p), Value::c(tt))],
    );
}

/// A Grizzly Bears (2/2) of P0's with +0/+3 until end of turn: 2/5 on the battlefield,
/// 2/2 in the graveyard.
fn pumped_bears(t: &mut TestGame) -> ObjectId {
    let b = t.battlefield(P0, "Grizzly Bears");
    pump(t, b, 0, 3);
    assert_eq!(t.pt(b), (2, 5));
    b
}

#[test]
fn sacrificed_creature_toughness_is_last_known() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Severed Strands",
        "The amount of life gained is equal to the sacrificed creature's toughness as it last existed on the battlefield."
    );
    ruling!(
        "Disciple of Griselbrand",
        "The amount of life you gain is equal to the toughness of the creature as it last existed on the battlefield, not its toughness in the graveyard."
    );
    ruling!(
        "Bushmeat Poacher",
        "Use the sacrificed creature’s toughness as it last existed on the battlefield to determine how much life you gain."
    );
    ruling!(
        "Arguel's Blood Fast // Temple of Aclazotz",
        "For the last ability of Temple of Aclazotz, use the creature's toughness as it last existed on the battlefield to determine how much life you gain."
    );
    ruling!(
        "Scourge of Skola Vale",
        "Use the toughness of the sacrificed creature as it last existed on the battlefield to determine how many +1/+1 counters to put on Scourge of Skola Vale."
    );
    ruling!(
        "Feed the Pack",
        "Check the sacrificed creature’s toughness as it last existed on the battlefield to determine how many Wolves you put onto the battlefield."
    );
    // Severed Strands: an additional cost.
    let mut t = TestGame::new(2);
    let bears = pumped_bears(&mut t);
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_choose(P0, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Severed Strands", &[obj(ogre)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 25);
    // Activated abilities that sacrifice a creature as a cost: (card, ability text).
    for (name, text) in [
        ("Disciple of Griselbrand", "You gain life"),
        ("Bushmeat Poacher", "You gain life"),
        ("Ayli, Eternal Pilgrim", "You gain life"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let bears = pumped_bears(&mut t);
        t.lands(P0, "Wastes", 1);
        t.answer_choose(P0, &[obj(bears)]);
        activate_containing(&mut t, P0, src, text).unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
        assert_eq!(t.life(P0), 25, "{name}");
    }
    // Temple of Aclazotz.
    let mut t = TestGame::new(2);
    let temple = battlefield_back(
        &mut t,
        P0,
        "Arguel's Blood Fast // Temple of Aclazotz",
    );
    assert_eq!(t.obj_now(temple).chars.name.as_str(), "Temple of Aclazotz");
    let bears = pumped_bears(&mut t);
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, temple, "Sacrifice a creature").unwrap();
    // The creature is sacrificed as the ability is activated, before anyone can respond.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    // Scourge of Skola Vale: counters.
    supported("Scourge of Skola Vale");
    let mut t = TestGame::new(2);
    let scourge = t.battlefield(P0, "Scourge of Skola Vale");
    crate::r_s29_common::put_counters(&mut t, scourge, "+1/+1", 2);
    let bears = pumped_bears(&mut t);
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, scourge, "Sacrifice another creature").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(scourge, "+1/+1"), 7);
    // Feed the Pack: Wolves at the beginning of the end step.
    supported("Feed the Pack");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feed the Pack");
    let bears = pumped_bears(&mut t);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(tokens(&t, P0).len(), 5);
}

#[test]
fn temple_of_aclazotz_sacrifice_is_a_cost() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Arguel's Blood Fast // Temple of Aclazotz",
        "Once you've announced the last ability of Temple of Aclazotz, it's too late for anyone to interrupt you by trying to remove the creature you sacrifice."
    );
    let mut t = TestGame::new(2);
    let temple = battlefield_back(
        &mut t,
        P0,
        "Arguel's Blood Fast // Temple of Aclazotz",
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, temple, "Sacrifice a creature").unwrap();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.g.stack.len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn bushmeat_poacher_draws_one_card() {
    cr!("608.2c");
    ruling!(
        "Bushmeat Poacher",
        "You draw only one card, no matter how much life you gain."
    );
    let mut t = TestGame::new(2);
    let poacher = t.battlefield(P0, "Bushmeat Poacher");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(wurm)]);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, poacher, "You gain life").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn feed_the_pack_chooses_on_resolution() {
    cr!("603.5", "608.2c");
    ruling!(
        "Feed the Pack",
        "You choose whether to sacrifice a nontoken creature and which creature to sacrifice when Feed the Pack’s ability resolves."
    );
    let mut t = TestGame::new(2);
    let feed = t.battlefield(P0, "Feed the Pack");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let token = create_token(&mut t, P0, "Spirit");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, feed), 1);
    // A creature that enters while the trigger is on the stack can be the one
    // sacrificed: the choice is made on resolution.
    let wurm = t.enter(P0, "Craw Wurm");
    t.settle();
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(wurm)]);
    t.resolve_all();
    let cands: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    // The token isn't offered (nontoken); the Bears and the Wurm are.
    assert_eq!(cands.len(), 1);
    assert!(cands[0].contains(&obj(bears)) && !cands[0].contains(&obj(token)));
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    assert_eq!(tokens(&t, P0).len(), 5);
}
