//! Rulings batch P178 — "threaten" effects: gain control of a permanent until end of
//! turn, untap it, it gains haste (Zealous Conscripts, Traitorous Blood, Wrangle,
//! Traitorous Instinct, Unwilling Recruit, Sarkhan Vol, Conquering Manticore, Magus of
//! the Unseen, Infernal Captor, Frenzied Fugue, The Nipton Lottery, Domineering Will,
//! Grab the Reins) and Centaur Chieftain's team pump.

use crate::r_p178_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// How a threaten case is used on its target.
enum Via {
    /// Cast the spell `name` (X = 1) targeting the creature.
    Spell(&'static str),
    /// `name` enters the battlefield under P0's control; its ETB trigger targets it.
    Enters(&'static str),
    /// Activate `name`'s activated ability with this index (P0 controls `name`).
    Activate(&'static str, usize),
}

/// A threaten effect, used by P0 on `target`: P0 controls it, it's untapped and hasty.
fn threaten(t: &mut TestGame, via: &Via, target: ObjectId) {
    match via {
        Via::Spell(name) => {
            t.answer(P0, DecisionKind::X, Answer::Number(1));
            cast_resolve(t, P0, name, &[obj(target)]);
        }
        Via::Enters(name) => {
            supported(name);
            t.answer_targets(P0, &[obj(target)]);
            t.enter(P0, name);
            t.resolve_all();
        }
        Via::Activate(name, index) => {
            supported(name);
            let source = t.battlefield(P0, name);
            t.lands(P0, "Island", 4);
            t.activate(P0, source, *index, &[obj(target)]).unwrap();
            t.resolve_all();
        }
    }
    assert_eq!(controller(t, target), P0, "P0 controls it");
    assert!(!tapped(t, target), "it's untapped");
    assert!(hasty(t, target), "it has haste");
}

#[test]
fn threaten_effects_can_target_your_own_or_an_untapped_permanent() {
    cr!("115.1", "613.1b", "701.26b");
    // (case, the target's owner, its card, whether it starts tapped)
    let cases: Vec<(Via, PlayerId, &str, bool)> = vec![
        (Via::Enters("Zealous Conscripts"), P0, "Grizzly Bears", true),
        (
            Via::Enters("Zealous Conscripts"),
            P1,
            "Grizzly Bears",
            false,
        ),
        (Via::Spell("Traitorous Blood"), P0, "Grizzly Bears", true),
        (Via::Spell("Wrangle"), P0, "Grizzly Bears", false),
        (
            Via::Spell("Traitorous Instinct"),
            P0,
            "Grizzly Bears",
            false,
        ),
        (
            Via::Spell("Traitorous Instinct"),
            P1,
            "Grizzly Bears",
            false,
        ),
        (Via::Spell("Unwilling Recruit"), P0, "Grizzly Bears", true),
        (Via::Activate("Sarkhan Vol", 1), P0, "Grizzly Bears", true),
        (
            Via::Enters("Conquering Manticore"),
            P1,
            "Grizzly Bears",
            false,
        ),
        (
            Via::Activate("Magus of the Unseen", 0),
            P1,
            "Sol Ring",
            false,
        ),
    ];
    ruling!(
        "Zealous Conscripts",
        "The triggered ability can target any permanent, including one that's untapped or one that you already control."
    );
    ruling!(
        "Traitorous Blood",
        "Traitorous Blood can target any creature, even one that’s tapped or one you already control."
    );
    ruling!(
        "Wrangle",
        "Wrangle can target any creature with power 4 or less, even one that you already control or that is already untapped."
    );
    ruling!(
        "Traitorous Instinct",
        "You may target a creature you already control. You may target a creature that’s already untapped."
    );
    ruling!(
        "Unwilling Recruit",
        "You may target a creature you already control with Unwilling Recruit."
    );
    ruling!(
        "Sarkhan Vol",
        "If you target a creature you already control, you’ll gain control of it (which will usually have no visible effect), it’ll untap, and it’ll gain haste until end of turn."
    );
    ruling!(
        "Conquering Manticore",
        "You may target a creature that’s already untapped."
    );
    ruling!(
        "Magus of the Unseen",
        "You may target an untapped artifact with this ability."
    );
    for (via, owner, name, start_tapped) in &cases {
        let mut t = TestGame::new(2);
        let target = t.battlefield(*owner, name);
        if *start_tapped {
            t.g.tap(target);
        }
        let power = t.pt(target).0;
        threaten(&mut t, via, target);
        if let Via::Spell("Unwilling Recruit") = via {
            assert_eq!(t.pt(target).0, power + 1, "+X/+0");
        }
        if let Via::Spell("Traitorous Instinct") = via {
            assert_eq!(t.pt(target).0, power + 2, "+2/+0");
        }
    }
}

#[test]
fn infernal_captor_can_untap_and_haste_itself() {
    cr!("702.110a", "613.1b");
    ruling!(
        "Infernal Captor",
        "You can target an artifact or creature you already control (including Infernal Captor) with Infernal Captor's last ability just to untap that permanent and give it haste."
    );
    supported("Infernal Captor");
    let mut t = TestGame::new(2);
    let fodder = t.battlefield(P0, "Grizzly Bears");
    // Exploit: sacrifice the Bears; the exploit trigger targets the (summoning-sick)
    // Captor itself.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(fodder)]);
    let captor = t.g.create_card_object(
        mtg_engine::card::card("Infernal Captor"),
        P0,
        mtg_engine::object::Zone::Nowhere,
    );
    t.answer_targets(P0, &[obj(captor)]);
    let captor = enter_existing(&mut t, captor);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "exploited");
    assert_eq!(controller(&mut t, captor), P0);
    assert!(hasty(&t, captor), "the Captor has haste");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, captor), "and can attack this turn");
}

/// Moves an already-created object onto the battlefield under its owner's control.
fn enter_existing(t: &mut TestGame, id: ObjectId) -> ObjectId {
    let p = t.obj_now(id).owner;
    t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: id,
        to: mtg_engine::object::Zone::Battlefield,
        pos: mtg_engine::ability::LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(p),
        etb: mtg_engine::replacement::EtbInfo {
            controller: Some(p),
            ..Default::default()
        },
        source: None,
    })
    .expect("entered")
}

#[test]
fn sarkhan_vol_plus_one_with_no_creatures_still_adds_loyalty() {
    cr!("606.4");
    ruling!(
        "Sarkhan Vol",
        "You may activate the first ability even if you control no creatures. Sarkhan Vol gains a loyalty counter, but the ability will have no effect."
    );
    supported("Sarkhan Vol");
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan Vol");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.counters(sarkhan, "loyalty"), 4);
    t.activate(P0, sarkhan, 0, &[]).unwrap();
    assert_eq!(t.counters(sarkhan, "loyalty"), 5, "paid as a cost");
    t.resolve_all();
    assert_eq!(t.counters(sarkhan, "loyalty"), 5);
    assert_eq!(
        t.pt(bears),
        (2, 2),
        "the opponent's creatures aren't affected"
    );
}

#[test]
fn conquering_manticore_steal_outlasts_losing_the_manticore() {
    cr!("611.2a", "613.1b");
    ruling!(
        "Conquering Manticore",
        "You retain control of the affected creature until end of turn, even if you lose control of Conquering Manticore before then."
    );
    supported("Conquering Manticore");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let manticore = t.enter(P0, "Conquering Manticore");
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    give_control(&mut t, manticore, P1);
    assert_eq!(controller(&mut t, manticore), P1);
    assert_eq!(controller(&mut t, bears), P0, "still P0's this turn");
    destroy(&mut t, manticore);
    assert_eq!(controller(&mut t, bears), P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&mut t, bears), P1, "returned at end of turn");
}

#[test]
fn frenzied_fugue_steals_when_it_enters_and_again_each_upkeep() {
    cr!("603.2", "611.2a");
    ruling!(
        "Frenzied Fugue",
        "The triggered ability triggers both when Frenzied Fugue enters the battlefield and at the beginning of your upkeep. You don't have to choose only one."
    );
    supported("Frenzied Fugue");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let fugue = cast_targeting(&mut t, P0, "Frenzied Fugue", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(fugue).attached_to, Some(obj(bears)));
    assert_eq!(controller(&mut t, bears), P0, "the enters trigger");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(controller(&mut t, bears), P1, "until end of turn");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0, "the upkeep trigger");
    assert!(hasty(&t, bears));
}

#[test]
fn the_nipton_lottery_can_choose_a_hexproof_or_shroud_creature() {
    cr!("702.11b", "702.18a");
    ruling!(
        "The Nipton Lottery",
        "The creature chosen at random isn’t targeted, so a creature with shroud or an opponent’s creature with hexproof can be chosen this way."
    );
    supported("The Nipton Lottery");
    for name in ["Gladecover Scout", "Silhana Ledgewalker", "Troll Ascetic"] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, name);
        t.g.tap(c);
        cast_resolve(&mut t, P0, "The Nipton Lottery", &[]);
        assert!(t.on_battlefield(c), "{name} isn't destroyed");
        assert_eq!(controller(&mut t, c), P0, "{name} was chosen");
        assert!(!tapped(&t, c));
        assert!(hasty(&t, c));
    }
}

#[test]
fn domineering_will_the_creatures_controller_chooses_their_blocks() {
    cr!("509.1a", "509.1c");
    ruling!(
        "Domineering Will",
        "The controller of the creatures still chooses which attacking creature each of the three target creatures blocks."
    );
    supported("Domineering Will");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P0, "Craw Wurm");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    // The defending player takes the Wurm, which must block; P1 chooses the Giant.
    cast_targeting(
        &mut t,
        P1,
        "Domineering Will",
        &[Entity::Player(P1), obj(wurm)],
    );
    t.resolve_all();
    assert_eq!(controller(&mut t, wurm), P1);
    block_and_finish(&mut t, P1, &[(wurm, giant)]);
    assert!(
        t.in_graveyard(P0, "Hill Giant"),
        "the Wurm blocked the Giant"
    );
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 18, "the Bears were unblocked");
}

#[test]
fn grab_the_reins_sacrifices_on_resolution_and_must_sacrifice() {
    cr!("601.2c", "608.2c", "701.21a");
    ruling!(
        "Grab the Reins",
        "you choose the target of the damage on announcement, but don’t choose what to sacrifice until resolution. You must sacrifice a creature when Grab the Reins resolves, even if you don’t want to. If you don’t control any creatures at that time, Grab the Reins deals no damage."
    );
    supported("Grab the Reins");
    // P0 chooses which creature to sacrifice as the spell resolves: the Giant, created
    // after the spell was cast.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Grab the Reins");
    let card = t.hand(P0, "Grab the Reins");
    t.cast(P0, card).modes(&[1]).target(Entity::Player(P1)).go();
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 17, "3 damage from the Giant's power");

    // Without an answer, a creature is still sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Grab the Reins");
    let card = t.hand(P0, "Grab the Reins");
    t.cast(P0, card).modes(&[1]).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "sacrificed");
    assert_eq!(t.life(P1), 18);

    // No creature to sacrifice: no damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Grab the Reins");
    let card = t.hand(P0, "Grab the Reins");
    t.cast(P0, card).modes(&[1]).target(Entity::Player(P1)).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "no damage");
}

#[test]
fn centaur_chieftain_pumps_only_the_creatures_you_control_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Centaur Chieftain",
        "All creatures under your control when Centaur Chieftain’s triggered ability resolves are affected, including Centaur Chieftain itself. Ones that come under your control or become creatures later in the turn are not."
    );
    supported("Centaur Chieftain");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, 7);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let chief = t.enter(P0, "Centaur Chieftain");
    t.resolve_all();
    assert_eq!(t.pt(chief), (4, 4), "itself");
    assert_eq!(t.pt(bears), (3, 3));
    assert!(has_kw(
        &t,
        bears,
        mtg_engine::keywords::KeywordKind::Trample
    ));
    // A creature that comes under P0's control later isn't affected.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    give_control(&mut t, theirs, P0);
    assert_eq!(t.pt(theirs), (2, 2));
    assert!(!has_kw(
        &t,
        theirs,
        mtg_engine::keywords::KeywordKind::Trample
    ));
    // Nor is one that becomes a creature later (Mishra's Factory, animated).
    let factory = t.battlefield(P0, "Mishra's Factory");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, factory, "becomes").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(factory), (2, 2));
    assert!(!has_kw(
        &t,
        factory,
        mtg_engine::keywords::KeywordKind::Trample
    ));
}
