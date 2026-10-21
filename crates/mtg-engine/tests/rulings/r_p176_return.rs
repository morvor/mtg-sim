//! Rulings batch P176 — taking opponents' creatures as they die or from graveyards:
//! Athreos, Shroud-Veiled's coin counters, replacement effects that exile opponents'
//! dying creatures (Gisa, Glorious Resurrector, Shelob, Dread Weaver, The Darkness
//! Crystal, Kalitas, Draugr Necromancer), Adarkar Valkyrie, Sorin, Lord of Innistrad,
//! Beacon of Unrest, Macabre Mockery, Treacherous Urge, Zara, Profane Procession and
//! Gather Specimens.

use crate::r_p176_common::*;
use crate::r_s02_common::{can_cast, create_token};
use mtg_engine::ability::LibraryPosition;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn coin(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    assert_eq!(t.g.add_counters(obj(id), "coin", 1, None), 1);
    t.g.flush_events();
}

/// Exiles the permanents at the same time (one simultaneous zone change), then settles.
fn exile_together(t: &mut TestGame, ids: &[ObjectId]) {
    let moves = ids
        .iter()
        .map(|id| MoveEv {
            obj: t.g.current(*id),
            to: Zone::Exile,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: None,
            etb: EtbInfo::default(),
            source: None,
        })
        .collect();
    t.g.move_objects(moves);
    t.g.flush_events();
    t.settle();
}

/// Exiles one permanent or card and settles.
fn exile(t: &mut TestGame, id: ObjectId) {
    exile_together(t, &[id]);
}

/// The objects on the battlefield named `name` controlled by `p`.
fn mine(t: &mut TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.recompute();
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.obj_now(*id).controller == p)
        .collect()
}

// ---------------------------------------------------------------------------------
// Athreos, Shroud-Veiled
// ---------------------------------------------------------------------------------

/// P0 controls Athreos (not a creature: devotion 2).
fn athreos() -> (TestGame, ObjectId) {
    supported("Athreos, Shroud-Veiled");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Athreos, Shroud-Veiled");
    assert!(!t.obj_now(a).is(CardType::Creature));
    (t, a)
}

#[test]
fn athreos_returns_a_creature_you_dont_own_that_leaves_with_it() {
    cr!("603.10a", "603.6c", "122.1");
    ruling!(
        "Athreos, Shroud-Veiled",
        "If Athreos and a creature with a coin counter on it are both put into graveyards and/or exiled at the same time, the other creature will be returned to the battlefield."
    );
    ruling!(
        "Athreos, Shroud-Veiled",
        "If a creature you don't own has a coin counter on it, that permanent will return to the battlefield under your control when it dies or is exiled while you control Athreos."
    );
    // Positive control: P1's Bears with a coin counter dies and comes back under P0.
    let (mut t, _) = athreos();
    let bears = t.battlefield(P1, "Grizzly Bears");
    coin(&mut t, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(mine(&mut t, P0, "Grizzly Bears").len(), 1);
    // Athreos and the Bears are exiled at the same time.
    let (mut t, a) = athreos();
    let bears = t.battlefield(P1, "Grizzly Bears");
    coin(&mut t, bears);
    exile_together(&mut t, &[a, bears]);
    assert_eq!(
        t.stack_len(),
        1,
        "Athreos's last ability looks back in time"
    );
    t.resolve_all();
    assert_eq!(mine(&mut t, P0, "Grizzly Bears").len(), 1);
    assert_eq!(t.zone(a), Zone::Exile);
}

#[test]
fn athreos_leaving_leaves_the_coin_counters_meaningless() {
    cr!("122.2", "603.2");
    ruling!(
        "Athreos, Shroud-Veiled",
        "If Athreos leaves the battlefield, creatures with coin counters on them keep them. The counters won't have any meaning or effect unless another Athreos is on the battlefield later."
    );
    let (mut t, a) = athreos();
    let bears = t.battlefield(P1, "Grizzly Bears");
    coin(&mut t, bears);
    exile(&mut t, a);
    assert_eq!(t.counters(bears, "coin"), 1, "keeps it");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Another Athreos later gives the counter meaning again.
    let (mut t, a) = athreos();
    let giant = t.battlefield(P1, "Hill Giant");
    coin(&mut t, giant);
    exile(&mut t, a);
    t.battlefield(P0, "Athreos, Shroud-Veiled");
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(
        mine(&mut t, P0, "Hill Giant").len(),
        1,
        "the new Athreos returns it"
    );
}

#[test]
fn athreos_with_a_coin_counter_returns_itself_only_while_a_creature() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Athreos, Shroud-Veiled",
        "If Athreos somehow gets a coin counter on itself, its last ability will return it when it dies or is exiled."
    );
    ruling!(
        "Athreos, Shroud-Veiled",
        "If a creature with a coin counter on it stops being a creature, it keeps its coin counter, but Athreos's last ability won't trigger when that permanent dies or is exiled unless it's a creature again by that time."
    );
    // Devotion 2 + 4 + 2 = 8: Athreos is a creature.
    let (mut t, a) = athreos();
    t.battlefield(P0, "Ashen Rider");
    t.battlefield(P0, "Tithe Drinker");
    t.g.recompute();
    assert!(t.obj_now(a).is(CardType::Creature));
    coin(&mut t, a);
    exile(&mut t, a);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(
        mine(&mut t, P0, "Athreos, Shroud-Veiled").len(),
        1,
        "returned itself"
    );

    // It stops being a creature (devotion drops) before it's exiled: no trigger.
    let (mut t, a) = athreos();
    let rider = t.battlefield(P0, "Ashen Rider");
    t.battlefield(P0, "Tithe Drinker");
    t.g.recompute();
    coin(&mut t, a);
    exile(&mut t, rider);
    t.g.recompute();
    assert!(!t.obj_now(a).is(CardType::Creature));
    assert_eq!(t.counters(a, "coin"), 1, "keeps the counter");
    exile(&mut t, a);
    assert_eq!(t.stack_len(), 0, "no trigger");
    assert_eq!(t.zone(a), Zone::Exile);
}

#[test]
fn athreos_doesnt_return_a_card_that_left_the_graveyard_or_a_token() {
    cr!("400.7", "111.8", "603.2");
    ruling!(
        "Athreos, Shroud-Veiled",
        "If a creature with a coin counter on it is put into a graveyard or exile but leaves that zone before Athreos's last ability resolves, that card stays in its new zone"
    );
    ruling!(
        "Athreos, Shroud-Veiled",
        "If a token with a coin counter on it dies or is exiled, Athreos's last ability triggers, but won't return it to the battlefield."
    );
    let (mut t, _) = athreos();
    let bears = t.battlefield(P1, "Grizzly Bears");
    coin(&mut t, bears);
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    // The card is exiled from the graveyard in response.
    let card = t.g.current(bears);
    exile(&mut t, card);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile, "stays in its new zone");
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());

    let (mut t, _) = athreos();
    let token = create_token(&mut t, P1, "Soldier");
    coin(&mut t, token);
    destroy(&mut t, token);
    assert_eq!(t.stack_len(), 1, "it triggers");
    t.resolve_all();
    assert!(t.named_on_battlefield("Soldier").is_empty(), "not returned");
}

#[test]
fn athreos_and_flicker_effects() {
    cr!("400.7", "603.2");
    ruling!(
        "Athreos, Shroud-Veiled",
        "If an effect exiles a creature with a coin counter on it and immediately returns it to the battlefield, Athreos's last ability triggers but will have no effect."
    );
    // Cloudshift: the Bears are back at once; the trigger does nothing.
    let (mut t, _) = athreos();
    let bears = t.battlefield(P0, "Grizzly Bears");
    coin(&mut t, bears);
    cast_only(&mut t, P0, "Cloudshift", &[obj(bears)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1, "it triggered");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Otherworldly Journey: Athreos returns it now (under P0's control); the delayed
    // return does nothing.
    let (mut t, _) = athreos();
    let bears = t.battlefield(P1, "Grizzly Bears");
    coin(&mut t, bears);
    t.set_step(P0, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Otherworldly Journey", &[obj(bears)]);
    assert_eq!(
        mine(&mut t, P0, "Grizzly Bears").len(),
        1,
        "Athreos returned it"
    );
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(mine(&mut t, P0, "Grizzly Bears").len(), 1);
}

// ---------------------------------------------------------------------------------
// Gisa, Glorious Resurrector
// ---------------------------------------------------------------------------------

#[test]
fn gisa_exiled_creatures_dont_die_and_return_without_haste() {
    cr!("614.6", "700.4", "302.6");
    ruling!(
        "Gisa, Glorious Resurrector",
        "Because creatures controlled by opponents aren't dying due to Gisa's replacement effect, any \"when [this creature] dies\" triggered abilities those creatures have won't trigger."
    );
    ruling!(
        "Gisa, Glorious Resurrector",
        "Gisa does not grant haste to the creatures that are returned to the battlefield"
    );
    supported("Gisa, Glorious Resurrector");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Gisa, Glorious Resurrector");
    let dissenter = t.battlefield(P1, "Doomed Dissenter");
    destroy(&mut t, dissenter);
    assert_eq!(t.zone(dissenter), Zone::Exile);
    assert_eq!(t.stack_len(), 0, "no dies trigger");
    assert!(t.named_on_battlefield("Zombie Token").is_empty());
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let back = mine(&mut t, P0, "Doomed Dissenter");
    assert_eq!(back.len(), 1, "returned at P0's upkeep");
    assert!(!t.obj_now(back[0]).has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, back[0]), "summoning sick, no haste");
}

#[test]
fn gisa_and_another_exile_effect_the_opponent_chooses() {
    cr!("616.1", "614.6");
    ruling!(
        "Gisa, Glorious Resurrector",
        "If a creature your opponent controls would die and more than one effect would cause it to be exiled, that opponent chooses which one to apply."
    );
    supported("Kalitas, Traitor of Ghet");
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.battlefield(P0, "Gisa, Glorious Resurrector");
        t.battlefield(P0, "Kalitas, Traitor of Ghet");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer(P1, DecisionKind::Replacement, Answer::Index(pick));
        let from = t.asked().len();
        destroy(&mut t, bears);
        let chosen = t.asked()[from..]
            .iter()
            .find_map(|(p, d)| match d {
                Decision::ChooseReplacement { options } if *p == P1 => Some(options[pick].clone()),
                _ => None,
            })
            .expect("P1 chooses");
        assert_eq!(t.zone(bears), Zone::Exile);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        let back = mine(&mut t, P0, "Grizzly Bears").len();
        if chosen.contains("Kalitas") {
            assert_eq!(back, 0, "exiled by Kalitas: Gisa doesn't return it");
            assert_eq!(mine(&mut t, P0, "Zombie Token").len(), 1);
        } else {
            assert_eq!(back, 1, "exiled by Gisa: returned");
        }
    }
}

// ---------------------------------------------------------------------------------
// Shelob, The Darkness Crystal, Kalitas
// ---------------------------------------------------------------------------------

#[test]
fn shelob_dying_with_the_creature_still_exiles_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Shelob, Dread Weaver",
        "If Shelob, Dread Weaver and a nontoken creature an opponent controls die at the same time, the opponent's creature will be exiled."
    );
    supported("Shelob, Dread Weaver");
    supported("Wrath of God");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shelob, Dread Weaver");
    t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Wrath of God", &[]);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Shelob, Dread Weaver"));
}

#[test]
fn shelob_card_that_went_elsewhere_is_a_new_object() {
    cr!("400.7", "614.6");
    ruling!(
        "Shelob, Dread Weaver",
        "Cards exiled with Shelob, Dread Weaver that would go to the graveyard but have that replaced by an effect such as Leyline of the Void are considered new objects."
    );
    let mut t = TestGame::new(2);
    let shelob = t.battlefield(P0, "Shelob, Dread Weaver");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    t.battlefield(P0, "Leyline of the Void");
    let exiled = t.g.current(bears);
    assert_eq!(t.zone(exiled), Zone::Exile);
    // Positive control: the third ability can return it.
    t.lands(P0, "Swamp", 6);
    assert!(can_activate(&mut t, P0, shelob));
    // Pay the second ability's cost with it: Leyline exiles it instead.
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(exiled)]);
    activate_containing(&mut t, P0, shelob, "Draw a card").expect("second ability");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(exiled), Zone::Exile, "Leyline exiled it");
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    t.g.recompute();
    assert!(
        !can_activate(&mut t, P0, shelob),
        "no longer exiled with Shelob"
    );
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn darkness_crystal_leaving_at_the_same_time_still_exiles() {
    cr!("614.12", "603.10a", "614.6");
    ruling!(
        "The Darkness Crystal",
        "If The Darkness Crystal leaves the battlefield at the same time as one or more nontoken creatures an opponent controls would die, those creature cards will be exiled and you'll gain 2 life for each of them."
    );
    supported("The Darkness Crystal");
    supported("Akroma's Vengeance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Darkness Crystal");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Akroma's Vengeance", &[]);
    assert!(t.in_graveyard(P0, "The Darkness Crystal"));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.life(P0), 24);
}

#[test]
fn opponents_dying_tokens_still_go_to_the_graveyard() {
    cr!("111.7", "700.4");
    ruling!(
        "Kalitas, Traitor of Ghet",
        "If a creature token an opponent controls dies, it goes to that player's graveyard as normal before ceasing to exist."
    );
    ruling!(
        "The Darkness Crystal",
        "If a creature token an opponent controls dies, it goes to that player's graveyard as normal before ceasing to exist."
    );
    for name in ["Kalitas, Traitor of Ghet", "The Darkness Crystal"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        // Blood Artist sees a creature die.
        t.battlefield(P0, "Blood Artist");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let token = create_token(&mut t, P1, "Soldier");
        destroy(&mut t, token);
        assert_eq!(t.stack_len(), 1, "{name}: the token died");
        t.resolve_all();
        assert_eq!(t.life(P1), 19);
        assert_eq!(t.life(P0), 21, "{name}: no 2 life");
        assert!(t.named_on_battlefield("Zombie Token").is_empty(), "{name}");
        // Positive control: a nontoken creature is exiled.
        let bears = t.battlefield(P1, "Grizzly Bears");
        destroy(&mut t, bears);
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"), "{name}");
    }
}

// ---------------------------------------------------------------------------------
// Draugr Necromancer
// ---------------------------------------------------------------------------------

/// P0's Draugr Necromancer exiles P1's dying Bears with an ice counter. Returns the card.
fn iced_bears(t: &mut TestGame) -> ObjectId {
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(t, bears);
    let iced = t.g.current(bears);
    assert_eq!(t.counters(iced, "ice"), 1);
    iced
}

#[test]
fn draugr_casting_follows_the_normal_rules() {
    cr!("601.2", "307.1", "601.2f");
    ruling!(
        "Draugr Necromancer",
        "Casting a spell from exile this way follows the normal rules for casting that spell. You must pay its costs, and you must follow all applicable timing rules."
    );
    supported("Draugr Necromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Draugr Necromancer");
    let iced = iced_bears(&mut t);
    // No mana: can't cast it.
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.cast(P0, iced).try_go().is_err(), "costs must be paid");
    t.lands(P0, "Snow-Covered Forest", 2);
    // Not on P1's turn (a creature spell).
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, iced, CastMethod::Normal), "timing");
    t.set_step(P0, Step::PrecombatMain);
    assert!(can_cast(&mut t, P0, iced, CastMethod::Normal));
    t.cast(P0, iced).go();
    t.resolve_all();
    assert_eq!(mine(&mut t, P0, "Grizzly Bears").len(), 1);
}

#[test]
fn draugr_any_iced_opponents_card_even_from_an_earlier_draugr() {
    cr!("122.1", "601.2");
    ruling!(
        "Draugr Necromancer",
        "Draugr Necromancer allows you to cast spells from among any cards in exile your opponents own with ice counters on them, including cards that were exiled by previous Draugr Necromancers."
    );
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Draugr Necromancer");
    let iced = iced_bears(&mut t);
    exile(&mut t, first);
    t.battlefield(P0, "Draugr Necromancer");
    t.lands(P0, "Snow-Covered Forest", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, iced).go();
    t.resolve_all();
    assert_eq!(mine(&mut t, P0, "Grizzly Bears").len(), 1);
}

#[test]
fn draugr_your_own_card_is_iced_but_not_castable() {
    cr!("122.1", "614.6");
    ruling!(
        "Draugr Necromancer",
        "If an opponent controls a nontoken creature that you own and that creature would die, it will be exiled with an ice counter on it. However, you won’t be able to cast that card from exile."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Draugr Necromancer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    destroy(&mut t, bears);
    let iced = t.g.current(bears);
    assert_eq!(t.zone(iced), Zone::Exile);
    assert_eq!(t.counters(iced, "ice"), 1);
    t.lands(P0, "Snow-Covered Forest", 2);
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, iced, CastMethod::Normal));
    assert!(t.cast(P0, iced).try_go().is_err());
}

// ---------------------------------------------------------------------------------
// Adarkar Valkyrie, Sorin, Beacon of Unrest
// ---------------------------------------------------------------------------------

#[test]
fn adarkar_valkyrie_can_target_a_token_but_cant_return_it() {
    cr!("111.8", "603.7");
    ruling!(
        "Adarkar Valkyrie",
        "Adarkar Valkyrie's ability can target a token creature, but since token creatures cease to exist when they leave the battlefield, it won't be returned to the battlefield."
    );
    supported("Adarkar Valkyrie");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Adarkar Valkyrie");
    let token = create_token(&mut t, P1, "Soldier");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, v, 0, &[obj(token)])
        .expect("a token is a legal target");
    t.resolve_all();
    destroy(&mut t, token);
    t.resolve_all();
    assert!(t.named_on_battlefield("Soldier").is_empty());
    // Positive control: a nontoken creature comes back.
    t.g.untap(v);
    t.activate(P0, v, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(mine(&mut t, P0, "Grizzly Bears").len(), 1);
}

/// P0 controls Sorin, Lord of Innistrad with 6 loyalty.
fn sorin(t: &mut TestGame) -> ObjectId {
    supported("Sorin, Lord of Innistrad");
    t.set_step(P0, Step::PrecombatMain);
    let s = t.battlefield(P0, "Sorin, Lord of Innistrad");
    set_loyalty(t, s, 6);
    s
}

#[test]
fn sorin_can_target_himself_as_a_creature() {
    cr!("606.3", "608.2c");
    ruling!(
        "Sorin, Lord of Innistrad",
        "If Sorin, Lord of Innistrad somehow becomes a creature, it can be chosen as a target of its own third ability."
    );
    supported("Sarkhan the Masterless");
    let mut t = TestGame::new(2);
    let s = sorin(&mut t);
    let sarkhan = t.battlefield(P0, "Sarkhan the Masterless");
    activate_containing(&mut t, P0, sarkhan, "Dragon").expect("Sarkhan +1");
    t.resolve_all();
    t.g.recompute();
    assert!(t.obj_now(s).is(CardType::Creature));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(s), obj(bears)]);
    activate_containing(&mut t, P0, s, "Destroy").expect("Sorin -6");
    t.resolve_all();
    let sorins = mine(&mut t, P0, "Sorin, Lord of Innistrad");
    assert_eq!(sorins.len(), 1, "destroyed and returned");
    assert_ne!(sorins[0], s);
    assert_eq!(mine(&mut t, P0, "Grizzly Bears").len(), 1);
}

#[test]
fn sorin_returns_an_undying_creature_before_undying_resolves() {
    cr!("702.93a", "400.7");
    ruling!(
        "Sorin, Lord of Innistrad",
        "If a creature with undying is destroyed by Sorin, Lord of Innistrad's third ability, the undying ability will trigger, but the creature will already have been returned to the battlefield by Sorin."
    );
    let mut t = TestGame::new(2);
    let s = sorin(&mut t);
    let wolf = t.battlefield(P1, "Young Wolf");
    t.answer_targets(P0, &[obj(wolf)]);
    activate_containing(&mut t, P0, s, "Destroy").expect("Sorin -6");
    t.resolve();
    let back = mine(&mut t, P0, "Young Wolf");
    assert_eq!(back.len(), 1, "Sorin returned it");
    assert_eq!(t.stack_len(), 1, "undying triggered");
    t.resolve_all();
    let wolves = t.named_on_battlefield("Young Wolf");
    assert_eq!(wolves.len(), 1);
    assert_eq!(t.counters(wolves[0], "+1/+1"), 0, "undying did nothing");
    assert_eq!(controller(&mut t, wolves[0]), P0);
}

#[test]
fn beacon_of_unrest_with_an_illegal_target_isnt_shuffled() {
    cr!("608.2b", "701.24a");
    ruling!(
        "Beacon of Unrest",
        "If the target card is an illegal target by the time Beacon of Unrest tries to resolve, the spell won't resolve. You won't shuffle it into your library."
    );
    supported("Beacon of Unrest");
    // Positive control: it returns the card and is shuffled away.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Beacon of Unrest", &[obj(bears)]);
    assert_eq!(mine(&mut t, P0, "Grizzly Bears").len(), 1);
    assert!(!t.in_graveyard(P0, "Beacon of Unrest"));
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    cast_only(&mut t, P0, "Beacon of Unrest", &[obj(bears)]);
    exile(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Beacon of Unrest"), "not shuffled");
}

// ---------------------------------------------------------------------------------
// Macabre Mockery, Treacherous Urge
// ---------------------------------------------------------------------------------

#[test]
fn macabre_mockery_cast_in_an_end_step() {
    cr!("603.7c", "514.2", "513.1");
    ruling!(
        "Macabre Mockery",
        "If you cast Macabre Mockery during a player’s end step, the creature will remain on the battlefield until the next turn’s end step. However, the +2/+0 and haste wear off during the cleanup step"
    );
    supported("Macabre Mockery");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.set_step(P1, Step::End);
    cast_resolve(&mut t, P0, "Macabre Mockery", &[obj(bears)]);
    let b = mine(&mut t, P0, "Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.pt(b[0]), (4, 2));
    assert!(t.obj_now(b[0]).has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::Upkeep);
    assert!(t.on_battlefield(b[0]), "still there next turn");
    assert_eq!(t.pt(b[0]), (2, 2), "the bonus wore off");
    assert!(!t.obj_now(b[0]).has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(b[0]), "sacrificed at the next end step");
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

/// P0 casts Treacherous Urge during `when`, taking P1's Grizzly Bears from P1's hand.
fn urge(when: (PlayerId, Step)) -> (TestGame, ObjectId) {
    supported("Treacherous Urge");
    let mut t = TestGame::new(2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.set_step(when.0, when.1);
    t.answer_choose(P0, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Treacherous Urge", &[Entity::Player(P1)]);
    let b = mine(&mut t, P0, "Grizzly Bears");
    assert_eq!(b.len(), 1, "took the Bears");
    (t, b[0])
}

#[test]
fn treacherous_urge_cast_in_the_end_step_sacrifices_next_turn() {
    cr!("603.7c", "513.1");
    ruling!(
        "Treacherous Urge",
        "If you cast Treacherous Urge during the End step, you won’t sacrifice the creature until the end of the following turn."
    );
    let (mut t, b) = urge((P1, Step::End));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.on_battlefield(b));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(b), "sacrificed");
}

#[test]
fn treacherous_urge_doesnt_sacrifice_a_creature_you_no_longer_control() {
    cr!("701.21a", "611.2c");
    ruling!(
        "Treacherous Urge",
        "If you don’t control the creature when the ability resolves, the creature won’t be sacrificed. It will have haste permanently."
    );
    let (mut t, b) = urge((P0, Step::PrecombatMain));
    give_control(&mut t, b, P1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(b), "not sacrificed");
    t.advance_to(P1, Step::Upkeep);
    assert!(
        t.obj_now(b).has_keyword(KeywordKind::Haste),
        "haste for good"
    );
}

// ---------------------------------------------------------------------------------
// Zara, Renegade Recruiter
// ---------------------------------------------------------------------------------

/// P0's Zara attacks P1; P0 puts P1's `name` from P1's hand onto the battlefield attacking.
fn zara(name: &str) -> (TestGame, ObjectId) {
    supported("Zara, Renegade Recruiter");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let z = t.battlefield(P0, "Zara, Renegade Recruiter");
    let card = t.hand(P1, name);
    t.answer_choose(P0, &[obj(card)]);
    attack_with(&mut t, &[(z, Entity::Player(P1))]);
    t.resolve_all();
    let c = mine(&mut t, P0, name);
    assert_eq!(c.len(), 1, "{name} put onto the battlefield");
    (t, c[0])
}

fn attacking(t: &TestGame, id: ObjectId) -> bool {
    t.g.combat
        .as_ref()
        .is_some_and(|c| c.attackers.iter().any(|a| a.id == t.g.current(id)))
}

#[test]
fn zara_creature_wasnt_declared_as_an_attacker() {
    cr!("508.4", "508.3a");
    ruling!(
        "Zara, Renegade Recruiter",
        "Although the creature is attacking, it was never declared as an attacking creature."
    );
    let (t, hero) = zara("Hero of Bladehold");
    assert!(attacking(&t, hero));
    assert!(
        crate::r_s01_common::tokens(&t, P0).is_empty(),
        "Hero's attack trigger didn't trigger"
    );
}

#[test]
fn zara_can_put_a_defender_onto_the_battlefield_attacking() {
    cr!("508.4", "702.3b");
    ruling!(
        "Zara, Renegade Recruiter",
        "Any effects that say the creature can't attack affect only the declaration of attackers."
    );
    let (t, wall) = zara("Wall of Omens");
    assert!(attacking(&t, wall));
    assert!(tapped(&t, wall));
}

// ---------------------------------------------------------------------------------
// Profane Procession // Tomb of the Dusk Rose
// ---------------------------------------------------------------------------------

#[test]
fn tomb_of_the_dusk_rose_only_returns_cards_exiled_by_profane_procession() {
    cr!("607.2a", "712.8");
    ruling!(
        "Profane Procession // Tomb of the Dusk Rose",
        "If Profane Procession or Tomb of the Dusk Rose somehow has exiled cards other than with Profane Procession's ability, those exiled cards aren't linked to Tomb of the Dusk Rose's second ability."
    );
    supported("Profane Procession // Tomb of the Dusk Rose");
    let mut t = TestGame::new(2);
    let pp = t.battlefield(P0, "Profane Procession // Tomb of the Dusk Rose");
    // A card exiled some other way.
    t.exile(P1, "Serra Angel");
    for name in ["Grizzly Bears", "Hill Giant", "Craw Wurm"] {
        let c = t.battlefield(P1, name);
        crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::W, 1);
        crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::B, 1);
        crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 3);
        t.answer_targets(P0, &[obj(c)]);
        activate_containing(&mut t, P0, pp, "Exile target creature").expect("activate");
        t.resolve_all();
    }
    let tomb = t.g.current(pp);
    assert_eq!(t.obj_now(tomb).chars.name.as_str(), "Tomb of the Dusk Rose");
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::W, 1);
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::B, 1);
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 2);
    let from = t.asked().len();
    activate_containing(&mut t, P0, tomb, "onto the battlefield").expect("activate");
    t.resolve_all();
    let offered = entity_choices_since(&t, P0, from);
    let names: Vec<String> = offered
        .iter()
        .flatten()
        .filter_map(|e| e.object())
        .map(|o| t.g.obj(o).chars.name.to_string())
        .collect();
    assert!(!names.is_empty(), "offered the linked cards");
    assert!(!names.iter().any(|n| n == "Serra Angel"), "{names:?}");
    assert!(mine(&mut t, P0, "Serra Angel").is_empty());
    let returned = ["Grizzly Bears", "Hill Giant", "Craw Wurm"]
        .iter()
        .filter(|n| !mine(&mut t, P0, n).is_empty())
        .count();
    assert_eq!(returned, 1);
}

// ---------------------------------------------------------------------------------
// Gather Specimens
// ---------------------------------------------------------------------------------

/// It's P1's precombat main phase; P0 casts Gather Specimens and it resolves.
fn specimens(n: usize) -> TestGame {
    supported("Gather Specimens");
    let mut t = if n == 2 { TestGame::new(2) } else { ffa(n) };
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Gather Specimens", &[]);
    t.set_step(P1, Step::PrecombatMain);
    t
}

#[test]
fn gather_specimens_takes_tokens_and_reanimated_creatures_of_every_opponent() {
    cr!("614.12", "614.1c");
    ruling!(
        "Gather Specimens",
        "Gather Specimens affects both token creatures and nontoken creatures. It affects creatures that would enter by any means."
    );
    ruling!(
        "Gather Specimens",
        "Gather Specimens isn't targeted. It affects creatures that would enter under any opponent's control."
    );
    let mut t = specimens(3);
    // P1 creates a token.
    cast_resolve(&mut t, P1, "Call of the Herd", &[]);
    assert_eq!(mine(&mut t, P0, "Elephant Token").len(), 1);
    // P1 reanimates a creature.
    let giant = t.graveyard(P1, "Hill Giant");
    cast_resolve(&mut t, P1, "Zombify", &[obj(giant)]);
    assert_eq!(mine(&mut t, P0, "Hill Giant").len(), 1);
    // P2 (another opponent) casts a creature.
    t.set_step(P2, Step::PrecombatMain);
    give_mana_for(&mut t, P2, "Grizzly Bears");
    let bears = t.hand(P2, "Grizzly Bears");
    t.cast(P2, bears).go();
    t.resolve_all();
    assert_eq!(mine(&mut t, P0, "Grizzly Bears").len(), 1);
}

#[test]
fn gather_specimens_isnt_retroactive() {
    cr!("614.12", "611.2a");
    ruling!(
        "Gather Specimens",
        "Gather Specimens won't retroactively change the control of creatures that have already entered that turn."
    );
    supported("Gather Specimens");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let early = t.enter(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Gather Specimens", &[]);
    assert_eq!(controller(&mut t, early), P1);
    let late = t.enter(P1, "Hill Giant");
    assert_eq!(controller(&mut t, late), P0);
}

#[test]
fn gather_specimens_kicked_creature_spell_and_its_etb_trigger_are_yours() {
    cr!("614.12", "603.3a", "702.33d");
    ruling!(
        "Gather Specimens",
        "If a creature spell controlled by an opponent would resolve, it resolves, but the creature enters under your control instead of the opponent's control. Choices made when casting that spell (such as whether it was kicked or the value of X in the spell's cost) are remembered."
    );
    supported("Gatekeeper of Malakir");
    let mut t = specimens(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1 casts Gatekeeper of Malakir kicked.
    t.lands(P1, "Swamp", 3);
    let gk = t.hand(P1, "Gatekeeper of Malakir");
    t.cast(P1, gk).kicked(true).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(mine(&mut t, P0, "Gatekeeper of Malakir").len(), 1);
    // P0 controlled the kicked ETB trigger and made P1 sacrifice.
    assert!(!t.on_battlefield(bears), "P1 sacrificed the Bears");
}

#[test]
fn gather_specimens_unearths_delayed_trigger_stays_with_the_opponent() {
    cr!("603.7d", "614.12");
    ruling!(
        "Gather Specimens",
        "If the effect that puts a creature onto the battlefield also creates a delayed triggered ability, Gather Specimens doesn't change who controls that ability."
    );
    let mut t = specimens(2);
    let zombie = t.graveyard(P1, "Dregscape Zombie");
    t.lands(P1, "Swamp", 1);
    activate_containing(&mut t, P1, zombie, "Unearth").expect("unearth");
    t.resolve_all();
    let now = t.g.current(zombie);
    assert_eq!(controller(&mut t, now), P0);
    t.advance_to_step(Step::End);
    t.settle();
    let ctrl: Vec<PlayerId> = t.g.stack.iter().map(|id| t.g.obj(*id).controller).collect();
    assert_eq!(ctrl, vec![P1], "P1 controls the delayed exile trigger");
    t.resolve_all();
    assert_eq!(t.zone(now), Zone::Exile);
}

#[test]
fn gather_specimens_two_copies_each_would_be_controller_chooses() {
    cr!("616.1", "614.12", "101.4");
    ruling!(
        "Gather Specimens",
        "If two or more players have each cast Gather Specimens during the same turn and a creature would enter, the creature's would-be controller (the controller of the creature spell, for example) chooses one of the applicable Gather Specimens to apply."
    );
    let mut outcomes = vec![];
    for pick in 0..2 {
        let mut t = specimens(3);
        cast_resolve(&mut t, P1, "Gather Specimens", &[]);
        t.set_step(P2, Step::PrecombatMain);
        t.answer(P2, DecisionKind::Replacement, Answer::Index(pick));
        let from = t.asked().len();
        let bears = t.enter(P2, "Grizzly Bears");
        let chosen = t.asked()[from..]
            .iter()
            .find_map(|(p, d)| match d {
                Decision::ChooseReplacement { options } if *p == P2 => Some(options.clone()),
                _ => None,
            })
            .expect("P2 chooses");
        assert_eq!(chosen.len(), 2);
        // Whichever P2 applies first, the other then applies too: the would-be
        // controller is now that Gather Specimens' controller, an opponent of the other.
        outcomes.push(controller(&mut t, bears));
    }
    outcomes.sort();
    assert_eq!(outcomes, vec![P0, P1], "P2's choice decides");
}
