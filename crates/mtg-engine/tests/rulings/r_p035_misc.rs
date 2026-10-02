//! Rulings batch P035 — assorted "counters matter" cards: exile-until effects and tokens,
//! commander-identity mana, Mutagen tokens, first main phase triggers, values locked in as
//! a spell resolves or is cast, and mana abilities.

use crate::r_p035_common::*;
use crate::r_s01_common::{attack_with, give_mana_for, supported, tokens};
use crate::r_s02_common::destroy;
use crate::r_s13_common::commander;
use mtg_engine::ability::{AbilityKind, Effect, PlayerRef, TokenSpec, Value};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType, ColorSet};
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

fn options_asked(t: &TestGame) -> Vec<Vec<String>> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .collect()
}

/// Creates a 1/1 token with the given card types for `p` (as an effect would).
fn token_of_types(t: &mut TestGame, p: PlayerId, types: Vec<CardType>) -> ObjectId {
    let spec = TokenSpec {
        name: "Spirit".into(),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: types,
        subtypes: vec!["Spirit".into()],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        scryfall_name: None,
        pt_values: None,
    };
    let before: Vec<ObjectId> = t.g.battlefield.clone();
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
    *t.g.battlefield
        .iter()
        .find(|id| !before.contains(id))
        .expect("token created")
}

// ---------------------------------------------------------------------------------------
// Exile until ~ leaves the battlefield
// ---------------------------------------------------------------------------------------

#[test]
fn isolation_zone_exiled_token_doesnt_return() {
    cr!("111.7", "704.5d", "610.3a");
    ruling!(
        "Isolation Zone",
        "If a token is exiled, it ceases to exist. It won’t be returned to the battlefield."
    );
    supported("Isolation Zone");
    let mut t = TestGame::new(2);
    let token = token_of_types(&mut t, P1, vec![CardType::Creature]);
    t.answer_targets(P0, &[Entity::Object(token)]);
    let zone = t.enter(P0, "Isolation Zone");
    t.resolve_all();
    assert!(!t.g.is_live(token));
    assert!(tokens(&t, P1).is_empty(), "the token is gone");
    destroy(&mut t, zone);
    t.resolve_all();
    assert!(tokens(&t, P1).is_empty(), "the token didn't come back");
    // A nontoken creature does return.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let zone = t.enter(P0, "Isolation Zone");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    destroy(&mut t, zone);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

// ---------------------------------------------------------------------------------------
// Festering Wound: two upkeep triggers, ordered by their controller
// ---------------------------------------------------------------------------------------

#[test]
fn festering_wound_on_your_creature_order_the_triggers() {
    cr!("603.3b");
    ruling!(
        "Festering Wound",
        "If it is on your creature, you can decide whether to add a counter before or after it damages you."
    );
    supported("Festering Wound");
    let mut damages = vec![];
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let wound = crate::r_s06_common::attach_new(&mut t, P0, "Festering Wound", bears);
        put(&mut t, wound, "infection", 2);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.answer_yes(P0, true);
        t.advance_to(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        t.resolve_all();
        assert_eq!(t.counters(wound, "infection"), 3);
        damages.push(20 - t.life(P0));
    }
    damages.sort();
    assert_eq!(
        damages,
        vec![2, 3],
        "damage before or after the new counter"
    );
}

// ---------------------------------------------------------------------------------------
// Hidden Hideout: mana of your commander's color identity
// ---------------------------------------------------------------------------------------

#[test]
fn hidden_hideout_no_commander_no_mana() {
    cr!("903.4f");
    ruling!(
        "Hidden Hideout",
        "If you don't have a commander, Hidden Hideout's first ability produces no mana."
    );
    supported("Hidden Hideout");
    let mut t = TestGame::new(2);
    commander(&mut t, P1, "Niv-Mizzet, Parun");
    let hideout = t.battlefield(P0, "Hidden Hideout");
    t.activate(P0, hideout, 0, &[]).unwrap();
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn hidden_hideout_colorless_commander_no_mana() {
    cr!("903.4f");
    ruling!(
        "Hidden Hideout",
        "If your commander is a card that has no colors in its color identity, Hidden Hideout's first ability produces no mana. It doesn't produce {C}."
    );
    let mut t = TestGame::new(2);
    commander(&mut t, P0, "Karn, Legacy Reforged");
    let hideout = t.battlefield(P0, "Hidden Hideout");
    t.activate(P0, hideout, 0, &[]).unwrap();
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn hidden_hideout_two_commanders_combined_identity() {
    cr!("903.4", "702.124c");
    ruling!(
        "Hidden Hideout",
        "If you have two commanders, Hidden Hideout's first ability adds one mana of any color in their combined color identities."
    );
    let mut t = TestGame::new(2);
    commander(&mut t, P0, "Thrasios, Triton Hero");
    commander(&mut t, P0, "Tymna the Weaver");
    let hideout = t.battlefield(P0, "Hidden Hideout");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, hideout, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
    assert_eq!(
        options_asked(&t),
        vec![vec![
            "W".to_string(),
            "U".to_string(),
            "B".to_string(),
            "G".to_string()
        ]]
    );
}

// ---------------------------------------------------------------------------------------
// Mutagen tokens
// ---------------------------------------------------------------------------------------

#[test]
fn mutagen_tokens_are_predefined_artifacts() {
    cr!("111.10v", "602.5d");
    ruling!(
        "Genghis Frog",
        "Mutagen tokens are a kind of predefined token. Each one is a colorless artifact with the artifact subtype Mutagen and the ability"
    );
    supported("Genghis Frog");
    let mut t = TestGame::new(2);
    t.enter(P0, "Genghis Frog");
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let m = toks[0];
    let o = t.obj_now(m);
    assert!(o.chars.colors.is_colorless(), "colorless");
    assert_eq!(
        o.chars.card_types,
        mtg_engine::types::CardTypeSet::single(CardType::Artifact)
    );
    assert!(o.chars.subtypes.iter().any(|s| s.as_str() == "Mutagen"));
    let activated = o
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count();
    assert_eq!(activated, 1);
    // Activate only as a sorcery: not while something is on the stack.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P1).go();
    assert!(t.activate(P0, m, 0, &[Entity::Object(bears)]).is_err());
    t.resolve_all();
    t.clear_answers();
    t.activate(P0, m, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert!(tokens(&t, P0).is_empty(), "sacrificed");
}

#[test]
fn return_to_the_sewers_illegal_target_no_mutagen() {
    cr!("608.2b");
    ruling!(
        "Return to the Sewers",
        "Some spells and abilities that create Mutagen tokens require targets. If each target chosen is an illegal target as that spell or ability tries to resolve, it won't resolve. The Mutagen token won't be created."
    );
    supported("Return to the Sewers");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Return to the Sewers");
    let card = t.hand(P0, "Return to the Sewers");
    t.cast(P0, card).target(bears).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty(), "no Mutagen token");
}

// ---------------------------------------------------------------------------------------
// Nev, the Practical Dean
// ---------------------------------------------------------------------------------------

#[test]
fn nev_first_x_spell_before_nev_entered_counts() {
    cr!("603.2");
    ruling!(
        "Nev, the Practical Dean",
        "If you cast your first spell with {X} in its mana cost during a turn before Nev is on the battlefield, casting another spell with {X} in its mana cost later in the turn won't cause his last ability to trigger."
    );
    supported("Nev, the Practical Dean");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(1).target(P1).go();
    t.resolve_all();
    let nev = t.battlefield(P0, "Nev, the Practical Dean");
    let blaze2 = t.hand(P0, "Blaze");
    t.cast(P0, blaze2).x(2).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 1, "no trigger");
    t.resolve_all();
    assert_eq!(t.counters(nev, counters::PLUS1), 0);
    // With Nev out first, the first one triggers.
    let mut t = TestGame::new(2);
    let nev = t.battlefield(P0, "Nev, the Practical Dean");
    t.lands(P0, "Mountain", 3);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(2).target(P1).go();
    t.resolve_all();
    assert_eq!(t.counters(nev, counters::PLUS1), 2);
}

// ---------------------------------------------------------------------------------------
// Blocked creatures stay blocked
// ---------------------------------------------------------------------------------------

#[test]
fn michelangelo_counter_after_blocks_doesnt_unblock() {
    cr!("509.1h", "509.1b");
    ruling!(
        "Michelangelo, Mutant BFF",
        "Once a creature you control has been blocked, putting a counter on that creature won't cause it to stop being blocked."
    );
    supported("Michelangelo, Mutant BFF");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Michelangelo, Mutant BFF");
    let giant = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Savannah Lions");
    crate::r_s03_common::to_blockers(
        &mut t,
        &[(giant, Entity::Player(P1))],
        &[(a, giant), (b, giant)],
    );
    put(&mut t, giant, counters::PLUS1, 1);
    let blocks = crate::r_s21_common::blocks_now(&t);
    assert!(blocks.contains(&(a, giant)) && blocks.contains(&(b, giant)));
    assert!(crate::r_p076_common::is_blocked(&t, giant));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------------------
// Lumbering Megasloth: cost reduction
// ---------------------------------------------------------------------------------------

#[test]
fn lumbering_megasloth_mana_value_is_always_12() {
    cr!("202.3", "601.2f");
    ruling!(
        "Lumbering Megasloth",
        "The cost-reduction ability of Lumbering Megasloth doesn't change its mana cost or mana value, only the total cost you pay. Specifically, the mana value of Lumbering Megasloth is always 12."
    );
    supported("Lumbering Megasloth");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 3);
    put(&mut t, P1, counters::POISON, 2);
    // 12 - 5 = 7 mana.
    t.lands(P0, "Forest", 7);
    let sloth = t.hand(P0, "Lumbering Megasloth");
    assert_eq!(t.g.mana_value_of(sloth), 12);
    let spell = t.cast(P0, sloth).go();
    assert_eq!(t.g.mana_value_of(spell), 12, "on the stack");
    t.resolve_all();
    assert_eq!(
        t.g.mana_value_of(t.g.current(sloth)),
        12,
        "on the battlefield"
    );
}

#[test]
fn lumbering_megasloth_total_cost_is_locked_in() {
    cr!("601.2f", "601.2h");
    ruling!(
        "Lumbering Megasloth",
        "Once you announce that you're casting Lumbering Megasloth, no player may take actions until you're done casting it."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 4);
    let forests = t.lands(P0, "Forest", 10);
    let sloth = t.hand(P0, "Lumbering Megasloth");
    let from = t.asked().len();
    t.cast(P0, sloth).go();
    // No player got priority while it was being cast, and 8 mana was paid.
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    let tapped = forests.iter().filter(|f| t.obj_now(**f).tapped).count();
    assert_eq!(tapped, 8);
    // Removing the counters afterwards changes nothing.
    let b = t.g.current(bears);
    t.g.remove_counters(Entity::Object(b), counters::PLUS1, 4);
    t.resolve_all();
    assert!(t.on_battlefield(sloth));
    assert_eq!(forests.iter().filter(|f| t.obj_now(**f).tapped).count(), 8);
}

// ---------------------------------------------------------------------------------------
// Mind Unbound
// ---------------------------------------------------------------------------------------

#[test]
fn mind_unbound_draws_are_mandatory() {
    cr!("603.5", "121.2");
    ruling!(
        "Mind Unbound",
        "The ability is mandatory. You can’t choose to draw fewer cards than the number of lore counters on Mind Unbound."
    );
    supported("Mind Unbound");
    let mut t = TestGame::new(2);
    let mind = t.battlefield(P0, "Mind Unbound");
    put(&mut t, mind, counters::LORE, 2);
    for _ in 0..8 {
        t.library_top(P0, "Island");
    }
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.counters(mind, counters::LORE), 3);
    assert_eq!(t.hand_size(P0) - hand, 3);
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::YesNo { .. } | Decision::ChooseNumber { .. })));
}

// ---------------------------------------------------------------------------------------
// First main phase triggers
// ---------------------------------------------------------------------------------------

#[test]
fn altar_of_shadows_triggers_only_in_the_first_main_phase() {
    cr!("505.1a");
    ruling!(
        "Altar of Shadows",
        "The precombat main phase is the first main phase of the turn. All others are postcombat main phases"
    );
    supported("Altar of Shadows");
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, "Altar of Shadows");
    put(&mut t, altar, counters::CHARGE, 2);
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    t.resolve_all();
    assert_eq!(pool(&t, P0), vec![ManaType::B, ManaType::B]);
    t.advance_to(P0, Step::PostcombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 0, "not in the second main phase");
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn blinkmoth_urn_triggers_in_each_players_first_main_phase() {
    cr!("505.1a");
    ruling!(
        "Blinkmoth Urn",
        "The precombat main phase is the first main phase of the turn. All others are postcombat main phases"
    );
    supported("Blinkmoth Urn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blinkmoth Urn");
    t.battlefield(P1, "Sol Ring");
    t.battlefield(P1, "Mind Stone");
    t.advance_to(P1, Step::PrecombatMain);
    t.settle();
    t.resolve_all();
    assert_eq!(pool(&t, P1), vec![ManaType::C, ManaType::C]);
    t.advance_to(P1, Step::PostcombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 0, "not in the second main phase");
}

// ---------------------------------------------------------------------------------------
// Values locked in as a spell resolves
// ---------------------------------------------------------------------------------------

#[test]
fn hazardous_conditions_affected_set_is_locked_in() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Hazardous Conditions",
        "The set of creatures affected by Hazardous Conditions is determined as the spell resolves."
    );
    supported("Hazardous Conditions");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    put(&mut t, wurm, counters::CHARGE, 1);
    give_mana_for(&mut t, P0, "Hazardous Conditions");
    let hc = t.hand(P0, "Hazardous Conditions");
    t.cast(P0, hc).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    assert_eq!(t.pt(wurm), (6, 4));
    // The Giant gets a counter: still -2/-2.
    put(&mut t, giant, counters::CHARGE, 1);
    assert_eq!(t.pt(giant), (1, 1));
    // The Wurm loses its counter: still unaffected.
    let w = t.g.current(wurm);
    t.g.remove_counters(Entity::Object(w), counters::CHARGE, 1);
    t.g.recompute();
    assert_eq!(t.pt(wurm), (6, 4));
    // A creature entering later isn't affected.
    let bears = t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn twenty_toed_toad_counts_are_checked_on_resolution() {
    cr!("603.2", "608.2h");
    ruling!(
        "Twenty-Toed Toad",
        "Twenty-Toed Toad's last ability will trigger whenever it attacks, no matter how many counters are on it or cards you have in your hand at that time."
    );
    supported("Twenty-Toed Toad");
    let mut t = TestGame::new(2);
    let toad = t.battlefield(P0, "Twenty-Toed Toad");
    put(&mut t, toad, counters::PLUS1, 19);
    attack_with(&mut t, &[(toad, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1, "it triggered with 19 counters");
    put(&mut t, toad, counters::PLUS1, 1);
    t.resolve_all();
    assert!(t.has_lost(P1), "twenty counters on resolution");
    // Twenty counters when it attacks, nineteen on resolution: nothing.
    let mut t = TestGame::new(2);
    let toad = t.battlefield(P0, "Twenty-Toed Toad");
    put(&mut t, toad, counters::PLUS1, 20);
    attack_with(&mut t, &[(toad, Entity::Player(P1))]);
    let s = t.g.current(toad);
    t.g.remove_counters(Entity::Object(s), counters::PLUS1, 1);
    t.resolve_all();
    assert!(!t.has_lost(P1));
}

// ---------------------------------------------------------------------------------------
// Mana abilities
// ---------------------------------------------------------------------------------------

#[test]
fn twitching_doll_mana_ability_doesnt_use_the_stack() {
    cr!("605.1a", "605.3b");
    ruling!(
        "Twitching Doll",
        "Twitching Doll's first ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Twitching Doll");
    let mut t = TestGame::new(2);
    let doll = t.battlefield(P0, "Twitching Doll");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let r = t.activate(P0, doll, 0, &[]).unwrap();
    assert!(r.is_none(), "nothing put on the stack");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0).len(), 1);
    assert_eq!(t.counters(doll, "nest"), 1);
}

#[test]
fn everflowing_chalice_unkicked_produces_no_mana() {
    cr!("702.33c", "106.5");
    ruling!(
        "Everflowing Chalice",
        "You can cast Everflowing Chalice without kicking it at all if you wish. However, if Everflowing Chalice has no charge counters on it, activating its last ability won't produce any mana."
    );
    supported("Everflowing Chalice");
    let mut t = TestGame::new(2);
    let chalice = t.hand(P0, "Everflowing Chalice");
    t.answer(P0, DecisionKind::Number, Answer::Number(0));
    t.cast(P0, chalice).kicked(false).go();
    t.resolve_all();
    assert!(t.on_battlefield(chalice));
    assert_eq!(t.counters(chalice, counters::CHARGE), 0);
    t.clear_answers();
    let c = t.g.current(chalice);
    t.activate(P0, c, 0, &[]).unwrap();
    assert!(t.obj_now(c).tapped);
    assert!(pool(&t, P0).is_empty());
}

// ---------------------------------------------------------------------------------------
// Daxos the Returned: cast triggers vs. enters triggers
// ---------------------------------------------------------------------------------------

#[test]
fn daxos_enchantment_token_doesnt_trigger_but_constellation_does() {
    cr!("603.2", "111.1");
    ruling!(
        "Daxos the Returned",
        "Putting an enchantment creature token onto the battlefield won’t cause Daxos’s first ability to trigger."
    );
    supported("Eidolon of Blossoms");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Daxos the Returned");
    t.battlefield(P0, "Eidolon of Blossoms");
    t.library_top(P0, "Island");
    let hand = t.hand_size(P0);
    token_of_types(&mut t, P0, vec![CardType::Enchantment, CardType::Creature]);
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter(counters::EXPERIENCE), 0);
    assert_eq!(t.hand_size(P0), hand + 1, "constellation triggered");
    // Casting an enchantment spell does give an experience counter.
    t.library_top(P0, "Island");
    give_mana_for(&mut t, P0, "Eidolon of Blossoms");
    let e = t.hand(P0, "Eidolon of Blossoms");
    t.cast(P0, e).go();
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter(counters::EXPERIENCE), 1);
}

// ---------------------------------------------------------------------------------------
// Nissa, Steward of Elements
// ---------------------------------------------------------------------------------------

#[test]
fn nissa_ultimate_sets_base_pt_counters_still_apply() {
    cr!("613.4b", "613.4c");
    ruling!(
        "Nissa, Steward of Elements",
        "Nissa's last ability sets the base power and toughness of the lands to 5/5. Any +1/+1 or -1/-1 counters that were on those lands will apply to those values."
    );
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa, Steward of Elements");
    put(&mut t, nissa, counters::LOYALTY, 6);
    let a = t.battlefield(P0, "Forest");
    let b = t.battlefield(P0, "Island");
    put(&mut t, a, counters::PLUS1, 2);
    put(&mut t, b, counters::MINUS1, 1);
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    crate::r_s06_common::activate_containing(&mut t, P0, nissa, "5/5").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(a), (7, 7));
    assert_eq!(t.pt(b), (4, 4));
}

// ---------------------------------------------------------------------------------------
// Lux Artillery: separate instances of sunburst
// ---------------------------------------------------------------------------------------

#[test]
fn lux_artillery_each_instance_of_sunburst_works_separately() {
    cr!("702.44d", "702.44a");
    ruling!(
        "Lux Artillery",
        "If the artifact creature already has sunburst (or if you control multiple copies of Lux Artillery), each instance of sunburst works separately."
    );
    supported("Bronze Sable");
    supported("Etched Oracle");
    // Two Lux Artilleries: two instances of sunburst, each counting two colors.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lux Artillery");
    t.battlefield(P0, "Lux Artillery");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let sable = t.hand(P0, "Bronze Sable");
    t.cast(P0, sable).go();
    t.resolve_all();
    assert_eq!(t.counters(sable, counters::PLUS1), 4);
    // An artifact creature that already has sunburst gets a second instance.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lux Artillery");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let oracle = t.hand(P0, "Etched Oracle");
    t.cast(P0, oracle).go();
    t.resolve_all();
    assert_eq!(t.counters(oracle, counters::PLUS1), 4);
}

// ---------------------------------------------------------------------------------------
// Wakka, Devoted Guardian: "if a counter was put on ~ this turn"
// ---------------------------------------------------------------------------------------

#[test]
fn wakka_checks_whether_a_counter_was_put_on_it_this_turn() {
    cr!("603.4", "122.6");
    ruling!(
        "Wakka, Devoted Guardian",
        "Wakka's last ability checks at the moment it would trigger to see if a counter was put on Wakka this turn."
    );
    supported("Wakka, Devoted Guardian");
    // A counter was put on Wakka earlier this turn (and removed again): it triggers.
    let mut t = TestGame::new(2);
    let wakka = t.battlefield(P0, "Wakka, Devoted Guardian");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, wakka, counters::CHARGE, 1);
    let w = t.g.current(wakka);
    t.g.remove_counters(Entity::Object(w), counters::CHARGE, 1);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(wakka, counters::PLUS1), 0, "each other creature");
    // A counter on another creature doesn't count; no trigger.
    let mut t = TestGame::new(2);
    let wakka = t.battlefield(P0, "Wakka, Devoted Guardian");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::CHARGE, 1);
    to_end_step(&mut t, P0);
    assert_eq!(t.stack_len(), 0, "didn't trigger");
    // Once the end step has begun, it's too late.
    put(&mut t, wakka, counters::CHARGE, 1);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    // A counter put on it last turn doesn't count either.
    let mut t = TestGame::new(2);
    let wakka = t.battlefield(P0, "Wakka, Devoted Guardian");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P1, Step::PrecombatMain);
    put(&mut t, wakka, counters::CHARGE, 1);
    t.advance_to(P0, Step::PrecombatMain);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
}

#[test]
fn wakka_combat_damage_destroys_an_artifact_and_feeds_blitzball_captain() {
    // Wakka's first triggered ability puts the counter its last ability looks for.
    cr!("510.3a", "603.4");
    let mut t = TestGame::new(2);
    let wakka = t.battlefield(P0, "Wakka, Devoted Guardian");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ring = t.battlefield(P1, "Sol Ring");
    t.answer_targets(P0, &[Entity::Object(ring)]);
    attack_with(&mut t, &[(wakka, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(!t.on_battlefield(ring), "destroyed");
    assert_eq!(t.counters(wakka, counters::PLUS1), 1);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}
