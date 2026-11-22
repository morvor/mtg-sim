//! Rulings residue P130 (item rulings-residue-reflexive-refund), part 3: the batch's other
//! cards: mana with spending restrictions and abilities that add mana but aren't mana
//! abilities (Jaya Ballard, The Ringhart Crest, Titans' Nest, Interdimensional Web Watch,
//! Path of Annihilation, Drain Power), cast triggers and delayed triggers that resolve
//! before their spell (Path of Annihilation, Brass Infiniscope), Coveted Jewel's combat
//! trigger, split cards, and the rest.

use crate::r_p129_common::*;
use crate::r_residue_reflexive_refund::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn pool_of(t: &TestGame, p: PlayerId, ty: ManaType) -> usize {
    t.g.player(p).mana_pool.count(ty)
}

/// Whether `p` could cast `card` now, by any method.
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    use mtg_engine::decision::Action;
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

// --- Coveted Jewel ------------------------------------------------------------------------

/// P1 attacks with `attackers` (Grizzly Bears each), P0 blocks with `blocks`; advances into
/// the declare blockers step with its triggers on the stack. P0 controls Coveted Jewel,
/// tapped.
fn jewel_combat(
    t: &mut TestGame,
    attackers: &[(ObjectId, Entity)],
    blocks: &[(ObjectId, ObjectId)],
) -> ObjectId {
    let jewel = t.battlefield(P0, "Coveted Jewel");
    t.g.tap(jewel);
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(t, attackers);
    t.resolve_all();
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    t.settle();
    jewel
}

#[test]
fn coveted_jewel_triggers_after_blocks_if_any_attacker_is_unblocked() {
    cr!("509.1", "603.2", "510.1");
    ruling!(
        "Coveted Jewel",
        "Coveted Jewel's last ability triggers after you declare blockers (or declare no blockers at all) if any attacking creatures are unblocked. It doesn't matter if some attacking creatures were blocked."
    );
    ruling!(
        "Coveted Jewel",
        "Coveted Jewel's last ability resolves after blockers are chosen but before combat damage is dealt."
    );
    supported("Coveted Jewel");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P0, "Wall of Stone");
    let jewel = jewel_combat(
        &mut t,
        &[(a, Entity::Player(P0)), (b, Entity::Player(P0))],
        &[(wall, a)],
    );
    assert_eq!(triggers_on_stack(&t, "aren't blocked"), 1);
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.obj_now(jewel).controller, P1);
    assert!(!t.obj_now(jewel).tapped, "untapped");
    assert_eq!(t.hand_size(P1), hand + 3);
    assert_eq!(t.life(P0), 20, "no combat damage yet");
    // P1 can tap it for mana before combat damage.
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.activate(P1, jewel, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P1), 3);
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 18);
    // No blockers declared at all: it triggers too.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    jewel_combat(&mut t, &[(a, Entity::Player(P0))], &[]);
    assert_eq!(triggers_on_stack(&t, "aren't blocked"), 1);
    // Every attacker blocked: no trigger.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P0, "Wall of Stone");
    jewel_combat(&mut t, &[(a, Entity::Player(P0))], &[(wall, a)]);
    assert_eq!(triggers_on_stack(&t, "aren't blocked"), 0);
}

#[test]
fn coveted_jewel_ignores_creatures_attacking_a_planeswalker() {
    cr!("508.1b");
    ruling!(
        "Coveted Jewel",
        "A creature attacking a planeswalker you control won't cause Coveted Jewel's last ability to trigger."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let pw = t.battlefield(P0, "Ajani Goldmane");
    let jewel = jewel_combat(&mut t, &[(a, obj(pw))], &[]);
    assert_eq!(triggers_on_stack(&t, "aren't blocked"), 0);
    t.resolve_all();
    assert_eq!(t.obj_now(jewel).controller, P0);
}

#[test]
fn coveted_jewel_triggers_for_each_opponent_attacking_you() {
    cr!("805.10a", "603.3b", "810.7");
    interp!("one-or-more-an-opponent-controls-per-opponent");
    ruling!(
        "Coveted Jewel",
        "If more than one opponent attacks you at the same time, Coveted Jewel's last ability triggers for each of them. You choose which player ends up with Coveted Jewel, but each of them draws three cards and has a chance to activate its mana ability."
    );
    // Two-Headed Giant: P2's and P3's creatures attack P0 together. P0 orders the two
    // triggers; each order gives the Jewel to the other player in the end.
    let mut holders = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = crate::r_p108_common::two_headed_giant();
        let jewel = t.battlefield(P0, "Coveted Jewel");
        t.g.tap(jewel);
        let a = t.battlefield(P2, "Grizzly Bears");
        let b = t.battlefield(P3, "Grizzly Bears");
        t.set_step(P2, Step::BeginningOfCombat);
        attack_with(&mut t, &[(a, Entity::Player(P0)), (b, Entity::Player(P0))]);
        t.resolve_all();
        let hands = (t.hand_size(P2), t.hand_size(P3));
        let seen = watch(&mut t, P0, |d| matches!(d, Decision::Order { .. }), |_| ());
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.advance_to(P2, Step::DeclareBlockers);
        t.settle();
        assert_eq!(seen.lock().unwrap().len(), 1, "P0 orders the triggers");
        assert_eq!(triggers_on_stack(&t, "aren't blocked"), 2);
        // Each trigger is for one player: its creature's controller.
        let player_of = |t: &TestGame, id: ObjectId| -> PlayerId {
            let si = t.g.obj(id).stack.as_deref().unwrap();
            let o = si.event.as_ref().and_then(|e| e.object).unwrap();
            t.g.obj(o).controller
        };
        let top_player = player_of(&t, top(&t));
        let bottom_player = player_of(&t, t.g.stack[t.g.stack.len() - 2]);
        assert_ne!(top_player, bottom_player);
        // The top one resolves: its player draws three, gets the Jewel untapped and may
        // tap it for mana.
        t.resolve();
        assert_eq!(t.obj_now(jewel).controller, top_player);
        assert!(!t.obj_now(jewel).tapped);
        t.answer(top_player, DecisionKind::Option, Answer::Index(0));
        t.activate(top_player, jewel, 0, &[]).unwrap();
        assert_eq!(pool_total(&t, top_player), 3, "a chance to tap it");
        t.resolve_all();
        assert_eq!(t.obj_now(jewel).controller, bottom_player);
        assert!(!t.obj_now(jewel).tapped, "untapped again");
        assert_eq!(
            (t.hand_size(P2), t.hand_size(P3)),
            (hands.0 + 3, hands.1 + 3)
        );
        holders.push(bottom_player);
    }
    assert_ne!(holders[0], holders[1], "P0 chooses who ends up with it");
}

#[test]
fn coveted_jewel_that_player_is_the_attacking_opponent_after_a_control_change() {
    cr!("506.4", "603.2");
    interp!("that-opponent-fixed-as-it-triggers");
    // P1's unblocked Bears trigger the Jewel; in response P0 gains control of them (they
    // leave combat, CR 506.4). "That player" is still P1, the opponent whose creature
    // attacked: P1 draws three cards and gains control of the Jewel.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let jewel = jewel_combat(&mut t, &[(a, Entity::Player(P0))], &[]);
    assert_eq!(triggers_on_stack(&t, "aren't blocked"), 1);
    let hands = (t.hand_size(P0), t.hand_size(P1));
    crate::r_p015_common::gain_control(&mut t, P0, a);
    assert_eq!(t.obj_now(a).controller, P0);
    assert!(!t.g.is_attacking(t.g.current(a)), "removed from combat");
    t.resolve_all();
    assert_eq!(t.obj_now(jewel).controller, P1);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (hands.0, hands.1 + 3));
}

#[test]
fn norns_decree_that_opponent_is_the_damaging_creatures_controller_as_it_triggered() {
    cr!("510.2", "603.2");
    interp!("that-opponent-fixed-as-it-triggers");
    // Norn's Decree: "Whenever one or more creatures an opponent controls deal combat
    // damage to you, that opponent gets a poison counter." P0 gains control of P1's Bears
    // after their combat damage: P1 still gets the poison counter, not P0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Norn's Decree");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P0), 18);
    assert_eq!(triggers_on_stack(&t, "poison counter"), 1);
    crate::r_p015_common::gain_control(&mut t, P0, bears);
    t.resolve_all();
    assert_eq!(t.g.player(P1).poison(), 1);
    assert_eq!(t.g.player(P0).poison(), 0);
}

// --- Indulge // Excess --------------------------------------------------------------------

#[test]
fn indulge_on_the_stack_has_only_its_own_half() {
    cr!("709.3", "709.3a");
    ruling!(
        "Indulge // Excess",
        "All split cards have two card faces on a single card, and you put a split card onto the stack with only the half you're casting. The characteristics of the half you didn't cast are ignored while the spell is on the stack."
    );
    supported("Indulge // Excess");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 3);
    let card = t.hand(P0, "Indulge // Excess");
    let spell = t.cast(P0, card).method(CastMethod::Half(0)).go();
    let o = t.obj(spell);
    assert_eq!(o.chars.name, "Indulge");
    assert_eq!(o.chars.mana_value(), 3);
}

#[test]
fn indulge_excess_is_one_card() {
    cr!("709.4", "709.4c");
    ruling!(
        "Indulge // Excess",
        "Each split card is a single card. For example, if you discard one, you've discarded one card, not two. If an effect counts the number of sorcery cards in your graveyard, Indulge /// Excess counts once, not twice."
    );
    let mut t = TestGame::new(2);
    t.hand(P1, "Indulge // Excess");
    cast_new(&mut t, P0, "Mind Rot", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 1, "one card discarded");
    // Spellheart Chimera counts instant and sorcery cards in its controller's graveyard.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Indulge // Excess");
    let chimera = t.battlefield(P0, "Spellheart Chimera");
    assert_eq!(t.pt(chimera).0, 1);
}

#[test]
fn excess_counts_creatures_you_no_longer_control() {
    cr!("608.2h", "702.127a");
    ruling!(
        "Indulge // Excess",
        "Excess counts the number of creatures you controlled that dealt combat damage to a player even if you no longer control one or more of those creatures as it resolves."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (lions, Entity::Player(P1))],
    );
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 16);
    // The Bears now belong to P1's side; the Lions are gone.
    crate::r_p015_common::gain_control(&mut t, P1, bears);
    destroy(&mut t, lions);
    let card = t.graveyard(P0, "Indulge // Excess");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Aftermath))
        .go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 2);
}

// --- Kolvori, God of Kinship // The Ringhart Crest -----------------------------------------

#[test]
fn kolvori_nonlethal_damage_becomes_lethal_when_the_bonus_ends() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Kolvori, God of Kinship // The Ringhart Crest",
        "Because damage remains marked on creatures until the damage is removed as the turn ends, nonlethal damage dealt to Kolvori may become lethal if you controlled three or more legendary creatures and some of them leave the battlefield that turn."
    );
    supported("Kolvori, God of Kinship // The Ringhart Crest");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kolvori, God of Kinship // The Ringhart Crest");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    let other = t.battlefield(P0, "Jasmine Boreal");
    let (p, tough) = t.pt(k);
    let base = mtg_engine::card::card("Kolvori, God of Kinship // The Ringhart Crest")
        .front()
        .chars
        .toughness
        .unwrap_or(0);
    assert_eq!(
        tough,
        base + 2,
        "+4/+2 with three legendary creatures (power {p})"
    );
    // Damage between its base toughness and its boosted one: it survives.
    crate::r_p114_common::deal(&mut t, other, tough - 1, obj(k));
    assert!(t.on_battlefield(k));
    destroy(&mut t, other);
    assert!(!t.on_battlefield(k), "now lethal");
}

#[test]
fn the_ringhart_crest_choice_and_its_mana() {
    cr!("205.3m", "106.6", "614.12");
    ruling!(
        "Kolvori, God of Kinship // The Ringhart Crest",
        "For The Ringhart Crest's first ability, you must choose an existing creature type, such as Fungus or Archer. You can't choose card types (e.g., artifact) or supertypes (e.g., snow)."
    );
    ruling!(
        "Kolvori, God of Kinship // The Ringhart Crest",
        "If The Ringhart Crest is somehow on the battlefield with no choice of creature type, the mana produced by its activated ability can still be spent to cast a legendary creature spell."
    );
    // Cast as The Ringhart Crest: the choice offers creature types only.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    let card = t.hand(P0, "Kolvori, God of Kinship // The Ringhart Crest");
    let from = t.asked().len();
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let crest = t.named_on_battlefield("The Ringhart Crest");
    assert_eq!(crest.len(), 1);
    let offered: Vec<String> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                options, prompt, ..
            } if prompt.to_lowercase().contains("creature type") => Some(options.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.iter().any(|o| o == "Fungus"));
    for not in ["Artifact", "Snow", "Legendary", "Creature"] {
        assert!(!offered.iter().any(|o| o == not), "{not} offered");
    }
    let chosen = t.obj_now(crest[0]).choices.creature_type.clone();
    assert!(chosen.is_some_and(|c| offered.iter().any(|o| *o == c.as_str())));
    // On the battlefield with no choice made: its mana can pay for a legendary creature
    // spell, not for a nonlegendary one.
    for (name, ok) in [("Fynn, the Fangbearer", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let crest = crate::r_p179_common::battlefield_back(
            &mut t,
            P0,
            "Kolvori, God of Kinship // The Ringhart Crest",
        );
        assert_eq!(t.obj_now(crest).chars.name, "The Ringhart Crest");
        assert!(t.obj_now(crest).choices.creature_type.is_none());
        t.activate(P0, crest, 0, &[]).unwrap();
        assert_eq!(pool_of(&t, P0, ManaType::G), 1);
        t.lands(P0, "Wastes", 1);
        let c = t.hand(P0, name);
        assert_eq!(t.cast(P0, c).try_go().is_ok(), ok, "{name}");
    }
}

// --- Jaya Ballard -------------------------------------------------------------------------

#[test]
fn jaya_ballards_mana_ability_is_a_loyalty_ability() {
    cr!("606.3", "605.1a", "606.4");
    ruling!(
        "Jaya Ballard",
        "Because it’s a loyalty ability, Jaya’s first ability isn’t a mana ability. It can be activated only any time you could cast a sorcery. It uses the stack and can be responded to."
    );
    ruling!(
        "Jaya Ballard",
        "Mana produced by Jaya’s first ability can be spent among any number of instant and/or sorcery spells."
    );
    supported("Jaya Ballard");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let jaya = t.battlefield(P0, "Jaya Ballard");
    let loyalty_before = t.counters(jaya, counters::LOYALTY);
    t.activate(P0, jaya, 0, &[]).unwrap();
    assert_eq!(t.counters(jaya, counters::LOYALTY), loyalty_before + 1);
    assert_eq!(t.stack_len(), 1, "it uses the stack");
    assert_eq!(pool_of(&t, P0, ManaType::R), 0);
    t.resolve();
    assert_eq!(pool_of(&t, P0, ManaType::R), 3);
    // Two instants from the three red mana; not a creature spell.
    let goblin = t.hand(P0, "Raging Goblin");
    assert!(t.cast(P0, goblin).try_go().is_err());
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // Not with something on the stack, and not on another player's turn.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let jaya = t.battlefield(P0, "Jaya Ballard");
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert!(t.activate(P0, jaya, 0, &[]).is_err());
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.activate(P0, jaya, 0, &[]).is_err());
}

#[test]
fn jayas_emblem_only_lets_you_cast_them() {
    cr!("114.4", "702.29a", "601.3");
    ruling!(
        "Jaya Ballard",
        "Jaya’s emblem doesn’t grant you permission to do anything with instant and sorcery cards in your graveyard except cast them. For example, you can’t cycle instant or sorcery cards with cycling from your graveyard."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let jaya = t.battlefield(P0, "Jaya Ballard");
    loyalty(&mut t, jaya, 8);
    t.activate(P0, jaya, 2, &[]).unwrap();
    t.resolve_all();
    let faith = t.graveyard(P0, "Renewed Faith");
    t.lands(P0, "Plains", 4);
    assert!(!can_cycle(&mut t, P0, faith));
    assert!(cycle(&mut t, P0, faith, 0).is_err());
    t.cast(P0, faith).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert!(is_in(&t, faith, Zone::Exile), "exiled instead");
}

// --- Drain Power --------------------------------------------------------------------------

#[test]
fn drain_power_target_player_chooses_a_variable_amount() {
    cr!("602.2b", "106.4");
    ruling!(
        "Drain Power",
        "If a land can draw a variable amount of mana, the target player (not the player of this spell) chooses how much to draw."
    );
    supported("Bottomless Vault");
    let mut t = TestGame::new(2);
    let skerry = t.battlefield(P1, "Bottomless Vault");
    put_counters(&mut t, skerry, "storage", 3);
    // P1 removes two of the three storage counters.
    t.answer(P1, DecisionKind::Number, Answer::Number(2));
    t.answer(P1, DecisionKind::X, Answer::Number(2));
    let from = t.asked().len();
    cast_new(&mut t, P0, "Drain Power", &[Entity::Player(P1)]);
    t.resolve_all();
    let p1_chose = t.asked()[from..].iter().any(|(p, d)| {
        *p == P1 && matches!(d, Decision::ChooseNumber { .. } | Decision::ChooseX { .. })
    });
    assert!(p1_chose, "P1 chooses how much");
    assert!(!t.asked()[from..].iter().any(|(p, d)| {
        *p == P0 && matches!(d, Decision::ChooseNumber { .. } | Decision::ChooseX { .. })
    }));
    assert_eq!(t.counters(skerry, "storage"), 1);
    assert_eq!(pool_of(&t, P0, ManaType::B), 2);
}

// --- Culmination of Studies ---------------------------------------------------------------

#[test]
fn culmination_of_studies_counts_a_card_for_each_quality() {
    cr!("105.2b", "120.2b");
    ruling!(
        "Culmination of Studies",
        "If an exiled card is both blue and red, Culmination of Studies deals 1 damage to each opponent and you draw a card."
    );
    ruling!(
        "Culmination of Studies",
        "If multiple red cards are exiled, all of the damage is dealt as a single event."
    );
    supported("Culmination of Studies");
    // Electrolyze (blue and red): 1 damage and a card.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Electrolyze", "Island", "Island"]);
    let hand = t.hand_size(P0);
    lands_for_cost(&mut t, P0, "Culmination of Studies");
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Culmination of Studies");
    t.cast(P0, card).x(1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Three red cards: one 3-damage event.
    let mut t = TestGame::new(2);
    stack_library(
        &mut t,
        P0,
        &["Lightning Bolt", "Shock", "Lava Spike", "Island"],
    );
    lands_for_cost(&mut t, P0, "Culmination of Studies");
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Culmination of Studies");
    let before = t.g.turn_events.len();
    t.cast(P0, card).x(3).go();
    t.resolve_all();
    let hits: Vec<u32> = t.g.turn_events[before..]
        .iter()
        .filter_map(|e| match e {
            Event::Damage { target, amount, .. } if *target == Entity::Player(P1) => Some(*amount),
            _ => None,
        })
        .collect();
    assert_eq!(hits, vec![3]);
}

// --- Teferi, Temporal Archmage and The Chain Veil -----------------------------------------

#[test]
fn teferis_emblem_with_the_chain_veil() {
    cr!("606.3", "114.4");
    ruling!(
        "Teferi, Temporal Archmage",
        "If you control The Chain Veil (from the Magic 2015 core set) and its activated ability has resolved, and you also control Teferi’s emblem, for each planeswalker you control, you can activate one of its loyalty abilities during that turn any time you could cast an instant, and then you may activate a loyalty ability again that turn (choosing the same loyalty ability or a different one) any time you could cast an instant."
    );
    supported("Teferi, Temporal Archmage");
    supported("The Chain Veil");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let teferi = t.battlefield(P0, "Teferi, Temporal Archmage");
    loyalty(&mut t, teferi, 12);
    t.activate(P0, teferi, 2, &[]).unwrap();
    t.resolve_all();
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    loyalty(&mut t, ajani, 4);
    let veil = t.battlefield(P0, "The Chain Veil");
    t.lands(P0, "Wastes", 4);
    activate_containing(&mut t, P0, veil, "For each planeswalker").unwrap();
    t.resolve_all();
    // In the beginning of combat step (instant timing), with a spell on the stack: Ajani
    // twice (+1, then +1 again), but not a third time.
    t.advance_to(P0, Step::BeginningOfCombat);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert!(t.activate(P0, ajani, 0, &[]).is_ok(), "first activation");
    t.resolve();
    assert!(
        t.activate(P0, ajani, 0, &[]).is_ok(),
        "again (The Chain Veil)"
    );
    t.resolve_all();
    assert_eq!(t.counters(ajani, counters::LOYALTY), 6);
    assert!(t.activate(P0, ajani, 0, &[]).is_err(), "not a third time");
    // Teferi, whose ability was already activated this turn, gets one more.
    assert!(t.activate(P0, teferi, 0, &[]).is_ok());
    t.resolve_all();
    assert!(t.activate(P0, teferi, 0, &[]).is_err());
}

// --- Urza, Lord High Artificer ------------------------------------------------------------

#[test]
fn urza_construct_counts_itself_and_the_exiled_card_stays_in_exile() {
    cr!("611.3a", "406.1", "611.2a");
    ruling!(
        "Urza, Lord High Artificer",
        "The token created by Urza's first ability will count itself, so it'll be at least 1/1."
    );
    ruling!(
        "Urza, Lord High Artificer",
        "If you don't play the card exiled with Urza's last ability, it remains in exile."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let urza = t.enter(P0, "Urza, Lord High Artificer");
    t.settle();
    t.resolve_all();
    let construct = with_subtype(&t, P0, "Construct");
    assert_eq!(construct.len(), 1);
    assert_eq!(t.pt(construct[0]), (1, 1));
    // The {5} ability: the exiled card isn't played and stays in exile.
    t.g.players[0].library.clear();
    let giant = t.library_top(P0, "Hill Giant");
    t.lands(P0, "Wastes", 5);
    activate_containing(&mut t, P0, urza, "Shuffle").unwrap();
    t.resolve_all();
    assert!(is_in(&t, giant, Zone::Exile));
    assert!(castable(&mut t, P0, giant));
    stack_library(&mut t, P0, &["Island", "Island"]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(is_in(&t, giant, Zone::Exile));
    t.lands(P0, "Wastes", 4);
    assert!(!castable(&mut t, P0, giant));
}

// --- Titans' Nest -------------------------------------------------------------------------

#[test]
fn titans_nest_surveil_keeps_the_card_on_top_for_the_draw() {
    cr!("701.25a", "504.1");
    ruling!(
        "Titans' Nest",
        "If you don’t put the card you look at into your graveyard, it remains on top of your library. You’ll draw it during your draw step unless something else moves it."
    );
    supported("Titans' Nest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Titans' Nest");
    t.set_step(P1, Step::End);
    t.library_top(P0, "Hill Giant");
    t.answer(P0, DecisionKind::Surveil, Answer::Indices(vec![]));
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert!(!t.in_graveyard(P0, "Hill Giant"));
    t.advance_to(P0, Step::Draw);
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn titans_nest_mana_pays_for_x_in_an_additional_cost() {
    cr!("106.6", "107.3", "702.33a");
    ruling!(
        "Titans' Nest",
        "Mana produced by Titans’ Nest can be used to cast a spell that has {X} in an additional or alternative cost."
    );
    supported("Thieving Skydiver");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let nest = t.battlefield(P0, "Titans' Nest");
    let a = t.graveyard(P0, "Island");
    let b = t.graveyard(P0, "Island");
    t.answer_choose(P0, &[obj(a)]);
    t.activate(P0, nest, 0, &[]).unwrap();
    t.answer_choose(P0, &[obj(b)]);
    t.activate(P0, nest, 0, &[]).unwrap();
    assert_eq!(pool_of(&t, P0, ManaType::C), 2);
    t.lands(P0, "Island", 1);
    let thopter = t.battlefield(P1, "Ornithopter");
    let sky = t.hand(P0, "Thieving Skydiver");
    t.answer_targets(P0, &[obj(thopter)]);
    t.cast(P0, sky).kicked(true).x(1).go();
    assert_eq!(
        pool_of(&t, P0, ManaType::C),
        0,
        "{{1}} and {{X}} paid with it"
    );
    t.resolve_all();
    assert_eq!(t.obj_now(thopter).controller, P0);
}

// --- Path of Annihilation -----------------------------------------------------------------

#[test]
fn path_of_annihilation_trigger_resolves_before_a_countered_spell() {
    cr!("603.3", "405.5", "608.2");
    ruling!(
        "Path of Annihilation",
        "Path of Annihilation's last ability resolves before the spell that causes it to trigger. The ability will resolve even if that spell is countered or otherwise leaves the stack."
    );
    supported("Path of Annihilation");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Path of Annihilation");
    let crusher = cast_new(&mut t, P0, "Ulamog's Crusher", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "you gain 4 life"), 1);
    // In response, P1 counters the Crusher.
    lands_for_cost(&mut t, P1, "Cancel");
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(obj(crusher)).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Ulamog's Crusher"));
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn path_of_annihilation_as_an_eldrazi_grants_itself_the_mana_ability() {
    cr!("613.1d", "613.1f", "205.3m");
    ruling!(
        "Path of Annihilation",
        "In the rare case where Path of Annihilation is an Eldrazi, it will grant itself the ability \"{T}: Add one mana of any color.\""
    );
    supported("Opalescence");
    supported("Maskwood Nexus");
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Path of Annihilation");
    t.battlefield(P0, "Opalescence");
    t.battlefield(P0, "Maskwood Nexus");
    t.g.recompute();
    assert!(t.obj_now(path).chars.has_subtype("Eldrazi"));
    assert!(t
        .obj_now(path)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("Add one mana of any color")));
    // (It's a creature now, so it needs haste to tap; it has been there since the turn
    // began.)
    let p = t.g.current(path);
    t.g.obj_mut(p).summoning_sick = false;
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    activate_containing(&mut t, P0, path, "Add one mana").unwrap();
    assert_eq!(pool_total(&t, P0), 1);
}

// --- Interdimensional Web Watch -----------------------------------------------------------

#[test]
fn web_watch_mana_pays_for_any_spell_cast_from_exile() {
    cr!("106.6", "601.2h");
    ruling!(
        "Interdimensional Web Watch",
        "Mana from Interdimensional Web Watch's last ability may be spent on any spells cast from exile, not just spells cast from exile via its first ability."
    );
    supported("Interdimensional Web Watch");
    supported("Misthollow Griffin");
    for (from_exile, ok) in [(true, true), (false, false)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let watch = t.battlefield(P0, "Interdimensional Web Watch");
        t.answer(P0, DecisionKind::Any, Answer::Index(1));
        t.activate(P0, watch, 0, &[]).unwrap();
        t.clear_answers();
        assert_eq!(pool_total(&t, P0), 2);
        t.lands(P0, "Island", 2);
        let griffin = if from_exile {
            t.exile(P0, "Misthollow Griffin")
        } else {
            t.hand(P0, "Misthollow Griffin")
        };
        let r = t.cast(P0, griffin).try_go();
        if ok {
            assert!(r.is_ok());
            assert_eq!(pool_total(&t, P0), 0);
        } else {
            // From hand, only the two Islands are usable: not enough.
            assert!(r.is_err());
            assert_eq!(pool_total(&t, P0), 2);
        }
    }
}

// --- Brass Infiniscope --------------------------------------------------------------------

#[test]
fn brass_infiniscope_trigger_uses_the_countered_spells_x() {
    cr!("603.7", "608.2h");
    ruling!(
        "Brass Infiniscope",
        "The delayed triggered ability created by Brass Infiniscope's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving. In that case, use the value of X as that spell last existed on the stack to determine the value of X for the delayed triggered ability."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let scope = t.battlefield(P0, "Brass Infiniscope");
    t.activate(P0, scope, 0, &[]).unwrap();
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let blaze = t.hand(P0, "Blaze");
    let spell = t.cast(P0, blaze).x(5).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "the delayed trigger is above the spell");
    lands_for_cost(&mut t, P1, "Cancel");
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(obj(spell)).go();
    let hand = t.hand_size(P0);
    t.resolve();
    assert!(t.in_graveyard(P0, "Blaze"));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 22, "half of 5, rounded down");
}
