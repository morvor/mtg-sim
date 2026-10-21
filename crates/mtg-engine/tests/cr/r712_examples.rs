//! The worked examples of CR 712 (double-faced cards, meld cards and melded permanents),
//! played out as tests.

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::merge;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const INSTIGATOR: &str = "Instigator Gang // Wildblood Pack";
const KRUIN: &str = "Kruin Outlaw // Terror of Kruin Pass";
const IRONSMITH: &str = "Village Ironsmith // Ironfang";

fn transform(t: &mut TestGame, id: ObjectId) {
    run_effect(
        t,
        P0,
        None,
        &[Entity::Object(id)],
        Effect::Transform {
            what: Sel::Target(0),
        },
    );
}

/// Melds Graf Rats (2/1 Rat) and Midnight Scavengers (3/3 Human Rogue), owned by `p`, into
/// Chittering Host (5/6).
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

fn yes_no_asked(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
        .count()
}

#[test]
fn a_clone_copying_wildblood_pack_cant_transform() {
    cr!("712.9", "701.27c");
    cr_example!("712.9", "A Clone enters the battlefield as a copy of Wildblood Pack");
    supported(INSTIGATOR);
    supported("Clone");
    let mut t = TestGame::new(2);
    let gang = t.battlefield(P1, INSTIGATOR);
    transform(&mut t, gang);
    assert_eq!(t.obj(gang).chars.name, "Wildblood Pack");
    // "You may have Clone enter as a copy of any creature on the battlefield."
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(gang)]);
    let clone = t.enter(P0, "Clone");
    let clone = t.g.current(clone);
    assert_eq!(t.obj(clone).chars.name, "Wildblood Pack");
    assert_eq!(t.pt(clone), (5, 5));
    // Transforming the Clone does nothing: it isn't a double-faced card.
    transform(&mut t, clone);
    assert!(t.g.is_live(clone));
    assert_eq!(t.obj(clone).face, FaceState::Front);
    assert_eq!(t.obj(clone).chars.name, "Wildblood Pack");
    assert_eq!(t.pt(clone), (5, 5));
    // The real Wildblood Pack transforms back into Instigator Gang.
    transform(&mut t, gang);
    assert_eq!(t.obj(gang).chars.name, "Instigator Gang");
}

#[test]
fn a_kruin_outlaw_copying_elite_vanguard_transforms_but_stays_a_copy_that_turn() {
    cr!("712.9", "707.2");
    cr_example!("712.9", "Cytoshape");
    supported(KRUIN);
    supported("Cytoshape");
    supported("Moonmist");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let kruin = t.battlefield(P0, KRUIN);
    let vanguard = t.battlefield(P1, "Elite Vanguard");
    // Cytoshape: "Choose a nonlegendary creature on the battlefield. Target creature
    // becomes a copy of that creature until end of turn."
    t.lands(P0, "Tropical Island", 5);
    t.answer_choose(P0, &[Entity::Object(vanguard)]);
    let cyto = t.hand(P0, "Cytoshape");
    t.cast(P0, cyto).target(kruin).go();
    t.resolve_all();
    assert_eq!(t.obj(kruin).chars.name, "Elite Vanguard");
    assert!(t.obj(kruin).chars.has_subtype("Human"));
    // Moonmist: "Transform all Humans." The copy is a double-faced card: it transforms.
    // The real Elite Vanguard (a Human too) isn't, and doesn't.
    let moonmist = t.hand(P0, "Moonmist");
    t.cast(P0, moonmist).go();
    t.resolve_all();
    assert!(t.g.is_live(kruin));
    assert_eq!(t.obj(kruin).face, FaceState::Back);
    assert_eq!(t.obj(vanguard).face, FaceState::Front);
    // Back face up, but still a copy of Elite Vanguard (a 2/1 Human Soldier) this turn.
    let c = &t.obj(kruin).chars;
    assert_eq!(c.name, "Elite Vanguard");
    assert!(c.has_subtype("Human") && c.has_subtype("Soldier"));
    assert_eq!(t.pt(kruin), (2, 1));
    // Once the copy effect ends, it's Terror of Kruin Pass.
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    let kruin = t.g.current(kruin);
    assert_eq!(t.obj(kruin).chars.name, "Terror of Kruin Pass");
    assert_eq!(t.pt(kruin), (3, 3));
}

#[test]
fn village_ironsmith_keeps_its_pump_as_it_transforms_into_ironfang() {
    cr!("712.18", "611.2c");
    cr_example!("712.18", "Village Ironsmith");
    supported(IRONSMITH);
    let mut t = TestGame::new(2);
    let smith = t.battlefield(P0, IRONSMITH);
    assert_eq!(t.pt(smith), (1, 1));
    // An effect gives it +2/+2 until end of turn.
    t.lands(P0, "Forest", 1);
    let growth = t.custom(
        P0,
        oracle_card(
            "Small Growth",
            "Instant",
            "{G}",
            None,
            "Target creature gets +2/+2 until end of turn.",
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, growth).target(smith).go();
    t.resolve_all();
    assert_eq!(t.pt(smith), (3, 3));
    // It transforms into Ironfang (3/1): the same object, still +2/+2.
    transform(&mut t, smith);
    assert!(t.g.is_live(smith));
    assert_eq!(t.obj(smith).chars.name, "Ironfang");
    assert_eq!(t.pt(smith), (5, 3));
    // Until end of turn. (Only one spell was cast this turn: Ironfang doesn't transform
    // back at the next upkeep.)
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let smith = t.g.current(smith);
    t.g.recompute();
    assert_eq!(t.obj(smith).chars.name, "Ironfang");
    assert_eq!(t.pt(smith), (3, 1));
}

#[test]
fn duplicants_controller_chooses_which_card_of_chittering_host_was_exiled_last() {
    cr!("712.21b", "607.2a");
    cr_example!("712.21b", "Duplicant");
    supported("Duplicant");
    // Answered as the cards are exiled: bottom to top (first to last) is the order the
    // components are given in: Graf Rats, then Midnight Scavengers.
    for (order, last, pt, ty) in [
        (vec![0, 1], "Midnight Scavengers", (3, 3), "Rogue"),
        (vec![1, 0], "Graf Rats", (2, 1), "Rat"),
    ] {
        let mut t = TestGame::new(2);
        let host = chittering_host(&mut t, P1);
        // "When Duplicant enters, you may exile target nontoken creature."
        t.answer_targets(P0, &[Entity::Object(host)]);
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        let dup = t.enter(P0, "Duplicant");
        t.resolve_all();
        let dup = t.g.current(dup);
        assert_eq!(cards_named(&t, Zone::Exile, "Graf Rats"), 1);
        assert_eq!(cards_named(&t, Zone::Exile, "Midnight Scavengers"), 1);
        // Duplicant's controller (P0) made the choice.
        assert!(t
            .asked()
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
        assert_eq!(t.pt(dup), pt, "last exiled: {last}");
        let c = &t.obj(dup).chars;
        assert!(c.has_subtype(ty), "{last}");
        assert!(c.has_subtype("Shapeshifter"));
    }
}

#[test]
fn otherworldly_journey_returns_both_cards_of_chittering_host_with_a_counter() {
    cr!("712.21c", "603.7c");
    cr_example!("712.21c", "Otherworldly Journey");
    supported("Otherworldly Journey");
    let mut t = TestGame::new(2);
    let host = chittering_host(&mut t, P1);
    t.lands(P0, "Plains", 2);
    let oj = t.hand(P0, "Otherworldly Journey");
    t.cast(P0, oj).target(host).go();
    t.resolve_all();
    assert!(!t.g.is_live(host));
    assert_eq!(cards_named(&t, Zone::Exile, "Graf Rats"), 1);
    assert_eq!(cards_named(&t, Zone::Exile, "Midnight Scavengers"), 1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    for name in ["Graf Rats", "Midnight Scavengers"] {
        let back = t.named_on_battlefield(name);
        assert_eq!(back.len(), 1, "{name} returned");
        assert_eq!(t.obj_now(back[0]).controller, P1, "under its owner's control");
        assert_eq!(t.counters(back[0], "+1/+1"), 1, "{name}");
    }
    assert!(t.named_on_battlefield("Chittering Host").is_empty());
}

#[test]
fn mimic_vat_makes_one_choice_for_both_cards_of_chittering_host() {
    cr!("712.21c", "603.10a");
    cr_example!("712.21c", "Mimic Vat");
    supported("Mimic Vat");
    for exile in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mimic Vat");
        let host = chittering_host(&mut t, P1);
        // "Whenever a nontoken creature dies, you may exile that card."
        t.answer_yes(P0, exile);
        t.g.move_object(host, Zone::Graveyard(P1), MoveCause::Destroy, None);
        t.resolve_all();
        assert_eq!(yes_no_asked(&t, P0), 1, "a single choice");
        let (ex, gy) = if exile { (1, 0) } else { (0, 1) };
        for name in ["Graf Rats", "Midnight Scavengers"] {
            assert_eq!(cards_named(&t, Zone::Exile, name), ex, "{name}");
            assert_eq!(cards_named(&t, Zone::Graveyard(P1), name), gy, "{name}");
        }
    }
}

#[test]
fn leyline_of_the_void_or_wheel_of_sun_and_moon_moves_both_cards_of_chittering_host() {
    cr!("712.21d", "616.1");
    cr_example!("712.21d", "Wheel of Sun and Moon");
    supported("Leyline of the Void");
    supported("Wheel of Sun and Moon");
    let mut outcomes = Vec::new();
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        // Leyline of the Void (P1): "If a card would be put into an opponent's graveyard
        // from anywhere, exile it instead." Wheel of Sun and Moon enchanting P0: "If a
        // card would be put into enchanted player's graveyard from anywhere, instead that
        // card is revealed and put on the bottom of that player's library."
        t.battlefield(P1, "Leyline of the Void");
        let wheel = t.battlefield(P0, "Wheel of Sun and Moon");
        t.g.obj_mut(wheel).attached_to = Some(Entity::Player(P0));
        let host = chittering_host(&mut t, P0);
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        let lib_before = t.library_size(P0);
        t.g.move_object(host, Zone::Graveyard(P0), MoveCause::Destroy, None);
        t.settle();
        let choices = t
            .asked()
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseReplacement { .. }))
            .count();
        assert_eq!(choices, 1, "the affected player chooses once for both cards");
        assert_eq!(t.graveyard_size(P0), 0);
        let exiled = cards_named(&t, Zone::Exile, "Graf Rats")
            + cards_named(&t, Zone::Exile, "Midnight Scavengers");
        let bottom = t.library_size(P0) - lib_before;
        if bottom == 2 {
            let lib = &t.g.player(P0).library;
            let mut names: Vec<&str> = lib[..2]
                .iter()
                .map(|id| t.obj(*id).chars.name.as_str())
                .collect();
            names.sort();
            assert_eq!(names, vec!["Graf Rats", "Midnight Scavengers"]);
        }
        outcomes.push((exiled, bottom));
    }
    // Each choice sends both cards to the same zone, and the two choices differ.
    outcomes.sort();
    assert_eq!(outcomes, vec![(0, 2), (2, 0)]);
}

const BIRD_ADMIRER: &str = "Bird Admirer // Wing Shredder";
const KYLEM: &str = "Invasion of Kylem // Valor's Reach Tag Team";

/// The situation of the CR 712.13a example: P0 controls Mycosynth Lattice and (if
/// `march`) March of the Machines, and a Clone that is a copy of Bird Admirer; it's
/// night. P0 casts Mystic Reflection targeting the Clone, then Invasion of Kylem.
/// Returns the game, the Clone and the Invasion card.
fn mystic_reflection_on_a_clone(march: bool) -> (TestGame, ObjectId, ObjectId) {
    use mtg_engine::mana::ManaType;
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let admirer = t.battlefield(P1, BIRD_ADMIRER);
    // "You may have Clone enter as a copy of any creature on the battlefield."
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(admirer)]);
    let clone = t.enter(P0, "Clone");
    let clone = t.g.current(clone);
    assert_eq!(t.obj(clone).chars.name, "Bird Admirer");
    // Mycosynth Lattice: "All permanents are artifacts in addition to their other types."
    // March of the Machines: "Each noncreature artifact is an artifact creature with power
    // and toughness each equal to its mana value."
    t.battlefield(P0, "Mycosynth Lattice");
    if march {
        t.battlefield(P0, "March of the Machines");
    }
    t.settle();
    // It becomes night: the real Bird Admirer transforms; the Clone isn't represented by
    // a double-faced card and can't.
    t.g.set_day(false);
    t.settle();
    assert_eq!(t.obj_now(admirer).chars.name, "Wing Shredder");
    assert_eq!(t.obj(clone).face, FaceState::Front);
    assert_eq!(t.obj(clone).chars.name, "Bird Admirer");
    // (Mana in the pool: with Mycosynth Lattice and March of the Machines, lands would be
    // 0/0 creatures.)
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::C, 3);
    // Mystic Reflection: "Choose target nonlegendary creature. The next time one or more
    // creatures or planeswalkers enter this turn, they enter as copies of the chosen
    // creature."
    let mr = t.hand(P0, "Mystic Reflection");
    t.cast(P0, mr).target(clone).go();
    t.resolve_all();
    // Invasion of Kylem: a Siege battle whose back face is a sorcery.
    let kylem = t.hand(P0, KYLEM);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.cast(P0, kylem).go();
    t.resolve_all();
    (t, clone, kylem)
}

#[test]
fn invasion_of_kylem_entering_as_a_night_copy_of_bird_admirer_goes_to_the_graveyard() {
    cr!("712.13a", "702.145b", "614.12", "707.9");
    cr_example!("712.13a", "Mystic Reflection");
    for name in [
        BIRD_ADMIRER,
        KYLEM,
        "Mystic Reflection",
        "Mycosynth Lattice",
        "March of the Machines",
        "Clone",
    ] {
        supported(name);
    }
    let (t, clone, kylem) = mystic_reflection_on_a_clone(true);
    // Entering as a creature, it would enter as a copy of Bird Admirer, with daybound;
    // at night it would enter transformed, but its back face is a sorcery: it's put into
    // its owner's graveyard instead.
    assert_eq!(t.zone(kylem), Zone::Graveyard(P0));
    assert_eq!(t.named_on_battlefield("Bird Admirer"), vec![clone]);
    assert!(t.named_on_battlefield("Invasion of Kylem").is_empty());
    assert!(t.named_on_battlefield("Valor's Reach Tag Team").is_empty());
    // Without March of the Machines it isn't a creature as it enters: Mystic Reflection
    // doesn't apply, and it enters as a battle.
    let (t, _, kylem) = mystic_reflection_on_a_clone(false);
    assert_eq!(t.zone(kylem), Zone::Battlefield);
    let kylem = t.g.current(kylem);
    assert_eq!(t.obj(kylem).chars.name, "Invasion of Kylem");
    assert!(t.obj(kylem).chars.is(CardType::Battle));
}
