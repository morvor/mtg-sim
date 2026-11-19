//! Rulings batch P105 — combat: attack requirements (Ursine Monstrosity), creatures that
//! stay in combat or stay blocked once their condition changes (Devourer of Memory,
//! Patchwork Beastie, Platypus-Bear, Agility Bobblehead), triggers during combat (Stinkweed
//! Imp, Harald Unites the Elves), Tezzeret the Schemer's emblem, Corsair Captain, and
//! "summoning sickness" (Cultivator's Caravan, Tyvar, Jubilant Brawler).

use crate::r_p104_common::obj;
use crate::r_p105_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, can_attack};
use crate::r_s04_common::crew;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, damage, has_kw};
use crate::r_s09_common::legal_attack;
use crate::r_s10_common::attacking;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

// ---------------------------------------------------------------------------
// Ursine Monstrosity: "At the beginning of combat on your turn, mill a card and choose an
// opponent at random. This creature attacks that player this combat if able. ..."
// ---------------------------------------------------------------------------

/// P0's Ursine Monstrosity, its beginning-of-combat trigger resolved.
fn ursine_at_combat(t: &mut TestGame, u: ObjectId) -> ObjectId {
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    t.g.current(u)
}

#[test]
fn ursine_monstrosity_doesnt_attack_when_it_cant() {
    cr!("508.1d", "508.1h", "302.6", "508.1a");
    ruling!(
        "Ursine Monstrosity",
        "If, during your declare attackers step, Ursine Monstrosity is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under your control continuously since your turn began (and doesn't have haste), then it doesn't attack. If there's a cost associated with having Ursine Monstrosity attack the chosen player, you aren't forced to pay that cost, so it doesn't have to attack that player in that case either."
    );
    supported("Ursine Monstrosity");
    // (tapped, summoning sick, Pacifism, Propaganda, attacks)
    let cases = [
        (true, false, false, false, false),
        (false, true, false, false, false),
        (false, false, true, false, false),
        (false, false, false, true, false),
        (false, false, false, false, true),
    ];
    for (tapped, sick, pacifism, propaganda, attacks) in cases {
        let mut t = TestGame::new(2);
        let u = if sick {
            t.battlefield_sick(P0, "Ursine Monstrosity")
        } else {
            t.battlefield(P0, "Ursine Monstrosity")
        };
        if tapped {
            set_tapped(&mut t, u, true);
        }
        if pacifism {
            attach_new(&mut t, P1, "Pacifism", obj(u));
        }
        if propaganda {
            t.battlefield(P1, "Propaganda");
        }
        let u = ursine_at_combat(&mut t, u);
        // P0 declares no attackers: the requirement is obeyed only if it can be.
        attack_with(&mut t, &[]);
        assert_eq!(attacking(&t, u), attacks, "{cases:?}");
    }
}

#[test]
fn ursine_monstrosity_must_attack_the_player_not_their_planeswalker_or_battle() {
    cr!("508.1d", "310.5");
    ruling!(
        "Ursine Monstrosity",
        "Ursine Monstrosity must attack the chosen player if able, not a planeswalker they control or a battle they protect."
    );
    supported("Liliana of the Veil");
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Ursine Monstrosity");
    let lili = t.battlefield(P1, "Liliana of the Veil");
    // P0's Invasion of Azgol (a Siege), protected by P1.
    let battle = t.battlefield(P0, "Invasion of Azgol // Ashen Reaper");
    t.g.objects[battle.0 as usize].choices.player = Some(P1);
    t.g.dirty = true;
    assert_eq!(battle::protector(&t.g, battle), Some(P1));
    let u = ursine_at_combat(&mut t, u);
    assert!(legal_attack(&mut t, &[(u, pl(P1))]));
    assert!(!legal_attack(&mut t, &[(u, obj(lili))]));
    assert!(!legal_attack(&mut t, &[(u, obj(battle))]));
    assert!(!legal_attack(&mut t, &[]));
    // Another creature may attack the planeswalker or the battle.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(legal_attack(&mut t, &[(u, pl(P1)), (bears, obj(lili))]));
    assert!(legal_attack(&mut t, &[(u, pl(P1)), (bears, obj(battle))]));
}

// ---------------------------------------------------------------------------
// Stinkweed Imp: "Whenever this creature deals combat damage to a creature, destroy that
// creature."
// ---------------------------------------------------------------------------

#[test]
fn stinkweed_imp_is_a_combat_damage_trigger_not_deathtouch() {
    cr!("510.2", "603.2", "702.2b", "704.5h");
    ruling!(
        "Stinkweed Imp",
        "Notably, Stinkweed Imp's ability isn't deathtouch. It's a triggered ability that triggers only on combat damage."
    );
    supported("Stinkweed Imp");
    supported("Prodigal Pyromancer");
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Stinkweed Imp");
    assert!(!has_kw(&t, imp, KeywordKind::Deathtouch));
    let giant = t.battlefield(P1, "Giant Spider");
    attack_with(&mut t, &[(imp, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(giant, imp)]));
    t.advance_to(P0, Step::CombatDamage);
    // State-based actions see 1 damage on a 3/3: no deathtouch, so it survives them; the
    // trigger goes on the stack and destroys it when it resolves.
    t.settle();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    // Noncombat damage dealt by the Imp doesn't trigger it: a 1/1 Imp-sourced ping won't
    // destroy anything. (The Imp has no damage ability of its own; its combat damage is
    // the only damage that matters.)
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Stinkweed Imp");
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, imp, 1, giant);
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(giant));
}

// ---------------------------------------------------------------------------
// Creatures stay attacking or blocked once their condition changes
// ---------------------------------------------------------------------------

#[test]
fn devourer_of_memory_cant_be_blocked_after_blocks_doesnt_unblock_it() {
    cr!("509.1h", "506.4", "510.1c");
    ruling!(
        "Devourer of Memory",
        "Once Devourer of Memory has been blocked, resolving its triggered ability (perhaps by activating its activated ability) won’t change or undo that block."
    );
    supported("Devourer of Memory");
    let mut t = TestGame::new(2);
    let dev = t.battlefield(P0, "Devourer of Memory");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    let wall = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(dev, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(wall, dev)]));
    t.advance_to(P0, Step::DeclareBlockers);
    // "{1}{U}{B}: Mill a card." The mill triggers its first ability.
    t.activate(P0, dev, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(dev), (3, 2));
    assert!(blocked(&t, dev));
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(wall));
}

#[test]
fn attackers_stay_in_combat_when_the_graveyard_changes() {
    cr!("506.4", "508.1c");
    ruling!(
        "Patchwork Beastie",
        "Once Patchwork Beastie is attacking or blocking, reducing the number of card types among cards in your graveyard won't remove Patchwork Beastie from combat."
    );
    ruling!(
        "Platypus-Bear",
        "Once Platypus-Bear has attacked, it will remain an attacking creature for that combat, even if the Lesson card is removed from your graveyard."
    );
    supported("Patchwork Beastie");
    supported("Platypus-Bear");
    // (creature, graveyard, power)
    let cases: [(&str, &[&str], i32); 2] = [
        (
            "Patchwork Beastie",
            &["Island", "Grizzly Bears", "Lightning Bolt", "Divination"],
            3,
        ),
        ("Platypus-Bear", &["Environmental Sciences"], 2),
    ];
    for (name, gy, power) in cases {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let cards: Vec<ObjectId> = gy.iter().map(|n| t.graveyard(P0, n)).collect();
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(can_attack(&mut t, c), "{name}");
        attack_with(&mut t, &[(c, pl(P1))]);
        assert!(attacking(&t, c), "{name} {}", t.dump_log());
        // The graveyard empties.
        for card in cards {
            move_to(&mut t, card, Zone::Exile);
        }
        t.settle();
        assert!(attacking(&t, c), "{name}");
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20 - power, "{name}");
    }
    // Blocking: Patchwork Beastie blocks, then the graveyard loses its types.
    let mut t = TestGame::new(2);
    let beastie = t.battlefield(P1, "Patchwork Beastie");
    let cards: Vec<ObjectId> = ["Island", "Grizzly Bears", "Lightning Bolt", "Divination"]
        .iter()
        .map(|n| t.graveyard(P1, n))
        .collect();
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(giant, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(beastie, giant)]));
    t.advance_to(P0, Step::DeclareBlockers);
    for card in cards {
        move_to(&mut t, card, Zone::Exile);
    }
    t.settle();
    assert!(t.g.is_blocking(t.g.current(beastie)));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn agility_bobblehead_doesnt_unblock_an_already_blocked_creature() {
    cr!("509.1h", "506.4", "602.2");
    ruling!(
        "Agility Bobblehead",
        "Activating Agility Bobblehead’s last ability targeting a creature that has already been blocked by a creature without haste won’t cause that creature to become unblocked."
    );
    supported("Agility Bobblehead");
    let mut t = TestGame::new(2);
    let bobble = t.battlefield(P0, "Agility Bobblehead");
    t.lands(P0, "Wastes", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(blocker, bears)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    // "{3}, {T}: Up to X target creatures you control each gain haste until end of turn
    // and can't be blocked this turn except by creatures with haste, ..."
    t.activate(P0, bobble, 1, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Haste));
    assert!(blocked(&t, bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// Harald Unites the Elves, chapter III: "Whenever an Elf you control attacks this turn,
// target creature an opponent controls gets -1/-1 until end of turn."
// ---------------------------------------------------------------------------

#[test]
fn harald_chapter_three_trigger_resolves_before_blockers() {
    cr!("508.1m", "508.2", "509.1", "603.7c");
    ruling!(
        "Harald Unites the Elves",
        "The triggered ability created by the chapter III ability resolves before blockers are declared. It may remove opposing creatures before they have the chance to block."
    );
    supported("Harald Unites the Elves");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Harald Unites the Elves");
    set_counters(&mut t, saga, counters::LORE, 2);
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    t.resolve_all();
    let elf = t.battlefield(P0, "Llanowar Elves");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.answer_targets(P0, &[obj(lions)]);
    // P1 would block with the Lions if it could.
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(lions, elf)]));
    attack_with(&mut t, &[(elf, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(!t.on_battlefield(lions));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn harald_chapter_two_counts_elves_as_it_resolves() {
    cr!("608.2h", "714.2c");
    ruling!(
        "Harald Unites the Elves",
        "Only Elves you control as the chapter II ability resolves will get a +1/+1 counter. Elves that come under your control later in the turn will not."
    );
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Harald Unites the Elves");
    set_counters(&mut t, saga, counters::LORE, 1);
    let early = t.battlefield(P0, "Llanowar Elves");
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    t.resolve_all();
    let late = t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.counters(early, counters::PLUS1), 1);
    assert_eq!(t.counters(late, counters::PLUS1), 0);
}

// ---------------------------------------------------------------------------
// Tezzeret the Schemer's emblem: "At the beginning of combat on your turn, target artifact
// you control becomes an artifact creature with base power and toughness 5/5."
// ---------------------------------------------------------------------------

/// P0 gets Tezzeret the Schemer's emblem (activating its −7).
fn tezzeret_emblem(t: &mut TestGame) {
    supported("Tezzeret the Schemer");
    let tez = t.battlefield(P0, "Tezzeret the Schemer");
    set_loyalty(t, tez, 7);
    t.activate(P0, tez, 2, &[]).unwrap();
    t.resolve_all();
}

/// The emblem's trigger at P0's beginning of combat, targeting `target`.
fn emblem_animates(t: &mut TestGame, target: ObjectId) {
    t.answer_targets(P0, &[obj(target)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
}

#[test]
fn tezzeret_emblem_sets_base_5_5_even_through_crewing() {
    cr!("613.4b", "702.122a", "613.1d", "301.7b");
    ruling!(
        "Tezzeret the Schemer",
        "An artifact creature targeted by the emblem's ability becomes 5/5 instead of its normal power and toughness."
    );
    ruling!(
        "Tezzeret the Schemer",
        "A Vehicle targeted by the emblem's ability becomes a 5/5 creature. Crewing it won't change its power and toughness."
    );
    supported("Smuggler's Copter");
    // An artifact creature (Ornithopter, 0/2).
    let mut t = TestGame::new(2);
    tezzeret_emblem(&mut t);
    let thopter = t.battlefield(P0, "Ornithopter");
    emblem_animates(&mut t, thopter);
    assert_eq!(t.pt(thopter), (5, 5));
    // A Vehicle (Smuggler's Copter, 3/3, crew 1).
    let mut t = TestGame::new(2);
    tezzeret_emblem(&mut t);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    emblem_animates(&mut t, copter);
    assert!(t.obj_now(copter).is(CardType::Creature));
    assert_eq!(t.pt(copter), (5, 5));
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert_eq!(t.pt(copter), (5, 5));
}

#[test]
fn tezzeret_emblem_artifact_must_have_been_controlled_since_the_turn_began() {
    cr!("302.6", "508.1a");
    ruling!(
        "Tezzeret the Schemer",
        "An artifact that becomes a creature due to the emblem's ability can't attack unless you've controlled it continuously since your turn began. It doesn't matter whether or not it was a creature at that time."
    );
    for (sick, can) in [(false, true), (true, false)] {
        let mut t = TestGame::new(2);
        tezzeret_emblem(&mut t);
        let stone = if sick {
            t.battlefield_sick(P0, "Mind Stone")
        } else {
            t.battlefield(P0, "Mind Stone")
        };
        emblem_animates(&mut t, stone);
        assert!(t.obj_now(stone).is(CardType::Creature));
        assert_eq!(can_attack(&mut t, stone), can, "sick: {sick}");
    }
}

// ---------------------------------------------------------------------------
// Corsair Captain: "Other Pirates you control get +1/+1."
// ---------------------------------------------------------------------------

#[test]
fn corsair_captain_leaving_can_make_marked_damage_lethal() {
    cr!("704.5g", "613.4c", "120.6");
    ruling!(
        "Corsair Captain",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Pirates you control may become lethal if Corsair Captain leaves the battlefield during that turn."
    );
    supported("Corsair Captain");
    supported("Kitesail Freebooter");
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Corsair Captain");
    let pirate = t.battlefield(P0, "Kitesail Freebooter");
    assert_eq!(t.pt(pirate), (2, 3));
    let src = t.battlefield(P1, "Hill Giant");
    damage(&mut t, src, 2, pirate);
    assert!(t.on_battlefield(pirate));
    move_to(&mut t, captain, Zone::Graveyard(P0));
    t.settle();
    assert!(!t.on_battlefield(pirate));
}

// ---------------------------------------------------------------------------
// {T} abilities of permanents that came under your control this turn
// ---------------------------------------------------------------------------

#[test]
fn cultivators_caravan_taps_for_mana_the_turn_it_arrives_unless_its_a_creature() {
    cr!("302.6", "702.122a");
    ruling!(
        "Cultivator's Caravan",
        "As long as Cultivator's Caravan isn't an artifact creature, its {T} ability can be activated if it came under your control this turn."
    );
    supported("Cultivator's Caravan");
    let mut t = TestGame::new(2);
    let caravan = t.battlefield_sick(P0, "Cultivator's Caravan");
    assert!(tap_for_mana(&mut t, P0, caravan, "Add one mana"));
    // Crewed the turn it arrived, it's a creature: its {T} ability can't be activated.
    let mut t = TestGame::new(2);
    let caravan = t.battlefield_sick(P0, "Cultivator's Caravan");
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(crew(&mut t, P0, caravan, &[giant]));
    t.resolve_all();
    assert!(t.obj_now(caravan).is(CardType::Creature));
    assert!(!tap_for_mana(&mut t, P0, caravan, "Add one mana"));
}

#[test]
fn tyvar_lets_tap_abilities_be_activated_but_grants_no_haste() {
    cr!("302.6", "702.10c", "508.1a");
    ruling!(
        "Tyvar, Jubilant Brawler",
        "Tyvar's first ability doesn't grant haste to any creatures, nor does it allow you to attack with creatures as though they had haste. However, you will be able to activate abilities with {T} in their activation costs as soon as creatures with such abilities come under your control."
    );
    supported("Tyvar, Jubilant Brawler");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield_sick(P0, "Prodigal Pyromancer");
    assert!(!can_activate(&mut t, P0, pyro));
    t.battlefield(P0, "Tyvar, Jubilant Brawler");
    t.g.recompute();
    assert!(!has_kw(&t, pyro, KeywordKind::Haste));
    assert!(can_activate(&mut t, P0, pyro));
    t.activate(P0, pyro, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // It still can't attack.
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, bears));
    assert!(!can_attack(&mut t, pyro));
}

// ---------------------------------------------------------------------------
// Domri's mana loyalty abilities
// ---------------------------------------------------------------------------

#[test]
fn domris_mana_loyalty_ability_uses_the_stack_at_sorcery_speed() {
    cr!("606.3", "605.1a");
    ruling!(
        "Domri, Anarch of Bolas",
        "Because it's a loyalty ability, Domri's first loyalty ability isn't a mana ability. It can be activated only any time you could cast a sorcery. It uses the stack and can be responded to."
    );
    ruling!(
        "Domri, Chaos Bringer",
        "Because it’s a loyalty ability, Domri’s first ability isn’t a mana ability. It can be activated only any time you could cast a sorcery. It uses the stack and can be responded to."
    );
    supported("Domri, Anarch of Bolas");
    supported("Domri, Chaos Bringer");
    for name in ["Domri, Anarch of Bolas", "Domri, Chaos Bringer"] {
        let mut t = TestGame::new(2);
        let domri = t.battlefield(P0, name);
        // Not during P1's turn, in combat, or with something on the stack.
        t.set_step(P1, Step::PrecombatMain);
        assert!(!can_activate(&mut t, P0, domri), "{name}");
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(!can_activate(&mut t, P0, domri), "{name}");
        t.set_step(P0, Step::PrecombatMain);
        let bolt = t.hand(P1, "Lightning Bolt");
        t.lands(P1, "Mountain", 1);
        t.cast(P1, bolt).target(pl(P0)).go();
        assert!(!can_activate(&mut t, P0, domri), "{name}");
        t.resolve_all();
        // In P0's main phase with an empty stack: the +1 goes on the stack, and the mana
        // is added only when it resolves.
        assert!(can_activate(&mut t, P0, domri), "{name}");
        t.activate(P0, domri, 0, &[]).unwrap();
        assert_eq!(t.stack_len(), 1, "{name}");
        assert_eq!(pool_total(&t, P0), 0, "{name}");
        // P1 can respond.
        let bolt = t.hand(P1, "Lightning Bolt");
        t.lands(P1, "Mountain", 1);
        t.cast(P1, bolt).target(pl(P0)).go();
        assert_eq!(t.stack_len(), 2, "{name}");
        t.resolve();
        t.resolve();
        assert_eq!(t.stack_len(), 0, "{name}");
        assert_eq!(pool_total(&t, P0), 1, "{name}");
    }
}
