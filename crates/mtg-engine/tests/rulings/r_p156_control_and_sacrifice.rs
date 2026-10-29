//! Rulings batch P156 — control changes (attachments stay with their controller, choosing
//! an opponent on resolution, control reverting when a player leaves, a targeted mana
//! ability) and spells whose additional cost sacrifices a creature whose power they use
//! (last known information, sacrificing the target).

use crate::r_s01_common::*;
use crate::r_s05_common::{enter, move_to};
use crate::r_s25_common::{cast_new, lands_for_cost, targets_of};
use crate::r_s29_common::put_counters;
use mtg_engine::ability::AbilityKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn controller(t: &TestGame, id: ObjectId) -> PlayerId {
    t.obj_now(id).controller
}

/// Attaches the attachment `what` to `to` (as an equip or enchant effect would).
fn attach(t: &mut TestGame, what: ObjectId, to: ObjectId) {
    t.g.attach(what, Entity::Object(to));
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

#[test]
fn wrong_turn_and_sky_swallower_dont_move_attachments() {
    cr!("701.3a", "303.4e", "301.5d", "613.1b");
    ruling!(
        "Wrong Turn",
        "A player gaining control of a creature doesn't cause that player to gain control of any Auras or Equipment attached to that creature."
    );
    ruling!(
        "Sky Swallower",
        "Auras your opponent gains control of won't move from where they are."
    );
    supported("Wrong Turn");
    supported("Sky Swallower");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Holy Strength");
    let sword = t.battlefield(P0, "Short Sword");
    attach(&mut t, aura, bears);
    attach(&mut t, sword, bears);
    cast_new(
        &mut t,
        P0,
        "Wrong Turn",
        &[Entity::Player(P1), obj(bears)],
    );
    t.resolve_all();
    assert_eq!(controller(&t, bears), P1);
    assert_eq!(controller(&t, aura), P0);
    assert_eq!(controller(&t, sword), P0);
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.obj_now(sword).attached_to, Some(Entity::Object(bears)));
    // Sky Swallower: P0's Aura on P1's creature goes to P1 and stays on that creature.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Holy Strength");
    attach(&mut t, aura, theirs);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let sky = enter(&mut t, P0, "Sky Swallower");
    t.resolve_all();
    assert_eq!(controller(&t, aura), P1);
    assert_eq!(controller(&t, mine), P1);
    assert_eq!(controller(&t, sky), P0);
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(theirs)));
    assert_eq!(t.pt(theirs), (4, 5));
}

#[test]
fn akroan_horse_chooses_the_opponent_as_it_resolves() {
    cr!("603.3d", "608.2b", "115.1");
    ruling!(
        "Akroan Horse",
        "Akroan Horse's enters-the-battlefield ability doesn't target any opponent. In a multiplayer game, you choose the opponent as the ability resolves."
    );
    supported("Akroan Horse");
    let mut t = TestGame::new(3);
    let horse = enter(&mut t, P0, "Akroan Horse");
    let trig = *t.g.stack.last().expect("trigger");
    assert!(targets_of(&t, trig).is_empty());
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.resolve();
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. })));
    assert_eq!(controller(&t, horse), P2);
}

#[test]
fn copied_tokens_enters_abilities_work() {
    cr!("707.2", "603.6a", "614.1c");
    ruling!(
        "Hazel of the Rootbloom",
        "Any \"enters\" abilities of the copied token will trigger when the token enters. Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the copied token will also work."
    );
    supported("Hazel of the Rootbloom");
    supported("Elvish Visionary");
    supported("Spike Feeder");
    for (name, counters, draws) in [("Elvish Visionary", 0, 1), ("Spike Feeder", 2, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Hazel of the Rootbloom");
        let card = t.exile(P0, name);
        let token = crate::r_s17_common::token_copy(&mut t, P0, card)[0];
        t.resolve_all();
        t.answer_targets(P0, &[obj(token)]);
        let hand = t.hand_size(P0);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let copies: Vec<ObjectId> = t
            .named_on_battlefield(name)
            .into_iter()
            .filter(|id| *id != t.g.current(token))
            .collect();
        assert_eq!(copies.len(), 1, "{name}");
        assert_eq!(t.counters(copies[0], "+1/+1"), counters, "{name}");
        assert_eq!(t.hand_size(P0), hand + draws, "{name}");
    }
}

#[test]
fn witch_engines_targeted_mana_ability_uses_the_stack() {
    cr!("605.1a", "605.3b");
    ruling!(
        "Witch Engine",
        "Because the ability is targeted, it is not a mana ability. It can only be activated when you have priority, and goes on the stack like other non-mana abilities."
    );
    supported("Witch Engine");
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Witch Engine");
    t.g.recompute();
    let ab = t
        .obj_now(engine)
        .chars
        .abilities
        .iter()
        .find_map(|a| match &a.kind {
            AbilityKind::Activated(x) => Some(x.is_mana_ability),
            _ => None,
        })
        .unwrap();
    assert!(!ab);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, engine, 0, &[]).expect("activate");
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.total(), 4);
    assert_eq!(controller(&t, engine), P1);
}

#[test]
fn eon_frolicker_opponents_creatures_can_attack_but_deal_no_damage() {
    cr!("702.16e", "508.1");
    ruling!(
        "Eon Frolicker",
        "Creatures that player controls can still attack you or planeswalkers you control, but the damage they would deal will be prevented."
    );
    supported("Eon Frolicker");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Eon Frolicker", &[]);
    t.resolve_all();
    // P1's (extra) turn: the Bears attack P0; the damage is prevented.
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    assert!(crate::r_s10_common::attacking(&t, bears));
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
    // Positive control: without Eon Frolicker, the attack deals 2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn humble_defector_can_be_activated_any_time_during_your_turn() {
    cr!("602.5b", "117.1b");
    ruling!(
        "Humble Defector",
        "Humble Defector's ability can be activated any time during your turn, including in response to a spell or ability."
    );
    supported("Humble Defector");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Humble Defector");
    cast_new(&mut t, P0, "Dark Ritual", &[]);
    assert_eq!(t.stack_len(), 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, d, 0, &[]).expect("in response, own turn");
    assert_eq!(t.stack_len(), 2);
    // During the opponent's turn it can't be activated.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Humble Defector");
    t.set_step(P1, Step::PrecombatMain);
    assert!(!crate::r_s27_common::can_activate_containing(
        &mut t, P0, d, "Draw"
    ));
}

#[test]
fn humble_defector_draws_even_if_it_left_and_reverts_when_a_controller_leaves() {
    cr!("608.2b", "800.4a", "800.4b");
    ruling!(
        "Humble Defector",
        "If Humble Defector isn't on the battlefield as its ability resolves, but the target player is still a legal target, the ability will resolve. You'll draw two cards, even though the player doesn't gain control of Humble Defector."
    );
    ruling!(
        "Humble Defector",
        "If Humble Defector is controlled by a player other than its owner, and its controller leaves the game, the effect giving that player control of Humble Defector ends. Humble Defector will return to the control of the player still in the game who most recently controlled it."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Humble Defector");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, d, 0, &[]).expect("activate");
    move_to(&mut t, d, Zone::Graveyard(P0));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.zone(d), Zone::Graveyard(P0));
    assert!(t.named_on_battlefield("Humble Defector").is_empty());
    // P0 owns it, gives it to P2 with its ability, then P1 takes it; P1 leaves: back to
    // P2, the player still in the game who most recently controlled it.
    let mut t = TestGame::new(3);
    let d = t.battlefield(P0, "Humble Defector");
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.activate(P0, d, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(controller(&t, d), P2);
    crate::r_p147_common::take_control(&mut t, P1, d);
    assert_eq!(controller(&t, d), P1);
    crate::r_p079_common::concede(&mut t, P1);
    t.g.recompute();
    assert!(t.on_battlefield(d));
    assert_eq!(controller(&t, d), P2);
}

#[test]
fn hungry_lynx_counts_itself_as_a_cat() {
    cr!("603.2", "122.1");
    ruling!(
        "Hungry Lynx",
        "Hungry Lynx’s last ability puts a +1/+1 counter on each Cat you control, including itself."
    );
    supported("Hungry Lynx");
    let mut t = TestGame::new(2);
    let lynx = t.battlefield(P0, "Hungry Lynx");
    let rat = crate::r_s02_common::create_token(&mut t, P1, "Rat");
    move_to(&mut t, rat, Zone::Graveyard(P1));
    t.resolve_all();
    assert_eq!(t.counters(lynx, "+1/+1"), 1);
}

#[test]
fn goblin_guide_attacking_a_planeswalker_its_controller_reveals() {
    cr!("506.2", "508.5");
    ruling!(
        "Goblin Guide",
        "If Goblin Guide is attacking a planeswalker, that planeswalker's controller is the defending player."
    );
    supported("Goblin Guide");
    let mut t = TestGame::new(3);
    let guide = t.battlefield(P0, "Goblin Guide");
    let pw = t.battlefield(P2, "Jace Beleren");
    t.library_top(P1, "Forest");
    let forest = t.library_top(P2, "Forest");
    t.attack(&[(guide, obj(pw))], &[]);
    assert_eq!(t.zone(forest), Zone::Hand(P2));
    assert!(!t.in_hand(P1, "Forest"));
}

#[test]
fn sacrificed_creatures_power_uses_last_known_information() {
    cr!("608.2h", "118.8", "601.2h");
    ruling!(
        "Call for Blood",
        "Uses last known information for the sacrificed creature’s power."
    );
    ruling!(
        "Ichor Explosion",
        "X is the power of the sacrificed creature as it last existed on the battlefield."
    );
    supported("Call for Blood");
    supported("Ichor Explosion");
    // Grizzly Bears with three +1/+1 counters: power 5 as it last existed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 3);
    let wurm = t.battlefield(P1, "Craw Wurm"); // 6/4
    t.answer_choose(P0, &[obj(bears)]);
    cast_new(&mut t, P0, "Call for Blood", &[obj(wurm)]);
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.zone(wurm), Zone::Graveyard(P1));
    // Ichor Explosion: all creatures get -5/-5.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(bears)]);
    cast_new(&mut t, P0, "Ichor Explosion", &[]);
    t.resolve_all();
    assert_eq!(t.zone(wurm), Zone::Graveyard(P1));
    assert_eq!(t.zone(giant), Zone::Graveyard(P1));
    // Control: an unpumped Bears (power 2) leaves the Craw Wurm alive.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_choose(P0, &[obj(bears)]);
    cast_new(&mut t, P0, "Ichor Explosion", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.pt(wurm), (4, 2));
}

#[test]
fn call_for_blood_can_sacrifice_its_own_target() {
    cr!("601.2c", "601.2h", "608.2b");
    ruling!(
        "Call for Blood",
        "You may sacrifice the creature chosen as the target for Call for Blood (if you control it) when you pay the additional cost for the spell. If you do, Call for Blood doesn’t resolve."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(bears)]);
    let spell = cast_new(&mut t, P0, "Call for Blood", &[obj(bears)]);
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(targets_of(&t, spell), vec![obj(bears)]);
    t.resolve_all();
    // It doesn't resolve: it's put into the graveyard, the Giant untouched.
    assert!(t.in_graveyard(P0, "Call for Blood"));
    assert_eq!(t.pt(other), (3, 3));
}

#[test]
fn tormented_thoughts_sacrifices_exactly_one_creature() {
    cr!("601.2b", "601.2h", "118.8");
    ruling!(
        "Tormented Thoughts",
        "You must sacrifice exactly one creature to cast Tormented Thoughts. You can't cast it without sacrificing a creature, and you can't sacrifice additional creatures."
    );
    supported("Tormented Thoughts");
    // No creature: it can't be cast.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Tormented Thoughts");
    let card = t.hand(P0, "Tormented Thoughts");
    assert!(t.cast_with(P0, card, &[Entity::Player(P1)]).is_err());
    // Two creatures offered: only one is sacrificed; P1 discards its power (3).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..4 {
        t.hand(P1, "Island");
    }
    t.answer_choose(P0, &[obj(giant), obj(bears)]);
    cast_new(&mut t, P0, "Tormented Thoughts", &[Entity::Player(P1)]);
    let gone = [giant, bears]
        .iter()
        .filter(|id| !t.on_battlefield(**id))
        .count();
    assert_eq!(gone, 1);
    t.resolve_all();
    let power = if t.on_battlefield(bears) { 3 } else { 2 };
    assert_eq!(t.hand_size(P1), 4 - power);
}
