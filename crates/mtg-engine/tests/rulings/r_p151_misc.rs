//! Rulings batch P151 — the other abilities of permanents that sacrifice themselves:
//! damage sources (CR 120.3, 609.7), creatures entering attacking (CR 508.4), copies and
//! modes, mana spent (CR 601.2h), card types shared, and linked exiled cards (CR 607.2a).

use crate::r_p151_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Adds `n` mana of each of `types` to `p`'s pool.
fn pool(t: &mut TestGame, p: PlayerId, types: &[ManaType], n: u32) {
    for ty in types {
        t.g.players[p.idx()].mana_pool.add_type(*ty, n);
    }
}

/// Cards in exile.
fn exiled(t: &TestGame) -> usize {
    t.g.objects
        .iter()
        .filter(|o| o.zone == Zone::Exile && t.g.is_live(o.id))
        .count()
}

#[test]
fn case_of_the_burning_masks_counts_sources_not_damage_events() {
    cr!("120.3", "113.7");
    ruling!(
        "Case of the Burning Masks",
        "A permanent that deals damage multiple times without changing zones still counts as only one source."
    );
    ruling!(
        "Case of the Burning Masks",
        "Activated and triggered abilities are not themselves sources of damage. The source of an activated ability is the object whose ability was activated"
    );
    supported("Case of the Burning Masks");
    for with_shock in [false, true] {
        let mut t = TestGame::new(2);
        let case = t.battlefield(P0, "Case of the Burning Masks");
        let pyro = t.battlefield(P0, "Prodigal Pyromancer");
        let bears = t.battlefield(P0, "Grizzly Bears");
        // The Pyromancer deals damage twice (its ability, then as a permanent), the Bears
        // once: two sources.
        t.activate(P0, pyro, 0, &[Entity::Player(P1)]).expect("Pyromancer");
        t.resolve_all();
        damage(&mut t, pyro, Entity::Player(P1), 1);
        damage(&mut t, bears, Entity::Player(P1), 2);
        if with_shock {
            cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
            t.resolve_all();
        }
        end_step(&mut t, P0);
        assert_eq!(mtg_engine::cases::is_solved(&t.g, case), with_shock);
    }
}

#[test]
fn time_elemental_targets_permanents_with_no_auras() {
    cr!("115.1");
    ruling!(
        "Time Elemental",
        "A “permanent that isn’t enchanted” is a permanent with no Auras on it."
    );
    supported("Time Elemental");
    let mut t = TestGame::new(2);
    let te = t.battlefield(P0, "Time Elemental");
    let enchanted = t.battlefield(P1, "Grizzly Bears");
    let strength = t.battlefield(P1, "Holy Strength");
    t.attach(strength, obj(enchanted));
    let plain = t.battlefield(P1, "Hill Giant");
    t.settle();
    pool(&mut t, P0, &[ManaType::U], 2);
    pool(&mut t, P0, &[ManaType::C], 2);
    let n = t.asked().len();
    t.activate(P0, te, 0, &[obj(plain)]).expect("Time Elemental");
    let cands = crate::r_s02_common::target_candidates(&t, P0, n);
    assert!(!cands[0].contains(&obj(enchanted)));
    assert!(cands[0].contains(&obj(strength)) && cands[0].contains(&obj(plain)));
}

#[test]
fn creatures_put_onto_the_battlefield_attacking_werent_declared_as_attackers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Chorale of the Void",
        "Although the creature you put onto the battlefield is attacking, it was never declared as an attacking creature."
    );
    supported("Chorale of the Void");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let chorale = t.battlefield(P0, "Chorale of the Void");
    t.attach(chorale, obj(bears));
    t.graveyard(P0, "Hill Giant");
    let cage = t.graveyard(P1, "Kessig Cagebreakers");
    t.settle();
    t.answer_targets(P0, &[obj(cage)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let cage = t.named_on_battlefield("Kessig Cagebreakers");
    assert_eq!(cage.len(), 1);
    assert!(t.g.is_attacking(cage[0]));
    assert!(tokens(&t, P0).is_empty(), "Cagebreakers' attack trigger didn't trigger");

    // Tokens created attacking: Connecting the Dots ("Whenever a creature you control
    // attacks") triggers only for the creature declared as an attacker.
    ruling!(
        "Kessig Cagebreakers",
        "Although the tokens are attacking, they were never declared as attacking creatures"
    );
    ruling!(
        "Launch the Fleet",
        "Although the tokens are attacking, they were never declared as attacking creatures"
    );
    ruling!(
        "Mardu Ascendancy",
        "Although the tokens are attacking, they were never declared as attacking creatures"
    );
    for name in ["Kessig Cagebreakers", "Launch the Fleet", "Mardu Ascendancy"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Connecting the Dots");
        let attacker = if name == "Kessig Cagebreakers" {
            t.graveyard(P0, "Hill Giant");
            t.battlefield(P0, name)
        } else {
            if name == "Mardu Ascendancy" {
                t.battlefield(P0, name);
            }
            t.battlefield(P0, "Grizzly Bears")
        };
        if name == "Launch the Fleet" {
            cast_new(&mut t, P0, name, &[obj(attacker)]);
            t.resolve_all();
        }
        attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
        t.resolve_all();
        let attacking_tokens = tokens(&t, P0)
            .into_iter()
            .filter(|x| t.g.is_attacking(*x))
            .count();
        assert_eq!(attacking_tokens, 1, "{name}");
        assert_eq!(exiled(&t), 1, "{name}: one Connecting the Dots trigger");
    }
}

#[test]
fn mardu_ascendancy_goblin_may_attack_another_opponent() {
    cr!("508.4");
    ruling!(
        "Mardu Ascendancy",
        "As the Goblin token enters the battlefield, you choose which opponent or opposing planeswalker it’s attacking."
    );
    supported("Mardu Ascendancy");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Mardu Ascendancy");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Player(P2)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let goblin = tokens(&t, P0)[0];
    let combat = t.g.combat.as_ref().unwrap();
    assert_eq!(combat.attack_target(goblin), Some(Entity::Player(P2)));
    assert_eq!(combat.attack_target(bears), Some(Entity::Player(P1)));
}

#[test]
fn from_beyond_finds_cards_with_the_creature_type_eldrazi() {
    cr!("205.3m");
    ruling!(
        "From Beyond",
        "An Eldrazi card is one with the creature type Eldrazi. Just having \"Eldrazi\" in its name or being an Eldrazi-themed card doesn't count."
    );
    supported("From Beyond");
    let mut t = TestGame::new(2);
    let beyond = t.battlefield(P0, "From Beyond");
    let monument = t.library_top(P0, "Eldrazi Monument");
    let devastator = t.library_top(P0, "Eldrazi Devastator");
    pool(&mut t, P0, &[ManaType::G, ManaType::C], 1);
    let n = t.asked().len();
    t.answer_choose(P0, &[obj(devastator)]);
    act(&mut t, P0, beyond, "Eldrazi card", &[]).expect("From Beyond");
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[n..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .flatten()
        .collect();
    assert!(!offered.contains(&obj(monument)));
    assert!(t.in_hand(P0, "Eldrazi Devastator"));
}

#[test]
fn angelic_shield_bonus_ends_as_it_is_sacrificed() {
    cr!("602.2", "704.5g");
    ruling!(
        "Angelic Shield",
        "Angelic Shield goes to the graveyard when you activate the ability, but the targeted creature does not get returned until the ability resolves."
    );
    supported("Angelic Shield");
    let mut t = TestGame::new(2);
    let shield = t.battlefield(P0, "Angelic Shield");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].damage = 2;
    t.settle();
    assert!(t.on_battlefield(bears), "2/3");
    t.activate(P0, shield, 0, &[obj(bears)]).expect("Angelic Shield");
    t.settle();
    assert!(t.in_graveyard(P0, "Angelic Shield"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "destroyed before the return");
    t.resolve_all();
    assert!(!t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn assemble_the_entmoot_counts_life_gained_ignoring_life_lost() {
    cr!("119.3");
    ruling!(
        "Assemble the Entmoot",
        "Assemble the Entmoot's activated ability counts the total amount of life you gained without taking into account any life you lost during that turn."
    );
    supported("Assemble the Entmoot");
    let mut t = TestGame::new(2);
    let entmoot = t.battlefield(P0, "Assemble the Entmoot");
    t.g.gain_life(P0, 3);
    t.g.lose_life(P0, 3);
    t.g.flush_events();
    t.activate(P0, entmoot, 0, &[]).expect("Assemble the Entmoot");
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 3);
    for x in toks {
        assert_eq!(t.pt(x), (3, 3));
        assert!(t.g.obj(x).tapped);
        assert_eq!(t.counters(x, "reach"), 1);
    }
}

#[test]
fn aura_of_silence_taxes_every_opponent() {
    cr!("601.2f");
    ruling!(
        "Aura of Silence",
        "Aura of Silence affects all opponents in a multiplayer game."
    );
    supported("Aura of Silence");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Aura of Silence");
    for p in [P1, P2] {
        t.set_step(p, Step::PrecombatMain);
        let thopter = t.hand(p, "Ornithopter");
        t.battlefield(p, "Wastes");
        assert!(!crate::r_s02_common::can_cast(
            &mut t,
            p,
            thopter,
            mtg_engine::object::CastMethod::Normal
        ));
        t.battlefield(p, "Wastes");
        assert!(crate::r_s02_common::can_cast(
            &mut t,
            p,
            thopter,
            mtg_engine::object::CastMethod::Normal
        ));
    }
    // Its controller isn't taxed.
    t.set_step(P0, Step::PrecombatMain);
    let thopter = t.hand(P0, "Ornithopter");
    assert!(crate::r_s02_common::can_cast(
        &mut t,
        P0,
        thopter,
        mtg_engine::object::CastMethod::Normal
    ));
}

#[test]
fn quest_for_pure_flame_sources() {
    cr!("120.3", "609.7");
    supported("Quest for Pure Flame");
    ruling!(
        "Quest for Pure Flame",
        "Both of Quest for Pure Flame’s abilities interact with any kind of damage, not just combat damage."
    );
    ruling!(
        "Quest for Pure Flame",
        "If a spell causes damage to be dealt, that spell will always identify the source of the damage. In most cases, the source is the spell itself."
    );
    ruling!(
        "Quest for Pure Flame",
        "If an ability causes damage to be dealt, that ability will always identify the source of the damage. The ability itself is never the source."
    );
    ruling!(
        "Quest for Pure Flame",
        "If the source of the damage is a permanent, Quest for Pure Flame checks whether you control that permanent at the time that damage is dealt. If the permanent has left the battlefield by then, its last known information is used."
    );
    let mut t = TestGame::new(2);
    let quest = t.battlefield(P0, "Quest for Pure Flame");
    // A spell (noncombat damage).
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.counters(quest, "quest"), 1);
    // An activated ability: its source is the Pyromancer.
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.answer_yes(P0, true);
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).expect("Pyromancer");
    t.resolve_all();
    assert_eq!(t.counters(quest, "quest"), 2);
    // Mogg Fanatic, sacrificed to pay for its ability: last known information.
    let fanatic = t.battlefield(P0, "Mogg Fanatic");
    t.answer_yes(P0, true);
    t.activate(P0, fanatic, 0, &[Entity::Player(P1)]).expect("Mogg Fanatic");
    t.resolve_all();
    assert_eq!(t.counters(quest, "quest"), 3);
    // P1's source dealing damage to P0's opponent? No: a source P1 controls.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    damage(&mut t, theirs, Entity::Player(P1), 1);
    t.resolve_all();
    assert_eq!(t.counters(quest, "quest"), 3);
    assert_eq!(t.life(P1), 20 - 2 - 1 - 1 - 1);

    ruling!(
        "Quest for Pure Flame",
        "If you activate the second ability of more than one Quest for Pure Flame in the same turn, the effects are cumulative."
    );
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        let q = t.battlefield(P0, "Quest for Pure Flame");
        put_counters(&mut t, q, "quest", 4);
        t.activate(P0, q, 0, &[]).expect("Quest for Pure Flame");
        t.resolve_all();
    }
    t.answer_yes(P0, false);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
}

#[test]
fn catharsis_checks_mana_spent_on_its_total_cost() {
    cr!("601.2h");
    ruling!(
        "Catharsis",
        "Catharsis's first and second abilities care about what mana was spent to pay its total cost, not just what mana was spent to pay the hybrid mana symbols in its cost."
    );
    supported("Catharsis");
    // Six mana, two of each of white and red: both abilities trigger whatever paid the
    // hybrid symbols (they're only two).
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 2);
    let c = t.hand(P0, "Catharsis");
    t.cast(P0, c).go();
    t.resolve_all();
    let cat = t.named_on_battlefield("Catharsis")[0];
    assert_eq!(tokens(&t, P0).len(), 2, "two Kithkin");
    assert_eq!(t.pt(cat), (4, 5));
    assert!(t.obj_now(cat).has_keyword(KeywordKind::Haste));

    ruling!(
        "Catharsis",
        "It doesn't matter if more than two mana of that color was spent; the effect isn't multiplied."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let c = t.hand(P0, "Catharsis");
    t.cast(P0, c).go();
    t.resolve_all();
    let cat = t.named_on_battlefield("Catharsis")[0];
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(t.pt(cat), (3, 4), "no {{R}}{{R}} spent");
}

#[test]
fn demonic_covenant_cases() {
    supported("Demonic Covenant");
    ruling!(
        "Demonic Covenant",
        "Demonic Covenant's first ability will trigger once for each player you attack with one or more Demons."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Demonic Covenant");
    let a = crate::r_s02_common::create_token(&mut t, P0, "Demon");
    let b = crate::r_s02_common::create_token(&mut t, P0, "Demon");
    let c = crate::r_s02_common::create_token(&mut t, P0, "Demon");
    for x in [a, b, c] {
        t.g.objects[x.0 as usize].summoning_sick = false;
    }
    attack_with(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P2)),
        ],
    );
    assert_eq!(abilities_on_stack(&t), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);

    ruling!(
        "Demonic Covenant",
        "Creatures that enter attacking are never declared as attackers, and as such, they won't cause Demonic Covenant's first ability to trigger."
    );
    supported("Kaalia of the Vast");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Demonic Covenant");
    let kaalia = t.battlefield(P0, "Kaalia of the Vast");
    let demon = t.hand(P0, "Lord of the Pit");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(demon)]);
    attack_with(&mut t, &[(kaalia, Entity::Player(P1))]);
    t.resolve_all();
    let pit = t.named_on_battlefield("Lord of the Pit");
    assert_eq!(pit.len(), 1);
    assert!(t.g.is_attacking(pit[0]));
    assert_eq!(t.life(P0), 20, "no Covenant trigger");

    ruling!(
        "Demonic Covenant",
        "Demonic Covenant's last ability will only cause you to sacrifice Demonic Covenant if both cards have the exact same combination of card types. Supertypes and subtypes are not taken into account."
    );
    for (cards, sacrificed) in [
        (["Grizzly Bears", "Hill Giant"], true),
        (["Ornithopter", "Grizzly Bears"], false),
        (["Snow-Covered Forest", "Swamp"], true),
    ] {
        let mut t = TestGame::new(2);
        let covenant = t.battlefield(P0, "Demonic Covenant");
        crate::r_s01_common::stack_library(&mut t, P0, &cards);
        end_step(&mut t, P0);
        assert_eq!(!t.on_battlefield(covenant), sacrificed, "{cards:?}");
    }
}

#[test]
fn connecting_the_dots_has_its_own_exiled_cards() {
    cr!("607.2a");
    ruling!(
        "Connecting the Dots",
        "Each Connecting the Dots you control has its own set of face-down exiled cards. Connecting the Dots's last ability puts only those cards into your hand, not those of any other Connecting the Dots."
    );
    supported("Connecting the Dots");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Connecting the Dots");
    t.battlefield(P0, "Connecting the Dots");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(exiled(&t), 2);
    t.advance_to(P0, Step::PostcombatMain);
    pool(&mut t, P0, &[ManaType::R, ManaType::C], 1);
    t.activate(P0, a, 0, &[]).expect("Connecting the Dots");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(exiled(&t), 1);

    ruling!(
        "Connecting the Dots",
        "If Connecting the Dots leaves the battlefield before you activate its last ability, any cards exiled by its triggered ability remain exiled face down for the rest of the game."
    );
    let mut t = TestGame::new(2);
    let dots = t.battlefield(P0, "Connecting the Dots");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    destroy(&mut t, dots);
    let card = t
        .g
        .objects
        .iter()
        .find(|o| o.zone == Zone::Exile && t.g.is_live(o.id))
        .unwrap()
        .id;
    assert!(t.g.obj(card).face_down);
    // A new Connecting the Dots has no connection to it.
    let dots = reanimate(&mut t, P0, dots);
    pool(&mut t, P0, &[ManaType::R, ManaType::C], 1);
    t.activate(P0, dots, 0, &[]).expect("Connecting the Dots");
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
}

#[test]
fn hopeless_nightmare_life_loss_without_a_discard() {
    cr!("608.2c");
    ruling!(
        "Hopeless Nightmare",
        "Hopeless Nightmare's first ability causes each opponent to lose 2 life even if some or all of those players were unable to discard a card."
    );
    supported("Hopeless Nightmare");
    let mut t = TestGame::new(3);
    t.hand(P2, "Grizzly Bears");
    t.enter(P0, "Hopeless Nightmare");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.hand_size(P2), 0);
}

#[test]
fn era_of_innovation_triggers_once_for_an_artifact_artificer() {
    cr!("603.2");
    ruling!(
        "Era of Innovation",
        "If a creature that is both an artifact and an Artificer enters the battlefield under your control, Era of Innovation's triggered ability triggers only once."
    );
    supported("Era of Innovation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Era of Innovation");
    t.enter(P0, "Etherium Sculptor");
    t.settle();
    assert_eq!(abilities_on_stack(&t), 1);
    pool(&mut t, P0, &[ManaType::C], 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter("energy"), 2);
}

#[test]
fn festival_of_embers_exiles_itself_and_discarded_cards() {
    cr!("614.1a", "702.35a");
    ruling!(
        "Festival of Embers",
        "If Festival of Embers is destroyed by a spell, Festivals of Embers will be exiled and then the spell will be put into its owner’s graveyard."
    );
    supported("Festival of Embers");
    let mut t = TestGame::new(2);
    let festival = t.battlefield(P0, "Festival of Embers");
    cast_new(&mut t, P0, "Naturalize", &[obj(festival)]);
    t.resolve_all();
    assert!(t.in_exile("Festival of Embers"));
    assert!(t.in_graveyard(P0, "Naturalize"));

    ruling!(
        "Festival of Embers",
        "If you discard a card while you control Festival of Embers, abilities that function when that card is discarded (such as madness) still work even though that card never reaches a graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Festival of Embers");
    let temper = t.hand(P0, "Fiery Temper");
    pool(&mut t, P0, &[ManaType::R], 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.discard(P0, temper, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "cast for its madness cost");
    assert!(t.in_exile("Fiery Temper"));
}

#[test]
fn two_aura_fluxes_give_each_other_the_ability() {
    cr!("613.1f");
    ruling!(
        "Aura Flux",
        "If two or more Aura Fluxes are on the battlefield, they will give the text to each other. And they will cause each other enchantment to have two triggered abilities for which the player would have to pay."
    );
    supported("Aura Flux");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Aura Flux");
    let b = t.battlefield(P0, "Aura Flux");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    next_upkeep(&mut t);
    assert_eq!(abilities_on_stack(&t), 4);
    t.resolve_all();
    for x in [a, b, anthem] {
        assert!(!t.on_battlefield(x));
    }
}

#[test]
fn a_killer_among_us_copies_have_their_own_choices() {
    cr!("607.2d");
    ruling!(
        "A Killer Among Us",
        "If you control multiple copies of A Killer Among Us, you may choose the same or different creature types for each one."
    );
    supported("A Killer Among Us");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // Human
    let human_killer = t.enter(P0, "A Killer Among Us");
    t.resolve_all();
    t.answer(P0, DecisionKind::Option, Answer::Index(2)); // Goblin
    let goblin_killer = t.enter(P0, "A Killer Among Us");
    t.resolve_all();
    let of = |t: &TestGame, ty: &str| -> Vec<ObjectId> {
        t.g.battlefield
            .iter()
            .copied()
            .filter(|id| t.g.obj(*id).chars.has_subtype(ty))
            .collect()
    };
    let (goblin, human) = (of(&t, "Goblin")[0], of(&t, "Human")[0]);
    for id in [goblin, human] {
        t.g.objects[id.0 as usize].summoning_sick = false;
    }
    attack_with(
        &mut t,
        &[(goblin, Entity::Player(P1)), (human, Entity::Player(P1))],
    );
    t.resolve_all();
    // The Goblin-choosing copy on the Human: nothing; the Human-choosing one: counters.
    t.activate(P0, goblin_killer, 0, &[obj(human)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(human, "+1/+1"), 0);
    t.activate(P0, human_killer, 0, &[obj(human)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(human, "+1/+1"), 3);
}

#[test]
fn no_rest_for_the_wicked_returns_creatures_that_died_before_it_entered() {
    cr!("400.7");
    ruling!(
        "No Rest for the Wicked",
        "If you sacrifice No Rest for the Wicked on the same turn you cast it, all creature cards that were put into your graveyard from the battlefield that turn will be returned to your hand, including any that were put into your graveyard before No Rest for the Wicked entered the battlefield."
    );
    supported("No Rest for the Wicked");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    let discarded = t.hand(P0, "Hill Giant");
    t.g.discard(P0, discarded, None);
    t.g.flush_events();
    cast_new(&mut t, P0, "No Rest for the Wicked", &[]);
    t.resolve_all();
    let nrftw = t.named_on_battlefield("No Rest for the Wicked")[0];
    t.activate(P0, nrftw, 0, &[]).expect("No Rest for the Wicked");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"), "discarded, not from the battlefield");
}

#[test]
fn an_animated_vehicle_stays_0_0_when_crewed() {
    cr!("613.4b", "702.122a");
    ruling!(
        "Kenku Artificer",
        "If the target noncreature artifact is a Vehicle, its power and toughness will be set to 0/0. Crewing that Vehicle will not restore its power and toughness."
    );
    ruling!(
        "Case of the Filched Falcon",
        "If the target noncreature artifact is a Vehicle, its power and toughness will be set to 0/0. Crewing that Vehicle will not restore its power and toughness."
    );
    supported("Kenku Artificer");
    supported("Case of the Filched Falcon");
    for name in ["Kenku Artificer", "Case of the Filched Falcon"] {
        let mut t = TestGame::new(2);
        let dragster = t.battlefield(P0, "Daredevil Dragster");
        let giant = t.battlefield(P0, "Hill Giant");
        let n = if name == "Kenku Artificer" {
            t.answer_targets(P0, &[obj(dragster)]);
            t.enter(P0, name);
            t.resolve_all();
            3
        } else {
            let case = t.battlefield(P0, name);
            mtg_engine::cases::solve(&mut t.g, case);
            pool(&mut t, P0, &[ManaType::U], 1);
            pool(&mut t, P0, &[ManaType::C], 2);
            act(&mut t, P0, case, "noncreature artifact", &[obj(dragster)]).expect(name);
            t.resolve_all();
            4
        };
        assert_eq!(t.pt(dragster), (n, n), "{name}");
        t.answer_choose(P0, &[obj(giant)]);
        act(&mut t, P0, dragster, "Crew", &[]).expect("crew");
        t.resolve_all();
        assert_eq!(t.pt(dragster), (n, n), "{name}: still 0/0 plus counters");
    }
}
