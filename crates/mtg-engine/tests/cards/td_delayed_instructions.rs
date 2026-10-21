//! Delayed triggered abilities created by resolving abilities, worded as a trailing "when"
//! ("Return ~ to the battlefield ... when that token dies.") or at a step of a later turn
//! ("at the beginning of the end step on your next turn"): Tatsumasa, the Dragon's Fang,
//! Portcullis, Kav Landseeker, Meandering Towershell.

use crate::combat2_common::*;
use crate::td_next_turn_durations::next_combat;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
}

#[test]
fn tatsumasa_returns_when_that_token_dies() {
    cr!("603.7a", "603.7c");
    supported("Tatsumasa, the Dragon's Fang");
    let mut t = TestGame::new(2);
    let tatsu = t.battlefield(P0, "Tatsumasa, the Dragon's Fang");
    t.lands(P0, "Plains", 6);
    t.activate(P0, tatsu, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(tatsu)), Zone::Exile);
    let token = *t
        .g
        .battlefield
        .iter()
        .find(|o| t.g.obj(**o).is_token())
        .expect("a Dragon Spirit");
    assert_eq!(t.pt(token), (5, 5));
    // Another creature dying doesn't return it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(named(&t, "Tatsumasa, the Dragon's Fang").is_empty());
    t.g.destroy(token, None);
    t.resolve_all();
    let back = named(&t, "Tatsumasa, the Dragon's Fang");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).controller, P0);
}

#[test]
fn portcullis_holds_the_creature_until_it_leaves() {
    cr!("603.7a");
    ruling!(
        "Portcullis",
        "It will trigger when Portcullis leaves the battlefield even if Portcullis has somehow lost its abilities at that time."
    );
    supported("Portcullis");
    let mut t = TestGame::new(2);
    let gate = t.battlefield(P0, "Portcullis");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    let elves = t.enter(P1, "Llanowar Elves");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(elves)), Zone::Exile);
    // Portcullis loses its abilities: the delayed ability was set up already.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    t.g.exec(
        &mtg_engine::ability::Effect::Modify {
            what: mtg_engine::ability::Sel::All(mtg_engine::ability::Filter::Objects(vec![gate])),
            mods: vec![mtg_engine::ability::Modification::RemoveAllAbilities],
            duration: mtg_engine::ability::Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert!(t.obj_now(gate).chars.abilities.is_empty());
    t.g.destroy(gate, None);
    t.resolve_all();
    let back = named(&t, "Llanowar Elves");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).controller, P1);
}

#[test]
fn kav_landseeker_sacrifices_the_token_at_the_end_step_of_your_next_turn() {
    cr!("603.7a", "603.7b");
    ruling!(
        "Kav Landseeker",
        "triggers at the beginning of the end step on your next turn"
    );
    supported("Kav Landseeker");
    let mut t = TestGame::new(2);
    t.enter(P0, "Kav Landseeker");
    t.resolve_all();
    let landers = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|o| t.g.obj(**o).is_token())
            .count()
    };
    assert_eq!(landers(&t), 1);
    // Not this turn's end step, nor the opponent's.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(landers(&t), 1);
    // The end step of P0's next turn.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(landers(&t), 0);
}

#[test]
fn meandering_towershell_returns_attacking_on_your_next_turn_only() {
    cr!("603.7a", "508.4");
    ruling!(
        "Meandering Towershell",
        "Abilities that trigger when a creature attacks, including its own triggered ability, won’t trigger."
    );
    supported("Meandering Towershell");
    let mut t = TestGame::new(2);
    let shell = t.battlefield(P0, "Meandering Towershell");
    declare(&mut t, P0, &[(shell, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(shell)), Zone::Exile);
    t.advance_to(P0, Step::End);
    // P1's turn: it doesn't come back.
    next_combat(&mut t, P1);
    t.advance_to(P1, Step::End);
    assert!(named(&t, "Meandering Towershell").is_empty());
    // P0's next turn: it returns tapped and attacking in the declare attackers step,
    // without attacking again (its ability doesn't trigger).
    next_combat(&mut t, P0);
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let back = named(&t, "Meandering Towershell");
    assert_eq!(back.len(), 1);
    assert!(t.obj_now(back[0]).tapped);
    assert!(t.g.combat.as_ref().is_some_and(|c| c.attacker(back[0]).is_some()));
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn giant_slug_chooses_its_landwalk_at_your_next_upkeep_for_that_turn() {
    cr!("603.7a", "702.14c");
    ruling!(
        "Giant Slug",
        "When that ability resolves, you choose a basic land type."
    );
    supported("Giant Slug");
    let mut t = TestGame::new(2);
    let slug = t.battlefield(P0, "Giant Slug");
    t.battlefield(P1, "Island");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Swamp", 5);
    t.activate(P0, slug, 0, &[]).unwrap();
    t.resolve_all();
    // Nothing yet this turn.
    let kw = |t: &TestGame| {
        t.obj_now(slug)
            .has_keyword(keywords::KeywordKind::Landwalk)
    };
    assert!(!kw(&t));
    t.advance_to(P1, Step::Upkeep);
    // Your next upkeep: choose Island (index 1).
    t.answer(P0, DecisionKind::Option, mtg_engine::decision::Answer::Index(1));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(kw(&t));
    to_combat(&mut t, P0);
    declare(&mut t, P0, &[(slug, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, &[(wall, slug)]));
    // Until the end of that turn only.
    t.advance_to(P1, Step::Upkeep);
    assert!(!kw(&t));
}

#[test]
fn disorder_in_the_court_returns_the_exiled_cards_tapped_at_the_next_end_step() {
    cr!("603.7a", "400.7", "701.16a");
    supported("Disorder in the Court");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "Disorder in the Court");
    t.cast(P0, spell)
        .x(2)
        .targets(&[Entity::Object(mine), Entity::Object(theirs)])
        .go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(mine)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(theirs)), Zone::Exile);
    let clues = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.g.obj(**o).is_token() && t.g.obj(**o).chars.has_subtype("Clue"))
        .count();
    assert_eq!(clues, 2);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    let elves = t.named_on_battlefield("Llanowar Elves");
    assert_eq!((bears.len(), elves.len()), (1, 1));
    assert_eq!(t.obj_now(bears[0]).controller, P0);
    assert_eq!(t.obj_now(elves[0]).controller, P1);
    assert!(t.obj_now(bears[0]).tapped && t.obj_now(elves[0]).tapped);
}
