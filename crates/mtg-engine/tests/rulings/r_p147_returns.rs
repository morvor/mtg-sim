//! Rulings batch P147 — "return a creature (permanent, land) you control to its owner's
//! hand": untargeted choices made as the ability resolves (CR 608.2c), the source itself
//! as a choice, returning as a cost (CR 118.3), targets "you own" controlled by another
//! player, illegal targets (CR 608.2b), and "as this enters" choices (CR 614.12).

use crate::r_p147_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether a target choice was asked of anyone since decision `from`.
fn asked_targets(t: &TestGame, from: usize) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
}

/// The candidates of the entity choices asked of `p` since `from`.
fn entity_candidates(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

// --- Untargeted choices made on resolution ------------------------------------------------

#[test]
fn untargeted_returns_are_chosen_as_the_ability_resolves() {
    cr!("608.2c", "115.1", "603.3d");
    ruling!(
        "Roaring Primadox",
        "The ability doesn’t target any creature. You choose which one to return when the ability resolves. No player can respond to this choice once the ability starts resolving."
    );
    ruling!(
        "Dust Elemental",
        "The ability of Dust Elemental doesn’t target any creature. Because of this, no player may take action between the time you choose the creature to return and the time you return it."
    );
    ruling!(
        "Emancipation Angel",
        "For Emancipation Angel’s triggered ability, the permanent to return is chosen as the ability resolves. No player may take actions between the time you choose the permanent and the time it’s returned to its owner’s hand."
    );
    ruling!(
        "Invasive Species",
        "The ability doesn’t target any permanent. You choose which one to return when the ability resolves. No player can respond to this choice once the ability starts resolving."
    );
    ruling!(
        "Loyal Gryff",
        "You choose whether to return a creature to your hand and which creature to return as Loyal Gryff's triggered ability resolves."
    );
    // (card, how it's set up: Some(step) for an upkeep trigger, None for an enters trigger)
    for name in [
        "Roaring Primadox",
        "Emancipation Angel",
        "Invasive Species",
        "Loyal Gryff",
        "Ambush Krotiq",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        // A hexproof creature can be chosen: it isn't targeted.
        let x = t.battlefield(P0, "Gladecover Scout");
        t.battlefield(P0, "Grizzly Bears");
        let from = n_asked(&t);
        if name == "Roaring Primadox" {
            t.battlefield(P0, name);
            into_upkeep(&mut t, P0);
        } else {
            enter(&mut t, P0, name);
        }
        assert_eq!(stacked_triggers(&t), 1, "{name}");
        assert!(!asked_targets(&t, from), "{name} targets");
        assert!(t.on_battlefield(x), "{name}");
        yes(&mut t, P0);
        t.answer_choose(P0, &[obj(x)]);
        t.resolve_all();
        assert!(t.in_hand(P0, "Gladecover Scout"), "{name}");
        assert!(!t.in_hand(P0, "Grizzly Bears"), "{name}");
    }
    // Dust Elemental: three of them.
    let mut t = TestGame::new(2);
    let x = t.battlefield(P0, "Gladecover Scout");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Savannah Lions");
    let from = n_asked(&t);
    enter(&mut t, P0, "Dust Elemental");
    assert!(!asked_targets(&t, from));
    t.answer_choose(P0, &[obj(x), obj(a), obj(b)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Gladecover Scout") && t.in_hand(P0, "Hill Giant"));
    assert!(!t.in_hand(P0, "Savannah Lions"));
    assert_eq!(t.named_on_battlefield("Dust Elemental").len(), 1);
}

#[test]
fn temur_sabertooths_choice_is_made_on_resolution() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Temur Sabertooth",
        "You choose whether to return a creature and which creature to return as the activated ability resolves. This doesn't target any creature."
    );
    supported("Temur Sabertooth");
    use mtg_engine::keywords::KeywordKind;
    for ret in [true, false] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Temur Sabertooth");
        let x = t.battlefield(P0, "Gladecover Scout");
        t.lands(P0, "Forest", 2);
        let from = n_asked(&t);
        t.activate(P0, s, 0, &[]).expect("activated");
        assert!(!asked_targets(&t, from));
        t.answer_yes(P0, ret);
        t.answer_choose(P0, &[obj(x)]);
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Gladecover Scout"), ret);
        assert_eq!(t.obj_now(s).has_keyword(KeywordKind::Indestructible), ret);
    }
}

#[test]
fn the_source_itself_may_be_returned_or_must_be() {
    cr!("608.2c", "603.3d");
    ruling!(
        "Dust Elemental",
        "You may return Dust Elemental creature itself to its owner’s hand as its triggered ability resolves. If you don’t control three other creatures, you must return it."
    );
    ruling!(
        "Emancipation Angel",
        "If Emancipation Angel is still on the battlefield as its triggered ability resolves, you may return Emancipation Angel itself."
    );
    ruling!(
        "Roaring Primadox",
        "Roaring Primadox’s ability is mandatory. When it resolves, if Roaring Primadox is the only creature you control, you must return it to its owner’s hand."
    );
    // Dust Elemental chosen with two others, of three others.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Savannah Lions");
    let d = enter(&mut t, P0, "Dust Elemental");
    t.answer_choose(P0, &[obj(d), obj(a), obj(b)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Dust Elemental"));
    assert!(!t.on_battlefield(now(&t, d)));
    assert!(!t.in_hand(P0, "Savannah Lions"));
    // Only one other creature: both are returned.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    enter(&mut t, P0, "Dust Elemental");
    t.resolve_all();
    assert!(t.in_hand(P0, "Dust Elemental") && t.in_hand(P0, "Grizzly Bears"));
    // Emancipation Angel returns itself.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let e = enter(&mut t, P0, "Emancipation Angel");
    t.answer_choose(P0, &[obj(e)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Emancipation Angel"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    // Roaring Primadox alone returns itself, without being asked whether to.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Roaring Primadox");
    into_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_hand(P0, "Roaring Primadox"));
}

#[test]
fn return_another_creature_with_none_does_nothing() {
    cr!("608.2c", "609.3");
    ruling!(
        "Ambush Krotiq",
        "If you control no other creatures as Ambush Krotiq’s triggered ability resolves, nothing happens. Ambush Krotiq remains on the battlefield."
    );
    ruling!(
        "Havengul Skaab",
        "If you don’t control any other creatures, the ability won’t do anything."
    );
    supported("Ambush Krotiq");
    supported("Havengul Skaab");
    let mut t = TestGame::new(2);
    let k = enter(&mut t, P0, "Ambush Krotiq");
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert!(t.on_battlefield(k));
    // Havengul Skaab attacks alone: it stays attacking.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Havengul Skaab");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[s]));
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert!(attacking(&t, s));
    // Positive control: with another creature, it's returned.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Havengul Skaab");
    t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[s]));
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn faerie_impostor_may_sacrifice_itself_instead() {
    cr!("608.2c", "118.12");
    ruling!(
        "Faerie Impostor",
        "Returning another creature you control to its owner’s hand is optional. You may choose to sacrifice Faerie Impostor, even if you could return a creature."
    );
    for ret in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        let f = enter(&mut t, P0, "Faerie Impostor");
        t.answer_yes(P0, ret);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(ret));
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), ret, "returned: {ret}");
        assert_eq!(t.on_battlefield(f), ret, "returned: {ret}");
        assert_eq!(t.in_graveyard(P0, "Faerie Impostor"), !ret);
    }
}

// --- Returning as a cost --------------------------------------------------------------------

#[test]
fn wirewood_symbiote_returns_the_elf_as_a_cost_and_may_untap_any_creature() {
    cr!("601.2h", "602.2b", "115.1");
    ruling!(
        "Wirewood Symbiote",
        "Returning the Elf you control to its owner's hand is the cost to activate the ability. Once you activate the ability, no one can try to do anything to the Elf to stop you from activating the ability."
    );
    ruling!(
        "Wirewood Symbiote",
        "You can target any creature with Wirewood Symbiote's ability, not just a tapped creature."
    );
    supported("Wirewood Symbiote");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wirewood Symbiote");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    let from = n_asked(&t);
    t.answer_choose(P0, &[obj(elf)]);
    t.activate(P0, w, 0, &[obj(giant)]).expect("activated");
    // The Elf was returned as the ability was activated, before anyone got priority.
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert_eq!(t.stack_len(), 1);
    assert!(offered_target(&t, P0, from, giant));
    t.resolve_all();
    assert!(!t.obj_now(giant).tapped);
}

#[test]
fn familiars_ruse_needs_another_spell_to_target() {
    cr!("601.2c", "115.5");
    ruling!(
        "Familiar's Ruse",
        "Familiar’s Ruse can’t be cast without a target spell. It can’t target itself."
    );
    supported("Familiar's Ruse");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let ruse = t.hand(P0, "Familiar's Ruse");
    assert!(!can_cast(&mut t, P0, ruse, CastMethod::Normal));
    // With an opponent's spell on the stack: castable, targeting only that spell.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.g.turn.priority = Some(P1);
    let s = t.cast(P1, shock).target(P1).go();
    assert!(can_cast(&mut t, P0, ruse, CastMethod::Normal));
    let from = n_asked(&t);
    t.answer_choose(P0, &[obj(bears)]);
    t.g.turn.priority = Some(P0);
    let r = t.cast(P0, ruse).target(s).go();
    let cands = target_candidates(&t, P0, from);
    assert!(cands.iter().all(|c| !c.contains(&obj(r))));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.life(P1), 20);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

// --- Targets you own, controlled by another player -----------------------------------------

#[test]
fn return_target_you_own_even_if_another_player_controls_it() {
    cr!("108.3", "115.1");
    ruling!(
        "Get Out",
        "Get Out's last mode can target any creatures and/or enchantments you own, including ones controlled by other players."
    );
    ruling!(
        "Saving Grasp",
        "Saving Grasp can target any creature you own, even one controlled by another player."
    );
    supported("Get Out");
    supported("Saving Grasp");
    for name in ["Get Out", "Saving Grasp"] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Hill Giant");
        take_control(&mut t, P1, mine);
        assert_eq!(t.obj_now(mine).controller, P1);
        lands_for_cost(&mut t, P0, name);
        let card = t.hand(P0, name);
        let from = n_asked(&t);
        let mut c = t.cast(P0, card);
        if name == "Get Out" {
            c = c.modes(&[1]);
        }
        c.target(mine).go();
        assert!(!offered_target(&t, P0, from, theirs), "{name}");
        t.resolve_all();
        assert!(t.in_hand(P0, "Grizzly Bears"), "{name}");
    }
}

#[test]
fn portal_of_sanctuary_leaves_equipment_behind() {
    cr!("301.5c", "704.5n");
    ruling!(
        "Portal of Sanctuary",
        "Equipment attached to the target creature remains on the battlefield."
    );
    supported("Portal of Sanctuary");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Portal of Sanctuary");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let eq = crate::r_s06_common::attach_new(&mut t, P0, "Bonesplitter", bears);
    crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", bears);
    t.lands(P0, "Wastes", 1);
    activate_resolve(&mut t, P0, p, 0, &[obj(bears)]);
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Holy Strength"));
    assert!(t.on_battlefield(eq));
    assert_eq!(t.obj_now(eq).attached_to, None);
}

// --- Dour Port-Mage ------------------------------------------------------------------------

#[test]
fn dour_port_mage_counts_any_zone_but_the_graveyard() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Dour Port-Mage",
        "Dour Port-Mage’s first ability triggers whenever one or more creatures you control go to any zone other than the graveyard from the battlefield."
    );
    supported("Dour Port-Mage");
    for (to, draws) in [
        (Zone::Hand(P0), true),
        (Zone::Exile, true),
        (Zone::Library(P0), true),
        (Zone::Graveyard(P0), false),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dour Port-Mage");
        let b = t.battlefield(P0, "Grizzly Bears");
        move_to(&mut t, b, to);
        assert_eq!(stacked_triggers(&t), usize::from(draws), "{to:?}");
    }
    // Its own ability.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Dour Port-Mage");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let hand = t.hand_size(P0);
    activate_resolve(&mut t, P0, m, 0, &[obj(b)]);
    assert_eq!(t.hand_size(P0), hand + 2);
}

// --- Leonardo, Cutting Edge ----------------------------------------------------------------

#[test]
fn leonardo_lifelink_gain_comes_too_late_for_lethal_damage() {
    cr!("510.2", "704.5g", "702.15b");
    ruling!(
        "Leonardo, Cutting Edge",
        "If Leonardo is dealt lethal damage at the same time that you gain life, he won't receive a counter from his last ability in time to save him."
    );
    supported("Leonardo, Cutting Edge");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Leonardo, Cutting Edge");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[l]));
    block_and_finish(&mut t, P1, &[(bears, l)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert!(t.in_graveyard(P0, "Leonardo, Cutting Edge"));
}

#[test]
fn leonardo_triggers_once_per_life_gain_event() {
    cr!("119.9", "603.2c");
    ruling!(
        "Leonardo, Cutting Edge",
        "that life is gained as one event and Leonardo's last ability triggers only once."
    );
    ruling!(
        "Leonardo, Cutting Edge",
        "Leonardo's last ability triggers just once for each life-gaining event"
    );
    supported("Congregate");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Leonardo, Cutting Edge");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    // 2 life for each of three creatures: one event.
    cast_new(&mut t, P0, "Congregate", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.counters(l, "+1/+1"), 1);
    // A 1-life gain and a 3-life gain: two events.
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    t.settle();
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(l, "+1/+1"), 3);
}

#[test]
fn leonardo_ignores_a_two_headed_giant_teammates_life_gain() {
    cr!("810.9", "603.2");
    ruling!(
        "Leonardo, Cutting Edge",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Leonardo's last ability to trigger"
    );
    let mut t = two_headed_giant();
    let l = t.battlefield(P0, "Leonardo, Cutting Edge");
    t.g.gain_life(P1, 2);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(l, "+1/+1"), 0);
    t.g.gain_life(P0, 2);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(l, "+1/+1"), 1);
}

// --- Chulane ---------------------------------------------------------------------------------

#[test]
fn chulanes_land_isnt_a_land_play() {
    cr!("305.4", "305.2");
    ruling!(
        "Chulane, Teller of Tales",
        "Chulane's triggered ability doesn't count as playing a land. It can put a land card onto the battlefield even if you've already played your land for the turn, and even if it's not your turn."
    );
    supported("Chulane, Teller of Tales");
    // P0 already played a land this turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chulane, Teller of Tales");
    let land = t.hand(P0, "Forest");
    t.play_land(P0, land).expect("land play");
    let f2 = t.hand(P0, "Forest");
    stack_library(&mut t, P0, &["Island"]);
    let played = t.g.player(P0).lands_played_this_turn;
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(f2)]);
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(f2));
    assert_eq!(t.g.player(P0).lands_played_this_turn, played);
    // During P1's turn (a flash creature).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chulane, Teller of Tales");
    let f = t.hand(P0, "Forest");
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(f)]);
    cast_new(&mut t, P0, "Loyal Gryff", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(f));
}

#[test]
fn cast_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i");
    ruling!(
        "Chulane, Teller of Tales",
        "Chulane's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chulane, Teller of Tales");
    let hand = t.hand_size(P0);
    let spell = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    counter(&mut t, spell);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

// --- Raphael's Technique, Arachne, Leonardo, Sewer Samurai ---------------------------------

#[test]
fn raphaels_technique_may_discard_an_empty_hand() {
    cr!("701.9a", "608.2c");
    ruling!(
        "Raphael's Technique",
        "A player may choose to discard their hand even if their hand contains zero cards."
    );
    supported("Raphael's Technique");
    let mut t = TestGame::new(2);
    let spell = cast_new(&mut t, P0, "Raphael's Technique", &[]);
    assert_eq!(t.hand_size(P0), 0);
    t.answer_yes(P0, true);
    t.answer_yes(P1, false);
    fill_hand(&mut t, P1, 2);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn arachne_chooses_a_card_type_other_than_creature_never_a_subtype() {
    cr!("205.2a", "601.2f");
    ruling!(
        "Arachne, Psionic Weaver",
        "Card types other than creature include artifact, battle, enchantment, instant, kindred, planeswalker, and sorcery. Technically, you could also choose land, but lands are never cast"
    );
    ruling!(
        "Arachne, Psionic Weaver",
        "You can't choose a subtype, such as creature types like Spider or Hero, artifact types like Treasure or Food, or enchantment types like Aura."
    );
    supported("Arachne, Psionic Weaver");
    // The options offered.
    let mut t = TestGame::new(2);
    let from = n_asked(&t);
    enter(&mut t, P0, "Arachne, Psionic Weaver");
    let options: Vec<String> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .next()
        .expect("a card type choice");
    let lower: Vec<String> = options.iter().map(|o| o.to_lowercase()).collect();
    for ty in [
        "artifact",
        "battle",
        "enchantment",
        "instant",
        "kindred",
        "planeswalker",
        "sorcery",
    ] {
        assert!(lower.iter().any(|o| o == ty), "{ty} missing from {options:?}");
    }
    for no in ["creature", "spider", "hero", "treasure", "food", "aura"] {
        assert!(!lower.iter().any(|o| o == no), "{no} offered: {options:?}");
    }
    // Choosing instant taxes Shock; choosing land has no effect on land plays.
    let pick = |ty: &str| {
        let mut t = TestGame::new(2);
        let opts = options.clone();
        let i = opts
            .iter()
            .position(|o| o.eq_ignore_ascii_case(ty))
            .expect("option");
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
        enter(&mut t, P0, "Arachne, Psionic Weaver");
        t.resolve_all();
        t
    };
    let mut t = pick("instant");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    assert!(!can_cast(&mut t, P0, shock, CastMethod::Normal));
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, shock, CastMethod::Normal));
    if lower.iter().any(|o| o == "land") {
        let mut t = pick("land");
        let f = t.hand(P0, "Forest");
        t.play_land(P0, f).expect("land play");
        assert!(t.on_battlefield(f) || !t.named_on_battlefield("Forest").is_empty());
    }
}

#[test]
fn leonardo_sewer_samurai_graveyard_casts_follow_costs_and_timing() {
    cr!("601.2", "307.1", "601.3");
    ruling!(
        "Leonardo, Sewer Samurai",
        "You pay all costs and follow all timing rules for cards played with the permission granted by Leonardo's last ability."
    );
    supported("Leonardo, Sewer Samurai");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leonardo, Sewer Samurai");
    let lions = t.graveyard(P0, "Savannah Lions"); // 2/1
    let elves = t.graveyard(P0, "Llanowar Elves"); // 1/1
    // No mana: not castable.
    assert!(!can_cast(&mut t, P0, elves, CastMethod::Normal));
    t.lands(P0, "Forest", 1);
    assert!(can_cast(&mut t, P0, elves, CastMethod::Normal));
    // Power 2, toughness 1: castable too.
    t.lands(P0, "Plains", 1);
    assert!(can_cast(&mut t, P0, lions, CastMethod::Normal));
    // Sorcery timing: not with a spell on the stack.
    let shock = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, shock).target(P1).go();
    assert!(!can_cast(&mut t, P0, elves, CastMethod::Normal));
    t.resolve_all();
    // Nor in combat.
    to_combat(&mut t, P0);
    assert!(!can_cast(&mut t, P0, elves, CastMethod::Normal));
}

// --- Lands ----------------------------------------------------------------------------------

#[test]
fn sea_drake_needs_two_land_targets_and_returns_what_it_can() {
    cr!("603.3d", "608.2b");
    ruling!(
        "Sea Drake",
        "If one of the lands becomes an illegal target in response, the remaining legal target will be returned to its owner's hand. If both targets have become illegal in response, nothing happens."
    );
    ruling!(
        "Sea Drake",
        "If you can't target two lands you control with the triggered ability (perhaps because you control fewer than two lands), the ability will be removed from the stack."
    );
    supported("Sea Drake");
    // One target becomes illegal.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Forest");
    let b = t.battlefield(P0, "Island");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    let d = enter(&mut t, P0, "Sea Drake");
    move_to(&mut t, a, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.in_hand(P0, "Island"));
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.on_battlefield(d));
    // Both become illegal.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Forest");
    let b = t.battlefield(P0, "Island");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    let d = enter(&mut t, P0, "Sea Drake");
    move_to(&mut t, a, Zone::Graveyard(P0));
    move_to(&mut t, b, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(d));
    assert_eq!(t.hand_size(P0), 0);
    // Only one land: the trigger is removed, nothing returned, the Drake stays.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    let d = enter(&mut t, P0, "Sea Drake");
    t.resolve_all();
    assert!(t.on_battlefield(d));
    assert!(!t.in_hand(P0, "Forest"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn tragic_lesson_choice_comes_after_the_draws() {
    cr!("608.2c", "118.12");
    ruling!(
        "Tragic Lesson",
        "You don’t choose which land to return to its owner’s hand or whether to discard a card instead until you see the two cards you draw."
    );
    supported("Tragic Lesson");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| {
            matches!(
                d,
                Decision::YesNo { .. }
                    | Decision::OptionalCost { .. }
                    | Decision::ChooseEntities { .. }
            )
        },
        |g| {
            g.player(P0)
                .hand
                .iter()
                .filter(|id| g.obj(**id).chars.name == "Hill Giant")
                .count()
        },
    );
    yes(&mut t, P0);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let spell = cast_new(&mut t, P0, "Tragic Lesson", &[]);
    t.resolve_all();
    assert!(resolved(&t, spell));
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty(), "no choice asked");
    assert!(seen.iter().all(|n| *n == 1), "{seen:?}");
    assert!(t.in_hand(P0, "Forest"));
}

// --- Rescuer Sphinx, Paradoxical Outcome ---------------------------------------------------

#[test]
fn rescuer_sphinx_returns_only_what_was_already_there_and_its_choice_is_immediate() {
    cr!("614.12", "614.1c", "603.6a");
    ruling!(
        "Rescuer Sphinx",
        "If Rescuer Sphinx somehow enters the battlefield at the same time as another permanent, you can’t return that permanent."
    );
    ruling!(
        "Rescuer Sphinx",
        "If the returned permanent had an ability that would otherwise have triggered when Rescuer Sphinx enters the battlefield (such as that of Kronch Wrangler), that ability won’t trigger."
    );
    ruling!(
        "Rescuer Sphinx",
        "You choose whether to return a nonland permanent and which one to return for Rescuer Sphinx’s ability while it’s entering the battlefield."
    );
    supported("Rescuer Sphinx");
    supported("Soul Warden");
    // Entering together with Grizzly Bears: the Bears aren't offered.
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Hill Giant");
    let from = n_asked(&t);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(old)]);
    let ids = enter_together(&mut t, P0, &["Rescuer Sphinx", "Grizzly Bears"]);
    let cands = entity_candidates(&t, P0, from);
    assert!(!cands.is_empty());
    assert!(cands.iter().all(|c| !c.contains(&obj(ids[1]))), "{cands:?}");
    assert!(t.in_hand(P0, "Hill Giant"));
    // Returning Soul Warden: it doesn't see the Sphinx enter; no stack in between.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Soul Warden");
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(w)]);
    let s = enter(&mut t, P0, "Rescuer Sphinx");
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.in_hand(P0, "Soul Warden"));
    assert_eq!(t.counters(s, "+1/+1"), 1);
    assert_eq!(t.life(P0), 20);
    // Positive control: kept, Soul Warden triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    t.answer_yes(P0, false);
    let s = enter(&mut t, P0, "Rescuer Sphinx");
    t.resolve_all();
    assert_eq!(t.counters(s, "+1/+1"), 0);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn paradoxical_outcome_draws_for_each_card_of_a_merged_permanent() {
    cr!("730.3c", "608.2c", "702.140e");
    ruling!(
        "Paradoxical Outcome",
        "If a melded or merged permanent is returned to your hand this way, you'll draw a card for each of its components that are cards."
    );
    supported("Paradoxical Outcome");
    use mtg_engine::keywords::KeywordKind;
    use mtg_engine::mana::ManaType;
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let parrot = t.hand(P0, "Porcuparrot");
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, parrot)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(bears)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Porcuparrot");
    let hand = t.hand_size(P0);
    let spell = cast_multi(&mut t, P0, "Paradoxical Outcome", &[obj(bears)]);
    t.resolve_all();
    assert!(resolved(&t, spell));
    // Both cards were returned, and two cards drawn.
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Porcuparrot"));
    assert_eq!(t.hand_size(P0), hand + 2 + 2);
}

#[test]
fn prehistoric_pet_stays_blocked_when_powers_change() {
    cr!("509.1b", "506.4", "509.1h");
    ruling!(
        "Prehistoric Pet",
        "Once Prehistoric Pet has been blocked, changing its power or the power of a blocking creature won't cause it to become unblocked."
    );
    supported("Prehistoric Pet");
    let mut t = TestGame::new(2);
    let pet = t.battlefield(P0, "Prehistoric Pet");
    let wall = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[pet]));
    // Greater power: can't block.
    assert!(!legal_blocks(&mut t, P1, &[(bears, pet)]));
    declare_blocks(&mut t, P1, &[(wall, pet)]);
    pump(&mut t, wall, 3, 0);
    assert!(blocking(&t, wall));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Prehistoric Pet"));
}
