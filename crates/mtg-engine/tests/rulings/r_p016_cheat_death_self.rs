//! Rulings batch P016 — permanents that return themselves to the battlefield after they
//! die or leave: the Enduring cycle, Loyal Cathar, Epochrasite, Ojer Taq, Flameskull,
//! Flayer of the Hatebound, Darigaaz Reincarnated, Screams from Within, Tenacious Dead,
//! Phytotitan, Temple of the Dead, Molten Firebird and Demonlord of Ashmouth.

use crate::r_p016_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::{can_activate, can_cast, can_play_land, target_candidates};
use crate::r_s05_common::move_to;
use crate::r_s17_common::become_copy;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::game::GameConfig;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn destroy_and_resolve(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.resolve_all();
}

const ENDURING: [(&str, &str); 3] = [
    ("Enduring Courage", "Dog"),
    ("Enduring Innocence", "Sheep"),
    ("Enduring Tenacity", "Snake"),
];

// ---------------------------------------------------------------------------
// The Enduring cycle
// ---------------------------------------------------------------------------

#[test]
fn enduring_returns_as_an_enchantment_without_its_creature_types() {
    cr!("205.3d", "611.2c", "603.10a");
    ruling!(
        "Enduring Courage",
        "Dog and Glimmer are both creature types. Enduring Courage won't have those creature types when its last ability returns it to the battlefield because it won't be a creature."
    );
    ruling!(
        "Enduring Innocence",
        "Sheep and Glimmer are both creature types. Enduring Innocence won't have those creature types when its last ability returns it to the battlefield because it won't be a creature."
    );
    ruling!(
        "Enduring Tenacity",
        "Snake and Glimmer are both creature types. Enduring Tenacity won't have those creature types when its last ability returns it to the battlefield because it won't be a creature."
    );
    for (name, ty) in ENDURING {
        supported(name);
        let mut t = TestGame::new(2);
        let e = t.battlefield(P0, name);
        assert!(t.obj_now(e).chars.has_subtype(ty));
        destroy_and_resolve(&mut t, e);
        let o = t.obj_now(e);
        assert_eq!(o.zone, Zone::Battlefield, "{name}");
        assert!(
            o.is(CardType::Enchantment) && !o.is(CardType::Creature),
            "{name}"
        );
        assert!(!o.chars.has_subtype(ty), "{name}");
        assert!(!o.chars.has_subtype("Glimmer"), "{name}");
    }
}

#[test]
fn a_token_copy_of_an_enduring_creature_doesnt_return() {
    cr!("111.7", "111.8", "707.2");
    ruling!(
        "Enduring Courage",
        "A token that's a copy of Enduring Courage won't return to the battlefield when its last ability resolves."
    );
    ruling!(
        "Enduring Innocence",
        "A token that's a copy of Enduring Innocence won't return to the battlefield when its last ability resolves."
    );
    ruling!(
        "Enduring Tenacity",
        "A token that's a copy of Enduring Tenacity won't return to the battlefield when its last ability resolves."
    );
    for (name, _) in ENDURING {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let e = t.battlefield(P0, name);
        cast_at(&mut t, P0, "Cackling Counterpart", &[o(e)]);
        t.resolve_all();
        let token =
            t.g.permanents()
                .find(|p| p.is_token())
                .map(|p| p.id)
                .expect("token copy");
        destroy_and_resolve(&mut t, token);
        assert!(t.g.permanents().all(|p| !p.is_token()), "{name}");
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
    }
}

#[test]
fn a_nontoken_copy_of_an_enduring_creature_returns_as_only_an_enchantment() {
    cr!("707.2", "611.2c", "205.1a");
    ruling!(
        "Enduring Courage",
        "If a nontoken permanent that's a copy of Enduring Courage dies while it's a creature, it will return to the battlefield as an enchantment when its last ability resolves. It won't have any card types other than enchantment."
    );
    ruling!(
        "Enduring Innocence",
        "If a nontoken permanent that's a copy of Enduring Innocence dies while it's a creature, it will return to the battlefield as an enchantment when its last ability resolves. It won't have any card types other than enchantment."
    );
    ruling!(
        "Enduring Tenacity",
        "If a nontoken permanent that's a copy of Enduring Tenacity dies while it's a creature, it will return to the battlefield as an enchantment when its last ability resolves. It won't have any card types other than enchantment."
    );
    for (name, _) in ENDURING {
        // P0's Grizzly Bears is a copy of P1's Enduring creature; it dies and the Bears
        // card returns as an enchantment.
        let mut t = TestGame::new(2);
        let orig = t.battlefield(P1, name);
        let bears = t.battlefield(P0, "Grizzly Bears");
        become_copy(&mut t, bears, orig);
        destroy_and_resolve(&mut t, bears);
        let o = t.obj_now(bears);
        assert_eq!(o.zone, Zone::Battlefield, "{name}");
        assert_eq!(o.chars.name, "Grizzly Bears", "{name}");
        assert_eq!(
            o.chars.card_types,
            CardTypeSet::single(CardType::Enchantment),
            "{name}"
        );
    }
}

#[test]
fn enduring_tenacity_triggers_once_per_life_gain_event() {
    cr!("119.10", "510.2", "702.15b");
    ruling!(
        "Enduring Tenacity",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event."
    );
    ruling!(
        "Enduring Tenacity",
        "If you gain an amount of life \"for each\" of something, that life is gained as one event and Enduring Tenacity's first ability triggers only once."
    );
    // Two lifelinkers deal combat damage: two triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enduring Tenacity");
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    crate::r_s01_common::attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "loses that much life"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    // One lifelinker dealing damage to two blockers at once: one trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enduring Tenacity");
    // Child of Night: a 2/1 with lifelink, blocked by two 1/1s.
    let a = t.battlefield(P0, "Child of Night");
    let x = t.battlefield(P1, "Llanowar Elves");
    let y = t.battlefield(P1, "Llanowar Elves");
    crate::r_s01_common::attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(x, a), (y, a)]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "loses that much life"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.graveyard_size(P1), 2);
    // "Gain 1 life for each land you control": one event, one trigger.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Enduring Tenacity");
    t.lands(P0, "Forest", 3);
    cast_at(&mut t, P0, "Bountiful Harvest", &[]);
    t.resolve();
    assert_eq!(t.life(P0), 23);
    assert_eq!(triggers_on_stack(&t, "loses that much life"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn enduring_tenacity_and_exquisite_blood_loop_until_a_player_wins() {
    cr!("104.3b", "704.5a");
    ruling!(
        "Enduring Tenacity",
        "If an ability triggers whenever an opponent loses life and its effect causes you to gain life, such as the ability of Exquisite Blood, this will loop until either you win the game"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Enduring Tenacity");
    t.battlefield(P0, "Exquisite Blood");
    cast_at(&mut t, P0, "Healing Salve", &[Entity::Player(P0)]);
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn enduring_tenacity_doesnt_see_a_teammates_life_gain() {
    cr!("810.9", "119.10");
    ruling!(
        "Enduring Tenacity",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Enduring Tenacity's first ability to trigger, even though it caused your team's life total to increase."
    );
    let mut t = TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]));
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Enduring Tenacity");
    let before = t.life(P0);
    cast_at(&mut t, P1, "Healing Salve", &[Entity::Player(P1)]);
    t.resolve();
    assert!(t.life(P0) > before);
    assert_eq!(triggers_on_stack(&t, "loses that much life"), 0);
    // P0 gaining life does trigger it.
    t.set_step(P0, Step::PrecombatMain);
    cast_at(&mut t, P0, "Healing Salve", &[Entity::Player(P0)]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "loses that much life"), 1);
}

#[test]
fn enduring_innocence_checks_power_as_creatures_enter() {
    cr!("603.2", "603.6a", "122.6");
    ruling!(
        "Enduring Innocence",
        "Enduring Innocence's second ability checks the power of the other creatures only at the moment they enter. If a creature enters with counters, those counters are included."
    );
    // Kalonian Hydra enters with four +1/+1 counters: power 4, no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enduring Innocence");
    crate::r_s05_common::enter(&mut t, P0, "Kalonian Hydra");
    assert_eq!(t.stack_len(), 0);
    // Grizzly Bears (power 2) triggers; it grows before the trigger resolves: still a draw.
    let bears = crate::r_s05_common::enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.stack_len(), 1);
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------
// Loyal Cathar
// ---------------------------------------------------------------------------

#[test]
fn a_copy_of_loyal_cathar_returns_transformed_only_if_double_faced() {
    cr!("712.14a", "707.2", "603.7");
    ruling!(
        "Loyal Cathar // Unhallowed Cathar",
        "Another double-faced card that becomes a copy of Loyal Cathar and then dies will return to the battlefield with its back face up."
    );
    ruling!(
        "Loyal Cathar // Unhallowed Cathar",
        "If a card that isn't a double-faced card becomes a copy of Loyal Cathar (due to Cytoshape, for example) and then dies, that card won't return to the battlefield. It will remain in its owner's graveyard."
    );
    supported("Loyal Cathar // Unhallowed Cathar");
    let run = |name: &str| -> (TestGame, ObjectId) {
        let mut t = TestGame::new(2);
        let cathar = t.battlefield(P1, "Loyal Cathar // Unhallowed Cathar");
        let c = t.battlefield(P0, name);
        become_copy(&mut t, c, cathar);
        destroy_and_resolve(&mut t, c);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        (t, c)
    };
    let (t, delver) = run("Delver of Secrets // Insectile Aberration");
    assert!(t.on_battlefield(delver));
    assert_eq!(t.obj_now(delver).chars.name, "Insectile Aberration");
    let (t, bears) = run("Grizzly Bears");
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
}

// ---------------------------------------------------------------------------
// Epochrasite, Ojer Taq
// ---------------------------------------------------------------------------

#[test]
fn epochrasite_gets_counters_unless_cast_from_hand() {
    cr!("614.1c");
    ruling!(
        "Epochrasite",
        "Epochrasite will enter with three +1/+1 counters on it if you cast it from a zone other than your hand, or if an effect let you put it directly onto the battlefield."
    );
    supported("Epochrasite");
    let mut t = TestGame::new(2);
    let e = t.enter(P0, "Epochrasite");
    assert_eq!(t.counters(e, counters::PLUS1), 3);
    t.set_step(P0, Step::PrecombatMain);
    let e = cast_at(&mut t, P0, "Epochrasite", &[]);
    t.resolve_all();
    assert_eq!(t.counters(e, counters::PLUS1), 0);
}

#[test]
fn ojer_taqs_extra_tokens_get_what_the_original_tokens_get() {
    cr!("614.1a", "111.1");
    ruling!(
        "Ojer Taq, Deepest Foundation // Temple of Civilization",
        "Everything that is specified by the effect creating the original token or tokens will also be true about the additional tokens created by Ojer Taq, Deepest Foundation's replacement effect."
    );
    supported("Ojer Taq, Deepest Foundation // Temple of Civilization");
    // Tapped and attacking: Hero of Bladehold's two Soldiers become six.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ojer Taq, Deepest Foundation // Temple of Civilization");
    let hero = t.battlefield(P0, "Hero of Bladehold");
    crate::r_s01_common::attack_with(&mut t, &[(hero, Entity::Player(P1))]);
    t.resolve_all();
    let soldiers: Vec<ObjectId> =
        t.g.permanents()
            .filter(|p| p.is_token())
            .map(|p| p.id)
            .collect();
    assert_eq!(soldiers.len(), 6);
    for s in &soldiers {
        assert!(t.obj(*s).tapped);
        assert!(t.g.is_attacking(*s));
    }
    // Counters put on "it": Fractal Summoning's one Fractal becomes three, each with X
    // counters.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Ojer Taq, Deepest Foundation // Temple of Civilization");
    lots(&mut t, P0);
    let s = t.hand(P0, "Fractal Summoning");
    t.cast(P0, s).x(2).go();
    t.resolve_all();
    let fractals: Vec<ObjectId> =
        t.g.permanents()
            .filter(|p| p.is_token())
            .map(|p| p.id)
            .collect();
    assert_eq!(fractals.len(), 3);
    for f in &fractals {
        assert_eq!(t.counters(*f, counters::PLUS1), 2, "{}", t.dump_log());
    }
}

// ---------------------------------------------------------------------------
// Flameskull
// ---------------------------------------------------------------------------

#[test]
fn flameskull_exiles_nothing_if_it_left_the_graveyard() {
    cr!("603.7c", "608.2c");
    ruling!(
        "Flameskull",
        "Exiling Flameskull as its triggered ability resolves isn't optional, but if Flameskull leaves the graveyard before the ability starts resolving, you won't exile it or the top card of your library."
    );
    supported("Flameskull");
    // Still in the graveyard: both are exiled (no choice asked).
    let mut t = TestGame::new(2);
    let skull = t.battlefield(P0, "Flameskull");
    let top = t.library_top(P0, "Lightning Bolt");
    t.answer_yes(P0, false);
    destroy_and_resolve(&mut t, skull);
    assert_eq!(t.zone(skull), Zone::Exile);
    assert_eq!(t.zone(top), Zone::Exile);
    // It left the graveyard first: nothing is exiled.
    let mut t = TestGame::new(2);
    let skull = t.battlefield(P0, "Flameskull");
    let top = t.library_top(P0, "Lightning Bolt");
    t.g.destroy(skull, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, skull, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.zone(skull), Zone::Hand(P0));
    assert_eq!(t.zone(top), Zone::Library(P0));
}

#[test]
fn flameskull_cards_are_played_with_the_normal_costs_and_timing() {
    cr!("601.2f", "305.2", "307.1");
    ruling!(
        "Flameskull",
        "You must still pay all costs and follow all timing rules for spells cast this way. Likewise, playing lands this way is subject to the normal rules for playing lands."
    );
    let setup = |top: &str| -> (TestGame, ObjectId, ObjectId) {
        let mut t = TestGame::new(2);
        let skull = t.battlefield(P0, "Flameskull");
        let c = t.library_top(P0, top);
        destroy_and_resolve(&mut t, skull);
        (t, skull, c)
    };
    // Flameskull itself: not without mana, not on the opponent's turn.
    let (mut t, skull, _) = setup("Lightning Bolt");
    let skull = t.g.current(skull);
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, skull, CastMethod::Normal));
    t.lands(P0, "Mountain", 3);
    assert!(can_cast(&mut t, P0, skull, CastMethod::Normal));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, skull, CastMethod::Normal));
    // Cast this way, it uses up the permission: the other card can't be played.
    let (mut t, skull, bolt) = setup("Lightning Bolt");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 4);
    let skull = t.g.current(skull);
    t.cast_with(P0, skull, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(skull));
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(t.cast_with(P0, bolt, &[Entity::Player(P1)]).is_err());
    // A land: only with a land play left.
    let (mut t, _, land) = setup("Mountain");
    t.set_step(P0, Step::PrecombatMain);
    let land = t.g.current(land);
    assert!(can_play_land(&mut t, P0, land));
    let other = t.hand(P0, "Forest");
    t.play_land(P0, other).unwrap();
    assert!(!can_play_land(&mut t, P0, land));
}

// ---------------------------------------------------------------------------
// Flayer of the Hatebound
// ---------------------------------------------------------------------------

/// P0's Flayer is on the battlefield; `who` reanimates `name` from P0's graveyard with
/// Reanimate, answering `target` for Flayer's trigger. Returns the reanimated creature.
fn flayer_reanimate(t: &mut TestGame, who: PlayerId, name: &str, target: Entity) -> ObjectId {
    supported("Flayer of the Hatebound");
    let c = t.graveyard(P0, name);
    cast_at(t, who, "Reanimate", &[o(c)]);
    t.answer_targets(P0, &[target]);
    t.resolve();
    t.settle();
    t.g.current(c)
}

#[test]
fn flayers_trigger_damage_comes_from_the_entering_creature() {
    cr!("702.16b", "702.16e", "113.7");
    ruling!(
        "Flayer of the Hatebound",
        "Flayer of the Hatebound is the source of the ability, but it may be another creature that is the source of the damage. If a black creature enters from your graveyard, the ability could target a creature with protection from black, although the damage will be prevented. It couldn't target a creature with protection from red."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Flayer of the Hatebound");
    let white = t.battlefield(P1, "White Knight");
    let fire = t.battlefield(P1, "Kor Firewalker");
    let from = t.asked().len();
    flayer_reanimate(&mut t, P0, "Vampire Nighthawk", o(white));
    let offered = target_candidates(&t, P0, from);
    let offered = offered.last().unwrap();
    assert!(offered.contains(&o(white)));
    assert!(!offered.contains(&o(fire)));
    t.resolve_all();
    assert_eq!(t.obj_now(white).damage, 0);
    assert!(t.on_battlefield(white));
}

#[test]
fn flayer_triggers_for_creatures_from_your_graveyard_under_any_control() {
    cr!("603.6a", "603.2");
    ruling!(
        "Flayer of the Hatebound",
        "Flayer of the Hatebound's last ability will trigger even if a creature enters from your graveyard under another player's control."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Flayer of the Hatebound");
    let giant = flayer_reanimate(&mut t, P1, "Hill Giant", Entity::Player(P1));
    assert_eq!(t.obj_now(giant).controller, P1);
    t.resolve_all();
    // (Reanimate costs P1 4 life.)
    assert_eq!(t.life(P1), 20 - 4 - 3);
}

#[test]
fn flayer_doesnt_trigger_for_a_creature_cast_from_the_graveyard() {
    cr!("601.2a", "603.6a");
    ruling!(
        "Flayer of the Hatebound",
        "If you cast a creature card from your graveyard, that card will be put on the stack before entering. Flayer of the Hatebound won't trigger."
    );
    supported("Gravecrawler");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Flayer of the Hatebound");
    t.battlefield(P0, "Gravecrawler");
    let crawler = t.graveyard(P0, "Gravecrawler");
    lots(&mut t, P0);
    t.cast(P0, crawler).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Gravecrawler").len(), 2);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn flayers_damage_uses_the_creatures_power_and_last_known_abilities() {
    cr!("608.2h", "702.15b", "702.2b", "702.93a");
    ruling!(
        "Flayer of the Hatebound",
        "Since damage is dealt by the creature, abilities like lifelink and deathtouch are taken into account, even if the creature has left the battlefield by the time it deals damage."
    );
    ruling!(
        "Flayer of the Hatebound",
        "The creature that entered from your graveyard deals damage equal to its current power (including any +1/+1 counters it entered with) to the target permanent or player. If it's no longer on the battlefield when the ability resolves, its last known existence on the battlefield is checked to determine its power."
    );
    // Vampire Nighthawk (lifelink, deathtouch) enters and is destroyed before the trigger
    // resolves: it still deals damage, with lifelink and deathtouch.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Flayer of the Hatebound");
    let giant = t.battlefield(P1, "Hill Giant");
    let hawk = flayer_reanimate(&mut t, P0, "Vampire Nighthawk", o(giant));
    modify_until_eot(
        &mut t,
        hawk,
        vec![Modification::ModifyPT(Value::c(1), Value::c(0))],
    );
    t.g.destroy(hawk, None);
    t.settle();
    let life = t.life(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), life + 3);
    // Flayer's undying returns it with a +1/+1 counter: 5 damage.
    let mut t = TestGame::new(2);
    let flayer = t.battlefield(P0, "Flayer of the Hatebound");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy_and_resolve(&mut t, flayer);
    assert_eq!(t.counters(flayer, counters::PLUS1), 1);
    assert_eq!(t.life(P1), 15);
}

// ---------------------------------------------------------------------------
// Darigaaz, Screams from Within, Tenacious Dead, Phytotitan
// ---------------------------------------------------------------------------

#[test]
fn darigaaz_chooses_its_own_replacement_first_and_needs_egg_counters() {
    cr!("616.1", "603.4");
    ruling!(
        "Darigaaz Reincarnated",
        "If another effect says to exile Darigaaz if it would die, you may apply Darigaaz's own effect first, giving it three egg counters."
    );
    ruling!(
        "Darigaaz Reincarnated",
        "If Darigaaz is exiled without any egg counters on it, its last ability won't trigger and won't return it to the battlefield."
    );
    supported("Darigaaz Reincarnated");
    let mut eggs = vec![];
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Rest in Peace");
        let d = t.battlefield(P0, "Darigaaz Reincarnated");
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        destroy_and_resolve(&mut t, d);
        assert_eq!(t.zone(d), Zone::Exile);
        let n = t.counters(d, "egg");
        eggs.push(n);
        // At P0's upkeep: an egg is removed only if it has any.
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "egg counter"), (n > 0) as usize);
        t.resolve_all();
        assert_eq!(t.counters(d, "egg"), n.saturating_sub(1));
    }
    eggs.sort();
    assert_eq!(eggs, vec![0, 3]);
}

#[test]
fn screams_from_within_controlled_by_another_player_does_nothing() {
    cr!("404.2", "603.3a");
    ruling!(
        "Screams from Within",
        "If another player has gained control of Screams from Within, its triggered ability does nothing when it's put into a graveyard."
    );
    supported("Screams from Within");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // P0's Screams, controlled by P1.
    let screams = t.battlefield(P0, "Screams from Within");
    t.g.attach(screams, o(bears));
    t.g.objects[screams.0 as usize].controller = P1;
    t.g.objects[screams.0 as usize].base_controller = P1;
    t.g.recompute();
    t.answer_choose(P1, &[o(giant)]);
    destroy_and_resolve(&mut t, bears);
    assert!(t.in_graveyard(P0, "Screams from Within"));
    // Under its owner's control it returns.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let screams = t.battlefield(P0, "Screams from Within");
    t.g.attach(screams, o(bears));
    t.answer_choose(P0, &[o(giant)]);
    destroy_and_resolve(&mut t, bears);
    assert_eq!(t.named_on_battlefield("Screams from Within").len(), 1);
}

#[test]
fn tenacious_dead_pays_on_resolution_and_returns_under_its_owner() {
    cr!("603.3a", "110.2", "117.3");
    ruling!(
        "Tenacious Dead",
        "If you control a Tenacious Dead owned by another player when it dies, you’ll control the triggered ability. However, if you choose to pay {1}{B}, it will be put onto the battlefield under its owner’s control."
    );
    ruling!(
        "Tenacious Dead",
        "You decide whether to pay {1}{B} when the ability resolves. If you choose not to, you won’t get another chance later."
    );
    supported("Tenacious Dead");
    let mut t = TestGame::new(2);
    let dead = t.battlefield(P1, "Tenacious Dead");
    t.g.objects[dead.0 as usize].controller = P0;
    t.g.objects[dead.0 as usize].base_controller = P0;
    t.g.recompute();
    t.lands(P0, "Swamp", 2);
    t.g.destroy(dead, None);
    t.settle();
    // The trigger is P0's; nothing was asked yet.
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(trig).controller, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(dead));
    assert_eq!(t.obj_now(dead).controller, P1);
    assert!(t.obj_now(dead).tapped);
    // Declined: it stays in the graveyard.
    let mut t = TestGame::new(2);
    let dead = t.battlefield(P0, "Tenacious Dead");
    t.lands(P0, "Swamp", 2);
    t.answer_yes(P0, false);
    destroy_and_resolve(&mut t, dead);
    assert!(t.in_graveyard(P0, "Tenacious Dead"));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_graveyard(P0, "Tenacious Dead"));
}

#[test]
fn phytotitan_returns_only_if_still_in_the_graveyard() {
    cr!("603.7c", "400.7");
    ruling!(
        "Phytotitan",
        "Phytotitan’s ability will return it to the battlefield only if it’s still in the graveyard when the delayed triggered ability resolves."
    );
    supported("Phytotitan");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Phytotitan");
    destroy_and_resolve(&mut t, p);
    move_to(&mut t, p, Zone::Exile);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(p), Zone::Exile);
    assert!(t.named_on_battlefield("Phytotitan").is_empty());
}

// ---------------------------------------------------------------------------
// Temple of the Dead, Molten Firebird, Demonlord of Ashmouth
// ---------------------------------------------------------------------------

#[test]
fn temple_of_the_dead_transforms_even_if_hands_grow_before_it_resolves() {
    cr!("602.5b", "701.27a");
    ruling!(
        "Aclazotz, Deepest Betrayal // Temple of the Dead",
        "The last ability of Temple of the Dead will resolve even if you have two or more cards in hand at that time."
    );
    const ACLAZOTZ: &str = "Aclazotz, Deepest Betrayal // Temple of the Dead";
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let god = t.battlefield(P0, ACLAZOTZ);
    destroy_and_resolve(&mut t, god);
    assert_eq!(t.obj_now(god).chars.name, "Temple of the Dead");
    let temple = t.g.current(god);
    t.g.objects[temple.0 as usize].tapped = false;
    // P1 has no cards in hand: the ability can be activated.
    assert_eq!(t.hand_size(P1), 0);
    lots(&mut t, P0);
    crate::r_s06_common::activate_containing(&mut t, P0, temple, "Transform").unwrap();
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
        t.hand(P1, "Grizzly Bears");
    }
    t.resolve_all();
    assert_eq!(t.obj_now(temple).chars.name, "Aclazotz, Deepest Betrayal");
}

#[test]
fn molten_firebirds_controller_skips_their_draw_step() {
    cr!("614.10", "603.10a");
    ruling!(
        "Molten Firebird",
        "The player who controlled Molten Firebird when it was put into a graveyard will skip their next draw step even if Molten Firebird is returned to the battlefield under a different player’s control"
    );
    supported("Molten Firebird");
    // P0 controls P1's Firebird when it dies: P0 skips their next draw step; the Firebird
    // returns under P1's control.
    let mut t = TestGame::new(2);
    let bird = t.battlefield(P1, "Molten Firebird");
    t.g.objects[bird.0 as usize].controller = P0;
    t.g.objects[bird.0 as usize].base_controller = P0;
    t.g.recompute();
    destroy_and_resolve(&mut t, bird);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bird));
    assert_eq!(t.obj_now(bird).controller, P1);
    // P1 draws normally; P0 doesn't.
    let p1_hand = t.hand_size(P1);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.hand_size(P1), p1_hand + 1);
    let p0_hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), p0_hand);
    // It left the graveyard before the end step: it doesn't return; the skip remains.
    let mut t = TestGame::new(2);
    let bird = t.battlefield(P0, "Molten Firebird");
    destroy_and_resolve(&mut t, bird);
    move_to(&mut t, bird, Zone::Exile);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.zone(bird), Zone::Exile);
    let p0_hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), p0_hand);
}

#[test]
fn molten_firebirds_exile_ability_works_only_on_the_battlefield() {
    cr!("113.6", "602.1");
    ruling!(
        "Molten Firebird",
        "The second ability can be activated only if Molten Firebird is on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    lots(&mut t, P0);
    let in_gy = t.graveyard(P0, "Molten Firebird");
    assert!(!can_activate(&mut t, P0, in_gy));
    let on_bf = t.battlefield(P0, "Molten Firebird");
    assert!(can_activate(&mut t, P0, on_bf));
}

#[test]
fn demonlord_of_ashmouth_may_be_exiled_instead_of_sacrificing() {
    cr!("118.12");
    ruling!(
        "Demonlord of Ashmouth",
        "You don't have to sacrifice another creature, even if you control one. You may choose to exile Demonlord of Ashmouth."
    );
    supported("Demonlord of Ashmouth");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[]);
    let d = crate::r_s05_common::enter(&mut t, P0, "Demonlord of Ashmouth");
    t.resolve_all();
    assert_eq!(t.zone(d), Zone::Exile);
    assert!(t.on_battlefield(bears));
}

#[test]
fn return_this_card_from_your_graveyard_wordings_still_work() {
    cr!("400.7", "603.6e");
    // An activated ability of the card in the graveyard (to hand).
    supported("Pilgrim of the Ages");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let pilgrim = t.graveyard(P0, "Pilgrim of the Ages");
    lots(&mut t, P0);
    crate::r_s06_common::activate_containing(&mut t, P0, pilgrim, "{6}").unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Pilgrim of the Ages"));
    // A triggered ability of the card in the graveyard (to the battlefield).
    supported("Bladewing's Thrall");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Bladewing's Thrall");
    t.answer_yes(P0, true);
    crate::r_s05_common::enter(&mut t, P0, "Shivan Dragon");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bladewing's Thrall").len(), 1);
    // A leaves-the-battlefield trigger of an Aura that went to the graveyard first.
    supported("Genju of the Cedars");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let genju = t.battlefield(P0, "Genju of the Cedars");
    t.g.attach(genju, o(forest));
    t.settle();
    t.answer_yes(P0, true);
    destroy_and_resolve(&mut t, forest);
    assert!(t.in_hand(P0, "Genju of the Cedars"));
    // The same, but the Genju left the graveyard and came back first: a new object.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let genju = t.battlefield(P0, "Genju of the Cedars");
    t.g.attach(genju, o(forest));
    t.settle();
    t.answer_yes(P0, true);
    t.g.destroy(forest, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Genju of the Cedars"));
    move_to(&mut t, genju, Zone::Exile);
    move_to(&mut t, genju, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Genju of the Cedars"));
}
