//! Event attribution (gap-event-causes): who puts counters on a permanent or player and
//! whether they're put by an effect, as a cost, as the result of damage, or by a
//! turn-based action (CR 122.6, 122.6a, 609.1); and which spell or ability destroyed a
//! permanent or countered a spell (CR 701.6, 701.8), including the destruction an umbra
//! armor Aura takes on instead (CR 702.89a).

use crate::r_s01_common::*;
use mtg_engine::card::CardDef;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts the real card `name` for `p` (with lands for its mana cost) and resolves the
/// stack.
fn cast_and_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    t.cast_with(p, c, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"));
    t.resolve_all();
    c
}

/// The permanent named `name` that `p` controls.
fn controlled(t: &TestGame, p: PlayerId, name: &str) -> ObjectId {
    *t.named_on_battlefield(name)
        .iter()
        .find(|o| t.obj(**o).controller == p)
        .unwrap_or_else(|| panic!("{name} under {p:?}'s control"))
}

/// "Put a +1/+1 counter on target creature." for {0}.
fn growth() -> CardDef {
    custom_card(
        "Gift of Growth",
        "Instant",
        "{0}",
        None,
        "Put a +1/+1 counter on target creature.",
    )
}

/// P0 casts Nessian Wilds Ravager; P1 (the chosen opponent) pays tribute.
fn ravager_with_tribute(t: &mut TestGame) -> ObjectId {
    give_mana_for(t, P0, "Nessian Wilds Ravager");
    let c = t.hand(P0, "Nessian Wilds Ravager");
    t.cast(P0, c).go();
    t.settle();
    t.answer_yes(P1, true);
    t.resolve();
    controlled(t, P0, "Nessian Wilds Ravager")
}

#[test]
fn the_player_who_pays_tribute_puts_the_counters() {
    cr!("122.6", "122.6a", "702.104a");
    for name in [
        "Nessian Wilds Ravager",
        "All Will Be One",
        "Vorinclex, Monstrous Raider",
    ] {
        supported(name);
    }
    ruling!(
        "Nessian Wilds Ravager",
        "For effects that check which player put counters on the entering creature, the player chosen to pay tribute puts those counters on it, not the creature’s controller."
    );
    // P1 pays tribute, so P1 puts the six counters: P1's All Will Be One triggers ("Whenever
    // you put one or more counters on a permanent or player"), P0's doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "All Will Be One");
    t.battlefield(P1, "All Will Be One");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let ravager = ravager_with_tribute(&mut t);
    assert_eq!(t.counters(ravager, counters::PLUS1), 6);
    t.resolve_all();
    assert_eq!(t.life(P0), 20 - 6, "P1's All Will Be One dealt 6 to P0");
    assert_eq!(t.life(P1), 20, "P0's All Will Be One didn't trigger");

    // P1's Vorinclex: "If you would put one or more counters ...": doubled.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Vorinclex, Monstrous Raider");
    let ravager = ravager_with_tribute(&mut t);
    assert_eq!(t.counters(ravager, counters::PLUS1), 12);
    // P0's Vorinclex: "If an opponent would put ...": halved, though P0 controls the
    // creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vorinclex, Monstrous Raider");
    let ravager = ravager_with_tribute(&mut t);
    assert_eq!(t.counters(ravager, counters::PLUS1), 3);
}

#[test]
fn a_creature_entering_under_another_players_control_gets_its_counters_from_that_player() {
    cr!("122.6a");
    ruling!(
        "Generous Patron",
        "If a creature enters the battlefield with counters under another player's control, that player is the player who puts those counters on it, even if you control the spell or ability putting that creature onto the battlefield."
    );
    ruling!(
        "Innkeeper's Talent",
        "If a permanent enters with counters on it, the effect causing the permanent to be given counters may specify which player puts those counters on it. If the effect doesn't specify a player, the object's controller puts those counters on it."
    );
    supported("Generous Patron");
    let mut t = TestGame::new(2);
    // P0's Generous Patron: "Whenever you put one or more counters on a creature you don't
    // control, draw a card."
    t.battlefield(P0, "Generous Patron");
    // P0's spell returns P1's Star Pupil to the battlefield under its owner's control: P1
    // puts its +1/+1 counter on it.
    let pupil = t.graveyard(P1, "Star Pupil");
    let def = custom_card(
        "Return Favor",
        "Sorcery",
        "{0}",
        None,
        "Return target creature card from a graveyard to the battlefield under its owner's control.",
    );
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.cast_with(P0, spell, &[Entity::Object(pupil)]).unwrap();
    let hand = t.hand_size(P0);
    t.resolve_all();
    let pupil = controlled(&t, P1, "Star Pupil");
    assert_eq!(t.counters(pupil, counters::PLUS1), 1);
    assert_eq!(t.hand_size(P0), hand, "P0 didn't put the counter");
    // P0 putting a counter on that creature does draw.
    let spell = t.custom(P0, growth(), Zone::Hand(P0));
    t.cast_with(P0, spell, &[Entity::Object(pupil)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(pupil, counters::PLUS1), 2);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn support_on_two_creatures_you_dont_control_triggers_generous_patron_twice() {
    cr!("122.6", "701.41a");
    ruling!(
        "Generous Patron",
        "If you put one or more counters on multiple creatures you don't control at the same time, such as by supporting two different creatures you don't control, Generous Patron's last ability triggers for each of those creatures."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    cast_and_resolve(&mut t, P0, "Generous Patron", &[]);
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    assert_eq!(t.hand_size(P0), hand + 2);
}

/// P0's planeswalker `name` (cast with Doubling Season-like effects in play) and its
/// loyalty.
fn cast_walker(t: &mut TestGame, name: &str) -> (ObjectId, u32) {
    cast_and_resolve(t, P0, name, &[]);
    let w = controlled(t, P0, name);
    (w, t.counters(w, counters::LOYALTY))
}

#[test]
fn doubling_season_doubles_entering_loyalty_but_not_loyalty_costs() {
    cr!("122.6", "306.5b", "606.4", "609.1", "614.1c");
    ruling!(
        "Doubling Season",
        "Planeswalkers will enter with double the normal number of loyalty counters. However, if you activate an ability whose cost has you put loyalty counters on a planeswalker, the number you put on isn't doubled. This is because those counters are put on as a cost, not as an effect."
    );
    ruling!(
        "Doubling Season",
        "Doubling Season affects permanents that enter with counters."
    );
    supported("Doubling Season");
    supported("Ajani, Caller of the Pride");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let (ajani, loyalty) = cast_walker(&mut t, "Ajani, Caller of the Pride");
    assert_eq!(loyalty, 8, "enters with twice its printed 4");
    // +1: Put a +1/+1 counter on up to one target creature. The cost's loyalty counter
    // isn't doubled; the effect's +1/+1 counters are.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, ajani, 0, &[Entity::Object(bears)]).unwrap();
    assert_eq!(t.counters(ajani, counters::LOYALTY), 9);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    // A creature that enters with counters.
    cast_and_resolve(&mut t, P0, "Star Pupil", &[]);
    let pupil = controlled(&t, P0, "Star Pupil");
    assert_eq!(t.counters(pupil, counters::PLUS1), 2);
}

#[test]
fn two_doubling_seasons_quadruple_and_battles_enter_with_double_defense() {
    cr!("310.4b", "614.1c", "616.1");
    ruling!(
        "Doubling Season",
        "If there are two Doubling Seasons on the battlefield, then the number of tokens or counters is four times the original number. If there are three on the battlefield, then the number of tokens or counters is eight times the original number, and so on."
    );
    ruling!(
        "Doubling Season",
        "Battles will enter with double the normal number of defense counters."
    );
    supported("Invasion of Belenon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    t.battlefield(P0, "Doubling Season");
    cast_and_resolve(&mut t, P0, "Star Pupil", &[]);
    let pupil = controlled(&t, P0, "Star Pupil");
    assert_eq!(t.counters(pupil, counters::PLUS1), 4);
    // One Doubling Season: a battle (printed defense 5) enters with 10 defense counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    cast_and_resolve(&mut t, P0, "Invasion of Belenon", &[]);
    let battle = controlled(&t, P0, "Invasion of Belenon");
    assert_eq!(t.counters(battle, counters::DEFENSE), 10);
}

#[test]
fn if_you_would_put_counters_applies_to_loyalty_costs_too() {
    cr!("122.6", "606.4", "614.1a");
    supported("Vorinclex, Monstrous Raider");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vorinclex, Monstrous Raider");
    let (ajani, loyalty) = cast_walker(&mut t, "Ajani, Caller of the Pride");
    assert_eq!(loyalty, 8);
    // P0 puts the cost's loyalty counter on Ajani: "If you would put one or more counters
    // on a permanent" applies to a cost as well (it isn't limited to effects).
    t.activate(P0, ajani, 0, &[]).unwrap();
    assert_eq!(t.counters(ajani, counters::LOYALTY), 10);
}

#[test]
fn selesnya_loft_gardens_doubles_effects_not_costs_or_damage() {
    cr!("120.3d", "606.4", "609.1", "702.80a");
    ruling!(
        "Selesnya Loft Gardens",
        "Effects that place a counter on a permanent include that permanent entering. For example, a creature that normally enters with one or more +1/+1 counters will enter that many +1/+1 counters on it. A planeswalker will enter with twice its starting loyalty counters."
    );
    ruling!(
        "Selesnya Loft Gardens",
        "Adding loyalty counters to a planeswalker in order to activate a loyalty ability isn't an effect, so those counters are not doubled."
    );
    ruling!(
        "Selesnya Loft Gardens",
        "Any -1/-1 counters that are placed on a creature because of wither or infect are also not doubled."
    );
    supported("Selesnya Loft Gardens");
    supported("Boggart Ram-Gang");
    let mut t = TestGame::new(2);
    t.command(P0, "Selesnya Loft Gardens");
    t.g.recompute();
    cast_and_resolve(&mut t, P0, "Star Pupil", &[]);
    let pupil = controlled(&t, P0, "Star Pupil");
    assert_eq!(t.counters(pupil, counters::PLUS1), 2);
    let (ajani, loyalty) = cast_walker(&mut t, "Ajani, Caller of the Pride");
    assert_eq!(loyalty, 8);
    t.activate(P0, ajani, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(ajani, counters::LOYALTY), 9);
    // Wither damage to a creature (any player's): -1/-1 counters not doubled.
    let ram = t.battlefield(P1, "Boggart Ram-Gang");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(ram, Entity::Object(bears), 1, false);
    t.settle();
    assert_eq!(t.counters(bears, counters::MINUS1), 1);
    let ram2 = t.battlefield(P0, "Boggart Ram-Gang");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(ram2, Entity::Object(theirs), 1, false);
    t.settle();
    assert_eq!(t.counters(theirs, counters::MINUS1), 1);
}

#[test]
fn all_will_be_one_triggers_for_effects_entering_counters_and_damage() {
    cr!("120.3b", "120.3d", "122.6", "702.80a", "702.90b");
    ruling!(
        "All Will Be One",
        "All Will Be One's ability will trigger any time you put one or more counters on a permanent or player. This might be due to a spell or ability resolving, a permanent you control entering the battlefield with counters, or combat damage from a source with toxic, infect, or wither."
    );
    supported("Glistener Elf");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "All Will Be One");
    let opp = |t: &mut TestGame| {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    };
    // A spell resolving: two +1/+1 counters → 2 damage.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let def = custom_card(
        "Double Growth",
        "Instant",
        "{0}",
        None,
        "Put two +1/+1 counters on target creature.",
    );
    let spell = t.custom(P0, def, Zone::Hand(P0));
    opp(&mut t);
    t.cast_with(P0, spell, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Entering with counters.
    opp(&mut t);
    cast_and_resolve(&mut t, P0, "Star Pupil", &[]);
    assert_eq!(t.life(P1), 17);
    // Combat damage from infect (poison counters) and wither (-1/-1 counters).
    let elf = t.battlefield(P0, "Glistener Elf");
    let ram = t.battlefield(P0, "Boggart Ram-Gang");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    opp(&mut t);
    opp(&mut t);
    attack_with(
        &mut t,
        &[(elf, Entity::Player(P1)), (ram, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(blocker, ram)]);
    t.resolve_all();
    assert_eq!(t.g.player(P1).counter(counters::POISON), 1);
    assert_eq!(t.counters(blocker, counters::MINUS1), 0, "the blocker died");
    // 1 (one poison counter) + 3 (three -1/-1 counters on the blocker).
    assert_eq!(t.life(P1), 17 - 4);
}

#[test]
fn toxic_counters_from_simultaneous_combat_damage_are_one_event() {
    cr!("702.164c", "510.2");
    ruling!(
        "All Will Be One",
        "If more than one creature with toxic deals combat damage to a player at the same time, those counters are placed as a single event, and the ability triggers one time."
    );
    supported("Bloated Contaminator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "All Will Be One");
    let a = t.battlefield(P0, "Bloated Contaminator");
    let b = t.battlefield(P0, "Bloated Contaminator");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.g.player(P1).counter(counters::POISON), 2);
    assert_eq!(
        triggers_on_stack(&t, "Whenever you put one or more counters"),
        1,
        "one trigger for both toxic creatures' counters"
    );
    t.resolve_all();
    // 8 combat damage, then 2 from the one trigger.
    assert_eq!(t.life(P1), 20 - 8 - 2);
}

#[test]
fn kate_stewart_counts_time_counters_on_permanents_not_on_exiled_cards() {
    cr!("122.6", "702.62a");
    ruling!(
        "Kate Stewart",
        "Kate Stewart's first ability triggers only when you put time counters on permanents. It does not trigger when you put time counters on cards in exile."
    );
    supported("Kate Stewart");
    let mut t = TestGame::new(2);
    let kate = t.battlefield(P0, "Kate Stewart");
    let tokens_before = tokens(&t, P0).len();
    // Suspending a card puts time counters on it in exile: no trigger.
    let rift = t.hand(P0, "Rift Bolt");
    t.lands(P0, "Mountain", 1);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::Suspend { card: rift }))
        .expect("suspend Rift Bolt");
    t.resolve_all();
    assert!(t.counters(t.g.current(rift), counters::TIME) > 0);
    assert_eq!(tokens(&t, P0).len(), tokens_before);
    // A time counter on a permanent: triggers.
    t.g.add_counters(Entity::Object(kate), counters::TIME, 1, Some(kate));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), tokens_before + 1);
}

#[test]
fn exemplar_of_light_triggers_for_counters_from_any_source() {
    cr!("122.6");
    ruling!(
        "Exemplar of Light",
        "Exemplar of Light's last ability triggers whenever you put one or more +1/+1 counters on it for any reason, not just as a result of its second ability."
    );
    supported("Exemplar of Light");
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Exemplar of Light");
    let hand = t.hand_size(P0);
    let spell = t.custom(P0, growth(), Zone::Hand(P0));
    t.cast_with(P0, spell, &[Entity::Object(ex)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Next turn, an opponent putting a counter on it isn't you putting one.
    t.g.turn.number += 1;
    t.set_step(P1, Step::PrecombatMain);
    let spell = t.custom(P1, growth(), Zone::Hand(P1));
    t.cast_with(P1, spell, &[Entity::Object(ex)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(ex, counters::PLUS1), 2);
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------------------
// What destroyed or countered something (CR 701.6, 701.8)
// ---------------------------------------------------------------------------------------

/// `p` casts the real card `name` (with lands for its mana cost) at `targets`.
fn cast_at(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    t.cast_with(p, c, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"));
}

/// The number of Karmic Justice triggers waiting or on the stack.
fn karmic_triggers(t: &mut TestGame) -> usize {
    t.settle();
    triggers_on_stack(t, "a spell or ability an opponent controls destroys")
}

#[test]
fn karmic_justice_triggers_on_an_opponents_destroy_effects_only() {
    cr!("603.10a", "701.8a", "701.8b");
    ruling!(
        "Karmic Justice",
        "Spells and abilities that cause you to sacrifice permanents will not cause Karmic Justice's ability to trigger."
    );
    ruling!(
        "Karmic Justice",
        "If a spell or ability an opponent controls destroys Karmic Justice, Karmic Justice's ability will trigger."
    );
    supported("Karmic Justice");
    // An opponent's Shatter destroys P0's artifact: triggers; P0 may destroy target
    // permanent that opponent controls.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karmic Justice");
    let stone = t.battlefield(P0, "Mind Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Shatter", &[Entity::Object(stone)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Mind Stone"));
    assert_eq!(karmic_triggers(&mut t), 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));

    // Sacrificing (an opponent's "sacrifice an artifact") doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karmic Justice");
    t.battlefield(P0, "Mind Stone");
    t.set_step(P1, Step::PrecombatMain);
    let edict = custom_card(
        "Artifact Edict",
        "Sorcery",
        "{0}",
        None,
        "Target player sacrifices an artifact.",
    );
    let s = t.custom(P1, edict, Zone::Hand(P1));
    t.cast_with(P1, s, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Mind Stone"));
    assert_eq!(karmic_triggers(&mut t), 0);
    // Nor does exiling (Revoke Existence) or P0's own destroy effect.
    let stone = t.battlefield(P0, "Mind Stone");
    cast_at(&mut t, P1, "Revoke Existence", &[Entity::Object(stone)]);
    t.resolve();
    assert!(t.in_exile("Mind Stone"));
    assert_eq!(karmic_triggers(&mut t), 0);
    t.set_step(P0, Step::PrecombatMain);
    let stone = t.battlefield(P0, "Mind Stone");
    cast_at(&mut t, P0, "Shatter", &[Entity::Object(stone)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Mind Stone"));
    assert_eq!(karmic_triggers(&mut t), 0);

    // Destroying Karmic Justice itself: it looks back in time and triggers.
    let mut t = TestGame::new(2);
    let kj = t.battlefield(P0, "Karmic Justice");
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Naturalize", &[Entity::Object(kj)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Karmic Justice"));
    assert_eq!(karmic_triggers(&mut t), 1);
}

#[test]
fn the_spell_that_would_destroy_an_umbras_creature_destroys_the_umbra() {
    cr!("614.6", "701.8b", "702.89a", "704.5g");
    ruling!(
        "Penumbra Umbra",
        "If a spell or ability says that it would “destroy” a creature enchanted with an Aura that has umbra armor, that spell or ability is what causes the Aura to be destroyed instead. Umbra armor doesn’t destroy the Aura; rather, it changes the effects of the spell or ability. On the other hand, if a spell or ability deals lethal damage to a creature enchanted with an Aura that has umbra armor, the game rules regarding lethal damage cause the Aura to be destroyed, not that spell or ability."
    );
    supported("Penumbra Umbra");
    supported("Hyena Umbra");
    // P1's Murder would destroy P0's enchanted Bears: Murder destroys the Aura instead, so
    // Karmic Justice (the Aura is a noncreature permanent P0 controls) triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karmic Justice");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = t.battlefield(P0, "Penumbra Umbra");
    assert!(t.g.attach(umbra, Entity::Object(bears)));
    t.g.recompute();
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Murder", &[Entity::Object(bears)]);
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Penumbra Umbra"));
    assert_eq!(karmic_triggers(&mut t), 1);

    // P1's Lightning Bolt deals lethal damage: the state-based action, not the Bolt,
    // destroys the Aura: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karmic Justice");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = t.battlefield(P0, "Hyena Umbra");
    assert!(t.g.attach(umbra, Entity::Object(bears)));
    t.g.recompute();
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Lightning Bolt", &[Entity::Object(bears)]);
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Hyena Umbra"));
    assert_eq!(karmic_triggers(&mut t), 0);
}

/// Cobra Trap's alternative cost ({G}) is available to P0 now.
fn cobra_trap_alternative(t: &mut TestGame, trap: ObjectId) -> bool {
    t.set_step(P0, Step::PrecombatMain);
    crate::r_s07_common::cast_methods(t, P0, trap)
        .iter()
        .any(|m| matches!(m, mtg_engine::object::CastMethod::Alternative(_)))
}

#[test]
fn cobra_traps_condition_needs_an_opponents_destroy_effect() {
    cr!("118.9", "701.8b");
    ruling!(
        "Cobra Trap",
        "A spell or ability destroys a permanent only if that spell or ability specifically contains the word “destroy” in its text. If a spell or ability an opponent controls exiles a noncreature permanent you control, causes you to sacrifice a noncreature permanent, removes all loyalty counters from a planeswalker you control, or causes an ability you control to trigger (and then that ability destroys a permanent you control), Cobra Trap’s alternative cost condition hasn’t been met."
    );
    supported("Cobra Trap");
    supported("Liliana of the Veil");
    let mut t = TestGame::new(2);
    let trap = t.hand(P0, "Cobra Trap");
    t.lands(P0, "Forest", 1);
    // P1 exiles P0's artifact, makes P0 sacrifice one, and burns P0's planeswalker to 0
    // loyalty: not destroyed.
    let stone = t.battlefield(P0, "Mind Stone");
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Revoke Existence", &[Entity::Object(stone)]);
    t.resolve_all();
    assert!(t.in_exile("Mind Stone"));
    t.battlefield(P0, "Mind Stone");
    let edict = custom_card(
        "Artifact Edict",
        "Sorcery",
        "{0}",
        None,
        "Target player sacrifices an artifact.",
    );
    t.set_step(P1, Step::PrecombatMain);
    let s = t.custom(P1, edict, Zone::Hand(P1));
    t.cast_with(P1, s, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mind Stone"));
    let lili = t.battlefield(P0, "Liliana of the Veil");
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Lightning Bolt", &[Entity::Object(lili)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Liliana of the Veil"));
    assert!(!cobra_trap_alternative(&mut t, trap));
    // P1's spell makes P0's own ability trigger, and that ability destroys P0's artifact.
    let watcher = custom_card(
        "Self Wrecker",
        "Enchantment",
        "{0}",
        None,
        "Whenever an opponent casts a spell, destroy target artifact you control.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    let stone = t.battlefield(P0, "Mind Stone");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(stone)]);
    let bears = t.hand(P1, "Grizzly Bears");
    give_mana_for(&mut t, P1, "Grizzly Bears");
    t.cast_with(P1, bears, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mind Stone"));
    assert!(!cobra_trap_alternative(&mut t, trap));
    // P1's Shatter destroys one: the condition is met.
    let mut t = TestGame::new(2);
    let trap = t.hand(P0, "Cobra Trap");
    t.lands(P0, "Forest", 1);
    let stone = t.battlefield(P0, "Mind Stone");
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Shatter", &[Entity::Object(stone)]);
    t.resolve_all();
    assert!(cobra_trap_alternative(&mut t, trap));
    // Cast for {G}.
    let alt = crate::r_s07_common::cast_methods(&mut t, P0, trap)
        .into_iter()
        .find(|m| matches!(m, mtg_engine::object::CastMethod::Alternative(_)))
        .unwrap();
    t.cast(P0, trap).method(alt).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Snake").len(), 4);
}

#[test]
fn cobra_trap_doesnt_count_a_destroyed_creature() {
    // Cobra Trap needs a noncreature permanent: a creature destroyed by an opponent's
    // spell doesn't count.
    cr!("701.8a", "701.8b");
    let mut t = TestGame::new(2);
    let trap = t.hand(P0, "Cobra Trap");
    t.lands(P0, "Forest", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Murder", &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!cobra_trap_alternative(&mut t, trap));
}

/// Summoning Trap's alternative cost ({0}) is available to P0 now.
fn summoning_trap_alternative(t: &mut TestGame, trap: ObjectId) -> bool {
    cobra_trap_alternative(t, trap)
}

#[test]
fn summoning_trap_needs_your_creature_spell_countered_by_an_opponent() {
    cr!("118.9", "701.6a");
    supported("Summoning Trap");
    supported("Essence Scatter");
    // P0's own counterspell on P0's creature spell doesn't count.
    let mut t = TestGame::new(2);
    let trap = t.hand(P0, "Summoning Trap");
    give_mana_for(&mut t, P0, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast_with(P0, bears, &[]).unwrap();
    cast_at(&mut t, P0, "Essence Scatter", &[Entity::Object(spell)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!summoning_trap_alternative(&mut t, trap));
    // An opponent's Essence Scatter does.
    give_mana_for(&mut t, P0, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast_with(P0, bears, &[]).unwrap();
    cast_at(&mut t, P1, "Essence Scatter", &[Entity::Object(spell)]);
    t.resolve_all();
    assert!(summoning_trap_alternative(&mut t, trap));
    // Next turn, it no longer does.
    t.g.turn.number += 1;
    t.g.turn_events.clear();
    assert!(!summoning_trap_alternative(&mut t, trap));
}

#[test]
fn baral_triggers_when_your_spell_counters_a_spell_not_when_one_fizzles() {
    cr!("608.2b", "701.6a");
    ruling!(
        "Baral, Chief of Compliance",
        "A spell or ability counters a spell only if it specifically contains the word \"counter\" in its text. If a spell or ability you control causes all the targets of a spell to become illegal, that spell doesn't resolve but it's not countered."
    );
    supported("Baral, Chief of Compliance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Baral, Chief of Compliance");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // P1's Giant Growth on its Bears; P0 bounces the Bears in response: the Growth
    // doesn't resolve, but it isn't countered.
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Giant Growth");
    let growth = t.hand(P1, "Giant Growth");
    t.cast_with(P1, growth, &[Entity::Object(theirs)]).unwrap();
    cast_at(&mut t, P0, "Unsummon", &[Entity::Object(theirs)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Giant Growth"));
    // P0's Cancel counters P1's spell: P0 may loot.
    give_mana_for(&mut t, P1, "Lightning Bolt");
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast_with(P1, bolt, &[Entity::Player(P0)]).unwrap();
    let asked = t.asked().len();
    cast_at(&mut t, P0, "Cancel", &[Entity::Object(spell)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 20);
    let looted = asked_since(&t, asked)
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
        .count();
    assert_eq!(looted, 1, "Baral triggered once (for Cancel, not for the fizzle)");
}

#[test]
fn lullmage_mentors_counter_ability_triggers_its_first_ability() {
    cr!("701.6a");
    ruling!(
        "Lullmage Mentor",
        "Lullmage Mentor’s second ability will cause its first ability to trigger."
    );
    ruling!(
        "Lullmage Mentor",
        "A spell or ability counters a spell only if it specifically contains the word “counter” in its text. If a spell or ability you control causes all the targets of a spell to become illegal, that spell doesn’t resolve but is not countered."
    );
    supported("Lullmage Mentor");
    let mut t = TestGame::new(2);
    let mentor = t.battlefield(P0, "Lullmage Mentor");
    for _ in 0..6 {
        t.battlefield(P0, "Merfolk of the Pearl Trident");
    }
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // A fizzle isn't a counter.
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Giant Growth");
    let growth = t.hand(P1, "Giant Growth");
    t.cast_with(P1, growth, &[Entity::Object(theirs)]).unwrap();
    cast_at(&mut t, P0, "Unsummon", &[Entity::Object(theirs)]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 0);
    // "Tap seven untapped Merfolk you control: Counter target spell."
    give_mana_for(&mut t, P1, "Lightning Bolt");
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast_with(P1, bolt, &[Entity::Player(P0)]).unwrap();
    t.activate(P0, mentor, 0, &[Entity::Object(spell)]).unwrap();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(with_subtype(&t, P0, "Merfolk").len(), 8, "a Merfolk token");
}

// ---------------------------------------------------------------------------------------
// Counters put this turn, on a permanent as it was then
// ---------------------------------------------------------------------------------------

/// `p` puts `n` +1/+1 counters on `on` with a resolving ability of `source` (a permanent
/// `p` controls).
fn put_plus1(t: &mut TestGame, source: ObjectId, on: ObjectId, n: u32) {
    t.g.add_counters(Entity::Object(on), counters::PLUS1, n, Some(source));
    t.g.flush_events();
    t.resolve_all();
    t.g.recompute();
}

/// Whether `id` has the keyword now.
fn has(t: &TestGame, id: ObjectId, kw: mtg_engine::keywords::KeywordKind) -> bool {
    t.obj_now(id).has_keyword(kw)
}

#[test]
fn sigardian_paladin_looks_at_what_the_permanent_was_when_the_counter_was_put_on() {
    use mtg_engine::keywords::KeywordKind;
    cr!("122.6", "611.3a");
    ruling!(
        "Sigardian Paladin",
        "Sigardian Paladin's first ability applies as long as you put a +1/+1 counter on a permanent this turn and that permanent was a creature at the time you put the counter on. It doesn't matter if that creature later left the battlefield, lost its counters, or somehow stopped being a creature."
    );
    supported("Sigardian Paladin");
    supported("Mutavault");
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Sigardian Paladin");
    assert!(!has(&t, paladin, KeywordKind::Trample));
    // A +1/+1 counter on a land that isn't a creature, which then becomes one: no.
    let vault = t.battlefield(P0, "Mutavault");
    put_plus1(&mut t, paladin, vault, 1);
    t.lands(P0, "Wastes", 1);
    crate::r_s06_common::activate_containing(&mut t, P0, vault, "becomes").unwrap();
    t.resolve_all();
    assert!(t.obj_now(vault).is(CardType::Creature));
    assert!(!has(&t, paladin, KeywordKind::Trample));
    // An opponent putting one on P0's creature: no.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    put_plus1(&mut t, theirs, bears, 1);
    assert!(!has(&t, paladin, KeywordKind::Lifelink));
    // P0 puts one on a creature, which then leaves the battlefield and loses its counters:
    // yes.
    put_plus1(&mut t, paladin, bears, 1);
    assert!(has(&t, paladin, KeywordKind::Trample));
    assert!(has(&t, paladin, KeywordKind::Lifelink));
    t.g.destroy(bears, None);
    t.settle();
    t.g.recompute();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(has(&t, paladin, KeywordKind::Trample));
    // Next turn, it no longer applies.
    t.g.turn.number += 1;
    t.g.turn_events.clear();
    t.g.recompute();
    assert!(!has(&t, paladin, KeywordKind::Trample));
}

/// Advances to the end step of `active`'s turn and puts the triggered abilities on the
/// stack; returns how many are Fairgrounds Trumpeter's.
fn trumpeter_triggers_at_end_step(t: &mut TestGame, active: PlayerId) -> usize {
    t.advance_to(active, Step::End);
    t.settle();
    triggers_on_stack(t, "if a +1/+1 counter was put on a permanent under your control")
}

#[test]
fn fairgrounds_trumpeter_counts_permanents_you_controlled_as_the_counter_was_placed() {
    cr!("122.6", "603.4");
    ruling!(
        "Fairgrounds Trumpeter",
        "Fairgrounds Trumpeter's ability triggers if, at any point during this turn, a +1/+1 counter was placed on a permanent that you controlled as the counter was placed. It doesn't matter whether you still control the permanent or whether it still has a counter."
    );
    supported("Fairgrounds Trumpeter");
    // A counter on P0's Bears, which P1 then gains control of: triggers.
    let mut t = TestGame::new(2);
    let trumpeter = t.battlefield(P0, "Fairgrounds Trumpeter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_plus1(&mut t, trumpeter, bears, 1);
    crate::r_s06_common::give_control(&mut t, bears, P1);
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(trumpeter_triggers_at_end_step(&mut t, P0), 1);
    t.resolve_all();
    assert_eq!(t.counters(trumpeter, counters::PLUS1), 1);
    // A counter on P1's Bears, which P0 then gains control of: doesn't.
    let mut t = TestGame::new(2);
    let trumpeter = t.battlefield(P0, "Fairgrounds Trumpeter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_plus1(&mut t, trumpeter, bears, 1);
    crate::r_s06_common::give_control(&mut t, bears, P0);
    assert_eq!(trumpeter_triggers_at_end_step(&mut t, P0), 0);
}

#[test]
fn fairgrounds_trumpeter_doesnt_see_counters_put_during_the_end_step() {
    cr!("603.4");
    ruling!(
        "Fairgrounds Trumpeter",
        "If a +1/+1 counter hasn't been placed yet at the moment an end step begins, Fairgrounds Trumpeter's ability doesn't trigger at all. If another ability triggers during the end step and puts a +1/+1 counter on a permanent you control, you won't put an additional +1/+1 counter on Fairgrounds Trumpeter."
    );
    let mut t = TestGame::new(2);
    let trumpeter = t.battlefield(P0, "Fairgrounds Trumpeter");
    // "At the beginning of your end step, put a +1/+1 counter on target creature you
    // control."
    let def = custom_card(
        "End Step Grower",
        "Enchantment",
        "{0}",
        None,
        "At the beginning of your end step, put a +1/+1 counter on target creature you control.",
    );
    t.custom(P0, def, Zone::Battlefield);
    t.answer_targets(P0, &[Entity::Object(trumpeter)]);
    assert_eq!(trumpeter_triggers_at_end_step(&mut t, P0), 0);
    t.resolve_all();
    assert_eq!(t.counters(trumpeter, counters::PLUS1), 1);
    assert_eq!(triggers_on_stack(&t, "if a +1/+1 counter was put"), 0);
}

#[test]
fn lord_jyscal_guado_and_lasting_tarfire_check_as_the_end_step_begins() {
    cr!("603.4");
    ruling!(
        "Lord Jyscal Guado",
        "Lord Jyscal Guado's last ability checks at the moment it would trigger to see if you put a counter on a creature this turn. If you didn't, the ability won't trigger at all. Once your end step begins, it's too late to put a counter on a creature in order to cause this ability to trigger."
    );
    ruling!(
        "Lasting Tarfire",
        "Lasting Tarfire's ability will check as your end step starts to see if you put a counter on a creature this turn. If you didn't, the ability won't trigger at all. Putting a counter on a creature during your end step won't cause the ability to trigger."
    );
    supported("Lord Jyscal Guado");
    supported("Lasting Tarfire");
    // No counter put this turn: neither triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lord Jyscal Guado");
    t.battlefield(P0, "Lasting Tarfire");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A counter put on any creature (even an opponent's) during the turn: both trigger.
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Lord Jyscal Guado");
    t.battlefield(P0, "Lasting Tarfire");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(theirs), counters::MINUS1, 1, Some(lord));
    t.g.flush_events();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(with_subtype(&t, P0, "Clue").len(), 1);
    // A counter P1 put: no.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lord Jyscal Guado");
    t.battlefield(P0, "Lasting Tarfire");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(theirs), counters::PLUS1, 1, Some(theirs));
    t.g.flush_events();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn putting_counters_on_several_creatures_at_once_triggers_for_each() {
    cr!("122.6", "603.2c");
    ruling!(
        "Hapatra, Vizier of Poisons",
        "If you put any number of -1/-1 counters on more than one creature at once, Hapatra's last ability triggers once for each of those creatures."
    );
    ruling!(
        "Hapatra, Vizier of Poisons",
        "If you put enough -1/-1 counters on Hapatra so that its toughness is 0 or less, its last ability triggers."
    );
    ruling!(
        "Obelisk Spider",
        "If you put one or more -1/-1 counter on each of multiple creatures at the same time, Obelisk Spider’s last ability triggers once for each of those creatures."
    );
    supported("Hapatra, Vizier of Poisons");
    supported("Obelisk Spider");
    supported("Black Sun's Zenith");
    // Black Sun's Zenith with X = 2 on Hapatra (2/2), Obelisk Spider (1/4) and two of
    // P1's creatures: four creatures get counters at once.
    let mut t = TestGame::new(2);
    let hapatra = t.battlefield(P0, "Hapatra, Vizier of Poisons");
    t.battlefield(P0, "Obelisk Spider");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 2);
    let zenith = t.hand(P0, "Black Sun's Zenith");
    t.cast(P0, zenith).x(2).go();
    t.resolve();
    // Hapatra's toughness is 0, but it was on the battlefield as the counters were put on
    // it: its ability triggered for each of the four creatures, Obelisk Spider's too.
    assert!(!t.on_battlefield(hapatra));
    assert_eq!(
        triggers_on_stack(&t, "create a 1/1 green Snake creature token"),
        4
    );
    assert_eq!(
        triggers_on_stack(&t, "each opponent loses 1 life and you gain 1 life"),
        4
    );
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Snake").len(), 4);
    assert_eq!((t.life(P0), t.life(P1)), (24, 16));
}

#[test]
fn wakka_checks_as_the_end_step_begins_whoever_put_the_counter() {
    cr!("603.4");
    ruling!(
        "Wakka, Devoted Guardian",
        "Wakka's last ability checks at the moment it would trigger to see if a counter was put on Wakka this turn. If none were, the ability won't trigger at all. Once your end step begins, it's too late to put a counter on Wakka in order to cause this ability to trigger."
    );
    supported("Wakka, Devoted Guardian");
    // No counter was put on Wakka this turn: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wakka, Devoted Guardian");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    // An opponent's ability puts a counter on Wakka ("a counter was put on Wakka", by
    // anyone): it triggers.
    let mut t = TestGame::new(2);
    let wakka = t.battlefield(P0, "Wakka, Devoted Guardian");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(wakka), counters::MINUS1, 1, Some(theirs));
    t.g.flush_events();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn stocking_the_pantry_counts_counters_you_put_on_your_creatures() {
    cr!("122.6");
    supported("Stocking the Pantry");
    // "Whenever you put one or more +1/+1 counters on a creature you control, put a supply
    // counter on this enchantment."
    let mut t = TestGame::new(2);
    let pantry = t.battlefield(P0, "Stocking the Pantry");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // P1 puts counters on P0's creature, P0 on P1's: no.
    put_plus1(&mut t, theirs, mine, 2);
    put_plus1(&mut t, pantry, theirs, 1);
    assert_eq!(t.counters(pantry, "supply"), 0);
    // P0 puts two counters on P0's creature: one trigger.
    put_plus1(&mut t, pantry, mine, 2);
    assert_eq!(t.counters(pantry, "supply"), 1);
    // A creature entering under P0's control with a counter: P0 put it.
    cast_and_resolve(&mut t, P0, "Star Pupil", &[]);
    assert_eq!(t.counters(pantry, "supply"), 2);
}

/// P0 casts "Put `n` -1/-1 counters on target creature." (`n` in words) at `target`.
fn cast_minus1(t: &mut TestGame, target: ObjectId, n: &str) {
    let def = custom_card(
        "Wither Away",
        "Instant",
        "{0}",
        None,
        &format!("Put {n} -1/-1 counters on target creature."),
    );
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.cast_with(P0, spell, &[Entity::Object(target)]).unwrap();
}

/// The number of -1/-1 counters put on `obj` by each counters event of this turn.
fn minus1_events(t: &TestGame, obj: ObjectId) -> Vec<u32> {
    t.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            mtg_engine::events::Event::CountersAdded {
                target: Entity::Object(o),
                kind,
                n,
                ..
            } if *o == obj && kind.as_str() == counters::MINUS1 => Some(*n),
            _ => None,
        })
        .collect()
}

#[test]
fn nest_of_scarabs_counts_every_counter_put_even_past_toughness() {
    cr!("122.6");
    ruling!(
        "Nest of Scarabs",
        "If an effect has you put more -1/-1 counters on a creature than it has toughness, you’ll put all of those counters on it and create that many Insects, even if that makes its toughness a negative number."
    );
    supported("Nest of Scarabs");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nest of Scarabs");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Three -1/-1 counters on a 2/2: all three are put on it.
    cast_minus1(&mut t, bears, "three");
    t.resolve_all();
    assert_eq!(minus1_events(&t, bears), vec![3]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(with_subtype(&t, P0, "Insect").len(), 3);
}

#[test]
fn defiant_greatmaw_triggers_on_the_counters_that_kill_it() {
    cr!("122.6", "704.5f");
    ruling!(
        "Defiant Greatmaw",
        "If you put enough -1/-1 counters on Defiant Greatmaw so that its toughness is 0 or less, its last ability triggers."
    );
    supported("Defiant Greatmaw");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Defiant Greatmaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::MINUS1, 1, Some(bears));
    // Five -1/-1 counters on the 4/5: it dies, and its ability triggers.
    cast_minus1(&mut t, maw, "five");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert_eq!(minus1_events(&t, maw), vec![5]);
    assert!(t.in_graveyard(P0, "Defiant Greatmaw"));
    assert_eq!(
        triggers_on_stack(&t, "remove a -1/-1 counter from another target creature"),
        1
    );
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::MINUS1), 0);
}

#[test]
fn earth_kingdom_general_stops_triggering_once_you_gain_life() {
    cr!("603.2h");
    ruling!(
        "Earth Kingdom General",
        "Once you choose to gain life using Earth Kingdom General's second ability, that ability won't trigger again that turn."
    );
    supported("Earth Kingdom General");
    let mut t = TestGame::new(2);
    let general = t.battlefield(P0, "Earth Kingdom General");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let trigger = "you may gain that much life";
    // Declined: it triggers again.
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 2, Some(general));
    t.settle();
    assert_eq!(triggers_on_stack(&t, trigger), 1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 3, Some(general));
    t.settle();
    assert_eq!(triggers_on_stack(&t, trigger), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Once you've gained life, it no longer triggers this turn.
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 1, Some(general));
    t.settle();
    assert_eq!(triggers_on_stack(&t, trigger), 0);
}
