//! Item `rulings-residue-misc-1` — spells and abilities on the stack, targets, casting
//! permissions and soft counters: Lightning Storm, Reflecting Mirror, Silver Wyvern,
//! Whirlwind Denial, Power Sink, Equinox, Academic Probation, Gauntlets of Chaos, Crooked
//! Scales, Isildur's Fateful Strike, Swift Reckoning, Graveyard Shift, Xiahou Dun, Mind
//! Extraction, Nebuchadnezzar and Creeping Dread.

use crate::r_p016_common::{cast_at, lots, stack_targets, target_choices};
use crate::r_residue_misc_1_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The uid of the first activated ability of `source` whose text contains `needle`.
fn ability_uid(t: &TestGame, source: ObjectId, needle: &str) -> u64 {
    t.g.obj(source)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
        .map(|a| a.uid)
        .expect("no such ability")
}

/// Whether the spell (followed across zone changes) is still on the stack.
fn on_stack(t: &TestGame, id: ObjectId) -> bool {
    t.zone(id) == Zone::Stack
}

// ---------------------------------------------------------------------------------------
// Lightning Storm: "Lightning Storm deals X damage to any target, where X is 3 plus the
// number of charge counters on Lightning Storm. / Discard a land card: Put two charge
// counters on Lightning Storm. You may choose a new target for it. Any player may
// activate this ability but only if Lightning Storm is on the stack."
// ---------------------------------------------------------------------------------------

/// P1 casts Lightning Storm at P0 during P0's turn, through the priority loop (P0 passes
/// first). Returns the spell.
fn p1_storms_p0(t: &mut TestGame) -> ObjectId {
    t.lands(P1, "Mountain", 3);
    let storm = t.hand(P1, "Lightning Storm");
    t.g.take_action(P0, Action::Pass);
    assert_eq!(t.g.turn.priority, Some(P1));
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.g.take_action(
        P1,
        Action::Cast {
            card: storm,
            method: CastMethod::Normal,
        },
    );
    t.g.current(storm)
}

#[test]
fn lightning_storm_the_active_player_gets_priority_after_its_ability_resolves() {
    cr!("117.3b", "602.5");
    ruling!(
        "Lightning Storm",
        "After an instance of Lightning Storm’s activated ability resolves, the active player (not Lightning Storm’s controller) receives priority."
    );
    ruling!(
        "Lightning Storm",
        "When you cast Lightning Storm, you immediately have a chance to activate its ability since you’ll have priority. If you don’t, and all other players pass, the spell will resolve. You won’t receive priority again — and thus won’t get another chance to activate its ability — unless someone else activates Lightning Storm’s activated ability, casts some other spell, or activates some other ability."
    );
    supported("Lightning Storm");
    let mut t = TestGame::new(2);
    let spell = p1_storms_p0(&mut t);
    // Its caster has priority right away and may activate it.
    assert_eq!(t.g.turn.priority, Some(P1));
    let land = t.hand(P1, "Mountain");
    t.answer_choose(P1, &[obj(land)]);
    t.answer_yes(P1, false);
    let uid = ability_uid(&t, spell, "Discard a land");
    t.g.take_action(
        P1,
        Action::Activate {
            source: spell,
            ability: uid,
        },
    );
    assert_eq!(t.stack_len(), 2);
    t.g.take_action(P1, Action::Pass);
    t.g.take_action(P0, Action::Pass);
    // The ability resolved: P0, the active player, has priority.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.counters(spell, counters::CHARGE), 2);
    assert_eq!(t.g.turn.priority, Some(P0));
    // Both pass again: the spell resolves; P1 got no other chance.
    t.g.take_action(P0, Action::Pass);
    t.g.take_action(P1, Action::Pass);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 15);
}

#[test]
fn lightning_storm_any_player_may_activate_it_while_its_on_the_stack() {
    cr!("602.5", "113.6", "608.1");
    ruling!(
        "Lightning Storm",
        "Lightning Storm has an activated ability that can be activated only while the spell is on the stack. Any player may activate that ability. When the ability is activated, it goes on the stack on top of Lightning Storm. When the ability resolves, the Lightning Storm spell is affected."
    );
    ruling!(
        "Lightning Storm",
        "When Lightning Storm leaves the stack, its charge counters go away."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let storm = t.hand(P0, "Lightning Storm");
    t.hand(P0, "Forest");
    // In a hand, it can't be activated.
    assert!(!can_activate(&mut t, P0, storm));
    t.cast(P0, storm).target(Entity::Player(P1)).go();
    let spell = t.g.current(storm);
    // P1, not its controller, activates it and redirects it to P0.
    let land = t.hand(P1, "Forest");
    t.answer_choose(P1, &[obj(land)]);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let ab = activate_containing(&mut t, P1, spell, "Discard a land")
        .unwrap()
        .unwrap();
    assert_eq!(*t.g.stack.last().unwrap(), ab);
    t.resolve();
    assert_eq!(t.counters(spell, counters::CHARGE), 2);
    assert_eq!(stack_targets(&t, spell), vec![Entity::Player(P0)]);
    t.resolve();
    assert_eq!(t.life(P0), 15);
    assert_eq!(t.life(P1), 20);
    let card = t.g.current(spell);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    assert_eq!(t.counters(card, counters::CHARGE), 0);
}

#[test]
fn lightning_storm_keeps_an_illegal_target_when_theres_no_other() {
    cr!("115.7d", "608.2b");
    ruling!(
        "Lightning Storm",
        "When Lightning Storm’s activated ability resolves, if there are no other legal targets to choose for the Lightning Storm spell, you must leave the target the same, even if it’s now an illegal target."
    );
    // P0 (Ivory Mask: shroud) casts it at P1's Grizzly Bears; P1 has hexproof (Leyline of
    // Sanctity). The Bears gain shroud: when P0's own activation resolves, nothing else
    // can be chosen, so the target stays the Bears and the spell doesn't resolve.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ivory Mask");
    t.battlefield(P1, "Leyline of Sanctity");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let storm = t.hand(P0, "Lightning Storm");
    t.cast(P0, storm).target(bears).go();
    let spell = t.g.current(storm);
    grant_kw(&mut t, bears, KeywordKind::Shroud);
    let land = t.hand(P0, "Forest");
    t.answer_choose(P0, &[obj(land)]);
    t.answer_yes(P0, true);
    activate_containing(&mut t, P0, spell, "Discard a land").unwrap();
    t.resolve();
    assert_eq!(stack_targets(&t, spell), vec![obj(bears)]);
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P0, "Lightning Storm"));
}

// ---------------------------------------------------------------------------------------
// Reflecting Mirror: "{X}, {T}: Change the target of target spell with a single target if
// that target is you. The new target must be a player. X is twice the mana value of that
// spell." Silver Wyvern: "{U}: Change the target of target spell or ability that targets
// only this creature. The new target must be a creature."
// ---------------------------------------------------------------------------------------

#[test]
fn reflecting_mirror_cant_target_a_spell_that_affects_you_without_targeting() {
    cr!("115.7a", "115.10");
    ruling!(
        "Reflecting Mirror",
        "This can’t be used on spells which affect “you” and which do not say they target you."
    );
    supported("Reflecting Mirror");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let mirror = t.battlefield(P0, "Reflecting Mirror");
    // Flame Rift ("deals 4 damage to each player") affects P0 without targeting.
    let rift = cast_at(&mut t, P1, "Flame Rift", &[]);
    lots(&mut t, P0);
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    assert!(t.activate(P0, mirror, 0, &[obj(rift)]).is_err());
    // Lava Spike targeting P0 can be.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let mirror = t.battlefield(P0, "Reflecting Mirror");
    let spike = cast_at(&mut t, P1, "Lava Spike", &[Entity::Player(P0)]);
    lots(&mut t, P0);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, mirror, 0, &[obj(spike)]).unwrap();
    t.resolve();
    assert_eq!(stack_targets(&t, spike), vec![Entity::Player(P1)]);
}

#[test]
fn silver_wyvern_must_pick_a_legal_new_target() {
    cr!("115.7a", "115.7d");
    ruling!(
        "Silver Wyvern",
        "You must pick a legal target when redirecting the spell or ability."
    );
    supported("Silver Wyvern");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let wyvern = t.battlefield(P0, "Silver Wyvern");
    // P0's Gladecover Scout has hexproof: P1's Shock can't target it. P1's Bears can be.
    let scout = t.battlefield(P0, "Gladecover Scout");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shock = cast_at(&mut t, P1, "Shock", &[obj(wyvern)]);
    t.lands(P0, "Island", 1);
    let from = t.asked().len();
    // P0 tries the Scout: illegal, so the target can't become it.
    t.answer_targets(P0, &[obj(scout)]);
    t.activate(P0, wyvern, 0, &[obj(shock)]).unwrap();
    t.resolve();
    let offered: Vec<Entity> = target_choices(&t, P0, from, "")
        .into_iter()
        .flatten()
        .collect();
    assert!(offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(scout)));
    assert!(!offered.contains(&Entity::Player(P1)));
    assert_ne!(stack_targets(&t, shock), vec![obj(scout)]);
}

// ---------------------------------------------------------------------------------------
// Whirlwind Denial: "For each spell and ability your opponents control, counter it unless
// its controller pays {4}."
// ---------------------------------------------------------------------------------------

#[test]
fn whirlwind_denial_opponents_choose_in_turn_order_then_all_are_countered_at_once() {
    cr!("101.4", "608.2c");
    ruling!(
        "Whirlwind Denial",
        "As Whirlwind Denial resolves, first the opponent whose turn it is (or, if it's your turn, the next opponent in turn order) chooses which spells and/or abilities to pay for, then pays that amount. Then each other opponent in turn order does the same. Then all spells and abilities that weren't paid for are countered at the same time."
    );
    supported("Whirlwind Denial");
    // P2's turn in a four-player game: P2 chooses first, then P3, then P1.
    let mut t = TestGame::new(4);
    t.set_step(P2, Step::PrecombatMain);
    let spells: Vec<ObjectId> = [P1, P2, P3]
        .into_iter()
        .map(|p| {
            t.lands(p, "Mountain", 4);
            cast_at(&mut t, p, "Shock", &[Entity::Player(P0)])
        })
        .collect();
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.stack.len(),
    );
    // P2 pays; P3 and P1 don't.
    t.answer_yes(P2, true);
    t.answer_yes(P3, false);
    t.answer_yes(P1, false);
    let from = t.asked().len();
    cast_at(&mut t, P0, "Whirlwind Denial", &[]);
    t.resolve();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P2, P3, P1]);
    // P3's unpaid Shock was still on the stack when P1 chose (countered at once, later).
    assert_eq!(*seen.lock().unwrap(), vec![3]);
    assert!(!on_stack(&t, spells[0]));
    assert!(on_stack(&t, spells[1]));
    assert!(!on_stack(&t, spells[2]));
}

// ---------------------------------------------------------------------------------------
// Power Sink: "Counter target spell unless its controller pays {X}. If that player
// doesn't, they tap all lands with mana abilities they control and lose all unspent mana."
// ---------------------------------------------------------------------------------------

#[test]
fn power_sink_is_a_separate_payment_not_a_higher_cost() {
    cr!("118.12", "202.3");
    ruling!(
        "Power Sink",
        "Does not increase the mana cost of the spell. It just requires a separate expenditure in order for it to succeed."
    );
    supported("Power Sink");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 4);
    let bears = t.hand(P1, "Grizzly Bears");
    t.cast(P1, bears).go();
    let spell = t.g.current(bears);
    t.lands(P0, "Island", 3);
    let sink = t.hand(P0, "Power Sink");
    t.answer_yes(P1, true);
    t.cast(P0, sink).x(2).target(spell).go();
    t.resolve();
    // Paid: its mana cost (and mana value) is unchanged, and it resolves.
    assert!(on_stack(&t, spell));
    assert_eq!(t.g.mana_value_of(spell), 2);
    t.resolve();
    assert!(t.on_battlefield(t.g.current(spell)));
}

#[test]
fn power_sink_taps_lands_without_tapping_them_for_mana() {
    cr!("106.12", "605.1a", "118.12");
    ruling!(
        "Power Sink",
        "When this spell resolves, you either pay X mana or let your lands become tapped. The lands that become tapped are not “tapped for mana”. If you choose to pay, you may pay the X mana using whatever mana abilities you want to use."
    );
    // P1 can't pay: the lands become tapped, and Mana Flare ("Whenever a player taps a
    // land for mana, that player adds one mana ...") doesn't trigger: no mana is made.
    for pays in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.battlefield(P0, "Mana Flare");
        let lands = t.lands(P1, "Forest", 2);
        t.lands(P1, "Wastes", 2);
        t.battlefield(P1, "Llanowar Elves");
        let bears = t.hand(P1, "Grizzly Bears");
        t.cast(P1, bears).go();
        let spell = t.g.current(bears);
        t.lands(P0, "Island", 1);
        let sink = t.hand(P0, "Power Sink");
        t.answer_yes(P1, pays);
        // X = 1: P1 can pay with Llanowar Elves (or a land) — a mana ability.
        lots(&mut t, P0);
        t.cast(P0, sink).x(1).target(spell).go();
        t.resolve();
        if pays {
            assert!(on_stack(&t, spell));
        } else {
            assert!(!on_stack(&t, spell));
            assert!(lands.iter().all(|l| tapped(&t, *l)));
            assert_eq!(pool(&t, P1), 0);
            t.settle();
            assert_eq!(t.stack_len(), 0);
        }
    }
}

// ---------------------------------------------------------------------------------------
// Equinox: "Enchanted land has '{T}: Counter target spell if it would destroy a land you
// control.'"
// ---------------------------------------------------------------------------------------

#[test]
fn equinox_doesnt_counter_a_spell_that_destroys_a_land_only_if_chosen_so() {
    cr!("608.2b", "601.2c");
    ruling!(
        "Equinox",
        "Will not counter a spell which would destroy a land only if a choice is made."
    );
    supported("Befoul");
    // Befoul ("Destroy target land or nonblack creature.") cast at a creature can't
    // destroy a land: not countered. Cast at P0's land, it is.
    for at_land in [false, true] {
        let mut t = TestGame::new(2);
        let land = t.battlefield(P0, "Plains");
        attach_new(&mut t, P0, "Equinox", land);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.set_step(P1, Step::PrecombatMain);
        let target = if at_land { land } else { bears };
        let befoul = cast_at(&mut t, P1, "Befoul", &[obj(target)]);
        t.answer_targets(P0, &[obj(befoul)]);
        activate_containing(&mut t, P0, land, "Counter target spell").unwrap();
        t.resolve();
        assert_eq!(on_stack(&t, befoul), !at_land);
        t.resolve_all();
        assert!(t.on_battlefield(land));
        assert_eq!(t.on_battlefield(bears), at_land);
    }
}

// ---------------------------------------------------------------------------------------
// Academic Probation: "Choose one — • Choose a nonland card name. Opponents can't cast
// spells with the chosen name until your next turn. • Choose target nonland permanent.
// Until your next turn, it can't attack or block, and its activated abilities can't be
// activated."
// ---------------------------------------------------------------------------------------

/// P0 casts Academic Probation's first mode naming `name` (as an instant, with Leyline of
/// Anticipation), and it resolves.
fn probation_names(t: &mut TestGame, name: &str) {
    t.battlefield(P0, "Leyline of Anticipation");
    t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
    lands_for_cost(t, P0, "Academic Probation");
    let c = t.hand(P0, "Academic Probation");
    t.cast(P0, c).modes(&[0]).go();
    t.resolve();
}

#[test]
fn academic_probation_doesnt_affect_spells_already_on_the_stack() {
    cr!("611.2c", "101.2");
    ruling!(
        "Academic Probation",
        "Academic Probation is a sorcery, but in some unusual cases it may be cast in response to other spells. Spells with the chosen name that somehow happen to already be on the stack when Academic Probation resolves will not be affected."
    );
    ruling!(
        "Academic Probation",
        "For the first mode, no one can cast spells between the time a card is named and the time that Academic Probation's effect starts to apply."
    );
    supported("Academic Probation");
    let mut t = TestGame::new(2);
    let shock = cast_at(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    let from = t.asked().len();
    probation_names(&mut t, "Shock");
    // Nobody had priority between the naming and the effect.
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    let another = t.hand(P1, "Shock");
    lots(&mut t, P1);
    assert!(!can_cast(&mut t, P1, another, CastMethod::Normal));
    assert!(on_stack(&t, shock));
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn academic_probation_named_permanent_cards_can_still_be_put_onto_the_battlefield() {
    cr!("101.2", "601.1a");
    ruling!(
        "Academic Probation",
        "Although spells with the chosen name can't be cast, a card with that name can still be put onto the battlefield by another effect (if it's a permanent card)."
    );
    let mut t = TestGame::new(2);
    probation_names(&mut t, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let bears = t.hand(P1, "Grizzly Bears");
    assert!(!can_cast(&mut t, P1, bears, CastMethod::Normal));
    move_to(&mut t, bears, Zone::Battlefield);
    assert!(t.on_battlefield(bears));
}

#[test]
fn academic_probation_names_one_face_or_half() {
    cr!("201.3", "709.3", "712.8c");
    ruling!(
        "Academic Probation",
        "You can name either face of a modal double-faced card, but not both, and you can name either half of a split card, but not both. Opponents may still cast the face or half you didn't name."
    );
    supported("Fire // Ice");
    // Naming Fire: P1 can cast Ice but not Fire.
    let mut t = TestGame::new(2);
    probation_names(&mut t, "Fire");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let fi = t.hand(P1, "Fire // Ice");
    assert!(!can_cast(&mut t, P1, fi, CastMethod::Half(0)));
    assert!(can_cast(&mut t, P1, fi, CastMethod::Half(1)));
    // Naming Flamescroll Celebrant: P1 can cast Revel in Silence, not the creature.
    let mut t = TestGame::new(2);
    probation_names(&mut t, "Flamescroll Celebrant");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let c = t.hand(P1, "Flamescroll Celebrant // Revel in Silence");
    assert!(!can_cast(&mut t, P1, c, CastMethod::Normal));
    assert!(can_cast(&mut t, P1, c, CastMethod::Half(1)));
}

#[test]
fn academic_probation_a_noncreature_that_becomes_a_creature_still_cant_attack() {
    cr!("611.2c", "506.4");
    ruling!(
        "Academic Probation",
        "If you target a noncreature permanent with the second mode and it later becomes a creature, that creature still won't be able to attack or block until your next turn."
    );
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Mind Stone");
    let other = t.battlefield(P1, "Mind Stone");
    lands_for_cost(&mut t, P0, "Academic Probation");
    let c = t.hand(P0, "Academic Probation");
    t.cast(P0, c).modes(&[1]).target(stone).go();
    t.resolve();
    t.advance_to(P1, Step::BeginningOfCombat);
    animate(&mut t, stone, 2, 2);
    animate(&mut t, other, 2, 2);
    assert!(!can_attack(&mut t, stone));
    assert!(can_attack(&mut t, other));
}

// ---------------------------------------------------------------------------------------
// Gauntlets of Chaos: "{5}, Sacrifice this artifact: Exchange control of target artifact,
// creature, or land you control and target permanent an opponent controls that shares
// one of those types with it. ..."
// ---------------------------------------------------------------------------------------

#[test]
fn gauntlets_of_chaos_targets_must_share_a_type_again_on_resolution() {
    cr!("608.2b", "701.12b");
    ruling!(
        "Gauntlets of Chaos",
        "As you activate the ability, the targets you choose must be two artifacts, two creatures, or two lands. As the ability resolves, both targets will be legal only if they are two artifacts, two creatures, or two lands at that time as well, though they may be a different type than they were at the time the ability was activated. For example, if the targets were a creature and an artifact creature when the ability was activated, but the first target became a noncreature artifact by the time the ability resolves, both targets will still be legal (since they’re both artifacts)."
    );
    supported("Gauntlets of Chaos");
    // P0's Ornithopter (an artifact creature) and P1's Memnite (an artifact creature):
    // Ornithopter stops being a creature (still an artifact): exchanged. If it stops being
    // an artifact too, they share nothing: no exchange.
    for strip in [
        vec![CardType::Creature],
        vec![CardType::Creature, CardType::Artifact],
    ] {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Gauntlets of Chaos");
        let thopter = t.battlefield(P0, "Ornithopter");
        let memnite = t.battlefield(P1, "Memnite");
        t.lands(P0, "Wastes", 5);
        t.activate(P0, g, 0, &[obj(thopter), obj(memnite)]).unwrap();
        let both = strip.len() == 2;
        run_from(
            &mut t,
            P0,
            None,
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::RemoveTypes(strip)],
                duration: Duration::EndOfTurn,
            },
            &[obj(thopter)],
        );
        t.resolve_all();
        let swapped = !both;
        assert_eq!(controller(&mut t, thopter) == P1, swapped, "both: {both}");
        assert_eq!(controller(&mut t, memnite) == P0, swapped, "both: {both}");
    }
}

// ---------------------------------------------------------------------------------------
// Crooked Scales: "{4}, {T}: Flip a coin. If you win the flip, destroy target creature an
// opponent controls. If you lose the flip, destroy target creature you control unless you
// pay {3} and repeat this process."
// ---------------------------------------------------------------------------------------

#[test]
fn crooked_scales_needs_both_targets_to_activate() {
    cr!("601.2c", "602.2b", "115.1");
    ruling!(
        "Crooked Scales",
        "You choose both targets on activation, so you can only activate this when you can legally choose both targets."
    );
    supported("Crooked Scales");
    let mut t = TestGame::new(2);
    let scales = t.battlefield(P0, "Crooked Scales");
    t.lands(P0, "Wastes", 4);
    let mine = t.battlefield(P0, "Grizzly Bears");
    assert!(t.activate(P0, scales, 0, &[obj(mine)]).is_err());
    t.clear_answers();
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(t.activate(P0, scales, 0, &[obj(theirs), obj(mine)]).is_ok());
}

// ---------------------------------------------------------------------------------------
// Isildur's Fateful Strike: "(You may cast a legendary instant only if you control a
// legendary creature or planeswalker.) / Destroy target creature. ..."
// ---------------------------------------------------------------------------------------

const ISILDUR: &str = "Isildur's Fateful Strike";

#[test]
fn isildurs_fateful_strike_needs_a_legendary_creature_or_planeswalker_to_cast() {
    cr!("205.4d");
    ruling!(
        "Isildur's Fateful Strike",
        "Isildur's Fateful Strike is the first legendary instant. It is subject to the same casting restrictions as existing legendary sorceries, as listed in the following rulings."
    );
    ruling!(
        "Isildur's Fateful Strike",
        "You can't cast a legendary instant unless you control a legendary creature or a legendary planeswalker. Once you begin to cast a legendary instant, losing control of your legendary creatures and planeswalkers won't affect that spell."
    );
    supported(ISILDUR);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    let strike = t.hand(P0, ISILDUR);
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, strike, CastMethod::Normal));
    let kaysa = t.battlefield(P0, "Kaysa");
    assert!(can_cast(&mut t, P0, strike, CastMethod::Normal));
    t.cast(P0, strike).target(bears).go();
    // Kaysa leaves in response: the spell is unaffected.
    destroy(&mut t, kaysa);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn isildurs_fateful_strike_any_number_a_turn_four_a_deck() {
    cr!("205.4d", "100.2a");
    ruling!(
        "Isildur's Fateful Strike",
        "Other than the casting restriction, the legendary supertype on an instant carries no additional rules. You may cast any number of legendary instants in a turn, and your deck may contain any number of legendary cards (but no more than four of any with the same name)."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kaysa");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    for target in [a, b] {
        lots(&mut t, P0);
        let s = t.hand(P0, ISILDUR);
        t.cast(P0, s).target(target).go();
        t.resolve_all();
    }
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(
        mtg_engine::deck::max_copies(&mtg_engine::card::card(ISILDUR)),
        Some(4)
    );
}

// ---------------------------------------------------------------------------------------
// Swift Reckoning: "Spell mastery — If there are two or more instant and/or sorcery cards
// in your graveyard, you may cast this spell as though it had flash." Graveyard Shift:
// "This spell has flash as long as there are five or more mana values among cards in your
// graveyard."
// ---------------------------------------------------------------------------------------

// Not cited, exempt as outdated (docs/rulings-exemptions/rulings-P011.tsv; logged in
// docs/RULING_DISCREPANCIES.md): Swift Reckoning's ruling (2018-12-07) "If an effect allows
// you to cast Swift Reckoning from your graveyard, you need two other instant and/or sorcery
// cards in your graveyard to cast it as though it had flash." It followed from 601.3 as
// worded in the CRs effective 2016-07-22 to 2019-08-23 ("If that player is no longer allowed
// to cast that spell after completing its proposal, the casting of the spell is illegal
// ..."; 601.4 from the CR effective 2019-10-04, now 601.6). The CR effective 2022-04-29 added
// 601.4a, now 601.6a: once a player has begun casting a spell "that could be cast as though
// it had flash because certain conditions were met", they "may continue to cast that spell
// as though it had flash even if those conditions stop being met". Graveyard Shift's ruling
// (2022-04-29) has its card count itself the same way "immediately before you begin to cast
// it, even if moving it to the stack would reduce that number to four" (work item
// ruling-conflicts-13; more cases in `flash_permission_conditions.rs`).
#[test]
fn swift_reckoning_cast_from_the_graveyard_counts_itself_as_casting_begins() {
    cr!("601.2a", "601.3", "601.6a", "702.34a");
    interp!("flash-permission-conditions-judged-as-casting-begins");
    ruling!(
        "Swift Reckoning",
        "The number of instant and/or sorcery cards in your graveyard matters only as you begin to cast Swift Reckoning. Once it's cast, that number doesn't matter and will have no effect on Swift Reckoning resolving."
    );
    supported("Swift Reckoning");
    // Snapcaster Mage gives it flashback this turn, on P1's turn (instant speed). As the
    // casting begins, the card is in the graveyard and counts itself: with one other
    // instant or sorcery card there, it's offered and can be cast; with none, neither.
    for others in [0usize, 1] {
        let mut t = TestGame::new(2);
        let reck = t.graveyard(P0, "Swift Reckoning");
        for _ in 0..others {
            t.graveyard(P0, "Shock");
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.objects[bears.0 as usize].tapped = true;
        t.set_step(P1, Step::PrecombatMain);
        t.answer_targets(P0, &[obj(reck)]);
        enter(&mut t, P0, "Snapcaster Mage");
        t.resolve_all();
        t.lands(P0, "Plains", 2);
        t.g.turn.priority = Some(P0);
        t.g.recompute();
        let offered = t.g.legal_actions(P0).into_iter().find_map(|a| match a {
            Action::Cast { card, method } if card == reck => Some(method),
            _ => None,
        });
        assert_eq!(offered.is_some(), others == 1, "others: {others}");
        // What's offered is what casting accepts.
        let method = offered.unwrap_or(CastMethod::Keyword(KeywordKind::Flashback));
        let cast = t.cast(P0, reck).method(method).target(bears).try_go();
        assert_eq!(cast.is_ok(), others == 1, "others: {others}");
        if others == 1 {
            t.resolve_all();
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
            assert_eq!(t.zone(reck), Zone::Exile);
        } else {
            assert_eq!(t.zone(reck), Zone::Graveyard(P0));
            assert!(t.on_battlefield(bears));
        }
    }
}

#[test]
fn graveyard_shift_legality_is_checked_before_costs_and_mana_abilities() {
    cr!("601.2a", "601.2g", "601.3");
    ruling!(
        "Graveyard Shift",
        "The game checks whether it is legal to cast a spell before costs are determined or mana abilities are activated while casting it. If cards are removed from your graveyard after that point, such as by exiling Jack-o'-Lantern from your graveyard to pay for its mana ability, this won't affect whether you can legally cast Graveyard Shift, even if that reduces the number of mana values in your graveyard to below five."
    );
    supported("Graveyard Shift");
    supported("Molt Tender");
    // Five mana values: 0, 1, 2, 3, 4 (Divination the only 3). Molt Tender's mana
    // ability ("{T}, Exile a card from your graveyard: Add one mana of any color.") pays
    // the {B}, exiling Divination: four values left once the spell's cast.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Ornithopter");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.graveyard(P0, "Grizzly Bears");
    let div = t.graveyard(P0, "Divination");
    t.graveyard(P0, "Hill Giant");
    t.battlefield(P0, "Molt Tender");
    t.lands(P0, "Wastes", 4);
    t.set_step(P1, Step::PrecombatMain);
    let gs = t.hand(P0, "Graveyard Shift");
    t.answer_choose(P0, &[obj(div)]);
    t.cast(P0, gs).target(elves).go();
    assert_eq!(t.zone(div), Zone::Exile);
    t.resolve_all();
    assert!(t.on_battlefield(elves));
}

// ---------------------------------------------------------------------------------------
// Xiahou Dun, the One-Eyed: "Sacrifice Xiahou Dun: Return target black card from your
// graveyard to your hand. Activate only during your turn, before attackers are declared."
// ---------------------------------------------------------------------------------------

#[test]
fn xiahou_dun_cant_return_itself() {
    cr!("602.2b", "601.2c", "601.2h");
    ruling!(
        "Xiahou Dun, the One-Eyed",
        "You can't activate Xiahou Dun's ability to return Xiahou Dun itself to your hand. You must choose a target before you sacrifice Xiahou Dun, so it's not a black card in your graveyard yet."
    );
    supported("Xiahou Dun, the One-Eyed");
    let mut t = TestGame::new(2);
    let dun = t.battlefield(P0, "Xiahou Dun, the One-Eyed");
    // No black card in the graveyard: no target, no activation.
    assert!(!can_activate(&mut t, P0, dun));
    let raven = t.graveyard(P0, "Gravedigger");
    let from = t.asked().len();
    t.activate(P0, dun, 0, &[obj(raven)]).unwrap();
    let offered: Vec<Entity> = target_choices(&t, P0, from, "")
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(offered, vec![obj(raven)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Gravedigger"));
    assert!(t.in_graveyard(P0, "Xiahou Dun, the One-Eyed"));
}

// ---------------------------------------------------------------------------------------
// Mind Extraction: "As an additional cost to cast this spell, sacrifice a creature. /
// Target player reveals their hand and discards all cards of each of the sacrificed
// creature's colors."
// ---------------------------------------------------------------------------------------

#[test]
fn mind_extraction_with_a_colorless_creature_discards_nothing() {
    cr!("105.2c", "608.2h");
    ruling!(
        "Mind Extraction",
        "If you sacrifice a colorless creature, you get to see their hand but they discard no cards."
    );
    supported("Mind Extraction");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Memnite");
    t.answer_choose(P0, &[obj(thopter)]);
    cast_new(&mut t, P0, "Mind Extraction", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!(t.hand_size(P1), 3);
}

// ---------------------------------------------------------------------------------------
// Nebuchadnezzar: "{X}, {T}: Choose a card name. Target opponent reveals X cards at random
// from their hand. Then that player discards all cards with that name revealed this way.
// Activate only during your turn."
// ---------------------------------------------------------------------------------------

#[test]
fn nebuchadnezzar_any_time_during_your_turn() {
    cr!("602.5b", "117.1b");
    ruling!(
        "Nebuchadnezzar",
        "You may activate the ability only during your turn, but you may do so any time you have priority. (In other words, it’s not restricted to only the times you could cast a sorcery.)"
    );
    supported("Nebuchadnezzar");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Nebuchadnezzar");
    t.hand(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    // P0's upkeep, and with a spell on the stack: fine.
    t.set_step(P0, Step::Upkeep);
    assert!(can_activate(&mut t, P0, n));
    t.set_step(P0, Step::PrecombatMain);
    cast_at(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    assert!(can_activate(&mut t, P0, n));
    t.resolve_all();
    // P1's turn: no.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P0, n));
}

// ---------------------------------------------------------------------------------------
// Creeping Dread: "At the beginning of your upkeep, each player discards a card. Each
// opponent who discarded a card that shares a card type with the card you discarded
// loses 3 life."
// ---------------------------------------------------------------------------------------

/// P1's Creeping Dread; each player holds the cards named (none for an empty entry); P1's
/// upkeep trigger resolves. Returns the life totals.
fn dread(hands: [&[&str]; 3]) -> [i32; 3] {
    supported("Creeping Dread");
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Creeping Dread");
    for (i, cards) in hands.iter().enumerate() {
        for c in *cards {
            t.hand(PlayerId(i as u8), c);
        }
    }
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    [t.life(P0), t.life(P1), t.life(P2)]
}

#[test]
fn creeping_dread_a_double_faced_card_compares_its_front_face() {
    cr!("712.8a", "205.2a");
    ruling!(
        "Creeping Dread",
        "Because you consider only the characteristics of a double-faced card's front face while it's not on the battlefield, a double-faced card with different card types among its faces (such as Skin Invasion) only compares its front face to the other discarded cards."
    );
    // P1 discards Skin Invasion (an enchantment; its back face is a creature): P0's
    // creature card shares no type; P2's enchantment does.
    assert_eq!(
        dread([
            &["Grizzly Bears"],
            &["Skin Invasion // Skin Shedder"],
            &["Pacifism"]
        ]),
        [20, 20, 17]
    );
}

#[test]
fn creeping_dread_with_nothing_discarded_by_you_no_one_loses_life() {
    cr!("701.9b", "603.4");
    ruling!(
        "Creeping Dread",
        "If a player has no cards in hand, that player doesn't discard a card while everyone else does. If you didn't discard a card, no player can have discarded a card that shares a card type with the card you discarded, so no player loses 3 life."
    );
    assert_eq!(
        dread([&["Grizzly Bears"], &[], &["Hill Giant"]]),
        [20, 20, 20]
    );
    // P2 has no card and discards none; P0's creature card shares a type.
    assert_eq!(
        dread([&["Grizzly Bears"], &["Hill Giant"], &[]]),
        [17, 20, 20]
    );
}
