//! Rulings batch P027 — the rest: costs of copy-making abilities (CR 118, 601.2f, 602.2),
//! what triggers them (CR 603.2, 603.10), last known information (CR 608.2h), choices on
//! resolution in turn order (CR 101.4, 608.2e), modes (CR 700.2), creature-type choices
//! (CR 205.3m), loyalty-ability timing (CR 606.3) and casting cards from a graveyard with
//! Finale of Promise (CR 608.2g, 707.10).

use crate::r_p027_common::*;
use crate::r_s01_common::{supported, tokens, watch};
use crate::r_s06_common::activate_containing;
use crate::r_s17_common::enter_transformed;
use crate::r_s18_common::lands_for;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::*;
use crate::r_s26_common::modify_until_eot;
use crate::r_s35_common::creature_type_options;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Inalla, Archmage Ritualist -------------------------------------------------------------

#[test]
fn inalla_taps_any_untapped_wizards_even_summoning_sick_ones() {
    cr!("302.6", "602.2b", "118.3");
    ruling!("Inalla, Archmage Ritualist", "To activate Inalla's last ability, you may tap any untapped Wizards you control, including ones you haven't controlled continuously since the beginning of your most recent turn, such as Inalla itself. (Note that tapping the creature doesn't use {T} [the tap symbol].)");
    // "Tap five untapped Wizards you control: Target player loses 7 life."
    supported("Inalla, Archmage Ritualist");
    for wizards in [4, 3] {
        let mut t = TestGame::new(2);
        let inalla = t.battlefield_sick(P0, "Inalla, Archmage Ritualist");
        let mut ws = vec![obj(inalla)];
        for _ in 0..wizards {
            ws.push(obj(t.battlefield_sick(P0, "Prodigal Sorcerer")));
        }
        t.answer_choose(P0, &ws);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let r = activate_containing(&mut t, P0, inalla, "Tap five untapped Wizards");
        t.resolve_all();
        if wizards == 4 {
            r.expect("five Wizards, all summoning sick");
            assert_eq!(t.life(P1), 13);
            assert!(t.obj_now(inalla).tapped);
        } else {
            assert!(r.is_err(), "only four Wizards");
            assert_eq!(t.life(P1), 20);
        }
    }
}

/// The "Pay ...?" prompts P0 was asked since `from`.
fn pay_prompts(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Pay")))
        .count()
}

#[test]
fn inalla_and_bramble_sovereign_pay_only_once_for_one_token() {
    cr!("603.5", "608.2c", "118.1");
    ruling!("Inalla, Archmage Ritualist", "You can pay {1} only once each time Inalla's first ability resolves. You can't pay more to get more than one token.");
    ruling!("Bramble Sovereign", "While resolving the triggered ability of Bramble Sovereign, you can't pay {1}{G} multiple times to have a player create more tokens.");
    for (source, cost_land, entering, paid) in [
        ("Inalla, Archmage Ritualist", "Wastes", "Prodigal Sorcerer", 1),
        ("Bramble Sovereign", "Forest", "Grizzly Bears", 2),
    ] {
        supported(source);
        let mut t = TestGame::new(2);
        t.battlefield(P0, source);
        t.lands(P0, cost_land, 8);
        let from = t.asked().len();
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.enter(P0, entering);
        t.resolve_all();
        assert_eq!(pay_prompts(&t, from), 1, "{source}");
        assert_eq!(t.named_on_battlefield(entering).len(), 2, "{source}");
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), paid, "{source}");
    }
}

// --- The Fire Crystal ------------------------------------------------------------------------

#[test]
fn the_fire_crystal_reduces_the_total_cost_not_the_mana_value() {
    cr!("601.2f", "118.7", "202.3b", "702.34a", "702.33d");
    ruling!("The Fire Crystal", "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of The Fire Crystal's first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was.");
    supported("The Fire Crystal");
    // Firebolt's flashback cost {4}{R} costs {3}{R}: four lands, not three.
    for lands in [3, 4] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "The Fire Crystal");
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", lands - 1);
        let fb = t.graveyard(P0, "Firebolt");
        let r = t
            .cast(P0, fb)
            .method(CastMethod::Keyword(KeywordKind::Flashback))
            .target(Entity::Player(P1))
            .try_go();
        assert_eq!(r.is_ok(), lands == 4, "{lands} lands");
        if let Ok(s) = r {
            assert_eq!(t.g.mana_value_of(s), 1);
        }
    }
    // Kicked Burst Lightning: {R} + {4} - {1}: four lands. Its mana value is still 1.
    for lands in [3, 4] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "The Fire Crystal");
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", lands - 1);
        let bl = t.hand(P0, "Burst Lightning");
        let r = t
            .cast(P0, bl)
            .kicked(true)
            .target(Entity::Player(P1))
            .try_go();
        match r {
            Ok(s) if lands == 4 => {
                assert_eq!(t.g.mana_value_of(s), 1);
                t.resolve_all();
                assert_eq!(t.life(P1), 16);
            }
            Ok(_) => {
                // Too little mana for the kicker: it wasn't kicked.
                t.resolve_all();
                assert_eq!(t.life(P1), 18);
            }
            Err(_) => assert_eq!(lands, 3),
        }
    }
}

// --- Lazav, Dimir Mastermind -----------------------------------------------------------------

#[test]
fn lazav_doesnt_trigger_for_dying_tokens() {
    cr!("603.2", "111.7", "704.5d");
    ruling!("Lazav, Dimir Mastermind", "Token creatures dying won't cause Lazav's triggered ability to trigger.");
    // "Whenever a creature card is put into an opponent's graveyard from anywhere, you may
    // have Lazav become a copy of that card, except ..."
    supported("Lazav, Dimir Mastermind");
    let mut t = TestGame::new(2);
    let lazav = t.battlefield(P0, "Lazav, Dimir Mastermind");
    let tok = crate::r_p023_common::wolf_token(&mut t, P1);
    kill(&mut t, tok);
    assert_eq!(t.stack_len(), 0, "a dying token triggered Lazav");
    // A creature card does.
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    kill(&mut t, giant);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(lazav), (3, 3));
    assert!(has_subtype(&t, lazav, "Giant"));
}

// --- Trostani, Selesnya's Voice --------------------------------------------------------------

#[test]
fn trostani_uses_the_toughness_as_it_resolves() {
    cr!("608.2h", "603.10", "119.3");
    ruling!("Trostani, Selesnya's Voice", "Trostani's first ability checks the creature's toughness as it resolves. If that creature has left the battlefield, use its toughness from when it was last on the battlefield. You can't lose life this way if that creature's toughness was less than 0.");
    // "Whenever another creature you control enters, you gain life equal to that
    // creature's toughness."
    supported("Trostani, Selesnya's Voice");
    // Pumped in response: the new toughness.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trostani, Selesnya's Voice");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    modify_until_eot(&mut t, bears, vec![Modification::ModifyPT(Value::c(0), Value::c(3))]);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    // Left the battlefield (after a +0/+1 pump): its last toughness, 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trostani, Selesnya's Voice");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    modify_until_eot(&mut t, bears, vec![Modification::ModifyPT(Value::c(0), Value::c(1))]);
    let b = t.g.current(bears);
    t.g.move_object(b, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Its toughness was -3 when it died: no life lost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trostani, Selesnya's Voice");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    modify_until_eot(&mut t, bears, vec![Modification::ModifyPT(Value::c(-5), Value::c(-5))]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

// --- Zndrsplt's Judgment ---------------------------------------------------------------------

/// P0 casts Zndrsplt's Judgment in a four-player game, calling each player friend (0)
/// or foe (1) as given; each player chooses their only creature.
fn judgment(calls: [usize; 4], creatures: bool) -> (TestGame, Vec<ObjectId>) {
    supported("Zndrsplt's Judgment");
    let mut t = TestGame::new(4);
    let mut cs = vec![];
    if creatures {
        for p in 0..4 {
            let c = t.battlefield(PlayerId(p), "Grizzly Bears");
            t.answer_choose(PlayerId(p), &[obj(c)]);
            cs.push(c);
        }
    }
    for c in calls {
        t.answer(P0, DecisionKind::Option, Answer::Index(c));
    }
    lands_for(&mut t, P0, "{4}{U}");
    let j = t.hand(P0, "Zndrsplt's Judgment");
    t.cast(P0, j).go();
    (t, cs)
}

/// Whether the decision is a choice among objects (a friend's or foe's choice).
fn is_choice(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { .. })
}

#[test]
fn zndrsplts_judgment_friends_then_foes_choose_in_turn_order() {
    cr!("101.4", "608.2e", "101.4a");
    ruling!("Zndrsplt's Judgment", "When Zndrsplt's Judgment resolves and you choose friend or foe, first the next friend in turn order (or, if it's a friend's turn, that friend) chooses a creature they control, then each other friend in turn order does the same, then each of those players create a token at the same time. Friends will know choices made by earlier players when making their choices. Repeat this process for foes, who choose a creature one at a time and then return them all to their owners' hands at the same time.");
    // P0 (whose turn it is) and P3 are foes; P1 and P2 are friends.
    let (mut t, cs) = judgment([1, 0, 0, 1], true);
    // When each player chooses: the number of Grizzly Bears on the battlefield.
    let count = |g: &mtg_engine::game::Game| {
        g.battlefield
            .iter()
            .filter(|id| g.obj(**id).chars.name == "Grizzly Bears")
            .count()
    };
    let seen: Vec<_> = (0..4)
        .map(|p| watch(&mut t, PlayerId(p), is_choice, count))
        .collect();
    let from = t.asked().len();
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| is_choice(d))
        .map(|(p, _)| *p)
        .collect();
    // Friends first (the next friend in turn order: P1), then the foes (P0's turn, so P0
    // first).
    assert_eq!(order, vec![P1, P2, P0, P3]);
    // No token had been created when P2 chose; both had (and no creature had left yet)
    // when the foes chose.
    let n = |p: usize| seen[p].lock().unwrap().clone();
    assert_eq!(n(1), vec![4]);
    assert_eq!(n(2), vec![4]);
    assert_eq!(n(0), vec![6]);
    assert_eq!(n(3), vec![6]);
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P3, "Grizzly Bears"));
    assert_eq!(tokens(&t, P1).len(), 1);
    assert_eq!(tokens(&t, P2).len(), 1);
    assert!(t.on_battlefield(cs[1]) && t.on_battlefield(cs[2]));
}

#[test]
fn zndrsplts_judgment_can_call_players_who_cant_comply() {
    cr!("101.3", "608.2e");
    ruling!("Zndrsplt's Judgment", "You may call a player a friend or a foe even if that player will be instructed to perform an impossible action. For example, for Regna's Sanction, a foe may control no untapped creatures.");
    // No one controls a creature: friends and foes alike do nothing.
    let (mut t, _) = judgment([0, 1, 0, 1], false);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Zndrsplt's Judgment"));
    assert!(tokens(&t, P0).is_empty() && tokens(&t, P2).is_empty());
}

// --- Grist, the Plague Swarm -----------------------------------------------------------------

#[test]
fn grist_can_activate_a_loyalty_ability_the_turn_it_enters_at_sorcery_speed() {
    cr!("606.3", "307.1");
    ruling!("Grist, Voracious Larva // Grist, the Plague Swarm", "You can activate one of Grist, the Plague Swarm's loyalty abilities the turn it enters the battlefield. However, you may do so only during one of your main phases when the stack is empty. For example, if Grist, the Plague Swarm enters the battlefield during combat, there will be an opportunity for your opponent to remove it before you can activate one of its abilities.");
    const GRIST: &str = "Grist, Voracious Larva // Grist, the Plague Swarm";
    supported(GRIST);
    // During combat: not yet.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let g = enter_transformed(&mut t, P0, GRIST);
    assert_eq!(name_of(&t, g), "Grist, the Plague Swarm");
    assert!(activate_containing(&mut t, P0, g, "Create a 1/1").is_err());
    // In the main phase with something on the stack: not yet.
    t.advance_to(P0, Step::PostcombatMain);
    let g = t.g.current(g);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert!(activate_containing(&mut t, P0, g, "Create a 1/1").is_err());
    t.resolve_all();
    // Empty stack in a main phase: yes, though it entered this turn.
    activate_containing(&mut t, P0, g, "Create a 1/1").expect("+1");
    t.resolve_all();
    assert_eq!(t.counters(g, counters::LOYALTY), 4);
}

// --- Choosing a creature type ----------------------------------------------------------------

#[test]
fn a_chosen_creature_type_is_one_existing_creature_type() {
    cr!("205.3m", "205.2a", "205.4a");
    ruling!("Kindred Charge", "You must choose an existing creature type, such as Bird or Wizard. Card types such as artifact can’t be chosen.");
    ruling!("Adaptive Automaton", "You must choose an existing creature type, such as Human or Warrior. Card types such as artifact and supertypes such as legendary can't be chosen.");
    ruling!("Mirror of the Forebears", "You must choose an existing creature type, such as Vampire or Cat. Card types such as “artifact” can’t be chosen.");
    ruling!("Mirror of the Forebears", "You can’t choose multiple creature types, such as “Cat Warrior.” A Cat Warrior is both a Cat and a Warrior. It’s affected by anything that affects either type and unaffected by things that affect non-Cat or non-Warrior creatures.");
    for name in [
        "Adaptive Automaton",
        "Instruments of War",
        "Kindred Discovery",
        "Molten Echoes",
        "Mirror of the Forebears",
        "Kindred Charge",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let from = t.asked().len();
        let card = t.hand(P0, name);
        give_mana_for(&mut t, name);
        choose_creature_type(&mut t, P0, "Wizard");
        t.cast(P0, card).go();
        t.resolve_all();
        let opts = creature_type_options(&t, P0, from);
        assert_eq!(opts.len(), 1, "{name}: one type chosen");
        let o = &opts[0];
        for ok in ["Bird", "Wizard", "Human", "Warrior", "Vampire", "Cat"] {
            assert!(o.iter().any(|x| x == ok), "{name}: {ok}");
        }
        for bad in ["Artifact", "Legendary", "Cat Warrior", "Creature", "Equipment"] {
            assert!(!o.iter().any(|x| x == bad), "{name}: {bad}");
        }
    }
    // Mirror of the Forebears choosing Cat can copy a Cat Wizard (Qasali Pridemage), and
    // Instruments of War choosing Wizard pumps it.
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Qasali Pridemage");
    choose_creature_type(&mut t, P0, "Cat");
    let mirror = t.enter(P0, "Mirror of the Forebears");
    choose_creature_type(&mut t, P0, "Wizard");
    t.enter(P0, "Instruments of War");
    t.settle();
    assert_eq!(t.pt(cat), (3, 3));
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(cat)]);
    activate_containing(&mut t, P0, mirror, "becomes a copy").expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, mirror), "Qasali Pridemage");
    // It's a Cat Wizard too: Instruments of War pumps it.
    assert_eq!(t.pt(mirror), (3, 3));
}

/// Lands for the mana cost of the front face of `name` (generic as Wastes).
fn give_mana_for(t: &mut TestGame, name: &str) {
    lands_for_cost(t, P0, name);
}

// --- Kaya, Spirits' Justice ------------------------------------------------------------------

#[test]
fn kaya_chooses_all_the_targets_of_her_last_ability() {
    cr!("115.1", "601.2c", "602.2b");
    ruling!("Kaya, Spirits' Justice", "You choose all targets for Kaya, Spirits' Justice's last ability.");
    supported("Kaya, Spirits' Justice");
    let mut t = TestGame::new(3);
    let kaya = t.battlefield(P0, "Kaya, Spirits' Justice");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P2, "Llanowar Elves");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(a)]);
    t.answer_targets(P0, &[obj(b)]);
    activate_containing(&mut t, P0, kaya, "Exile target creature you control").expect("-2");
    t.resolve_all();
    let target_askers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert!(!target_askers.is_empty());
    assert!(target_askers.iter().all(|p| *p == P0), "{target_askers:?}");
    for c in [mine, a, b] {
        assert_eq!(t.zone(c), Zone::Exile);
    }
}

// --- Seasons ---------------------------------------------------------------------------------

#[test]
fn seasons_needn_t_choose_five_pawprints() {
    cr!("700.2i", "700.2");
    ruling!("Season of Weaving", "You don't have to choose modes that add up to exactly five pawprints. For example, you could choose the two pawprint mode twice and leave it at that. You could even choose no modes at all. You probably shouldn't, but you could.");
    supported("Season of Weaving");
    // {P}{P} twice: two token copies of P0's Hill Giant.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Season of Weaving");
    let s = t.hand(P0, "Season of Weaving");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_choose(P0, &[obj(giant)]);
    let id = t.cast(P0, s).modes(&[1, 1]).go();
    assert_eq!(crate::r_s26_common::modes_on_stack(&t, id), vec![1, 1]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 3);
    // No modes at all: it's cast and resolves, doing nothing.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Season of Weaving");
    let s = t.hand(P0, "Season of Weaving");
    let hand = t.hand_size(P0);
    let id = t.cast(P0, s).modes(&[]).go();
    assert!(crate::r_s26_common::modes_on_stack(&t, id).is_empty());
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Season of Weaving"));
    assert_eq!(t.hand_size(P0), hand - 1);
    assert!(t.on_battlefield(giant));
}

// --- Mimic Vat ---------------------------------------------------------------------------------

#[test]
fn mimic_vat_with_nothing_exiled_makes_no_token() {
    cr!("602.2", "607.2a", "608.2");
    ruling!("Mimic Vat", "You may activate the second ability even if no card has been exiled with Mimic Vat. If no card has been exiled with Mimic Vat by the time the ability resolves, no token will be created.");
    supported("Mimic Vat");
    let mut t = TestGame::new(2);
    let vat = t.battlefield(P0, "Mimic Vat");
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, vat, "Create a token").expect("activation");
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert!(t.obj_now(vat).tapped);
}

#[test]
fn mimic_vat_can_exile_a_noncreature_card() {
    cr!("603.10a", "607.2a", "707.2");
    ruling!("Mimic Vat", "You may exile a noncreature card with Mimic Vat's first ability. For example, if a nontoken artifact that's become a creature is put into a graveyard from the battlefield, Mimic Vat's first ability triggers and you may exile that card.");
    let mut t = TestGame::new(2);
    let vat = t.battlefield(P0, "Mimic Vat");
    let ring = t.battlefield(P1, "Sol Ring");
    crate::r_p017_common::animate(&mut t, ring, 2);
    assert!(t.obj_now(ring).is(CardType::Creature));
    t.answer_yes(P0, true);
    kill(&mut t, ring);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_exile("Sol Ring"));
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, vat, "Create a token").expect("activation");
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(name_of(&t, toks[0]), "Sol Ring");
    assert!(!t.obj_now(toks[0]).is(CardType::Creature));
}

// --- Deceiver of Form ------------------------------------------------------------------------

#[test]
fn deceiver_of_form_may_put_the_card_on_the_bottom_either_way() {
    cr!("701.20a", "401.4");
    ruling!("Deceiver of Form", "You may put the card on the bottom of your library in any of the possible scenarios that could occur with this card. It doesn’t matter whether the card is a creature card or not. If it is a creature card, it doesn’t matter whether you chose to have the copy effect apply.");
    supported("Deceiver of Form");
    for (top, copy) in [("Lightning Bolt", false), ("Hill Giant", false), ("Hill Giant", true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Deceiver of Form");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let card = t.library_top(P0, top);
        if top == "Hill Giant" {
            t.answer_yes(P0, copy);
        }
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        let lib = &t.g.player(P0).library;
        assert_eq!(*lib.first().unwrap(), t.g.current(card), "{top}: not on the bottom");
        let want = if copy { "Hill Giant" } else { "Grizzly Bears" };
        assert_eq!(name_of(&t, bears), want, "{top}");
    }
}

// --- Finale of Promise -----------------------------------------------------------------------

/// P0 casts Finale of Promise with X = `x`, targeting the graveyard cards `targets`.
fn finale(t: &mut TestGame, x: usize, targets: &[Entity]) -> ObjectId {
    supported("Finale of Promise");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", x);
    let f = t.hand(P0, "Finale of Promise");
    let mut c = t.cast(P0, f).x(x as i64);
    for e in targets {
        c = c.target(*e);
    }
    c.go()
}

#[test]
fn finale_of_promise_casts_then_copies_each_twice() {
    cr!("608.2g", "707.10", "601.2");
    ruling!("Finale of Promise", "As Finale of Promise resolves, first you cast the target instant card and/or the target sorcery card in either order. Then, if X is 10 or more, you copy each of those twice and put the copies on the stack in any order. The copies will resolve before the original spells.");
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let div = t.graveyard(P0, "Divination");
    finale(&mut t, 10, &[obj(bolt), obj(div)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    for _ in 0..4 {
        keep_copy_targets(&mut t, P0);
    }
    t.resolve();
    // Both cast, and four copies above them.
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 4);
    let originals: Vec<ObjectId> = t
        .g
        .stack
        .iter()
        .copied()
        .filter(|s| !copies.contains(s))
        .collect();
    assert_eq!(originals.len(), 2);
    let pos = |id: &ObjectId| t.g.stack.iter().position(|x| x == id).unwrap();
    for c in &copies {
        for o in &originals {
            assert!(pos(c) > pos(o));
        }
    }
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 9);
    assert_eq!(t.hand_size(P0), hand + 6);
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Divination"));
}

#[test]
fn finale_of_promise_casts_a_split_card_only_once() {
    cr!("709.3", "608.2g", "115.3");
    ruling!("Finale of Promise", "A split card that's an instant and a sorcery card may be the target of Finale of Promise twice. However, once you cast that card once, it can't be cast a second time.");
    // Thrash // Threat: Thrash an instant, Threat ("Create a 4/4 red and green Beast
    // creature token with trample.") a sorcery; its mana value is 6.
    supported("Thrash // Threat");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Thrash // Threat");
    let f = finale(&mut t, 6, &[obj(card), obj(card)]);
    // Targeted twice: as the instant card and as the sorcery card.
    assert_eq!(targets_of(&t, f), vec![obj(card), obj(card)]);
    // Cast it (as Threat).
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve();
    // It was cast once.
    let spells: Vec<ObjectId> = t
        .g
        .stack
        .iter()
        .copied()
        .filter(|s| t.g.obj(*s).kind == mtg_engine::object::ObjKind::Card)
        .collect();
    assert_eq!(spells.len(), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert!(t.in_exile("Thrash // Threat"));
}
