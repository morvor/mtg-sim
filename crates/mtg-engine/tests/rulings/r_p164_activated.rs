//! Rulings batch P164 — activated-ability synergies: Illusionist's Bracers and Crackdown
//! Construct (what an activated and a mana ability are, CR 602.1, 605.1a), the
//! activation-cost reducers Biomancer's Familiar, Heartstone and Training Grounds (CR
//! 601.2f, 602.2b), Ertha Jo copying a linked ability (CR 607.2, 707.10), Magewright's
//! Stone ("has an activated ability with {T} in its cost") and Laboratory Drudge.

use crate::r_p164_common::*;
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::events::MoveCause;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn illusionists_bracers_copies_activated_abilities_written_cost_colon_effect() {
    cr!("602.1", "605.1a", "603.2");
    ruling!(
        "Illusionist's Bracers",
        "An activated ability is written in the form \"Cost: Effect.\""
    );
    supported("Illusionist's Bracers");
    // Prodigal Sorcerer's "{T}: This creature deals 1 damage to any target." is copied.
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    attach_new(&mut t, P0, "Illusionist's Bracers", sorcerer);
    t.activate(P0, sorcerer, 0, &[pl(P1)]).unwrap();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Llanowar Elves' "{T}: Add {G}." is a mana ability: not copied.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P0, "Illusionist's Bracers", elves);
    t.activate(P0, elves, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn crackdown_construct_ignores_mana_abilities_but_not_targeted_mana_producers() {
    cr!("605.1a", "602.2", "603.2");
    ruling!(
        "Crackdown Construct",
        "An activated mana ability is one that could add mana as it resolves, doesn't have a target, and isn't a loyalty ability."
    );
    supported("Deathrite Shaman");
    let mut t = TestGame::new(2);
    let construct = t.battlefield(P0, "Crackdown Construct");
    // A mana ability: no trigger.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.activate(P0, elves, 0, &[]).unwrap();
    t.settle();
    assert_eq!(triggers_of(&t, construct), 0);
    // A non-mana ability: +1/+1.
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.activate(P0, sorcerer, 0, &[pl(P1)]).unwrap();
    t.settle();
    assert_eq!(triggers_of(&t, construct), 1);
    t.resolve_all();
    assert_eq!(t.pt(construct), (3, 3));
    // Deathrite Shaman's "{T}: Exile target land card from a graveyard. Add one mana of
    // any color." could add mana but has a target: not a mana ability, so it triggers.
    let shaman = t.battlefield(P0, "Deathrite Shaman");
    let land = t.graveyard(P1, "Forest");
    t.activate(P0, shaman, 0, &[o(land)]).unwrap();
    t.settle();
    assert_eq!(triggers_of(&t, construct), 2);
    assert!(t.stack_len() >= 2, "the Shaman's ability uses the stack");
    t.resolve_all();
    assert_eq!(t.pt(construct), (4, 4));
}

#[test]
fn biomancers_familiar_reduces_only_creatures_on_the_battlefield() {
    cr!("602.2b", "601.2f", "702.29a");
    ruling!(
        "Biomancer's Familiar",
        "Biomancer’s Familiar affects only abilities of creatures you control on the battlefield. The costs of activated abilities of creature cards that work in other zones (such as cycling) won’t be reduced."
    );
    supported("Biomancer's Familiar");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Biomancer's Familiar");
    let ballista = t.battlefield(P0, "Walking Ballista");
    assert_eq!(ability_mana(&mut t, P0, ballista, 0, 0), "{2}");
    // Yoked Plowbeast, a creature card with cycling {2}, in hand.
    let plowbeast = t.hand(P0, "Yoked Plowbeast");
    assert_eq!(ability_mana(&mut t, P0, plowbeast, 0, 0), "{2}");
    // An opponent's creature isn't affected.
    let theirs = t.battlefield(P1, "Walking Ballista");
    assert_eq!(ability_mana(&mut t, P1, theirs, 0, 0), "{4}");
}

#[test]
fn heartstone_reduces_only_generic_mana_including_additional_costs() {
    cr!("601.2f", "602.2b", "107.4h", "118.7a", "118.8");
    ruling!("Heartstone", "Can’t reduce Snow mana costs.");
    ruling!(
        "Heartstone",
        "It will never reduce any colored mana portion of an activation cost."
    );
    ruling!(
        "Heartstone",
        "The cost reduction can be applied to additional costs."
    );
    supported("Heartstone");
    supported("Chilling Shade");
    supported("Frostpeak Yeti");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heartstone");
    // Chilling Shade "{S}: ..." stays {S}; Frostpeak Yeti "{1}{S}: ..." becomes {S}.
    let shade = t.battlefield(P0, "Chilling Shade");
    assert_eq!(ability_mana(&mut t, P0, shade, 0, 0), "{S}");
    let yeti = t.battlefield(P0, "Frostpeak Yeti");
    assert_eq!(ability_mana(&mut t, P0, yeti, 0, 0), "{S}");
    // Shivan Dragon "{R}: ..." stays {R}.
    let dragon = t.battlefield(P0, "Shivan Dragon");
    assert_eq!(ability_mana(&mut t, P0, dragon, 0, 0), "{R}");
    // Heartstone affects every player's creatures.
    let theirs = t.battlefield(P1, "Walking Ballista");
    assert_eq!(ability_mana(&mut t, P1, theirs, 0, 0), "{3}");
    // "Additional costs": no activated ability of a creature lists a mana cost in addition
    // to its activation cost, so the additional mana costs an ability can have are those
    // applied to it from another effect and paid with its activation cost (CR 118.8):
    // Suppression Field's {2}. The reduction comes off the total cost, after the
    // additional costs and cost increases are added (CR 601.2f): Heartstone reduces
    // Prodigal Sorcerer's "{T}: ..." with that {2} to {1}.
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    assert_eq!(ability_mana(&mut t, P0, sorcerer, 0, 0), "");
    t.battlefield(P1, "Suppression Field");
    assert_eq!(ability_mana(&mut t, P0, sorcerer, 0, 0), "{1}");
}

/// The generic mana of the waterbend part of the total cost of `src`'s first activated
/// ability for `p`.
fn waterbend_n(t: &mut TestGame, p: PlayerId, src: ObjectId) -> i32 {
    use mtg_engine::ability::*;
    t.g.recompute();
    let a = t
        .g
        .obj(src)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .cloned()
        .unwrap();
    let AbilityKind::Activated(act) = &a.kind else {
        unreachable!()
    };
    let act = act.clone();
    let c = t.g.ability_total_cost_with(p, src, &a, &act, None, 0, None);
    c.parts
        .iter()
        .filter_map(|part| match part {
            CostPart::Effect(e) => match &**e {
                Effect::KeywordAction {
                    action: KeywordAction::Waterbend,
                    n: Value::Const(k),
                    ..
                } => Some(*k),
                _ => None,
            },
            _ => None,
        })
        .sum()
}

#[test]
fn training_grounds_reduces_waterbend_activation_costs_but_not_spell_costs() {
    cr!("602.2b", "601.2f", "701.67a", "701.67b");
    ruling!(
        "Training Grounds",
        "Training Grounds reduces the generic mana in waterbend costs for activated abilities. It will not reduce waterbend costs that are additional costs for casting spells."
    );
    supported("Ruthless Waterbender");
    supported("Water Whip");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Training Grounds");
    // Ruthless Waterbender: "Waterbend {2}: ..." costs {1} (not less than one mana).
    let wb = t.battlefield(P0, "Ruthless Waterbender");
    assert_eq!(waterbend_n(&mut t, P0, wb), 1);
    assert_eq!(ability_mana(&mut t, P0, wb, 0, 0), "");
    t.lands(P0, "Swamp", 1);
    t.activate(P0, wb, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(wb), (2, 4));
    // Water Whip, {U}{U} plus "waterbend {5}": still seven mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Training Grounds");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 4);
    let whip = t.hand(P0, "Water Whip");
    assert!(t.cast(P0, whip).try_go().is_err());
    t.lands(P0, "Wastes", 1);
    let whip = t.g.current(whip);
    assert!(t.cast(P0, whip).try_go().is_ok());
}

#[test]
fn training_grounds_ignores_costs_paid_on_resolution_and_kicker() {
    cr!("602.2b", "601.2f", "118.8", "118.12");
    ruling!(
        "Training Grounds",
        "Training Grounds won't affect a cost that isn't the cost to activate a creature's activated ability. For example, it won't affect Flameblast Dragon's {X}{R} cost, since that's a cost paid when a triggered ability resolves, and it won't affect a kicker cost, since that's an additional cost to cast a spell."
    );
    supported("Kavu Titan");
    // Flameblast Dragon: X = 2 costs {2}{R}, all three Mountains.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Training Grounds");
    let dragon = t.battlefield(P0, "Flameblast Dragon");
    t.lands(P0, "Mountain", 3);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    attack_with(&mut t, &[(dragon, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    // Kavu Titan kicked ({1}{G} + kicker {2}{G}) still costs five.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Training Grounds");
    t.lands(P0, "Forest", 4);
    let titan = t.hand(P0, "Kavu Titan");
    assert!(t.cast(P0, titan).kicked(true).try_go().is_err());
    t.lands(P0, "Forest", 1);
    let titan = t.g.current(titan);
    assert!(t.cast(P0, titan).kicked(true).try_go().is_ok());
}

#[test]
fn copies_of_a_linked_ability_are_linked_too() {
    cr!("607.2a", "707.10", "603.2");
    ruling!(
        "Ertha Jo, Frontier Mentor",
        "If an ability is linked to a second ability, copies of that ability are also linked to that second ability. If the second ability refers to \"the exiled card,\" it refers to all cards exiled by the ability and the copy."
    );
    supported("Ertha Jo, Frontier Mentor");
    supported("Helvault");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ertha Jo, Frontier Mentor");
    let helvault = t.battlefield(P0, "Helvault");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Wastes", 1);
    // Helvault's "{1}, {T}: Exile target creature you control." targets the Bears; Ertha
    // Jo copies it, and the copy targets the Elves.
    t.activate(P0, helvault, 0, &[o(bears)]).unwrap();
    crate::r_s25_common::change_copy_targets(&mut t, P0, &[Some(o(elves))]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(elves));
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Llanowar Elves"));
    // Helvault goes to the graveyard: both cards return.
    t.g.move_object(helvault, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

/// The creatures offered as targets for Magewright's Stone's ability.
fn stone_candidates(t: &mut TestGame) -> Vec<Entity> {
    let stone = t.battlefield(P0, "Magewright's Stone");
    t.lands(P0, "Wastes", 1);
    let from = t.asked().len();
    let _ = t.activate(P0, stone, 0, &[]);
    crate::r_s02_common::target_candidates(t, P0, from)
        .into_iter()
        .next()
        .unwrap_or_default()
}

#[test]
fn magewrights_stone_looks_for_the_tap_symbol_granted_or_printed() {
    cr!("602.1", "107.5", "115.1");
    ruling!(
        "Magewright's Stone",
        "If an activated ability with {T} in its cost is granted to a creature by an effect (such as Hypervolt Grasp’s effect, for instance), Magewright’s Stone can untap that creature."
    );
    ruling!(
        "Magewright's Stone",
        "This looks for the {T} symbol. The phrase “tap an untapped creature you control” isn’t the same and doesn’t count."
    );
    supported("Magewright's Stone");
    supported("Hypervolt Grasp");
    supported("Bramblesnap");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let granted = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Hypervolt Grasp", granted);
    // Bramblesnap: "Tap an untapped creature you control: This creature gets +1/+1".
    let snap = t.battlefield(P0, "Bramblesnap");
    let c = stone_candidates(&mut t);
    assert!(c.contains(&o(sorcerer)));
    assert!(c.contains(&o(granted)));
    assert!(!c.contains(&o(bears)));
    assert!(!c.contains(&o(snap)));
    // It untaps the creature with the granted ability.
    let mut t = TestGame::new(2);
    let granted = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Hypervolt Grasp", granted);
    t.g.objects[granted.0 as usize].tapped = true;
    let stone = t.battlefield(P0, "Magewright's Stone");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, stone, 0, &[o(granted)]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(granted).tapped);
}

#[test]
fn laboratory_drudge_triggers_every_end_step_and_you_may_respond() {
    cr!("603.2b", "117.1a", "702.34a");
    ruling!(
        "Laboratory Drudge",
        "Laboratory Drudge's ability will trigger at the beginning of each end step whether you've cast a spell from a graveyard or activated an ability of a card in a graveyard that turn or not. You can respond to that ability and do one (or both) of the things to draw a card."
    );
    ruling!(
        "Laboratory Drudge",
        "It doesn't matter where the card is during your end step, as long as it was in a graveyard when you cast it or activated one of its abilities."
    );
    supported("Laboratory Drudge");
    supported("Think Twice");
    supported("Dregscape Zombie");
    // Nothing from a graveyard: it triggers, but no card is drawn.
    let mut t = TestGame::new(2);
    let drudge = t.battlefield(P0, "Laboratory Drudge");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_of(&t, drudge), 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // In response to the trigger, flash back Think Twice (it ends in exile): a card.
    let mut t = TestGame::new(2);
    let drudge = t.battlefield(P0, "Laboratory Drudge");
    let tt = t.graveyard(P0, "Think Twice");
    t.lands(P0, "Island", 3);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_of(&t, drudge), 1);
    t.cast(P0, tt)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_exile("Think Twice"));
    // One from Think Twice, one from the Drudge.
    assert_eq!(t.hand_size(P0), hand + 2);
    // Unearth (an activated ability of a card in a graveyard) counts, wherever the card
    // is now (on the battlefield).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Laboratory Drudge");
    let zombie = t.graveyard(P0, "Dregscape Zombie");
    t.lands(P0, "Swamp", 1);
    t.activate(P0, zombie, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Dregscape Zombie").len(), 1);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Each end step: it triggers in P1's end step too.
    let drudge = t.named_on_battlefield("Laboratory Drudge")[0];
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(triggers_of(&t, drudge), 1);
}
