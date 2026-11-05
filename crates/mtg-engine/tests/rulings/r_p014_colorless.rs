//! Rulings batch P014 — colorless and all-colors characteristic-defining abilities (CR
//! 105.2, 604.3) and the Eldrazi built around them: devoid spells and their sources,
//! "another colorless creature" checks, cost reduction for colorless spells, processors
//! (exiled cards as costs), Kozilek's Unsealing, Inverter of Truth, Eldrazi Displacer,
//! Flaying Tendrils and Turn Against.

use crate::r_p014_common::*;
use crate::r_p038_common::{cast_only, cast_resolve, enter, kill, mana_all, tokens_of};
use crate::r_s01_common::*;
use crate::r_s06_common::activate_containing;
use mtg_engine::commander_rules::color_identity;
use mtg_engine::decision::Decision;
use mtg_engine::deck::{check_commander, DeckProblem};
use mtg_engine::facedown;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn colors(t: &TestGame, id: ObjectId) -> ColorSet {
    t.obj_now(id).chars.colors
}

fn add(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

// ---------------------------------------------------------------------------
// All colors, colorless, in every zone
// ---------------------------------------------------------------------------

#[test]
fn all_colors_abilities_apply_in_every_zone() {
    cr!("604.3", "105.2a", "105.4");
    ruling!(
        "Sphinx of the Guildpact",
        "The ability that makes Sphinx of the Guildpact all colors applies in all zones, not just the battlefield."
    );
    ruling!(
        "Fallaji Wayfarer",
        "The first ability of Fallaji Wayfarer functions in all zones."
    );
    ruling!(
        "Transguild Courier",
        "This card is white, blue, black, red, and green in all zones. It is multicolored."
    );
    ruling!(
        "Transguild Courier",
        "Transguild Courier appears in a multicolored card frame, but it's still an artifact."
    );
    for name in [
        "Sphinx of the Guildpact",
        "Fallaji Wayfarer",
        "Transguild Courier",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let ids = [
            t.hand(P0, name),
            t.graveyard(P0, name),
            t.exile(P0, name),
            t.library_top(P0, name),
            t.battlefield(P0, name),
        ];
        t.g.recompute();
        for id in ids {
            assert_eq!(colors(&t, id), ColorSet::ALL, "{name} in {:?}", t.zone(id));
            assert!(colors(&t, id).count() > 1);
        }
        if name != "Fallaji Wayfarer" {
            assert!(t.obj_now(ids[0]).chars.is(CardType::Artifact), "{name}");
        }
    }
}

#[test]
fn fallaji_wayfarers_color_identity_is_green() {
    cr!("903.4", "903.5c");
    ruling!(
        "Fallaji Wayfarer",
        "Fallaji Wayfarer's color identity is green."
    );
    let w = card("Fallaji Wayfarer");
    let mut green = ColorSet::NONE;
    green.insert(Color::Green);
    assert_eq!(color_identity(&w), green);
    // A mono-green commander's deck may include it; a mono-white one's may not.
    for (commander, ok) in [
        ("Marwyn, the Nurturer", true),
        ("Isamaru, Hound of Konda", false),
    ] {
        let cmd = card(commander);
        let mut deck: Vec<Arc<_>> = vec![cmd.clone(), w.clone()];
        deck.extend((0..98).map(|_| card(if ok { "Forest" } else { "Plains" })));
        let outside = check_commander(&deck, &cmd, &[], false)
            .into_iter()
            .any(|p| matches!(p, DeckProblem::OutsideColorIdentity { name } if name == "Fallaji Wayfarer"));
        assert_eq!(outside, !ok, "{commander}");
    }
}

#[test]
fn ghostfire_is_colorless_and_so_is_its_damage() {
    cr!("105.2c", "604.3", "702.16a");
    ruling!(
        "Ghostfire",
        "Ghostfire is colorless in all zones. Its damage comes from a colorless source."
    );
    supported("Ghostfire");
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, "Ghostfire");
    t.g.recompute();
    assert_eq!(colors(&t, gy), ColorSet::NONE);
    // Kor Firewalker has protection from red: Ghostfire can target it and isn't prevented.
    let kor = t.battlefield(P1, "Kor Firewalker");
    let g = cast_only(&mut t, P0, "Ghostfire", &[obj(kor)]);
    assert_eq!(colors(&t, g), ColorSet::NONE);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Kor Firewalker"));
}

#[test]
fn sphinx_of_the_guildpact_colorless_spells_arent_monocolored() {
    cr!("105.2c", "702.11d");
    ruling!(
        "Sphinx of the Guildpact",
        "An object is monocolored if it has exactly one color. A colorless object isn't monocolored."
    );
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Sphinx of the Guildpact");
    t.set_step(P1, Step::PrecombatMain);
    mana_all(&mut t, P1, 4);
    let bolt = t.hand(P1, "Lightning Bolt");
    let from = t.asked().len();
    let _ = t.cast(P1, bolt).target(pl(P0)).try_go();
    let offered = crate::r_p016_common::target_choices(&t, P1, from, "");
    assert!(!offered.is_empty() && offered.iter().all(|c| !c.contains(&obj(sphinx))));
    t.resolve_all();
    let from = t.asked().len();
    let fire = cast_only(&mut t, P1, "Ghostfire", &[obj(sphinx)]);
    let offered = crate::r_p016_common::target_choices(&t, P1, from, "");
    assert!(offered.iter().any(|c| c.contains(&obj(sphinx))));
    assert!(t.g.stack.contains(&fire));
    t.resolve_all();
    assert_eq!(t.obj_now(sphinx).damage, 3);
}

// ---------------------------------------------------------------------------
// "Another colorless creature"
// ---------------------------------------------------------------------------

#[test]
fn dominator_drone_intervening_if_another_colorless_creature() {
    cr!("603.4", "105.2c");
    ruling!(
        "Dominator Drone",
        "Dominator Drone’s last ability checks to see if you control another colorless creature at the moment it enters the battlefield."
    );
    supported("Dominator Drone");
    // No other colorless creature (Grizzly Bears is green): no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    enter(&mut t, P0, "Dominator Drone");
    assert_eq!(t.stack_len(), 0);
    // Ornithopter: it triggers; Ornithopter leaves before it resolves: no effect.
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let thopter = t.battlefield(P0, "Ornithopter");
        enter(&mut t, P0, "Dominator Drone");
        assert_eq!(t.stack_len(), 1);
        if leaves {
            kill(&mut t, thopter);
        }
        t.resolve_all();
        assert_eq!(t.life(P1), if leaves { 20 } else { 18 });
    }
}

#[test]
fn reaver_drone_checks_on_resolution_even_if_it_has_left() {
    cr!("608.2h", "603.4");
    ruling!(
        "Reaver Drone",
        "Whether you control another colorless creature is checked as the ability resolves. This check will happen even if Reaver Drone leaves the battlefield in response to the ability."
    );
    supported("Reaver Drone");
    // (Reaver Drone leaves, Ornithopter stays) and (Ornithopter leaves).
    for (drone_leaves, thopter_leaves, loss) in
        [(true, false, 0), (false, true, 1), (true, true, 1)]
    {
        let mut t = TestGame::new(2);
        let drone = t.battlefield(P0, "Reaver Drone");
        let thopter = t.battlefield(P0, "Ornithopter");
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "lose 1 life"), 1);
        if drone_leaves {
            kill(&mut t, drone);
        }
        if thopter_leaves {
            kill(&mut t, thopter);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), 20 - loss, "{drone_leaves} {thopter_leaves}");
    }
}

#[test]
fn barrage_tyrant_deals_the_damage_not_the_sacrificed_creature() {
    cr!("609.7", "702.15b", "118.3");
    ruling!(
        "Barrage Tyrant",
        "Barrage Tyrant deals the damage, not the sacrificed creature. For example, if the sacrificed creature had lifelink, you won't gain life."
    );
    supported("Barrage Tyrant");
    let mut t = TestGame::new(2);
    let tyrant = t.battlefield(P0, "Barrage Tyrant");
    let wurm = t.battlefield(P0, "Wurmcoil Engine");
    mana_all(&mut t, P0, 3);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_choose(P0, &[obj(wurm)]);
    activate_containing(&mut t, P0, tyrant, "Sacrifice another colorless creature").unwrap();
    assert!(t.in_graveyard(P0, "Wurmcoil Engine"));
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 20);
}

// ---------------------------------------------------------------------------
// Herald of Kozilek
// ---------------------------------------------------------------------------

#[test]
fn herald_of_kozilek_reduces_only_generic_mana() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Herald of Kozilek",
        "Herald of Kozilek's last ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic component of that mana cost."
    );
    supported("Herald of Kozilek");
    // Reaver Drone ({B}, devoid) still costs {B}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Kozilek");
    let drone = t.hand(P0, "Reaver Drone");
    assert!(t.cast(P0, drone).try_go().is_err());
    add(&mut t, P0, ManaType::B, 1);
    t.cast(P0, drone).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // Turn Against ({4}{R}, devoid) costs {3}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Kozilek");
    let bears = t.battlefield(P1, "Grizzly Bears");
    add(&mut t, P0, ManaType::R, 1);
    add(&mut t, P0, ManaType::C, 3);
    let ta = t.hand(P0, "Turn Against");
    t.cast(P0, ta).target(obj(bears)).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn herald_of_kozilek_doesnt_change_mana_value_and_x_is_chosen_first() {
    cr!("601.2f", "202.3", "107.3a");
    ruling!(
        "Herald of Kozilek",
        "Herald of Kozilek's last ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you actually pay."
    );
    ruling!(
        "Herald of Kozilek",
        "If a colorless spell you cast has {X} in its mana cost, you choose the value of X before calculating the spell's total cost."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Kozilek");
    let bears = t.battlefield(P1, "Grizzly Bears");
    add(&mut t, P0, ManaType::R, 1);
    add(&mut t, P0, ManaType::C, 3);
    let ta = t.hand(P0, "Turn Against");
    let spell = t.cast(P0, ta).target(obj(bears)).go();
    assert_eq!(t.g.mana_value_of(spell), 5);
    t.resolve_all();
    // Endless One with X = 5 for {4}.
    add(&mut t, P0, ManaType::C, 4);
    let one = t.hand(P0, "Endless One");
    let spell = t.cast(P0, one).x(5).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    assert_eq!(t.g.mana_value_of(spell), 5);
    t.resolve_all();
    let one = t.named_on_battlefield("Endless One")[0];
    assert_eq!(t.counters(one, counters::PLUS1), 5);
}

// ---------------------------------------------------------------------------
// Kozilek's Unsealing
// ---------------------------------------------------------------------------

#[test]
fn kozileks_unsealing_uses_the_chosen_x_and_resolves_before_the_spell() {
    cr!("202.3e", "603.3", "601.2i");
    ruling!(
        "Kozilek's Unsealing",
        "If you cast a creature spell with {X} in its mana cost, use the value of X that was chosen when it was cast to determine its mana value."
    );
    ruling!(
        "Kozilek's Unsealing",
        "Kozilek's Unsealing's triggered abilities trigger before the spell that causes it to trigger. The ability will resolve even if that spell is countered or otherwise leaves the stack."
    );
    supported("Kozilek's Unsealing");
    // Endless One for X = 4: two Spawn; X = 7: three cards.
    for (x, spawn, drawn) in [(4, 2, 0), (7, 0, 3), (3, 0, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Kozilek's Unsealing");
        add(&mut t, P0, ManaType::C, x as u32);
        let one = t.hand(P0, "Endless One");
        t.cast(P0, one).x(x).go();
        t.settle();
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(tokens_of(&t, P0, "Spawn").len(), spawn, "X={x}");
        assert_eq!(t.hand_size(P0), hand + drawn, "X={x}");
    }
    // The spell is countered in response to the trigger: the trigger still resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kozilek's Unsealing");
    add(&mut t, P0, ManaType::C, 4);
    let one = t.hand(P0, "Endless One");
    let spell = t.cast(P0, one).x(4).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(
        t.g.stack.first(),
        Some(&spell),
        "the trigger is above the spell"
    );
    t.g.turn.priority = Some(P1);
    cast_only(&mut t, P1, "Cancel", &[obj(spell)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Endless One"));
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Spawn").len(), 2);
}

// ---------------------------------------------------------------------------
// Processors and exiled cards
// ---------------------------------------------------------------------------

#[test]
fn murk_strider_does_nothing_if_its_target_is_illegal() {
    cr!("608.2b", "603.3d");
    ruling!(
        "Murk Strider",
        "If the target creature becomes illegal before the triggered ability resolves, it won’t resolve and none of its effects will happen."
    );
    ruling!(
        "Murk Strider",
        "Players can respond to the triggered ability, but once it starts resolving and you decide whether to put a card from exile into the player’s graveyard, it’s too late for anyone to respond."
    );
    supported("Murk Strider");
    for target_dies in [true, false] {
        let mut t = TestGame::new(2);
        let exiled = t.exile(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[obj(giant)]);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(exiled)]);
        enter(&mut t, P0, "Murk Strider");
        assert_eq!(t.stack_len(), 1);
        if target_dies {
            kill(&mut t, giant);
        }
        t.resolve_all();
        if target_dies {
            assert_eq!(t.zone(exiled), Zone::Exile);
        } else {
            // Both happen in the one resolution.
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
            assert!(t.in_hand(P1, "Hill Giant"));
        }
    }
}

#[test]
fn processor_costs_are_paid_while_casting_or_activating() {
    cr!("601.2h", "602.2b", "118.3");
    ruling!(
        "Processor Assault",
        "Players can respond to Processor Assault only after it’s been cast and all of its costs have been paid."
    );
    ruling!(
        "Void Attendant",
        "Once you announce that you're activating this ability, players can't respond until after you have paid its costs and completed activating it."
    );
    ruling!(
        "Processor Assault",
        "You must put exactly one card an opponent owns from exile into that player’s graveyard to cast Processor Assault."
    );
    supported("Processor Assault");
    supported("Void Attendant");
    // No card an opponent owns in exile (P0's own doesn't count): can't cast it.
    let mut t = TestGame::new(2);
    t.exile(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    mana_all(&mut t, P0, 2);
    let pa = t.hand(P0, "Processor Assault");
    assert!(t.cast(P0, pa).target(obj(giant)).try_go().is_err());
    t.clear_answers();
    // Two cards: exactly one is put into the graveyard as it's cast.
    let a = t.exile(P1, "Grizzly Bears");
    t.exile(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_choose(P0, &[obj(a)]);
    let spell = t.cast(P0, pa).try_go().unwrap();
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Grizzly Bears").len(), 2);
    assert!(t.g.stack.contains(&spell));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Void Attendant: the card is gone by the time the ability is on the stack.
    let mut t = TestGame::new(2);
    let va = t.battlefield(P0, "Void Attendant");
    t.exile(P1, "Grizzly Bears");
    mana_all(&mut t, P0, 2);
    activate_containing(&mut t, P0, va, "Eldrazi Scion").unwrap();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Scion").len(), 1);
}

#[test]
fn inverter_of_truth_exiles_face_down_and_processors_take_one_at_random() {
    cr!("406.3", "406.4");
    ruling!(
        "Inverter of Truth",
        "No player can look at the cards once they’re exiled."
    );
    ruling!(
        "Inverter of Truth",
        "If your opponent puts one of the face-down exiled cards into your graveyard (for example, to activate an ability of an Eldrazi Processor), choose one of the cards at random."
    );
    supported("Inverter of Truth");
    let mut t = TestGame::new(2);
    let lib: Vec<ObjectId> = t.g.player(P0).library.clone();
    let n = lib.len();
    t.graveyard(P0, "Grizzly Bears");
    enter(&mut t, P0, "Inverter of Truth");
    t.resolve_all();
    let exiled: Vec<ObjectId> = lib.iter().map(|c| t.g.current(*c)).collect();
    for c in &exiled {
        assert_eq!(t.g.obj(*c).zone, Zone::Exile);
        assert!(t.g.obj(*c).face_down);
        assert!(!facedown::can_look_at(&t.g, P0, *c));
        assert!(!facedown::can_look_at(&t.g, P1, *c));
    }
    assert_eq!(t.library_size(P0), 1);
    // P1's Void Attendant: one card stands for the pile; a random one goes to P0's graveyard.
    let va = t.battlefield(P1, "Void Attendant");
    t.set_step(P1, Step::PrecombatMain);
    mana_all(&mut t, P1, 2);
    let from = t.asked().len();
    activate_containing(&mut t, P1, va, "Eldrazi Scion").unwrap();
    let offers: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P1 => Some(candidates.len()),
            _ => None,
        })
        .collect();
    assert!(offers.iter().all(|k| *k <= 1), "{offers:?}");
    assert_eq!(t.graveyard_size(P0), 1);
    let exiled_now =
        t.g.exile
            .iter()
            .filter(|c| t.g.obj(**c).owner == P0)
            .count();
    assert_eq!(exiled_now, n - 1);
}

// ---------------------------------------------------------------------------
// Eldrazi Displacer, Flaying Tendrils, Turn Against
// ---------------------------------------------------------------------------

#[test]
fn eldrazi_displacer_returns_a_new_object() {
    cr!("400.7", "506.4", "704.5m", "704.5n");
    ruling!(
        "Eldrazi Displacer",
        "After the creature returns to the battlefield, it will be a new object with no connection to the creature that was exiled."
    );
    supported("Eldrazi Displacer");
    let mut t = TestGame::new(2);
    let disp = t.battlefield(P0, "Eldrazi Displacer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), counters::PLUS1, 1, None);
    let aura = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(aura, obj(bears)));
    let eq = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(eq, obj(bears)));
    t.g.recompute();
    attack_with(&mut t, &[(bears, pl(P1))]);
    mana_all(&mut t, P0, 3);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, disp, "Exile another target creature").unwrap();
    t.resolve_all();
    let now = t.g.current(bears);
    assert_ne!(now, bears);
    let o = t.g.obj(now);
    assert_eq!(o.zone, Zone::Battlefield);
    assert!(o.tapped);
    assert_eq!(o.counter(counters::PLUS1), 0);
    assert_eq!((o.power(), o.toughness()), (2, 2));
    assert!(t
        .g
        .combat
        .as_ref()
        .is_some_and(|c| c.attack_target(now).is_none()));
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert_eq!(t.obj_now(eq).attached_to, None);
    assert!(t.on_battlefield(eq));
}

#[test]
fn flaying_tendrils_exiles_any_creature_that_would_die_this_turn() {
    cr!("614.1a", "704.3", "611.2c");
    ruling!(
        "Flaying Tendrils",
        "Creatures that Flaying Tendrils would cause to die will be exiled instead."
    );
    ruling!(
        "Flaying Tendrils",
        "Only creatures that are on the battlefield as Flaying Tendrils resolves will get -2/-2. However, any creature that would die that turn will be exiled, even if it wasn’t on the battlefield or wasn’t a creature as Flaying Tendrils resolved."
    );
    supported("Flaying Tendrils");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Flaying Tendrils", &[]);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.pt(giant), (1, 1));
    // A creature that arrives afterwards isn't shrunk, but dies into exile.
    let late = t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(late), (3, 3));
    kill(&mut t, late);
    assert_eq!(t.zone(late), Zone::Exile);
    assert!(!t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn turn_against_can_target_any_creature() {
    cr!("115.1", "701.26b");
    ruling!(
        "Turn Against",
        "Turn Against can target any creature, even one that’s untapped or one you already control."
    );
    supported("Turn Against");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield_sick(P0, "Hill Giant");
    t.g.tap(mine);
    cast_resolve(&mut t, P0, "Turn Against", &[obj(theirs)]);
    assert_eq!(t.obj_now(theirs).controller, P0);
    cast_resolve(&mut t, P0, "Turn Against", &[obj(mine)]);
    assert!(!t.obj_now(mine).tapped);
    assert!(t
        .obj_now(mine)
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}
