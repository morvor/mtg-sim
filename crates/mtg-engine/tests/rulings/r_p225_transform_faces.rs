//! Rulings batch P225 — the faces of double-faced cards: which face a card enters or is
//! cast with (CR 712.11, 712.14), sorcery faces that can't be transformed into or put
//! onto the battlefield (CR 701.27d, 712.10, 712.14a), "transformed permanents"
//! (CR 701.27g), face-down double-faced permanents (CR 701.27b, 712.15a), entering back
//! face up isn't transforming (CR 701.27e, 712.20), and planeswalker faces that a
//! creature transforms into with no loyalty counters (CR 306.5b, 704.5i).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s17_common::*;
use mtg_engine::facedown;
use mtg_engine::kwa::manifest::{put_face_down, MANIFESTED};
use mtg_engine::object::{FaceState, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

const BOLAS: &str = "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen";
const CAPTIVE: &str = "Wolfbitten Captive // Krallenhorde Killer";
const LILIANA: &str = "Liliana, Heretical Healer // Liliana, Defiant Necromancer";
const SEPHIROTH: &str = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";
const ZENOS: &str = "Zenos yae Galvus // Shinryu, Transcendent Rival";
const CHOCOBO: &str = "Sidequest: Raise a Chocobo // Black Chocobo";

fn emblems(t: &TestGame, p: PlayerId) -> usize {
    t.g.command
        .iter()
        .filter(|id| t.obj(**id).kind == ObjKind::Emblem && t.obj(**id).controller == p)
        .count()
}

#[test]
fn a_double_faced_card_enters_and_is_cast_front_face_up_by_default() {
    cr!("712.14", "712.14a", "712.11");
    ruling!(
        "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen",
        "A double-faced card enters the battlefield with its front face up by default"
    );
    supported(BOLAS);
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, BOLAS);
    let a = put_onto_battlefield(&mut t, P0, a, false).unwrap();
    assert_eq!(name_of(&t, a), "Nicol Bolas, the Ravager");
    let b = t.graveyard(P1, BOLAS);
    let b = put_onto_battlefield(&mut t, P1, b, true).unwrap();
    assert_eq!(name_of(&t, b), "Nicol Bolas, the Arisen");
    assert_eq!(t.counters(b, counters::LOYALTY), 7);
    // Cast from hand: the front face is on the stack.
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, BOLAS);
    let card = t.hand(P0, BOLAS);
    let spell = t.cast(P0, card).go();
    assert_eq!(t.obj(spell).chars.name, "Nicol Bolas, the Ravager");
    assert_eq!(face(&t, spell), FaceState::Front);
}

#[test]
fn a_sorcery_front_face_cant_be_put_onto_the_battlefield_or_transformed_into() {
    cr!("712.14a", "712.10", "701.27d");
    ruling!(
        "Startled Awake // Persistent Nightmare",
        "If an effect instructs you to transform Persistent Nightmare, the instruction is ignored."
    );
    ruling!(
        "Esper Origins // Summon: Esper Maduin",
        "If an effect instructs you to transform Summon: Esper Maduin, the instruction is ignored."
    );
    supported("Startled Awake // Persistent Nightmare");
    for (name, back) in [
        (
            "Startled Awake // Persistent Nightmare",
            "Persistent Nightmare",
        ),
        (
            "Esper Origins // Summon: Esper Maduin",
            "Summon: Esper Maduin",
        ),
    ] {
        let mut t = TestGame::new(2);
        // Exiled, then returned (not transformed): it stays in exile.
        let card = t.exile(P0, name);
        assert!(
            put_onto_battlefield(&mut t, P0, card, false).is_none(),
            "{name}"
        );
        assert_eq!(t.zone(card), Zone::Exile, "{name}");
        // Returned transformed: its back face.
        let perm = put_onto_battlefield(&mut t, P0, card, true).expect(name);
        assert_eq!(name_of(&t, perm), back);
        // Transforming it is ignored.
        transform(&mut t, perm);
        assert!(t.on_battlefield(perm), "{name}");
        assert_eq!(name_of(&t, perm), back);
        assert_eq!(face(&t, perm), FaceState::Back);
    }
}

#[test]
fn only_a_double_faced_permanent_back_face_up_is_transformed() {
    cr!("701.27g");
    // (The rulings of Oculus Whelp and Corruption of Towashi that also exclude modal
    // double-faced permanents predate those being able to transform; the current
    // CR 701.27a and 701.27g don't exclude them — see `transform_rules.rs` and
    // docs/rulings-exemptions/rulings-P225.tsv.)
    supported("Oculus Whelp");
    let whelp_triggers = |t: &mut TestGame| {
        let whelp = t.battlefield(P0, "Oculus Whelp");
        destroy(t, whelp);
        triggers_on_stack(t, "draw a card")
    };
    // No double-faced permanent: no trigger.
    let mut t = TestGame::new(2);
    assert_eq!(whelp_triggers(&mut t), 0);
    // A double-faced card that transformed and transformed back: not transformed.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, CAPTIVE);
    transform(&mut t, c);
    transform(&mut t, c);
    assert_eq!(whelp_triggers(&mut t), 0);
    // Back face up: a transformed permanent.
    let mut t = TestGame::new(2);
    enter_transformed(&mut t, P0, CAPTIVE);
    assert_eq!(whelp_triggers(&mut t), 1);
}

#[test]
fn turning_a_permanent_face_up_isnt_transforming() {
    cr!("701.27b", "701.27a");
    ruling!(
        "Norn's Inquisitor",
        "A face-up permanent turning face down doesn't count as transforming, nor does a face-down permanent turning face up."
    );
    ruling!(
        "Corruption of Towashi",
        "Similarly, only transforming double-faced permanents (including transforming double-faced cards and Incubator tokens) can transform."
    );
    supported("Norn's Inquisitor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Corruption of Towashi");
    t.battlefield(P0, "Norn's Inquisitor");
    // A manifested Phyrexian creature card turned face up.
    let card = t.library_top(P0, "Norn's Inquisitor");
    let fd = put_face_down(&mut t.g, card, P0, MANIFESTED, None).unwrap();
    t.settle();
    t.resolve_all();
    assert!(facedown::turn_face_up(&mut t.g, fd, true));
    t.g.flush_events();
    t.settle();
    assert!(t.obj_now(fd).chars.has_subtype("Phyrexian"));
    assert_eq!(t.stack_len(), 0);
    // A non-double-faced permanent told to transform: nothing happens.
    let bears = t.battlefield(P0, "Grizzly Bears");
    transform(&mut t, bears);
    assert_eq!(name_of(&t, bears), "Grizzly Bears");
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_manifested_double_faced_card_cant_transform_and_turns_face_up_front_face_up() {
    cr!("712.15", "712.15a", "701.40b");
    ruling!(
        "Liliana, Heretical Healer // Liliana, Defiant Necromancer",
        "While face down, it can’t transform."
    );
    supported(LILIANA);
    let mut t = TestGame::new(2);
    let card = t.library_top(P0, LILIANA);
    let fd = put_face_down(&mut t.g, card, P0, MANIFESTED, None).unwrap();
    t.settle();
    assert_eq!(t.pt(fd), (2, 2));
    assert!(t.obj_now(fd).chars.name.is_empty());
    transform(&mut t, fd);
    assert!(t.obj_now(fd).face_down);
    assert_eq!(t.pt(fd), (2, 2));
    assert!(facedown::turn_face_up(&mut t.g, fd, true));
    t.g.recompute();
    assert_eq!(name_of(&t, fd), "Liliana, Heretical Healer");
    assert_eq!(face(&t, fd), FaceState::Front);
}

#[test]
fn entering_back_face_up_isnt_transforming() {
    cr!("701.27e", "712.20", "712.14a");
    ruling!(
        "Zenos yae Galvus // Shinryu, Transcendent Rival",
        "it didn't transform, so you won't choose an opponent"
    );
    ruling!(
        "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel",
        "it didn't transform, so you won't get an emblem"
    );
    ruling!(
        "Sidequest: Raise a Chocobo // Black Chocobo",
        "it didn't transform, so you won't search"
    );
    supported(SEPHIROTH);
    supported(CHOCOBO);
    // Sephiroth: an emblem only when he transforms.
    let mut t = TestGame::new(2);
    enter_transformed(&mut t, P0, SEPHIROTH);
    t.resolve_all();
    assert_eq!(emblems(&t, P0), 0);
    let s = t.battlefield(P1, SEPHIROTH);
    transform(&mut t, s);
    assert_eq!(emblems(&t, P1), 1);
    // Black Chocobo: no search when it enters back face up.
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    enter_transformed(&mut t, P0, CHOCOBO);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Library(P0));
    // Transforming it does search.
    let c = t.battlefield(P0, CHOCOBO);
    transform(&mut t, c);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    // Zenos (Shinryu's Burning Chains doesn't compile): no opponent is chosen when it
    // enters back face up; one is as it transforms.
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    enter_transformed(&mut t, P0, ZENOS);
    assert!(asked_since(&t, from).is_empty());
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, ZENOS);
    let from = t.asked().len();
    transform(&mut t, z);
    assert_eq!(name_of(&t, z), "Shinryu, Transcendent Rival");
    assert!(asked_since(&t, from).iter().any(|(p, _)| *p == P0));
}

#[test]
fn a_creature_that_transforms_into_a_planeswalker_has_no_loyalty_and_dies() {
    cr!("306.5b", "704.5i", "712.18");
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "Ajani, Nacatl Avenger won't have any loyalty counters on him and will subsequently be put into his owner's graveyard."
    );
    ruling!(
        "Grist, Voracious Larva // Grist, the Plague Swarm",
        "Grist, the Plague Swarm won't have any loyalty counters on it and will subsequently be put into its owner's graveyard."
    );
    ruling!(
        "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen",
        "In some rare cases, a spell or ability may cause Nicol Bolas to transform while it's a creature on the battlefield."
    );
    ruling!(
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
        "Ral, Leyline Prodigy won't have any loyalty counters on him and will subsequently be put into his owner's graveyard."
    );
    ruling!(
        "Sorin of House Markov // Sorin, Ravenous Neonate",
        "Sorin, Ravenous Neonate won't have any loyalty counters on him and will subsequently be put into his owner's graveyard."
    );
    ruling!(
        "Tamiyo, Inquisitive Student // Tamiyo, Seasoned Scholar",
        "Tamiyo, Seasoned Scholar won't have any loyalty counters on her and will subsequently be put into her owner's graveyard."
    );
    ruling!(
        "Liliana, Heretical Healer // Liliana, Defiant Necromancer",
        "In some rare cases, a spell or ability may cause one of these five cards to transform"
    );
    // (Some of these have abilities that don't compile; none is involved.)
    for name in [
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "Grist, Voracious Larva // Grist, the Plague Swarm",
        BOLAS,
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
        "Sorin of House Markov // Sorin, Ravenous Neonate",
        "Tamiyo, Inquisitive Student // Tamiyo, Seasoned Scholar",
        LILIANA,
    ] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        transform(&mut t, c);
        assert_eq!(t.zone(c), Zone::Graveyard(P0), "{name}");
        assert_eq!(face(&t, c), FaceState::Front, "{name}");
    }
}
