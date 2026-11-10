//! Rulings batch P049 — the order things happen in: instructions performed in the order
//! written (Cruel Ultimatum, Vicious Rumors), choices made in APNAP order with earlier
//! choices known and the actions then taken at once (Social Snub, Professor Onyx, Kaya,
//! Ghost Assassin, CR 101.4), and Creeping Inn's phasing (CR 702.26: phasing in during
//! the untap step of the player who controlled it as it phased out, removal from combat,
//! control-change effects that expire while it's phased out).

use crate::r_p049_common::*;
use crate::r_p053_common::concede;
use crate::r_p212_common::back_face;
use crate::r_s02_common::can_attack;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn is_choose(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { .. })
}

#[test]
fn cruel_ultimatum_acts_in_order_and_chooses_the_creature_card_on_resolution() {
    cr!("608.2c", "608.2d", "115.1", "400.3");
    ruling!(
        "Cruel Ultimatum",
        "All of the actions for Cruel Ultimatum are performed sequentially, in the order listed. Earlier actions may affect how you perform later actions. For example, if the opponent sacrifices a creature that they control but you own, it will end up in your graveyard. When Cruel Ultimatum lets you return a creature card from your graveyard to your hand, you can choose that one."
    );
    ruling!(
        "Cruel Ultimatum",
        "Cruel Ultimatum's only target is an opponent. You don't choose which creature card in your graveyard you'll return to your hand until Cruel Ultimatum resolves."
    );
    supported("Cruel Ultimatum");
    let mut t = TestGame::new(2);
    // P1 controls P0's Grizzly Bears, its only creature.
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    for _ in 0..3 {
        t.hand(P1, "Hill Giant");
    }
    t.graveyard(P0, "Serra Angel");
    let from = t.asked().len();
    // The Bears, in P0's graveyard by then, is chosen as the spell resolves.
    t.answer_choose_moved(P0, &[obj(bears)]);
    let spell = cast_new(&mut t, P0, "Cruel Ultimatum", &[Entity::Player(P1)]);
    // Casting asked for one target, a player; no card was chosen yet.
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].iter().all(|e| matches!(e, Entity::Player(_))));
    assert!(!t.asked()[from..].iter().any(|(_, d)| is_choose(d)));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    assert!(t.in_graveyard(P0, "Serra Angel"));
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.hand_size(P0), hand + 1 + 3);
    assert_eq!((t.life(P0), t.life(P1)), (25, 15));
}

#[test]
fn vicious_rumors_damages_then_each_opponent_chooses_in_turn_order_then_discards_mills_and_you_gain() {
    cr!("608.2c", "101.4", "701.9b");
    ruling!(
        "Vicious Rumors",
        "As Vicious Rumors resolves, it deals 1 damage to each opponent, then the next opponent in turn order (or, if it’s an opponent’s turn, that opponent) chooses a card in hand without revealing it, then each other opponent in turn order does the same. Then the chosen cards are discarded at the same time. Next, each opponent puts the top card of their library into their graveyard at the same time. Finally, you gain 1 life."
    );
    supported("Vicious Rumors");
    let mut t = TestGame::new(3);
    let c1 = t.hand(P1, "Grizzly Bears");
    let c2 = t.hand(P2, "Hill Giant");
    // What each opponent sees as they choose: both already damaged, nobody has
    // discarded or milled yet, P0 hasn't gained life yet; P2 sees P1 has chosen.
    let seen1 = watch(&mut t, P1, is_choose, |g| {
        (
            g.player(P1).life,
            g.player(P2).life,
            g.player(P1).hand.len(),
            g.player(P2).hand.len(),
            g.player(P1).graveyard.len(),
            g.player(P0).life,
        )
    });
    let seen2 = watch(&mut t, P2, is_choose, |g| {
        (
            g.player(P1).hand.len(),
            g.player(P1).graveyard.len(),
            g.known_apnap_choices(P2).len(),
        )
    });
    t.answer_choose(P1, &[obj(c1)]);
    t.answer_choose(P2, &[obj(c2)]);
    let lib = (t.library_size(P1), t.library_size(P2));
    cast_new(&mut t, P0, "Vicious Rumors", &[]);
    t.resolve_all();
    assert_eq!(seen1.lock().unwrap().first(), Some(&(19, 19, 1, 1, 0, 20)));
    assert_eq!(seen2.lock().unwrap().first(), Some(&(1, 0, 1)));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P2, "Hill Giant"));
    assert_eq!((t.library_size(P1), t.library_size(P2)), (lib.0 - 1, lib.1 - 1));
    assert_eq!((t.graveyard_size(P1), t.graveyard_size(P2)), (2, 2));
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (21, 19, 19));
}

#[test]
fn kayas_discards_are_chosen_in_turn_then_made_at_the_same_time() {
    cr!("101.4", "701.9b", "608.2c");
    ruling!(
        "Kaya, Ghost Assassin",
        "As the last ability resolves, each opponent will choose a card to discard and set it aside. Then all those cards are revealed and discarded simultaneously."
    );
    supported("Kaya, Ghost Assassin");
    let mut t = TestGame::new(3);
    let kaya = t.battlefield(P0, "Kaya, Ghost Assassin");
    let c1 = t.hand(P1, "Grizzly Bears");
    let c2 = t.hand(P2, "Hill Giant");
    // As P2 chooses, P1's card is still in P1's hand.
    let seen = watch(&mut t, P2, is_choose, |g| {
        (g.player(P1).hand.len(), g.player(P1).graveyard.len())
    });
    t.answer_choose(P1, &[obj(c1)]);
    t.answer_choose(P2, &[obj(c2)]);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, kaya, "discards a card").unwrap();
    t.resolve_all();
    assert_eq!(seen.lock().unwrap().first(), Some(&(1, 0)));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P2, "Hill Giant"));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(kaya, counters::LOYALTY), 3);
}

#[test]
fn social_snubs_sacrifices_are_chosen_in_apnap_order_and_made_at_once() {
    cr!("101.4", "101.4b", "701.21a");
    ruling!(
        "Social Snub",
        "First the player whose turn it is chooses which creature they're going to sacrifice, then each other player in turn order does the same. Players will know the choices of previous players when making their choices. Then all chosen creatures are sacrificed by their controllers simultaneously."
    );
    supported("Social Snub");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Grizzly Bears");
    let a2 = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P2, "Grizzly Bears");
    // As P2 chooses, P0 and P1 have chosen (and P2 knows what) but nothing has been
    // sacrificed.
    let seen = watch(&mut t, P2, is_choose, |g| {
        (
            g.find_in_zone(Zone::Battlefield, "Grizzly Bears").len(),
            g.known_apnap_choices(P2)
                .iter()
                .map(|(p, _)| *p)
                .collect::<Vec<_>>(),
        )
    });
    let seen1 = watch(&mut t, P1, is_choose, |g| g.known_apnap_choices(P1).len());
    // P0 declines to copy the spell.
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P1, &[obj(b)]);
    t.answer_choose(P2, &[obj(c)]);
    cast_new(&mut t, P0, "Social Snub", &[]);
    t.resolve_all();
    assert_eq!(seen.lock().unwrap().first(), Some(&(3, vec![P0, P1])));
    assert_eq!(seen1.lock().unwrap().first(), Some(&1));
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b) && !t.on_battlefield(c));
    assert!(t.on_battlefield(a2));
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (21, 19, 19));
}

#[test]
fn professor_onyxs_sacrifices_let_each_opponent_choose_among_their_greatest_power() {
    cr!("101.4", "101.4b", "608.2c");
    ruling!(
        "Professor Onyx",
        "For Professor Onyx's second loyalty ability, if an opponent has multiple creatures with the greatest power, that player choose which one to sacrifice. The next opponent in turn order chooses which creature they are sacrificing, then each other opponent in turn order does the same, knowing the choices made before them. Then all chosen creatures are sacrificed at the same time."
    );
    supported("Professor Onyx");
    let mut t = TestGame::new(3);
    let onyx = t.battlefield(P0, "Professor Onyx");
    let g1 = t.battlefield(P1, "Hill Giant");
    let g2 = t.battlefield(P1, "Hill Giant");
    let small = t.battlefield(P1, "Grizzly Bears");
    let g3 = t.battlefield(P2, "Hill Giant");
    let seen = watch(&mut t, P2, is_choose, |g| {
        (
            g.find_in_zone(Zone::Battlefield, "Hill Giant").len(),
            g.known_apnap_choices(P2)
                .iter()
                .map(|(p, _)| *p)
                .collect::<Vec<_>>(),
        )
    });
    let from = t.asked().len();
    t.answer_choose(P1, &[obj(g2)]);
    t.answer_choose(P2, &[obj(g3)]);
    activate_containing(&mut t, P0, onyx, "greatest power").unwrap();
    t.resolve_all();
    // P1 chose among the two Giants only, before P2.
    let choices: Vec<(PlayerId, Vec<Entity>)> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some((*p, candidates.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].0, P1);
    assert_eq!(choices[1].0, P2);
    assert!(choices[0].1.contains(&obj(g1)) && choices[0].1.contains(&obj(g2)));
    assert!(!choices[0].1.contains(&obj(small)));
    assert_eq!(seen.lock().unwrap().first(), Some(&(3, vec![P1])));
    assert!(t.on_battlefield(g1) && !t.on_battlefield(g2) && !t.on_battlefield(g3));
    assert!(t.on_battlefield(small));
}

/// P0's Creeping Inn (Hostile Hostel's back face) on the battlefield, with four lands for
/// its "{4}: This creature phases out."
fn creeping_inn(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Hostile Hostel // Creeping Inn");
    t.lands(p, "Wastes", 4);
    back_face(t, p, "Hostile Hostel // Creeping Inn")
}

/// `p` activates the Inn's phasing ability and it resolves.
fn phase_out(t: &mut TestGame, p: PlayerId, inn: ObjectId) {
    activate_containing(t, p, inn, "phases out").unwrap();
    t.resolve_all();
    assert!(t.obj_now(inn).phased_out);
}

#[test]
fn creeping_inn_phases_in_during_its_controllers_untap_step_and_can_attack() {
    cr!("702.26a", "702.26d", "302.6");
    ruling!(
        "Hostile Hostel // Creeping Inn",
        "Creeping Inn will phase in during its controller's untap step. It will be able to attack the same turn."
    );
    let mut t = TestGame::new(2);
    let inn = creeping_inn(&mut t, P0);
    phase_out(&mut t, P0, inn);
    // Not during P1's untap step...
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(inn).phased_out);
    // ...but during P0's; it has been under P0's control continuously, so it can attack.
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!t.obj_now(inn).phased_out);
    assert!(can_attack(&mut t, inn));
}

#[test]
fn creeping_inn_phasing_out_while_attacking_is_removed_from_combat() {
    cr!("702.26b", "506.4");
    ruling!(
        "Hostile Hostel // Creeping Inn",
        "If Creeping Inn is attacking or blocking when it phases out, it is removed from combat."
    );
    let mut t = TestGame::new(2);
    let inn = creeping_inn(&mut t, P0);
    // It attacks (declining to exile a card for its trigger), then phases out in the
    // declare attackers step: no combat damage.
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(inn, Entity::Player(P1))]);
    t.resolve_all();
    phase_out(&mut t, P0, inn);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.obj_now(inn).phased_out);
}

#[test]
fn creeping_inn_stolen_and_phased_out_phases_in_under_its_owners_control_in_the_thiefs_untap_step() {
    cr!("702.26a", "702.26f", "702.26n");
    ruling!(
        "Hostile Hostel // Creeping Inn",
        "If an opponent gains control of Creeping Inn and it phases out and the duration of the control-change effect expires before it phases in, it phases in under your control as that opponent's appropriate untap step begins. If they leave the game before that untap step, it phases in as the next untap step begins after that turn would have begun."
    );
    supported("Act of Treason");
    let mut t = TestGame::new(2);
    let inn = creeping_inn(&mut t, P0);
    // P1's turn: P1 takes the Inn until end of turn and phases it out.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 4);
    cast_new(&mut t, P1, "Act of Treason", &[obj(inn)]);
    t.resolve_all();
    assert_eq!(t.obj_now(inn).controller, P1);
    phase_out(&mut t, P1, inn);
    // P0's untap step: it phased out under P1's control, so it stays phased out.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(inn).phased_out);
    // P1's untap step: it phases in, under P0's control (the control effect expired).
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(inn).phased_out);
    assert_eq!(t.obj_now(inn).controller, P0);
    // Three players: P1 steals it and phases it out, then leaves the game. P1's next turn
    // would have begun after P0's; the Inn phases in as the untap step after that (P2's)
    // begins.
    let mut t = TestGame::new(3);
    let inn = creeping_inn(&mut t, P0);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 4);
    cast_new(&mut t, P1, "Act of Treason", &[obj(inn)]);
    t.resolve_all();
    phase_out(&mut t, P1, inn);
    concede(&mut t, P1);
    assert!(t.has_lost(P1));
    t.advance_to(P2, Step::Upkeep);
    assert!(t.obj_now(inn).phased_out, "P2's untap step after P1 left");
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(inn).phased_out, "P0's untap step");
    t.advance_to(P2, Step::Upkeep);
    assert!(!t.obj_now(inn).phased_out, "the untap step after P1's turn would have begun");
    assert_eq!(t.obj_now(inn).controller, P0);
}
