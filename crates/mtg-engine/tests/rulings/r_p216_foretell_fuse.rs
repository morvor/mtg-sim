//! Rulings batch P216 — foretell (CR 702.143), formidable, "party" ability words, fuse
//! (CR 702.102), gift (CR 702.174) and gravestorm (CR 702.69).

use crate::r_p216_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::damage;
use crate::r_s11_common::triggered_from;
use crate::r_s17_common::become_copy;
use crate::r_s25_common::*;
use mtg_engine::ability::Duration;
use mtg_engine::casting::PlayGrant;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------
// Foretell.
// ---------------------------------------------------------------------------

#[test]
fn foretold_cards_follow_their_own_timing_rules() {
    cr!("702.143a", "304.1", "307.1");
    ruling!(
        "Surge of Brilliance",
        "Casting a foretold card from exile follows the timing rules for that card. If you foretell an instant card, you can cast it as soon as the next player's turn."
    );
    ruling!(
        "Frost Fair Lure Fish",
        "Casting a foretold card from exile follows the timing rules for that card. If you foretell an instant card, you can cast it as soon as the next player's turn. In most cases, if you foretell a card that isn't an instant (like Frost Fair Lure Fish), you'll have to wait until your next turn to cast it."
    );
    ruling!(
        "Impending Flux",
        "Casting a foretold card from exile follows the timing rules for that card. If you foretell an instant card, you can cast it as soon as the next player's turn. In most cases, if you foretell a card that isn't an instant (like Impending Flux), you'll have to wait until your next turn to cast it."
    );
    ruling!(
        "The Foretold Soldier",
        "Casting a foretold card from exile follows the timing rules for that card. If you foretell an instant card, you can cast it as soon as the next player's turn. In most cases, if you foretell a card that isn't an instant (like The Foretold Soldier), you'll have to wait until your next turn to cast it."
    );
    let names = [
        "Surge of Brilliance",
        "Frost Fair Lure Fish",
        "Impending Flux",
        "The Foretold Soldier",
    ];
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 2);
    let foretold: Vec<ObjectId> = names
        .iter()
        .map(|n| {
            supported(n);
            let c = t.hand(P0, n);
            foretell(&mut t, P0, c)
        })
        .collect();
    for c in &foretold {
        assert!(!can_cast(&mut t, P0, *c, FORETELL));
    }
    // The next player's turn: only the instant.
    t.advance_to(P1, Step::Upkeep);
    let now: Vec<bool> = foretold
        .iter()
        .map(|c| can_cast(&mut t, P0, *c, FORETELL))
        .collect();
    assert_eq!(now, vec![true, false, false, false]);
    // P0's next main phase: all of them.
    t.advance_to(P0, Step::PrecombatMain);
    for c in &foretold {
        assert!(can_cast(&mut t, P0, *c, FORETELL));
    }
}

#[test]
fn cosmos_charger_only_works_on_the_battlefield() {
    cr!("702.143a", "116.2h", "113.6");
    ruling!(
        "Cosmos Charger",
        "Cosmos Charger’s third ability applies only if it’s on the battlefield. Notably, it doesn’t affect when you can foretell Cosmos Charger itself or how much you pay to do so."
    );
    supported("Cosmos Charger");
    // Alone in hand: only during P0's turn, for {2}.
    let mut t = TestGame::new(2);
    let charger = t.hand(P0, "Cosmos Charger");
    t.set_step(P1, Step::PrecombatMain);
    add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 2);
    assert!(!can_foretell(&mut t, P0, charger));
    t.set_step(P0, Step::PrecombatMain);
    t.g.players[0].mana_pool = Default::default();
    foretell(&mut t, P0, charger);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // With another Cosmos Charger on the battlefield: during P1's turn, for {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cosmos Charger");
    let charger = t.hand(P0, "Cosmos Charger");
    t.set_step(P1, Step::PrecombatMain);
    add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 2);
    assert!(can_foretell(&mut t, P0, charger));
    t.g.players[0].mana_pool = Default::default();
    foretell(&mut t, P0, charger);
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn delayed_blast_fireball_deals_5_when_cast_from_exile_for_any_reason() {
    cr!("601.2");
    ruling!(
        "Delayed Blast Fireball",
        "Delayed Blast Fireball deals 5 damage if it was cast from exile for any reason; it doesn't have to be from its foretell ability."
    );
    supported("Delayed Blast Fireball");
    for from_exile in [false, true] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        lands_for_cost(&mut t, P0, "Delayed Blast Fireball");
        let card = if from_exile {
            // Face up in exile, with an effect's permission to cast it.
            let c = t.exile(P0, "Delayed Blast Fireball");
            t.g.play_grants.push(PlayGrant {
                player: P0,
                object: c,
                duration: Duration::EndOfTurn,
                free: false,
                source: None,
                turn: t.g.turn.number,
                terms: Default::default(),
            });
            c
        } else {
            t.hand(P0, "Delayed Blast Fireball")
        };
        t.cast(P0, card).go();
        t.resolve_all();
        let n = if from_exile { 5 } else { 2 };
        assert_eq!(t.life(P1), 20 - n);
        assert_eq!(t.on_battlefield(wurm), !from_exile);
    }
}

#[test]
fn a_copy_of_the_foretold_soldier_becomes_foretold_but_has_no_foretell_cost() {
    cr!("702.143a", "702.143d", "707.2");
    ruling!(
        "The Foretold Soldier",
        "If a card is a copy of The Foretold Soldier on the battlefield, it can be exiled by the triggered ability and will become foretold. However, it may only be cast this way if it has a foretell cost in exile."
    );
    supported("The Foretold Soldier");
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "The Foretold Soldier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    become_copy(&mut t, bears, soldier);
    let target = t.battlefield(P1, "Craw Wurm");
    for s in [soldier, bears] {
        damage(&mut t, s, 1, target);
        t.resolve_all();
    }
    let soldier = t.g.current(soldier);
    let bears = t.g.current(bears);
    for c in [soldier, bears] {
        assert_eq!(t.zone(c), Zone::Exile);
        assert!(t.obj(c).face_down);
    }
    t.lands(P0, "Forest", 4);
    t.advance_to(P0, Step::PrecombatMain);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, soldier, FORETELL));
    assert!(!can_cast(&mut t, P0, bears, FORETELL));
}

#[test]
fn starnheim_unleashed_chooses_x_in_its_foretell_cost() {
    cr!("702.143a", "107.3a", "601.2b");
    ruling!(
        "Starnheim Unleashed",
        "You choose the value of X in the foretell cost as you cast the spell. You’ll pay {2}{W} for one Angel Warrior token, {4}{W} for two, {6}{W} for three, and so on."
    );
    supported("Starnheim Unleashed");
    for x in [1usize, 2, 3] {
        let mut t = TestGame::new(2);
        let card = t.hand(P0, "Starnheim Unleashed");
        let card = foretell(&mut t, P0, card);
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Wastes", 2 * x);
        t.cast(P0, card).method(FORETELL).x(x as i64).go();
        assert_eq!(untapped_lands(&t, P0), 0);
        t.resolve_all();
        assert_eq!(with_subtype(&t, P0, "Angel").len(), x);
    }
}

// ---------------------------------------------------------------------------
// Formidable; the "party" ability words.
// ---------------------------------------------------------------------------

#[test]
fn gimlis_formidable_is_checked_again_on_resolution() {
    cr!("603.4", "701.14a");
    ruling!(
        "Gimli's Reckless Might",
        "Gimli's Reckless Might's formidable ability checks the total power of creatures you control twice: once at the appropriate time to see if the ability will trigger, and again as the ability tries to resolve. If, at that time, the total power of creatures you control is no longer 8 or greater, the ability will have no effect."
    );
    supported("Gimli's Reckless Might");
    for shrink in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Gimli's Reckless Might");
        let wurm = t.battlefield(P0, "Craw Wurm");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let elves = t.battlefield(P1, "Llanowar Elves");
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_targets(P0, &[obj(elves)]);
        attack_with(
            &mut t,
            &[(wurm, Entity::Player(P1)), (bears, Entity::Player(P1))],
        );
        assert_eq!(t.stack_len(), 1);
        if shrink {
            destroy(&mut t, wurm);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(elves), shrink);
    }
}

#[test]
fn champions_from_beyonds_party_abilities_dont_need_a_party() {
    cr!("207.2c", "508.1m");
    ruling!(
        "Champions from Beyond",
        "While the second and third abilities of Champions from Beyond have \"Party\" in their flavor words, neither of them refer to a player's party"
    );
    supported("Champions from Beyond");
    // Eight Grizzly Bears: no Cleric, Rogue, Warrior or Wizard among them.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Champions from Beyond");
    let bears: Vec<ObjectId> = (0..8).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let hand = t.hand_size(P0);
    attack_with(
        &mut t,
        &bears
            .iter()
            .map(|b| (*b, Entity::Player(P1)))
            .collect::<Vec<_>>(),
    );
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.pt(bears[0]), (6, 6));
}

// ---------------------------------------------------------------------------
// Fuse: the halves of a fused split spell resolve in order (CR 702.102d).
// ---------------------------------------------------------------------------

#[test]
fn entering_chooses_on_resolution_including_a_card_breaking_milled() {
    cr!("702.102d", "608.2c");
    ruling!(
        "Breaking // Entering",
        "Entering doesn’t target any creature card. You choose which card you’re putting onto the battlefield as Entering resolves. You can choose any creature card in a graveyard at time, including one put into a graveyard because of Breaking if you cast Breaking // Entering as a fused split spell."
    );
    supported("Breaking // Entering");
    let mut t = TestGame::new(2);
    let wurm = t.library_top(P1, "Craw Wurm");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Breaking // Entering");
    let from = t.asked().len();
    t.cast(P0, card).method(FUSED).target(P1).go();
    // Only Breaking's target was chosen.
    assert_eq!(target_candidates(&t, P0, from).len(), 1);
    t.resolve_all();
    let wurm = t.g.current(wurm);
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj(wurm).controller, P0);
}

#[test]
fn fused_halves_see_the_earlier_halfs_results() {
    cr!("702.102d", "608.2c");
    ruling!(
        "Alive // Well",
        "If you cast Alive // Well as a fused split spell, the Centaur creature token will count toward the amount of life you gain."
    );
    ruling!(
        "Beck // Call",
        "If you cast Beck // Call as a fused split spell, the triggered ability it creates will be in effect when the Bird creature tokens enter the battlefield. You’ll draw up to four cards."
    );
    ruling!(
        "Flesh // Blood",
        "If you cast Flesh // Blood as a fused split card, the +1/+1 counters will have been placed before the amount of damage is determined."
    );
    ruling!(
        "Catch // Release",
        "If you cast Catch/Release as a fused split spell, you can sacrifice the permanent you just gained control of."
    );
    // Alive // Well: a Grizzly Bears and the Centaur: 4 life.
    supported("Alive // Well");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Alive // Well");
    t.cast(P0, card).method(FUSED).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    // Beck // Call: four Birds, four cards.
    supported("Beck // Call");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Beck // Call");
    let hand = t.hand_size(P0) - 1;
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    t.cast(P0, card).method(FUSED).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Bird").len(), 4);
    assert_eq!(t.hand_size(P0), hand + 4);
    // Flesh // Blood: Craw Wurm's power 6 on a Grizzly Bears, then 8 damage.
    supported("Flesh // Blood");
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P1, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Flesh // Blood");
    t.cast(P0, card)
        .method(FUSED)
        .targets(&[obj(wurm)])
        .targets(&[obj(bears)])
        .targets(&[obj(bears)])
        .targets(&[Entity::Player(P1)])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 6);
    assert_eq!(t.life(P1), 12);
    // Catch // Release: P0's only creature is the one it just took.
    supported("Catch // Release");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 5);
    let card = t.hand(P0, "Catch // Release");
    t.cast(P0, card).method(FUSED).target(bears).go();
    t.resolve_all();
    assert!(!t.g.is_live(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn gallifrey_falls_damage_comes_before_no_mores_phasing() {
    cr!("702.102d", "702.26b", "603.2");
    ruling!(
        "Gallifrey Falls // No More",
        "If you cast both halves of this spell fused, the targeted creatures will be dealt 4 damage first and then be phased out, even if they had less than 4 toughness. Any abilities that trigger based on those creatures being dealt damage will trigger."
    );
    supported("Gallifrey Falls // No More");
    supported("Boros Reckoner");
    // Boros Reckoner (3/3): "Whenever this creature is dealt damage, it deals that much
    // damage to any target."
    let mut t = TestGame::new(2);
    let reckoner = t.battlefield(P0, "Boros Reckoner");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 6);
    let card = t.hand(P0, "Gallifrey Falls // No More");
    t.cast(P0, card)
        .method(FUSED)
        .targets(&[obj(reckoner)])
        .go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    // The Reckoner was dealt 4 damage, then phased out: it isn't destroyed or exiled.
    let r = t.g.current(reckoner);
    assert_eq!(t.zone(r), Zone::Battlefield);
    assert!(t.obj(r).phased_out);
    assert!(triggered_from(&t, reckoner) >= 1);
    // The Bears weren't phased out: exiled instead of dying.
    assert!(!t.g.is_live(bears));
    assert!(t.in_exile("Grizzly Bears"));
}

// ---------------------------------------------------------------------------
// Gift; gravestorm.
// ---------------------------------------------------------------------------

#[test]
fn wildfire_howl_with_a_promised_gift_and_an_illegal_target_does_nothing() {
    cr!("608.2b", "702.174a");
    ruling!(
        "Wildfire Howl",
        "If the gift was promised and the target is illegal as Wildfire Howl tries to resolve, it won’t resolve and none of its effects will happen. No creatures will be dealt damage."
    );
    supported("Wildfire Howl");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let hand = t.hand_size(P1);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        cast_new(&mut t, P0, "Wildfire Howl", &[obj(bears)]);
        if illegal {
            move_to(&mut t, bears, Zone::Exile);
        }
        t.resolve_all();
        let giant_damage = t.obj_now(giant).damage;
        assert_eq!(giant_damage, if illegal { 0 } else { 2 });
        // The gift (a card) wasn't given either.
        assert_eq!(t.hand_size(P1), hand + usize::from(!illegal));
    }
}

#[test]
fn gravestorm_copies_are_not_cast() {
    cr!("702.69a", "707.10");
    ruling!(
        "Follow the Bodies",
        "The copies of Follow the Bodies created by its gravestorm ability are created on the stack, so they're not cast and won't cause abilities that trigger when a player casts a spell to trigger."
    );
    ruling!(
        "Ominous Harvest",
        "The copies of Ominous Harvest created by its gravestorm ability are created on the stack, so they're not cast and won't cause abilities that trigger when a player casts a spell to trigger."
    );
    supported("Monastery Swiftspear");
    for name in ["Follow the Bodies", "Ominous Harvest"] {
        supported(name);
        let mut t = TestGame::new(2);
        let spear = t.battlefield(P0, "Monastery Swiftspear");
        // Two permanents were put into graveyards this turn: two copies.
        for _ in 0..2 {
            let b = t.battlefield(P1, "Grizzly Bears");
            destroy(&mut t, b);
        }
        let targets = if name == "Ominous Harvest" {
            for _ in 0..2 {
                t.answer_targets(P0, &[Entity::Player(P1)]);
            }
            vec![Entity::Player(P1)]
        } else {
            vec![]
        };
        cast_new(&mut t, P0, name, &targets);
        t.resolve_all();
        // Prowess triggered once, for the spell that was cast.
        assert_eq!(t.pt(spear), (2, 3), "{name}");
        if name == "Ominous Harvest" {
            assert_eq!(t.life(P1), 17);
        } else {
            assert_eq!(with_subtype(&t, P0, "Clue").len(), 3);
        }
    }
}
