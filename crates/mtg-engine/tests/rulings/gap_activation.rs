//! Activated abilities' total costs and activation permissions (gap-activation): cost
//! changes applied increases first (CR 601.2f), the "can't reduce the mana in that cost to
//! less than one mana" floor, target-dependent changes, alternative activation costs
//! (CR 118.9), instant-speed and twice-per-turn permissions (CR 602.5d, 606.3), "as though
//! it had haste" (CR 302.6, 609.4) and "Activate only as an instant" (CR 602.5e).

use crate::r_s01_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The `n`th activated ability of `src`.
fn nth(t: &TestGame, src: ObjectId, n: usize) -> (Ability, ActivatedAbility) {
    let a = t
        .g
        .obj(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .nth(n)
        .cloned()
        .expect("no such ability");
    let AbilityKind::Activated(act) = &a.kind else {
        unreachable!()
    };
    let act = act.clone();
    (a, act)
}

/// The mana `p` pays to activate the `n`th activated ability of `src` (X announced as `x`).
fn mana(t: &mut TestGame, p: PlayerId, src: ObjectId, n: usize, x: u32) -> String {
    t.g.recompute();
    let (a, act) = nth(t, src, n);
    let c = t.g.ability_total_cost_with(p, src, &a, &act, None, x, None);
    c.mana.map(|m| m.to_string()).unwrap_or_default()
}

#[test]
fn training_grounds_reduces_generic_mana_but_keeps_one_mana() {
    cr!("601.2f", "602.2b", "118.7a");
    ruling!(
        "Training Grounds",
        "if an activation cost is {2}{G}, you'd have to pay only {G}. If an activation cost is {2}, though, you'd still have to pay {1}."
    );
    ruling!(
        "Training Grounds",
        "Training Grounds won't affect the part of an activation cost represented by colored mana symbols"
    );
    supported("Training Grounds");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Training Grounds");
    // Walking Ballista: "{4}: Put a +1/+1 counter on this creature."
    let ballista = t.battlefield(P0, "Walking Ballista");
    assert_eq!(mana(&mut t, P0, ballista, 0, 0), "{2}");
    // Shivan Dragon: "{R}: This creature gets +1/+0 until end of turn."
    let dragon = t.battlefield(P0, "Shivan Dragon");
    assert_eq!(mana(&mut t, P0, dragon, 0, 0), "{R}");
    // An opponent's creature isn't affected.
    let theirs = t.battlefield(P1, "Walking Ballista");
    assert_eq!(mana(&mut t, P1, theirs, 0, 0), "{4}");
}

#[test]
fn training_grounds_never_adds_a_mana_payment() {
    cr!("601.2f");
    ruling!(
        "Training Grounds",
        "In particular, it won't increase the cost to include a mana payment of {1}."
    );
    ruling!(
        "Heartstone",
        "It will not add a {1} to abilities with no generic mana in their activation cost."
    );
    ruling!("Power Artifact", "It does not increase the cost to one.");
    supported("Heartstone");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Training Grounds");
    t.battlefield(P0, "Heartstone");
    // Prodigal Sorcerer: "{T}: This creature deals 1 damage to any target."
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    assert_eq!(mana(&mut t, P0, sorcerer, 0, 0), "");
    let dragon = t.battlefield(P0, "Shivan Dragon");
    assert_eq!(mana(&mut t, P0, dragon, 0, 0), "{R}");
}

#[test]
fn reductions_apply_after_increases() {
    cr!("601.2f", "602.2b");
    ruling!(
        "Training Grounds",
        "Training Grounds takes the total cost to activate a creature's activated ability into account, not just the cost printed on it."
    );
    supported("Suppression Field");
    let mut t = TestGame::new(2);
    // The reduction's source comes first: it still applies after the increase.
    t.battlefield(P0, "Training Grounds");
    t.battlefield(P0, "Suppression Field");
    let dragon = t.battlefield(P0, "Shivan Dragon");
    assert_eq!(mana(&mut t, P0, dragon, 0, 0), "{R}");
    // Suppression Field alone: {2}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Suppression Field");
    let dragon = t.battlefield(P0, "Shivan Dragon");
    assert_eq!(mana(&mut t, P0, dragon, 0, 0), "{2}{R}");
}

#[test]
fn training_grounds_reduces_x_once_announced() {
    cr!("601.2f", "107.3b");
    ruling!(
        "Training Grounds",
        "If you control Training Grounds and you activate the ability with X equal to 5, you'll have to pay only {3}{B}{B}."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Training Grounds");
    // Drana, Kalastria Bloodchief: "{X}{B}{B}: Target creature gets -0/-X ..."
    let drana = t.battlefield(P0, "Drana, Kalastria Bloodchief");
    assert_eq!(mana(&mut t, P0, drana, 0, 5), "{3}{B}{B}");
}

#[test]
fn training_grounds_does_not_reduce_cycling_from_hand() {
    cr!("602.2b");
    ruling!(
        "Training Grounds",
        "Training Grounds affects only creatures you control on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Training Grounds");
    // Yoked Plowbeast, a creature card, has cycling {2}; it isn't a creature
    // on the battlefield either.
    let card = t.hand(P0, "Yoked Plowbeast");
    assert_eq!(mana(&mut t, P0, card, 0, 0), "{2}");
}

#[test]
fn power_artifact_reduces_only_the_enchanted_artifacts_generic_mana() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Power Artifact",
        "Only affects the generic mana part of activation costs."
    );
    supported("Power Artifact");
    let mut t = TestGame::new(2);
    let ballista = t.battlefield(P0, "Walking Ballista");
    let other = t.battlefield(P0, "Walking Ballista");
    let aura = t.battlefield(P0, "Power Artifact");
    t.g.objects[aura.0 as usize].attached_to = Some(Entity::Object(ballista));
    t.g.dirty = true;
    assert_eq!(mana(&mut t, P0, ballista, 0, 0), "{2}");
    assert_eq!(mana(&mut t, P0, other, 0, 0), "{4}");
}

#[test]
fn zirda_reduces_non_mana_abilities_down_to_one_mana() {
    cr!("601.2f", "605.1a");
    ruling!(
        "Zirda, the Dawnwaker",
        "a cycling cost of {2} would become {1}, and one of {1}{R} would become {R}"
    );
    ruling!(
        "Zirda, the Dawnwaker",
        "An activated mana ability is one that produces mana as it resolves, not one that costs mana to activate."
    );
    let mut t = TestGame::new(2);
    let zirda = t.battlefield(P0, "Zirda, the Dawnwaker");
    // Zirda's own "{1}, {T}: ..." already costs one mana.
    assert_eq!(mana(&mut t, P0, zirda, 0, 0), "{1}");
    let ballista = t.battlefield(P0, "Walking Ballista");
    assert_eq!(mana(&mut t, P0, ballista, 0, 0), "{2}");
    // A cycling cost of {2} (in any zone) becomes {1}.
    let plowbeast = t.hand(P0, "Yoked Plowbeast");
    assert_eq!(mana(&mut t, P0, plowbeast, 0, 0), "{1}");
    // A mana ability that costs mana isn't affected: Azorius Signet's
    // "{1}, {T}: Add {W}{U}." (with {2} more from Suppression Field, it would be).
    t.battlefield(P0, "Suppression Field");
    let signet = t.battlefield(P0, "Azorius Signet");
    assert_eq!(mana(&mut t, P0, signet, 0, 0), "{1}");
    assert_eq!(mana(&mut t, P0, ballista, 0, 0), "{4}");
    // An opponent's abilities aren't affected ("Abilities you activate"): {4} + {2}.
    let theirs = t.battlefield(P1, "Walking Ballista");
    assert_eq!(mana(&mut t, P1, theirs, 0, 0), "{6}");
}

#[test]
fn kopala_taxes_abilities_targeting_merfolk_once() {
    cr!("601.2c", "601.2f", "602.2b");
    ruling!(
        "Kopala, Warden of Waves",
        "Spells and abilities that target more than one Merfolk you control cost only {2} more to cast or activate."
    );
    supported("Kopala, Warden of Waves");
    let mut t = TestGame::new(2);
    let kopala = t.battlefield(P0, "Kopala, Warden of Waves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    // With no lands, P1 can't pay {2} to target Kopala (a Merfolk) ...
    let before = t.g.clone();
    assert!(t
        .activate(P1, sorcerer, 0, &[Entity::Object(kopala)])
        .is_err());
    t.g = before;
    // ... but can target a non-Merfolk.
    assert!(t.activate(P1, sorcerer, 0, &[Entity::Object(bears)]).is_ok());
    // With two lands it can target Kopala, paying {2} more once.
    let mut t = TestGame::new(2);
    let kopala = t.battlefield(P0, "Kopala, Warden of Waves");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    let lands = t.lands(P1, "Island", 3);
    assert!(t
        .activate(P1, sorcerer, 0, &[Entity::Object(kopala)])
        .is_ok());
    let tapped = lands.iter().filter(|l| t.obj_now(**l).tapped).count();
    assert_eq!(tapped, 2);
}

#[test]
fn dwarven_mauler_reduces_equip_abilities_that_target_it() {
    cr!("601.2c", "601.2f", "702.6a");
    supported("Dwarven Mauler");
    let mut t = TestGame::new(2);
    let mauler = t.battlefield(P0, "Dwarven Mauler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Bonesplitter: "Equip {1}".
    let sword = t.battlefield(P0, "Bonesplitter");
    let before = t.g.clone();
    assert!(t.activate(P0, sword, 0, &[Entity::Object(bears)]).is_err());
    t.g = before;
    assert!(t.activate(P0, sword, 0, &[Entity::Object(mauler)]).is_ok());
    t.resolve_all();
    assert_eq!(t.obj_now(t.g.current(sword)).attached_to, Some(Entity::Object(mauler)));
}

#[test]
fn new_perspectives_cycles_for_zero_but_still_discards() {
    cr!("118.9", "702.29a");
    ruling!(
        "New Perspectives",
        "If you choose to pay New Perspectives’s alternate activation cost, you still discard the card with cycling to activate the ability."
    );
    ruling!(
        "New Perspectives",
        "The card you wish to cycle counts when determining whether you have seven or more cards in hand."
    );
    supported("New Perspectives");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "New Perspectives");
    let card = t.hand(P0, "Yoked Plowbeast");
    // Seven cards in hand, counting the one cycled.
    while t.hand_size(P0) < 7 {
        t.hand(P0, "Grizzly Bears");
    }
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    assert!(t.activate(P0, card, 0, &[]).is_ok());
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Yoked Plowbeast"));
    // Discarded one, drew one.
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn forge_anew_pays_zero_for_the_first_equip_each_of_your_turns_at_instant_speed() {
    cr!("118.9", "118.9a", "602.5d");
    ruling!("Forge Anew", "Reconfigure is not an equip ability.");
    supported("Forge Anew");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forge Anew");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Bonesplitter");
    // During combat on P0's turn (not sorcery timing), for {0}.
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    assert!(t.activate(P0, sword, 0, &[Entity::Object(bears)]).is_ok());
    t.resolve_all();
    assert_eq!(t.obj_now(t.g.current(sword)).attached_to, Some(Entity::Object(bears)));
    // The second equip ability this turn has no alternative cost: with no lands, it
    // can't be paid.
    let other = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    assert!(t.activate(P0, sword, 0, &[Entity::Object(other)]).is_err());
    // Not on an opponent's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forge Anew");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Plains", 1);
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(t.activate(P0, sword, 0, &[Entity::Object(bears)]).is_err());
}

#[test]
fn leonin_shikari_allows_equip_at_instant_speed_on_any_turn() {
    cr!("602.5d", "702.6a");
    ruling!(
        "Leonin Shikari",
        "Leonin Shikari allows you to move your Equipment around during the combat phase or in response to a spell or ability."
    );
    ruling!(
        "Leonin Shikari",
        "Normally, equip abilities can be activated only any time you could cast a sorcery."
    );
    supported("Leonin Shikari");
    // Without Shikari, no equipping during combat.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Plains", 1);
    t.set_step(P1, Step::DeclareAttackers);
    assert!(t.activate(P0, sword, 0, &[Entity::Object(bears)]).is_err());
    // With it, even on an opponent's turn.
    t.battlefield(P0, "Leonin Shikari");
    assert!(t.activate(P0, sword, 0, &[Entity::Object(bears)]).is_ok());
}

#[test]
fn oath_of_teferi_allows_two_loyalty_activations_each_turn() {
    cr!("606.3");
    ruling!(
        "Oath of Teferi",
        "you may activate the same ability of a planeswalker twice"
    );
    ruling!(
        "Oath of Teferi",
        "If you somehow control more than one Oath of Teferi, you won’t be able to activate abilities of planeswalkers you control more than twice in one turn."
    );
    supported("Oath of Teferi");
    supported("Jace Beleren");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    t.g.objects[jace.0 as usize]
        .counters
        .insert("loyalty".into(), 3);
    // Without Oath: once.
    let before = t.g.clone();
    assert!(t.activate(P0, jace, 0, &[]).is_ok());
    t.resolve_all();
    assert!(t.activate(P0, jace, 0, &[]).is_err());
    t.g = before;
    t.battlefield(P0, "Oath of Teferi");
    t.battlefield(P0, "Oath of Teferi");
    for _ in 0..2 {
        assert!(t.activate(P0, jace, 0, &[]).is_ok());
        t.resolve_all();
    }
    assert!(t.activate(P0, jace, 0, &[]).is_err());
}

#[test]
fn urza_planeswalker_activates_twice() {
    cr!("606.3");
    supported("Urza, Planeswalker");
    let mut t = TestGame::new(2);
    let urza = t.battlefield(P0, "Urza, Planeswalker");
    t.g.objects[urza.0 as usize]
        .counters
        .insert("loyalty".into(), 7);
    // "0: Create two 1/1 colorless Soldier artifact creature tokens."
    for _ in 0..2 {
        assert!(t.activate(P0, urza, 2, &[]).is_ok());
        t.resolve_all();
    }
    assert!(t.activate(P0, urza, 2, &[]).is_err());
}

#[test]
fn the_wandering_emperor_activates_at_instant_speed_the_turn_she_entered() {
    cr!("606.3", "602.5d");
    ruling!(
        "The Wandering Emperor",
        "You may still only activate one of The Wandering Emperor's loyalty abilities on the turn she entered the battlefield."
    );
    supported("The Wandering Emperor");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::DeclareAttackers);
    let emperor = t.enter(P0, "The Wandering Emperor");
    t.settle();
    // "−1: Create a 2/2 white Samurai creature token with vigilance."
    assert!(t.activate(P0, emperor, 1, &[]).is_ok());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Samurai Token").len(), 1);
    assert!(t.activate(P0, emperor, 1, &[]).is_err());
    // Without having entered this turn: sorcery timing again.
    let mut t = TestGame::new(2);
    let emperor = t.battlefield(P0, "The Wandering Emperor");
    t.g.objects[emperor.0 as usize]
        .counters
        .insert("loyalty".into(), 4);
    // (A later turn.)
    t.g.turn.number += 1;
    t.g.history.permanents_entered.clear();
    t.g.dirty = true;
    t.set_step(P1, Step::DeclareAttackers);
    assert!(t.activate(P0, emperor, 1, &[]).is_err());
}

#[test]
fn thousand_year_elixir_lets_summoning_sick_creatures_tap_but_not_attack() {
    cr!("302.6", "602.5a", "609.4");
    ruling!(
        "Thousand-Year Elixir",
        "Thousand-Year Elixir doesn’t actually grant haste to creatures you control, nor does it let you attack with them as though they had haste."
    );
    supported("Thousand-Year Elixir");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield_sick(P0, "Prodigal Sorcerer");
    let before = t.g.clone();
    assert!(t
        .activate(P0, sorcerer, 0, &[Entity::Player(P1)])
        .is_err());
    t.g = before;
    t.battlefield(P0, "Thousand-Year Elixir");
    let life = t.life(P1);
    assert!(t
        .activate(P0, sorcerer, 0, &[Entity::Player(P1)])
        .is_ok());
    t.resolve_all();
    assert_eq!(t.life(P1), life - 1);
    let s = t.g.current(sorcerer);
    assert!(!t.obj_now(s).has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    t.set_step(P0, Step::DeclareAttackers);
    assert!(!t.g.can_attack(bears));
}

#[test]
fn lions_eye_diamond_is_activated_only_as_an_instant() {
    cr!("602.5e", "605.3a");
    ruling!(
        "Lion's Eye Diamond",
        "it can only be activated at times when you can cast an instant. Yes, this is a bit weird."
    );
    supported("Lion's Eye Diamond");
    let mut t = TestGame::new(2);
    let led = t.battlefield(P0, "Lion's Eye Diamond");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.hand(P0, "Grizzly Bears");
    t.g.recompute();
    let activatable = |t: &mut TestGame, id: ObjectId| {
        t.g.activatable_abilities(P0)
            .iter()
            .any(|(src, _)| *src == id)
    };
    // In the middle of a mana payment (CR 605.3a), other mana abilities may be
    // activated, but not one that's activated only as an instant.
    t.g.turn.priority = None;
    t.g.mana_hint = Some(vec![]);
    assert!(activatable(&mut t, elves));
    assert!(!activatable(&mut t, led));
    // With priority and nothing being cast, it can be activated for mana.
    t.g.mana_hint = None;
    t.g.turn.priority = Some(P0);
    assert!(activatable(&mut t, led));
    assert!(t.activate(P0, led, 0, &[]).is_ok());
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}
