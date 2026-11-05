//! Rulings batch P047 — Duplicity's face-down cards and lose-control trigger (CR 406.3,
//! 603.2), Umaro's random mode without a legal target (CR 700.2b, 603.3c), and The Book of
//! Vile Darkness's Vecna looking at the exiled cards (CR 400.7, 608.2h).

use crate::r_p047_common::*;
use crate::r_s01_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn duplicity_lose_control_puts_the_cards_into_the_graveyard() {
    cr!("603.2", "613.1b");
    ruling!(
        "Duplicity",
        "Taking control of another player’s Duplicity will trigger the ability that causes the cards they exiled to be put into their owner’s graveyard."
    );
    ruling!(
        "Duplicity",
        "It has been restored to triggering when you lose control of Duplicity."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let d = t.enter(P1, "Duplicity");
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 5);
    let gy = t.graveyard_size(P1);
    t.set_step(P0, Step::PrecombatMain);
    spell(&mut t, P0, "Confiscate", &[obj(d)]);
    assert_eq!(t.g.obj(d).controller, P0);
    assert_eq!(t.graveyard_size(P1), gy + 5);
    assert!(t.g.exile.is_empty());
}

#[test]
fn duplicity_cards_are_face_down_and_unseen() {
    cr!("406.3", "708.2");
    ruling!(
        "Duplicity",
        "The cards are face down all the time. You can’t look at them. This applies even to cards from your hand that got swapped out. You have to remember what they were if you care."
    );
    let mut t = TestGame::new(2);
    t.enter(P0, "Duplicity");
    t.resolve_all();
    let first: Vec<ObjectId> = t.g.exile.clone();
    // At the next upkeep, the hand is exiled face down and the five come to hand.
    t.advance_to(P1, Step::Upkeep);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    let now: Vec<ObjectId> = t.g.exile.clone();
    assert_eq!(now.len(), 1);
    assert!(first.iter().all(|c| !t.g.is_live(*c)));
    for c in now {
        assert!(t.g.obj(c).face_down);
        for p in [P0, P1] {
            assert!(!mtg_engine::facedown::can_look_at(&t.g, p, c));
        }
    }
}

#[test]
fn umaro_last_mode_without_a_legal_target_is_never_chosen() {
    cr!("700.2", "700.2b", "603.3c");
    ruling!(
        "Umaro, Raging Yeti",
        "In the incredibly rare case where there is no legal target for the last mode, that mode can't be chosen. The first two modes are always eligible to be chosen."
    );
    supported("Umaro, Raging Yeti");
    // Both players have shroud (Ivory Mask), Umaro has shroud (Lightning Greaves) and so
    // does the only other creature: nothing can be targeted. Over several seeds, the trigger always picks one of the first two modes.
    let mut picked = [false; 2];
    for seed in 0..12u64 {
        let mut t = TestGame::with_config(
            2,
            mtg_engine::game::GameConfig {
                seed,
                ..Default::default()
            },
        );
        let u = t.battlefield(P0, "Umaro, Raging Yeti");
        // Argothian Enchantress (0/1, shroud) shows the first mode.
        let bears = t.battlefield(P0, "Argothian Enchantress");
        let greaves = t.battlefield(P0, "Lightning Greaves");
        t.g.attach(greaves, obj(u));
        t.battlefield(P0, "Ivory Mask");
        t.battlefield(P1, "Ivory Mask");
        t.hand(P0, "Hill Giant");
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        let pumped = t.pt(bears) == (3, 1);
        let wheeled = t.hand_size(P0) == 4;
        assert!(pumped ^ wheeled, "seed {seed}");
        picked[usize::from(wheeled)] = true;
        assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    }
    assert!(picked[0] && picked[1], "both eligible modes are chosen sometimes");
}

#[test]
fn book_of_vile_darkness_vecna_gets_the_exiled_cards_abilities() {
    cr!("400.7", "707.2", "608.2h");
    ruling!(
        "The Book of Vile Darkness",
        "In some strange cases, you might make something into a copy of the Eye and/or Hand of Vecna and then exile that copy to activate The Book of Vile Darkness's ability. The ability that creates Vecna will look at the cards in exile, not the permanents on the battlefield, so it's possible to have Vecna gain different triggered abilities."
    );
    supported("The Book of Vile Darkness");
    supported("Eye of Vecna");
    supported("Hand of Vecna");
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "The Book of Vile Darkness");
    // The opponent's Eye of Vecna; P0's Phyrexian Metamorph enters as a copy of it (an
    // artifact P0 controls named Eye of Vecna), exiled with the Hand by the Book.
    t.battlefield(P1, "Eye of Vecna");
    t.battlefield(P0, "Hand of Vecna");
    t.answer_yes(P0, true);
    let eye = t.named_on_battlefield("Eye of Vecna")[0];
    t.answer_choose(P0, &[obj(eye)]);
    let mm = t.enter(P0, "Phyrexian Metamorph");
    t.resolve_all();
    assert_eq!(t.g.obj(mm).chars.name, "Eye of Vecna");
    let hand = t.named_on_battlefield("Hand of Vecna")[0];
    t.answer_choose(P0, &[obj(mm)]);
    t.answer_choose(P0, &[obj(hand)]);
    t.activate(P0, book, 0, &[]).unwrap();
    t.resolve_all();
    let vecna = t.named_on_battlefield("Vecna");
    assert_eq!(vecna.len(), 1);
    assert!(t.on_battlefield(eye), "the opponent's Eye stays");
    let texts: Vec<String> = t
        .g
        .obj(vecna[0])
        .chars
        .abilities
        .iter()
        .map(|a| a.text.to_string())
        .collect();
    // The Book's and the Hand's triggered abilities; nothing from the Eye (the exiled
    // card is a Phyrexian Metamorph, which has none).
    assert!(texts.iter().any(|x| x.contains("end step")), "{texts:?}");
    assert!(texts.iter().any(|x| x.contains("beginning of combat")), "{texts:?}");
    assert!(!texts.iter().any(|x| x.contains("upkeep")), "{texts:?}");
}
