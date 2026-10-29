//! Rulings batch P098 — the rest: {Q} and unattach costs (Order of Whiteclay, Sunforger),
//! "whenever you cast a historic spell" (lands aren't cast; the trigger resolves first),
//! shield counters (Brotherhood Outcast), Bilbo's evasion, Primary Research's end step
//! check, Battle of Hoover Dam copied without a choice, Izzet Guildmage, Tezzeret's
//! emblem, Ral Zarek's discard order, modal Commands, Ranger-Captain of Eos, Starting
//! Town and Lurrus as a companion.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s12_common::pregame;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::{castable, legal_blocks};
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn untap_and_unattach_costs_need_a_tapped_or_attached_source() {
    cr!("107.6", "602.2b", "701.3d");
    ruling!(
        "Order of Whiteclay",
        "If the permanent is already untapped, you can't activate its {Q} ability. That's because you can't pay the “Untap this permanent” cost."
    );
    ruling!(
        "Sunforger",
        "You can't pay the cost of unattaching Sunforger unless Sunforger is attached to a creature."
    );
    supported("Order of Whiteclay");
    supported("Sunforger");
    let mut t = TestGame::new(2);
    let order = t.battlefield(P0, "Order of Whiteclay");
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    assert!(!can_activate(&mut t, P0, order));
    t.g.tap(order);
    assert!(can_activate(&mut t, P0, order));
    let mut t = TestGame::new(2);
    let forger = t.battlefield(P0, "Sunforger");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.library_top(P0, "Lightning Bolt");
    let has = |t: &mut TestGame, src| {
        t.g.turn.priority = Some(P0);
        t.g.recompute();
        t.g.legal_actions(P0).iter().any(|a| {
            matches!(a, mtg_engine::decision::Action::Activate { source, .. } if *source == src)
        })
    };
    // Unattached: only equip ({3}, unaffordable) remains; the {R}{W} ability can't be.
    assert!(!has(&mut t, forger));
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(forger, obj(bears)));
    assert!(has(&mut t, forger));
}

#[test]
fn historic_cast_triggers_ignore_lands_and_resolve_before_the_spell() {
    cr!("700.6", "305.1", "603.3");
    ruling!(
        "Lingering Phantom",
        "Lands are never cast, so abilities that trigger \"whenever you cast a historic spell\" won't trigger if you play a legendary land. They also won't trigger if a card on the battlefield transforms into a legendary land, as the Ixalan and Rivals of Ixalan double-faced cards do."
    );
    ruling!(
        "Teshar, Ancestor's Apostle",
        "Lands are never cast, so abilities that trigger \"whenever you cast a historic spell\" won't trigger if you play a legendary land."
    );
    ruling!(
        "Traxos, Scourge of Kroog",
        "Lands are never cast, so abilities that trigger \"whenever you cast a historic spell\" won't trigger if you play a legendary land."
    );
    ruling!(
        "Elena, Turk Recruit",
        "Some abilities trigger \"whenever you cast a historic spell.\" Such an ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Glóin, Dwarf Emissary",
        "Some abilities trigger \"whenever you cast a historic spell.\" Such an ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    for name in [
        "Teshar, Ancestor's Apostle",
        "Traxos, Scourge of Kroog",
        "Elena, Turk Recruit",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.graveyard(P0, "Grizzly Bears");
        let haven = t.hand(P0, "Kor Haven");
        t.play_land(P0, haven).unwrap();
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 0, "{name}");
        // Search for Azcanta transforming into Azcanta, the Sunken Ruin (legendary land).
        let search = t.battlefield(P0, "Search for Azcanta // Azcanta, the Sunken Ruin");
        assert!(mtg_engine::dfc::transform(&mut t.g, search));
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 0, "{name}");
    }
    // Elena and Glóin: the trigger resolves even though Sol Ring is countered.
    supported("Glóin, Dwarf Emissary");
    let mut t = TestGame::new(2);
    let elena = t.battlefield(P0, "Elena, Turk Recruit");
    t.battlefield(P0, "Glóin, Dwarf Emissary");
    let ring = t.hand(P0, "Sol Ring");
    t.lands(P0, "Wastes", 1);
    let spell = t.cast(P0, ring).go();
    t.settle();
    assert_eq!(t.stack_len(), 3);
    let counter = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sol Ring"));
    assert_eq!(t.counters(elena, counters::PLUS1), 1);
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn shield_counters_arent_regeneration_and_dont_stop_sacrifice() {
    cr!("122.1c", "701.19c", "701.21a");
    ruling!(
        "Brotherhood Outcast",
        "Removing a shield counter when a permanent would be dealt damage or destroyed isn’t the same as regenerating a creature."
    );
    ruling!(
        "Brotherhood Outcast",
        "Shield counters don’t prevent players from sacrificing creatures."
    );
    supported("Brotherhood Outcast");
    supported("Wrath of God");
    let shielded = |t: &mut TestGame| {
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
        t.answer_targets(P0, &[obj(bears)]);
        t.enter(P0, "Brotherhood Outcast");
        t.resolve_all();
        assert_eq!(t.counters(bears, counters::SHIELD), 1);
        bears
    };
    // Wrath of God ("They can't be regenerated."): the shield counter still saves it.
    let mut t = TestGame::new(2);
    let bears = shielded(&mut t);
    let wrath = t.hand(P0, "Wrath of God");
    t.lands(P0, "Plains", 4);
    t.cast(P0, wrath).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, counters::SHIELD), 0);
    // Sacrificed: it's gone, counter and all.
    let mut t = TestGame::new(2);
    let bears = shielded(&mut t);
    t.g.sacrifice(bears, P0);
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn bilbo_stays_blocked_when_the_blocker_grows() {
    cr!("509.1b", "506.4");
    ruling!(
        "Bilbo, Unexpected Adventurer",
        "Once Bilbo has been blocked, increasing the power of a creature blocking him to 3 or greater won't cause him to become unblocked."
    );
    supported("Bilbo, Unexpected Adventurer");
    let mut t = TestGame::new(2);
    let bilbo = t.battlefield(P0, "Bilbo, Unexpected Adventurer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(bilbo, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(giant, bilbo)]));
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, bilbo)]),
    );
    crate::r_s21_common::go_to(&mut t, Step::DeclareBlockers);
    let growth = t.hand(P1, "Giant Growth");
    t.lands(P1, "Forest", 1);
    t.cast(P1, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears).0, 5);
    assert!(!t.g.combat.as_ref().unwrap().is_unblocked(bilbo));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn primary_research_checks_as_the_end_step_starts() {
    cr!("603.4", "513.1");
    ruling!(
        "Primary Research",
        "Primary Research's last ability will check as your end step starts to see if any cards have left your graveyard this turn. If none did, the ability won't trigger at all. It's not possible to remove cards from your graveyard during your end step in time to have the ability trigger."
    );
    supported("Primary Research");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Primary Research");
    let card = t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 0);
    crate::r_s05_common::move_to(&mut t, card, Zone::Exile);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 0);
    // A card that left earlier in the turn: it triggers.
    t.advance_to(P0, Step::PrecombatMain);
    let card = t.graveyard(P0, "Grizzly Bears");
    crate::r_s05_common::move_to(&mut t, card, Zone::Exile);
    let before = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 1);
}

#[test]
fn battle_of_hoover_dam_copied_without_a_choice_has_neither_ability() {
    cr!("707.2", "614.12");
    ruling!(
        "Battle of Hoover Dam",
        "If you somehow control Battle of Hoover Dam and no choice was made for it (perhaps because another permanent on the battlefield became a copy of it), it has neither of the two triggered abilities."
    );
    supported("Battle of Hoover Dam");
    supported("Mirage Mirror");
    let mut t = TestGame::new(2);
    let battle = t.battlefield(P1, "Battle of Hoover Dam");
    let mirror = t.battlefield(P0, "Mirage Mirror");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, mirror, 0, &[obj(battle)]).expect("activate");
    t.resolve_all();
    t.recompute();
    assert_eq!(t.obj_now(mirror).chars.name, "Battle of Hoover Dam");
    // A creature of P0's dies and the end step comes: nothing triggers for P0.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.graveyard(P0, "Llanowar Elves");
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn izzet_guildmage_copies_an_untargeted_instant() {
    cr!("707.10");
    ruling!(
        "Izzet Guildmage",
        "Izzet Guildmage's abilities can copy any instant or sorcery spell with an appropriate mana value, not just one with targets."
    );
    supported("Izzet Guildmage");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Izzet Guildmage");
    let opt = t.hand(P0, "Opt");
    t.lands(P0, "Island", 4);
    let spell = t.cast(P0, opt).go();
    t.activate(P0, mage, 0, &[obj(spell)]).expect("copy");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn tezzeret_emblem_animates_without_removing_anything() {
    cr!("114.4", "205.1b", "613.1d");
    ruling!(
        "Tezzeret, Cruel Captain",
        "The emblem’s triggered ability doesn’t remove any abilities, types, subtypes, or supertypes the artifact has."
    );
    supported("Tezzeret, Cruel Captain");
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret, Cruel Captain");
    t.g.objects[tez.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    t.activate(P0, tez, 2, &[]).expect("-7");
    t.resolve_all();
    let ring = t.battlefield(P0, "Sol Ring");
    t.answer_targets(P0, &[obj(ring)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    t.recompute();
    let c = &t.obj_now(ring).chars;
    assert!(c.is(CardType::Artifact) && c.is(CardType::Creature));
    assert!(c.has_subtype("Robot"));
    assert_eq!(t.pt(ring), (3, 3));
    assert!(tap_for(&mut t, ring));
}

fn tap_for(t: &mut TestGame, id: ObjectId) -> bool {
    crate::r_s20_common::tap_for_mana(t, P0, id, "Add")
}

#[test]
fn ral_zarek_discards_are_chosen_in_turn_order_then_made_together() {
    cr!("101.4", "608.2e");
    ruling!(
        "Ral Zarek, Guest Lecturer",
        "When Ral Zarek's second ability resolves, the next target player in turn order (or, if it's a target player's turn, that player) chooses a card in hand without revealing it, then each other target player in turn order (if any) does the same. All chosen cards are then discarded at the same time."
    );
    supported("Ral Zarek, Guest Lecturer");
    let mut t = TestGame::new(4);
    let ral = t.battlefield(P0, "Ral Zarek, Guest Lecturer");
    for p in [P0, P1, P2, P3] {
        t.hand(p, "Grizzly Bears");
        t.hand(p, "Hill Giant");
    }
    let from = t.asked().len();
    t.answer_targets(
        P0,
        &[Entity::Player(P3), Entity::Player(P0), Entity::Player(P2)],
    );
    activate_containing(&mut t, P0, ral, "discard").expect("activate");
    t.resolve_all();
    let choosers: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.to_lowercase().contains("discard")))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(choosers, vec![P0, P2, P3]);
    for p in [P0, P2, P3] {
        assert_eq!(t.hand_size(p), 1, "{p:?}");
    }
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn commands_choose_only_modes_with_legal_targets() {
    cr!("700.2a", "601.2c");
    ruling!(
        "Ojutai's Command",
        "You can choose a mode only if you can choose legal targets for that mode. Ignore the targeting requirements for modes that aren’t chosen. For example, you can cast Ojutai’s Command without targeting a creature spell provided you don’t choose the third mode."
    );
    ruling!(
        "Silumgar's Command",
        "You can choose a mode only if you can choose legal targets for that mode. Ignore the targeting requirements for modes that aren’t chosen."
    );
    supported("Ojutai's Command");
    supported("Silumgar's Command");
    let available = |t: &TestGame, from: usize| -> Vec<usize> {
        asked_since(t, from)
            .into_iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseModes { available, .. } => Some(available),
                _ => None,
            })
            .expect("modes asked")
    };
    // Ojutai's Command with no creature spell (nor creature card in the graveyard): gain
    // 4 life and draw a card.
    let mut t = TestGame::new(2);
    let cmd = t.hand(P0, "Ojutai's Command");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Plains", 2);
    let from = t.asked().len();
    t.cast(P0, cmd).modes(&[1, 3]).go();
    assert_eq!(available(&t, from), vec![1, 3]);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.hand_size(P0), 1);
    // Silumgar's Command with only a creature around: bounce it and -3/-3.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let cmd = t.hand(P0, "Silumgar's Command");
    t.lands(P0, "Island", 3);
    t.lands(P0, "Swamp", 2);
    let from = t.asked().len();
    t.cast(P0, cmd)
        .modes(&[1, 2])
        .targets(&[obj(giant), obj(giant)])
        .go();
    assert_eq!(available(&t, from), vec![1, 2]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
}

#[test]
fn ranger_captain_of_eos_cant_counter_spells_already_cast() {
    cr!("602.2", "608.2");
    ruling!(
        "Ranger-Captain of Eos",
        "Your opponents may cast spells in response to the last ability of Ranger-Captain of Eos. The ability won’t affect those spells and it won’t affect spells that those players cast before you activated it. (In other words, the ability can’t be used to counter a spell.)"
    );
    supported("Ranger-Captain of Eos");
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Ranger-Captain of Eos");
    // P1's Shock is on the stack; P0 sacrifices the Ranger-Captain in response, and P1
    // responds with another Shock.
    let shock = t.hand(P1, "Shock");
    let shock2 = t.hand(P1, "Shock");
    let shock3 = t.hand(P1, "Shock");
    let bears = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Mountain", 3);
    t.lands(P1, "Forest", 2);
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    activate_containing(&mut t, P0, captain, "Sacrifice").expect("sacrifice");
    t.cast(P1, shock2).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    // Afterwards, P1 can't cast a noncreature spell this turn, but can cast a creature.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P1, shock3));
    assert!(castable(&mut t, P1, bears));
}

#[test]
fn starting_town_counts_the_turns_you_have_taken() {
    cr!("500.7", "614.1c");
    ruling!(
        "Squall, SeeD Mercenary",
        "Starting Town's first ability cares about how many turns you have taken, not necessarily how many turns the game has had, in case you take an extra turn."
    );
    supported("Starting Town");
    // "This land enters tapped unless it's your first, second, or third turn of the game."
    // P1's third turn is the game's sixth turn.
    let mut t = TestGame::new(2);
    let enters_untapped = |t: &mut TestGame, p: PlayerId| {
        let town = t.hand(p, "Starting Town");
        t.play_land(p, town).unwrap();
        t.settle();
        !t.obj_now(t.g.current(town)).tapped
    };
    for _ in 0..2 {
        t.advance_to(P1, Step::PrecombatMain);
        t.advance_to(P0, Step::PrecombatMain);
    }
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.g.turn.number >= 6);
    assert!(enters_untapped(&mut t, P1));
    // P0's fourth turn: tapped.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!enters_untapped(&mut t, P0));
}

fn copies(name: &str, n: usize) -> Vec<Arc<CardDef>> {
    (0..n).map(|_| card(name)).collect()
}

#[test]
fn lurrus_ignores_instants_and_sorceries_in_the_starting_deck() {
    cr!("702.139a");
    ruling!(
        "Lurrus of the Dream-Den",
        "Lurrus doesn't care about instant and sorcery cards in your starting deck. They may have any mana value."
    );
    let mut deck = copies("Grizzly Bears", 20);
    deck.extend(copies("Forest", 24));
    deck.extend(copies("Divine Verdict", 8));
    deck.extend(copies("Stroke of Genius", 8));
    let config = GameConfig {
        starting_player: Some(P0),
        ..Default::default()
    };
    let mut t = pregame(config, vec![deck, copies("Forest", 60)]);
    let side =
        t.g.add_to_sideboard(P0, vec![card("Lurrus of the Dream-Den")]);
    t.answer_choose(P0, &[obj(side[0])]);
    t.g.start();
    assert_eq!(t.g.companion_of(P0), Some(side[0]));
}
