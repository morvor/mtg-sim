//! Rulings for targets constrained together (gap-targeting): exact target counts ("X
//! target creatures", multikicker), targets with a relationship ("with equal toughness",
//! "with total power 10 or less", "that share a creature type"), targets chosen for each
//! opponent, and "each mode must target a different player".

use crate::r_s01_common::*;
use crate::r_s04_common::{add_mana, untapped_lands};
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::eval::Ctx;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn stack_targets(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.obj(id)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect()
}

/// (min, max) of each target choice asked since `from`.
fn bounds_since(t: &TestGame, from: usize) -> Vec<(u32, u32)> {
    asked_since(t, from)
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { min, max, .. } => Some((*min, *max)),
            _ => None,
        })
        .collect()
}

/// Executes `effect` as if a spell or ability `p` controls with these targets resolved.
fn run(t: &mut TestGame, p: PlayerId, effect: Effect, targets: &[Entity]) {
    let mut ctx = Ctx::new(None, p);
    ctx.targets = targets.iter().map(|e| vec![*e]).collect();
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

fn pump_toughness(t: &mut TestGame, id: ObjectId, n: i32) {
    run(
        t,
        P0,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![id])),
            mods: vec![Modification::ModifyPT(Value::c(0), Value::c(n))],
            duration: Duration::EndOfTurn,
        },
        &[],
    );
}

fn gain_control(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    run(
        t,
        p,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn multikicker_number_of_targets_is_one_more_than_the_kicks() {
    cr!("601.2b", "601.2c", "702.33c", "702.33d");
    ruling!(
        "Comet Storm",
        "The number of targets you choose for Comet Storm is one more than the number of times it's kicked. First you declare how many times you're going to kick the spell (at the same time you declare the value of X), then you choose the targets accordingly"
    );
    ruling!(
        "Strength of the Tajuru",
        "The number of targets you choose for Strength of the Tajuru is one more than the number of times it's kicked. First you declare how many times you're going to kick the spell (at the same time you declare the value of X), then you choose the targets accordingly"
    );
    supported("Comet Storm");
    supported("Strength of the Tajuru");
    for (name, land) in [
        ("Comet Storm", "Mountain"),
        ("Strength of the Tajuru", "Forest"),
    ] {
        // Kicked once: exactly two targets are chosen, the kicks and X announced first.
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Craw Wurm");
        t.battlefield(P1, "Hill Giant");
        t.lands(P0, land, 5);
        let spell = t.hand(P0, name);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
        let from = t.asked().len();
        let id = t.cast(P0, spell).x(2).target(a).go();
        let order: Vec<&str> = asked_since(&t, from)
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::OptionalCost { .. } => Some("kick"),
                Decision::ChooseX { .. } => Some("x"),
                Decision::ChooseTargets { .. } => Some("targets"),
                _ => None,
            })
            .collect();
        let targets_at = order.iter().position(|s| *s == "targets").unwrap();
        assert!(order[..targets_at].contains(&"kick"), "{name}: {order:?}");
        assert!(order[..targets_at].contains(&"x"), "{name}: {order:?}");
        assert_eq!(bounds_since(&t, from), vec![(2, 2)], "{name}");
        let chosen = stack_targets(&t, id);
        assert_eq!(chosen.len(), 2, "{name}");
        assert!(chosen.contains(&Entity::Object(a)), "{name}");
    }
}

#[test]
fn multikicker_worked_example() {
    cr!("601.2b", "601.2f", "702.33c");
    ruling!(
        "Comet Storm",
        "For example, if you want Comet Storm to deal 4 damage to each of three different targets, that means X is 4 and you're kicking the spell twice."
    );
    ruling!(
        "Strength of the Tajuru",
        "For example, if you want to put four +1/+1 counters on each of three different targets, that means X is 4 and you're kicking the spell twice."
    );
    // Comet Storm, X = 4, kicked twice: {6}{R}{R} for 4 damage to each of three targets.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 8);
    let spell = t.hand(P0, "Comet Storm");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    t.cast(P0, spell)
        .x(4)
        .targets(&[Entity::Object(a), Entity::Object(b), Entity::Player(P1)])
        .go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 16);

    // Strength of the Tajuru, X = 4, kicked twice: {6}{G}{G} for four counters on each.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 8);
    let spell = t.hand(P0, "Strength of the Tajuru");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    t.cast(P0, spell)
        .x(4)
        .targets(&[Entity::Object(a), Entity::Object(b), Entity::Object(c)])
        .go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    for x in [a, b, c] {
        assert_eq!(t.counters(x, "+1/+1"), 4);
    }
}

#[test]
fn multikicker_still_affects_the_legal_targets() {
    cr!("608.2b");
    ruling!(
        "Comet Storm",
        "As long as any of its targets are legal at the time Comet Storm resolves, Comet Storm will deal X damage to each of those legal targets."
    );
    ruling!(
        "Strength of the Tajuru",
        "As long as any of its targets are legal at the time Strength of the Tajuru resolves, you'll put X +1/+1 counters on each of those legal targets."
    );
    // Comet Storm kicked once at a Wurm and P1; the Wurm leaves: P1 still takes 3.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 6);
    let spell = t.hand(P0, "Comet Storm");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.cast(P0, spell)
        .x(3)
        .targets(&[Entity::Object(wurm), Entity::Player(P1)])
        .go();
    t.g.move_object(
        wurm,
        Zone::Hand(P1),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.resolve();
    assert_eq!(t.life(P1), 17);

    // Strength of the Tajuru kicked once: one target leaves, the other gets the counters.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 5);
    let spell = t.hand(P0, "Strength of the Tajuru");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.cast(P0, spell)
        .x(2)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.g.move_object(a, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    t.resolve();
    assert_eq!(t.counters(b, "+1/+1"), 2);
}

#[test]
fn thrive_x_different_creatures() {
    cr!("601.2c", "115.3");
    ruling!("Thrive", "It means X different creatures.");
    supported("Thrive");
    // X = 3: three different creatures get one counter each; naming one creature three
    // times isn't a legal choice.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 4);
    let spell = t.hand(P0, "Thrive");
    let from = t.asked().len();
    t.cast(P0, spell)
        .x(3)
        .targets(&[Entity::Object(a), Entity::Object(a), Entity::Object(a)])
        .go();
    assert_eq!(bounds_since(&t, from), vec![(3, 3)]);
    t.resolve();
    for x in [a, b, c] {
        assert_eq!(t.counters(x, "+1/+1"), 1);
    }
}

#[test]
fn vats_targets_with_equal_toughness_on_resolution() {
    cr!("608.2b", "601.2c");
    ruling!(
        "V.A.T.S.",
        "In the rare case where the legal targets no longer all have equal toughness"
    );
    ruling!(
        "V.A.T.S.",
        "If you choose just one target for V.A.T.S., that creature will be destroyed when V.A.T.S. resolves as long as it’s still a legal target, regardless of whether or not its toughness has changed"
    );
    supported("V.A.T.S.");
    // Two 2/2s; one becomes 2/3 before V.A.T.S. resolves: neither is destroyed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let runeclaw = t.battlefield(P1, "Runeclaw Bear");
    add_mana(&mut t, P0, ManaType::B, 4);
    let spell = t.hand(P0, "V.A.T.S.");
    t.cast(P0, spell)
        .targets(&[Entity::Object(bears), Entity::Object(runeclaw)])
        .go();
    pump_toughness(&mut t, bears, 1);
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(runeclaw));
    assert!(t.in_graveyard(P0, "V.A.T.S."));

    // The one that's no longer a legal target (it left) isn't compared: the other is
    // destroyed even though its toughness changed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let runeclaw = t.battlefield(P1, "Runeclaw Bear");
    add_mana(&mut t, P0, ManaType::B, 4);
    let spell = t.hand(P0, "V.A.T.S.");
    t.cast(P0, spell)
        .targets(&[Entity::Object(bears), Entity::Object(runeclaw)])
        .go();
    t.g.move_object(
        bears,
        Zone::Hand(P1),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    pump_toughness(&mut t, runeclaw, 2);
    t.resolve();
    assert!(!t.on_battlefield(runeclaw));
    assert!(t.in_graveyard(P1, "Runeclaw Bear"));

    // A single target is destroyed whatever its toughness has become.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::B, 4);
    let spell = t.hand(P0, "V.A.T.S.");
    t.cast(P0, spell).targets(&[Entity::Object(bears)]).go();
    pump_toughness(&mut t, bears, 3);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn mass_mutiny_one_target_per_opponent() {
    cr!("601.2c", "115.6");
    ruling!(
        "Mass Mutiny",
        "You can cast Mass Mutiny even if an opponent doesn't control any creatures. You simply won't choose a target for that opponent."
    );
    ruling!("Mass Mutiny", "Mass Mutiny can target untapped creatures.");
    ruling!(
        "Mass Mutiny",
        "The phrase \"up to one\" was inadvertently omitted from Mass Mutiny's rules text."
    );
    supported("Mass Mutiny");
    // P1 controls nothing; P2's untapped creature is targeted and stolen.
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P2, "Hill Giant");
    add_mana(&mut t, P0, ManaType::R, 5);
    let spell = t.hand(P0, "Mass Mutiny");
    let from = t.asked().len();
    t.cast(P0, spell).target(giant).go();
    // Only P2 is asked about (P1 has no creature), for up to one.
    assert_eq!(bounds_since(&t, from), vec![(0, 1)]);
    t.resolve();
    assert_eq!(t.obj_now(giant).controller, P0);
    assert!(t.obj_now(giant).has_keyword(mtg_engine::keywords::KeywordKind::Haste));

    // "Up to one": no target need be chosen for an opponent who has creatures.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    add_mana(&mut t, P0, ManaType::R, 5);
    let spell = t.hand(P0, "Mass Mutiny");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(t.obj_now(giant).controller, P0);
}

#[test]
fn molten_primordial_one_target_per_opponent() {
    cr!("603.3d", "601.2c");
    ruling!(
        "Molten Primordial",
        "You can choose a number of targets up to the number of opponents you have, one target per opponent."
    );
    ruling!(
        "Molten Primordial",
        "Molten Primordial’s triggered ability can target a creature that’s already untapped."
    );
    supported("Molten Primordial");
    // Three opponents: one choice each; the untapped creatures can be targeted.
    let mut t = TestGame::new(4);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    let c = t.battlefield(P3, "Llanowar Elves");
    let d = t.battlefield(P3, "Runeclaw Bear");
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.answer_targets(P0, &[Entity::Object(c), Entity::Object(d)]);
    let from = t.asked().len();
    t.enter(P0, "Molten Primordial");
    t.settle();
    let maxes: Vec<u32> = bounds_since(&t, from).iter().map(|b| b.1).collect();
    assert_eq!(maxes, vec![1, 1, 1]);
    t.resolve_all();
    assert_eq!(t.obj_now(a).controller, P0);
    assert_eq!(t.obj_now(b).controller, P0);
    // Two of P3's creatures weren't a legal choice; neither was taken.
    assert_eq!(t.obj_now(c).controller, P3);
    assert_eq!(t.obj_now(d).controller, P3);
}

#[test]
fn blatant_thievery_one_permanent_from_each_player() {
    cr!("608.2b", "601.2c");
    ruling!(
        "Blatant Thievery",
        "If a permanent changes controller after being targeted but before this spell resolves, you won't gain control of that permanent."
    );
    ruling!(
        "Blatant Thievery",
        "You gain control of only one permanent from each player."
    );
    supported("Blatant Thievery");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P2, "Hill Giant");
    add_mana(&mut t, P0, ManaType::U, 7);
    let spell = t.hand(P0, "Blatant Thievery");
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(elves)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let id = t.cast(P0, spell).go();
    // One of P1's permanents, one of P2's.
    let chosen = stack_targets(&t, id);
    assert_eq!(chosen.len(), 2);
    assert!(chosen.contains(&Entity::Object(giant)));
    let from_p1 = chosen
        .iter()
        .find(|e| **e != Entity::Object(giant))
        .and_then(|e| e.object())
        .unwrap();
    // P2 gains control of the permanent targeted for P1.
    gain_control(&mut t, P2, from_p1);
    t.resolve();
    assert_eq!(t.obj_now(from_p1).controller, P2);
    assert_eq!(t.obj_now(giant).controller, P0);
}

fn lich_dies(t: &mut TestGame) {
    let lich = t.battlefield(P0, "Vindictive Lich");
    t.g.destroy(lich, None);
    t.g.flush_events();
    t.settle();
}

#[test]
fn vindictive_lich_no_more_modes_than_opponents() {
    cr!("700.2", "700.2b", "603.3c");
    ruling!(
        "Vindictive Lich",
        "You can’t choose more modes for Vindictive Lich’s triggered ability than you have opponents, but you need to choose at least one target if you can."
    );
    ruling!(
        "Vindictive Lich",
        "You don’t have to choose a mode for each opponent. For example, you could make one opponent lose 5 life and spare each other opponent."
    );
    supported("Vindictive Lich");
    // Two players: asking for all three modes isn't legal; one mode is chosen.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    for _ in 0..2 {
        t.hand(P1, "Hill Giant");
    }
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1, 2]));
    lich_dies(&mut t);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(top).stack.as_ref().unwrap().chosen.len(), 1);

    // Three opponents, one mode: one opponent loses 5 life, the others are spared.
    let mut t = TestGame::new(4);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.answer_targets(P0, &[Entity::Player(P2)]);
    lich_dies(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P2), 15);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P3), 20);

    // Three opponents, three modes: each targets a different opponent.
    let mut t = TestGame::new(4);
    for p in [P1, P2, P3] {
        t.battlefield(p, "Grizzly Bears");
        for _ in 0..2 {
            t.hand(p, "Hill Giant");
        }
    }
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1, 2]));
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    lich_dies(&mut t);
    let top = *t.g.stack.last().unwrap();
    let mut players: Vec<Entity> = stack_targets(&t, top);
    players.sort();
    assert_eq!(
        players,
        vec![Entity::Player(P1), Entity::Player(P2), Entity::Player(P3)]
    );
}

#[test]
fn creature_cards_must_share_a_creature_type() {
    cr!("601.2c", "205.3m");
    ruling!(
        "Return from Extinction",
        "If you choose the second mode, the cards must share at least one creature type, such as Sliver or Warrior. Card types such as artifact, and supertypes such as legendary or snow, aren’t creature types."
    );
    ruling!(
        "Unbury",
        "If you choose the second mode, the cards must share at least one creature type, such as Faerie or Goblin. Card types such as artifact, and supertypes such as legendary or snow, aren't creature types."
    );
    ruling!(
        "Raise the Draugr",
        "If you choose the second mode, the cards must share at least one creature type, such as Knight or Djinn. Card types (e.g., artifact) and supertypes (e.g., legendary or snow) aren’t creature types."
    );
    for name in ["Return from Extinction", "Unbury", "Raise the Draugr"] {
        supported(name);
        // Ornithopter (a Thopter) and Memnite (a Construct) are both artifact creature
        // cards but share no creature type: the second mode can't be chosen, and the
        // first returns one card.
        let mut t = TestGame::new(2);
        let thopter = t.graveyard(P0, "Ornithopter");
        let memnite = t.graveyard(P0, "Memnite");
        add_mana(&mut t, P0, ManaType::B, 3);
        let spell = t.hand(P0, name);
        let id = t
            .cast(P0, spell)
            .modes(&[1])
            .targets(&[Entity::Object(thopter), Entity::Object(memnite)])
            .go();
        let si = t.obj(id).stack.clone().unwrap();
        assert_eq!(si.chosen[0].mode, Some(0), "{name}");
        assert_eq!(stack_targets(&t, id).len(), 1, "{name}");
        t.resolve();
        assert_eq!(t.hand_size(P0), 1, "{name}");
    }
}

#[test]
fn secret_tunnel_creatures_share_a_creature_type() {
    cr!("601.2c", "602.2b", "608.2b");
    ruling!(
        "Secret Tunnel",
        "The creatures must share at least one creature type, such as Ally or Lemur."
    );
    ruling!(
        "Secret Tunnel",
        "If one of the two creatures leaves the battlefield before Secret Tunnel's ability resolves, the other still can't be blocked this turn as long as it has a creature type that the other card had as it left the battlefield."
    );
    supported("Secret Tunnel");
    // Ornithopter and Memnite (artifact creatures, no common creature type): the
    // ability can't be activated.
    let mut t = TestGame::new(2);
    let tunnel = t.battlefield(P0, "Secret Tunnel");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Memnite");
    t.lands(P0, "Wastes", 4);
    // The second activated ability ({T}: Add {C} is the first).
    let ability = 1;
    assert!(t.activate(P0, tunnel, ability, &[]).is_err());

    // Two Bears: one leaves, the other still can't be blocked.
    let mut t = TestGame::new(2);
    let tunnel = t.battlefield(P0, "Secret Tunnel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let runeclaw = t.battlefield(P0, "Runeclaw Bear");
    t.lands(P0, "Wastes", 4);
    let blocker = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(runeclaw)]);
    t.activate(P0, tunnel, ability, &[]).unwrap();
    t.g.move_object(
        runeclaw,
        Zone::Hand(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.resolve();
    // The Giant's block isn't legal: the Bears deal their damage to P1.
    t.attack(&[(bears, Entity::Player(P1))], &[(blocker, bears)]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn reunion_of_the_house_total_power() {
    cr!("601.2c", "608.2b", "604.3");
    ruling!(
        "Reunion of the House",
        "Reunion of the House can return any combination of creature cards whose powers total 10 or less."
    );
    ruling!(
        "Reunion of the House",
        "Use the power of the creature cards as they exist in your graveyard to determine whether they can be the targets of Reunion of the House."
    );
    ruling!(
        "Reunion of the House",
        "If the power of the target creature cards changes while in your graveyard, most likely because one has a characteristic-defining ability that defines a * in its power, the total power of the creature cards may become greater than 10. If this happens, the entire selection of targets is illegal."
    );
    supported("Reunion of the House");
    // Craw Wurm (6), Hill Giant (3) and two 0-power cards (Ornithopter, Clone as a 0/0
    // in the graveyard): 9 in all, so all of them can be returned.
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P0, "Craw Wurm");
    let giant = t.graveyard(P0, "Hill Giant");
    let thopter = t.graveyard(P0, "Ornithopter");
    let clone = t.graveyard(P0, "Clone");
    add_mana(&mut t, P0, ManaType::W, 7);
    let spell = t.hand(P0, "Reunion of the House");
    let all = [
        Entity::Object(wurm),
        Entity::Object(giant),
        Entity::Object(thopter),
        Entity::Object(clone),
    ];
    let id = t.cast(P0, spell).targets(&all).go();
    assert_eq!(stack_targets(&t, id).len(), 4);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);

    // Tarmogoyf's power in the graveyard counts the card types among cards in all
    // graveyards: with only creature cards there it's 1, so Wurm + Giant + Tarmogoyf is
    // 10. A land put into a graveyard makes it 2 and the total 11: nothing is returned.
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P0, "Craw Wurm");
    let giant = t.graveyard(P0, "Hill Giant");
    let goyf = t.graveyard(P0, "Tarmogoyf");
    add_mana(&mut t, P0, ManaType::W, 7);
    let spell = t.hand(P0, "Reunion of the House");
    t.recompute();
    assert_eq!(t.obj(goyf).power(), 1);
    let id = t
        .cast(P0, spell)
        .targets(&[
            Entity::Object(wurm),
            Entity::Object(giant),
            Entity::Object(goyf),
        ])
        .go();
    assert_eq!(stack_targets(&t, id).len(), 3);
    t.graveyard(P1, "Forest");
    t.recompute();
    assert_eq!(t.obj(goyf).power(), 2);
    t.resolve();
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Tarmogoyf"));
    assert!(t.in_graveyard(P0, "Reunion of the House"));
}

/// Puts the Aura `name` onto the battlefield under `p`'s control attached to `host`.
fn aura_on(t: &mut TestGame, p: PlayerId, name: &str, host: ObjectId) -> ObjectId {
    let a = t.battlefield(p, name);
    assert!(t.g.attach(a, Entity::Object(host)));
    t.recompute();
    a
}

#[test]
fn simic_guildmage_moves_a_counter_between_creatures_with_the_same_controller() {
    cr!("122.5", "608.2b", "601.2c", "602.2b");
    ruling!(
        "Simic Guildmage",
        "For the first ability, if the two target creatures aren’t controlled by the same player when the ability resolves, the ability does nothing. The player who controls the two creatures doesn’t have to be the same player who controlled them when the ability was activated, and that player doesn’t have to be Simic Guildmage’s controller."
    );
    ruling!(
        "Simic Guildmage",
        "For the first ability, the first target creature doesn’t need to have a +1/+1 counter on it. If it doesn’t, the ability does nothing."
    );
    supported("Simic Guildmage");
    let setup = |t: &mut TestGame| {
        let gm = t.battlefield(P0, "Simic Guildmage");
        add_mana(t, P0, ManaType::G, 2);
        gm
    };
    // Two of P1's creatures: both change control to P2 before it resolves — still the
    // same controller, so the counter moves.
    let mut t = TestGame::new(3);
    let gm = setup(&mut t);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(a), "+1/+1", 1, None);
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.activate(P0, gm, 0, &[]).unwrap();
    gain_control(&mut t, P2, a);
    gain_control(&mut t, P2, b);
    t.resolve();
    assert_eq!(t.counters(a, "+1/+1"), 0);
    assert_eq!(t.counters(b, "+1/+1"), 1);

    // Only one of them changes control: nothing moves.
    let mut t = TestGame::new(3);
    let gm = setup(&mut t);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(a), "+1/+1", 1, None);
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.activate(P0, gm, 0, &[]).unwrap();
    gain_control(&mut t, P2, b);
    t.resolve();
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 0);

    // The first target needn't have a counter: it can be activated, and does nothing.
    let mut t = TestGame::new(2);
    let gm = setup(&mut t);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.activate(P0, gm, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(b, "+1/+1"), 0);
}

#[test]
fn simic_guildmage_moves_an_aura_to_a_permanent_with_the_same_controller() {
    cr!("701.3a", "303.4d", "115.10");
    ruling!(
        "Simic Guildmage",
        "For the second ability, only the Aura is targeted. When the ability resolves, you choose a permanent to move the Aura onto."
    );
    // P1's Bears wear P0's Pacifism. Answering P0's own creature (not P1's) isn't a
    // legal choice; the Aura moves to P1's other creature. P1's land can't be enchanted
    // by Pacifism.
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Simic Guildmage");
    add_mana(&mut t, P0, ManaType::U, 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Llanowar Elves");
    let pacifism = aura_on(&mut t, P0, "Pacifism", bears);
    t.answer_targets(P0, &[Entity::Object(pacifism)]);
    t.answer_choose(P0, &[Entity::Object(mine)]);
    let from = t.asked().len();
    t.activate(P0, gm, 1, &[]).unwrap();
    t.resolve();
    let offered: Vec<Vec<Entity>> = asked_since(&t, from)
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![vec![Entity::Object(giant)]]);
    assert_eq!(t.obj_now(pacifism).attached_to, Some(Entity::Object(giant)));

    // No other creature P1 controls: the Aura stays.
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Simic Guildmage");
    add_mana(&mut t, P0, ManaType::U, 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    let pacifism = aura_on(&mut t, P0, "Pacifism", bears);
    t.answer_targets(P0, &[Entity::Object(pacifism)]);
    t.activate(P0, gm, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(pacifism).attached_to, Some(Entity::Object(bears)));
}

#[test]
fn bioshift_any_number_between_creatures_with_the_same_controller() {
    cr!("122.5", "608.2b", "601.2c");
    ruling!(
        "Bioshift",
        "If one of the two creatures is an illegal target when Bioshift tries to resolve, or if the creatures are controlled by different players at that time, no counters will move."
    );
    ruling!(
        "Bioshift",
        "You decide how many counters to move when Bioshift resolves."
    );
    ruling!(
        "Bioshift",
        "To move a counter from one creature to another, the counter is removed from the first creature and placed on the second. Any abilities that care about a counter being removed or placed on a creature will apply."
    );
    supported("Bioshift");
    // Two counters of three are moved; Hardened Scales (P0's) adds one as they're put on
    // P0's creature.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.g.add_counters(Entity::Object(a), "+1/+1", 3, None);
    t.battlefield(P0, "Hardened Scales");
    add_mana(&mut t, P0, ManaType::G, 1);
    let spell = t.hand(P0, "Bioshift");
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 3);

    // A creature P1 controls can't be the second target with P0's as the first; a
    // creature of P0's is chosen instead.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Llanowar Elves");
    t.g.add_counters(Entity::Object(a), "+1/+1", 1, None);
    add_mana(&mut t, P0, ManaType::G, 1);
    let spell = t.hand(P0, "Bioshift");
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    let id = t
        .cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(theirs)])
        .go();
    assert_eq!(
        stack_targets(&t, id),
        vec![Entity::Object(a), Entity::Object(b)]
    );
    // Then the second changes controller: no counters move.
    gain_control(&mut t, P1, b);
    t.resolve();
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 0);

    // With no two creatures sharing a controller, Bioshift can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::G, 1);
    let spell = t.hand(P0, "Bioshift");
    assert!(t.cast(P0, spell).try_go().is_err());
}

#[test]
fn glamer_spinners_moves_all_auras_to_a_permanent_with_the_same_controller() {
    cr!("701.3a", "303.4d", "603.3d");
    ruling!(
        "Glamer Spinners",
        "When Glamer Spinners enters, you target only one permanent: the one that will be losing its Auras."
    );
    ruling!(
        "Glamer Spinners",
        "It can't be the targeted permanent, it must have the same controller as the targeted permanent, and it must be able to be enchanted by all the Auras attached to the targeted permanent. If you can't choose a permanent that meets all those criteria, the Auras won't move."
    );
    ruling!(
        "Glamer Spinners",
        "You may target a permanent that has no Auras enchanting it."
    );
    supported("Glamer Spinners");
    // P1's Bears wear Pacifism and Holy Strength; both move to P1's Giant.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Llanowar Elves");
    let p = aura_on(&mut t, P0, "Pacifism", bears);
    let h = aura_on(&mut t, P1, "Holy Strength", bears);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Glamer Spinners");
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(p).attached_to, Some(Entity::Object(giant)));
    assert_eq!(t.obj_now(h).attached_to, Some(Entity::Object(giant)));

    // P1's only other permanent is a land, which Pacifism can't enchant: nothing moves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Forest");
    let p = aura_on(&mut t, P0, "Pacifism", bears);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Glamer Spinners");
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(p).attached_to, Some(Entity::Object(bears)));

    // A permanent without Auras can be targeted.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[Entity::Object(forest)]);
    t.enter(P0, "Glamer Spinners");
    t.settle();
    let top = *t.g.stack.last().unwrap();
    assert_eq!(stack_targets(&t, top), vec![Entity::Object(forest)]);
    t.resolve_all();
}

#[test]
fn crown_of_the_ages_targets_only_the_aura() {
    cr!("701.3a", "115.10", "702.11b");
    ruling!(
        "Crown of the Ages",
        "This only targets the Aura and not either creature. This means it can move Auras onto a creature which can’t normally be targeted by spells and abilities if the Aura is legal on that creature."
    );
    supported("Crown of the Ages");
    // Pacifism moves onto P1's hexproof Gladecover Scout.
    let mut t = TestGame::new(2);
    let crown = t.battlefield(P0, "Crown of the Ages");
    t.lands(P0, "Wastes", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let p = aura_on(&mut t, P0, "Pacifism", bears);
    t.answer_targets(P0, &[Entity::Object(p)]);
    t.answer_choose(P0, &[Entity::Object(scout)]);
    t.activate(P0, crown, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(p).attached_to, Some(Entity::Object(scout)));
}
