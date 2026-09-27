//! Rulings batch S02 — choose a Background (CR 702.124k): a legendary creature with
//! "Choose a Background" and a legendary Background enchantment can be two commanders.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use mtg_engine::ability::*;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn commander_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    )
}

/// Puts the real card `name` into `p`'s command zone as one of their commanders.
fn commander(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.command(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.players[p.idx()].commander_names.push(name.into());
    id
}

/// Makes a permanent on the battlefield one of its owner's commanders.
fn make_commander(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let owner = t.obj(id).owner;
    let name = t.obj(id).chars.name.clone();
    t.g.players[owner.idx()].commander_names.push(name);
}

fn untapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land) && !o.tapped)
        .count()
}

#[test]
fn a_creature_commander_and_its_background_each_have_their_own_commander_tax() {
    cr!("702.124k", "702.124d", "903.8");
    ruling!(
        "Ganax, Astral Hunter",
        "Once the game begins, your two commanders are tracked separately. If you cast one, you won't have to pay an additional {2} the first time you cast the other."
    );
    supported("Ganax, Astral Hunter");
    supported("Raised by Giants");
    let mut t = commander_game();
    let ganax = commander(&mut t, P0, "Ganax, Astral Hunter");
    let giants = commander(&mut t, P0, "Raised by Giants");
    // Ganax ({4}{R}) is cast from the command zone, dies, and returns there.
    t.lands(P0, "Mountain", 5);
    t.cast(P0, ganax).go();
    t.resolve_all();
    assert!(t.on_battlefield(ganax));
    t.answer_yes(P0, true);
    destroy(&mut t, ganax);
    assert_eq!(t.zone(ganax), Zone::Command);
    // Raised by Giants ({5}{G}) costs no more for Ganax having been cast.
    t.lands(P0, "Forest", 6);
    t.cast(P0, giants).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(t.on_battlefield(giants));
    // Ganax costs {2} more the second time.
    let ganax = t.g.current(ganax);
    t.lands(P0, "Mountain", 5);
    assert!(!can_cast(&mut t, P0, ganax, CastMethod::Normal));
    t.lands(P0, "Wastes", 2);
    assert!(can_cast(&mut t, P0, ganax, CastMethod::Normal));
    t.cast(P0, ganax).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn combat_damage_from_a_creature_commander_and_its_background_is_counted_separately() {
    cr!("702.124k", "702.124d", "903.10a");
    ruling!(
        "Ganax, Astral Hunter",
        "A player loses the game after having been dealt 21 combat damage from any one of them, not from both of them combined (although your Background won't usually be a creature anyway)."
    );
    let mut t = commander_game();
    t.g.players[1].life = 40;
    let ganax = t.battlefield(P0, "Ganax, Astral Hunter");
    make_commander(&mut t, ganax);
    // Raised by Giants: "Commander creatures you own have base power and toughness 10/10
    // and are Giants in addition to their other types."
    let giants = t.battlefield(P0, "Raised by Giants");
    make_commander(&mut t, giants);
    assert_eq!(t.pt(ganax), (10, 10));
    // Ganax has already dealt P1 11 combat damage.
    t.g.players[1]
        .commander_damage
        .insert("Ganax, Astral Hunter".into(), 11);
    // An effect makes the Background a creature until end of turn: a 10/10 commander
    // creature.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(giants)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddTypes(vec![CardType::Creature])],
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert!(t.obj(giants).is_creature());
    assert_eq!(t.pt(giants), (10, 10));
    // It deals 10: 21 combat damage from P0's commanders combined, but not from one.
    attack_with(&mut t, &[(giants, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    assert_eq!(t.life(P1), 30);
    assert_eq!(
        t.player(P1).commander_damage.get("Raised by Giants"),
        Some(&10)
    );
    assert!(!t.has_lost(P1));
    // Next turn, Ganax deals 10 more: 21 from Ganax alone.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(ganax, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    assert!(t.g.run_until(10_000, |g| g.result.is_some()));
    assert!(t.has_lost(P1));
    assert!(t.life(P1) > 0);
}

#[test]
fn your_commander_is_the_one_of_your_two_commanders_you_choose() {
    cr!("702.124k", "702.124e");
    ruling!(
        "Amber Gristle O'Maul",
        "If something refers to your commander while you have two commanders, it refers to one of them of your choice. If you are instructed to perform an action on your commander (e.g. put it from the command zone into your hand due to Command Beacon), you choose one of your commanders at the time the effect happens."
    );
    supported("Amber Gristle O'Maul");
    supported("Command Beacon");
    for pick_background in [true, false] {
        let mut t = commander_game();
        let amber = commander(&mut t, P0, "Amber Gristle O'Maul");
        let giants = commander(&mut t, P0, "Raised by Giants");
        // Command Beacon: "{T}, Sacrifice this land: Put your commander into your hand
        // from the command zone."
        let beacon = t.battlefield(P0, "Command Beacon");
        let (chosen, other) = if pick_background {
            (giants, amber)
        } else {
            (amber, giants)
        };
        t.answer_choose(P0, &[Entity::Object(chosen)]);
        // Its owner doesn't put it back into the command zone instead (CR 903.9b).
        t.answer_yes(P0, false);
        t.activate(P0, beacon, 1, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.zone(chosen), Zone::Hand(P0));
        assert_eq!(t.zone(other), Zone::Command);
    }
}

#[test]
fn amber_gristle_draws_a_card_for_each_player_being_attacked() {
    cr!("508.1b", "802.3");
    let mut t = TestGame::new(4);
    // Amber Gristle O'Maul (3/3 haste): "Whenever Amber Gristle O'Maul attacks, you may
    // discard your hand. If you do, draw a card for each player being attacked."
    let amber = t.battlefield(P0, "Amber Gristle O'Maul");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let jace = t.battlefield(P3, "Jace Beleren");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    // P1 and P2 are attacked; P3 isn't (only its planeswalker is).
    t.answer_yes(P0, true);
    attack_with(
        &mut t,
        &[
            (amber, Entity::Player(P1)),
            (bears, Entity::Player(P2)),
            (giant, Entity::Object(jace)),
        ],
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), 3);
}
