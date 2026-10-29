//! CR 408: the command zone.

use crate::r100_common::*;
use crate::r703_common::{oracle_card, run_effect, supported};
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::eval::Ctx;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn emblems(t: &TestGame) -> Vec<ObjectId> {
    t.g.command
        .iter()
        .copied()
        .filter(|id| t.obj(*id).kind == ObjKind::Emblem)
        .collect()
}

#[test]
fn command_zone_objects_have_overarching_effects_but_arent_permanents_and_cant_be_destroyed() {
    cr!("408.1");
    let mut t = TestGame::new(2);
    // An emblem with "Creatures you control get +1/+1."
    let anthem = abilities(&card("Glorious Anthem"));
    run_effect(
        &mut t,
        P0,
        None,
        Effect::CreateEmblem {
            who: PlayerRef::You,
            abilities: anthem,
        },
        &[],
    );
    let emblem = emblems(&t)[0];
    let plane = t.command(P0, "Goldmeadow");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    // Its effect applies to the game.
    assert_eq!(t.pt(bears), (3, 3));
    // Neither it nor a plane is a permanent.
    let ctx = Ctx::new(None, P0);
    for id in [emblem, plane] {
        assert!(!t.g.matches(id, &Filter::Permanent, &ctx));
    }
    // They can't be destroyed: "destroy all permanents" leaves them, and destroying them
    // directly does nothing.
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Destroy {
            what: Sel::All(Filter::Permanent),
            no_regen: false,
        },
        &[],
    );
    assert!(!t.on_battlefield(bears));
    for id in [emblem, plane] {
        t.g.destroy(id, None);
        assert!(t.is_live(id));
        assert_eq!(t.zone(id), Zone::Command);
    }
}

fn abilities(def: &mtg_engine::CardDef) -> Vec<Ability> {
    def.faces[0].chars.abilities.clone()
}

#[test]
fn emblems_are_created_in_the_command_zone() {
    cr!("408.2");
    supported("Ob Nixilis Reignited");
    let mut t = TestGame::new(2);
    let ob = t.battlefield(P0, "Ob Nixilis Reignited");
    t.g.objects[ob.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 8);
    t.g.recompute();
    let index = t
        .obj(ob)
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Activated(act) => Some(act),
            _ => None,
        })
        .position(|act| format!("{:?}", act.body.effect).contains("CreateEmblem"))
        .unwrap();
    t.activate(P0, ob, index, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    let e = emblems(&t);
    assert_eq!(e.len(), 1);
    assert_eq!(t.zone(e[0]), Zone::Command);
}

#[test]
fn variants_start_their_special_cards_in_the_command_zone() {
    cr!("408.3");
    // Commander: the commander.
    let mut deck = fillers(30);
    deck.push(card("Isamaru, Hound of Konda"));
    let mut t = pregame(
        GameConfig {
            variant: Variant::Commander,
            skip_mulligans: true,
            ..Default::default()
        },
        vec![deck, fillers(30)],
    );
    assert!(t.g.designate_commander(P0, "Isamaru, Hound of Konda"));
    t.g.start();
    assert_eq!(
        t.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda").len(),
        1
    );
    // Planechase: the planar deck (and the face-up starting plane).
    let deck: Vec<Arc<mtg_engine::CardDef>> = fillers(20)
        .into_iter()
        .chain(["Goldmeadow", "Strixhaven"].iter().map(|n| card(n)))
        .collect();
    let mut t = pregame(
        GameConfig {
            variant: Variant::Planechase,
            skip_mulligans: true,
            ..Default::default()
        },
        vec![deck, fillers(20)],
    );
    t.g.start();
    let planes: Vec<ObjectId> = t
        .g
        .command
        .iter()
        .copied()
        .filter(|id| {
            t.obj(*id)
                .card
                .as_ref()
                .is_some_and(|c| c.front().chars.is(CardType::Plane))
        })
        .collect();
    assert_eq!(planes.len(), 2);
    assert!(t.g.player(P0).library.iter().all(|id| !planes.contains(id)));
    // Vanguard: the vanguard card.
    let mut deck = fillers(40);
    deck.push(card("Titania"));
    let mut t = pregame(
        GameConfig {
            variant: Variant::Vanguard,
            skip_mulligans: true,
            ..Default::default()
        },
        vec![deck, fillers(40)],
    );
    t.g.start();
    assert_eq!(t.g.find_in_zone(Zone::Command, "Titania").len(), 1);
    // Archenemy: the scheme deck.
    let mut deck = fillers(40);
    deck.push(card("What's Yours Is Now Mine"));
    let mut t = pregame(
        GameConfig {
            variant: Variant::Archenemy,
            teams: Some(vec![0, 1, 0]),
            skip_mulligans: true,
            ..Default::default()
        },
        vec![fillers(40), deck, fillers(40)],
    );
    t.g.start();
    let schemes: Vec<ObjectId> = t
        .g
        .command
        .iter()
        .copied()
        .filter(|id| t.obj(*id).owner == P1)
        .collect();
    assert_eq!(schemes.len(), 1);
    // Conspiracy Draft: a conspiracy put into the command zone before the game.
    let conspiracy = oracle_card("Idle Conspiracy", "Conspiracy", "", None, "");
    let mut deck = fillers(40);
    deck.push(Arc::new(conspiracy));
    let mut t = pregame(
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
        vec![deck, fillers(40)],
    );
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(
            t.g.player(P0)
                .sideboard
                .iter()
                .map(|id| Entity::Object(*id))
                .collect(),
        ),
    );
    t.g.start();
    assert_eq!(t.g.find_in_zone(Zone::Command, "Idle Conspiracy").len(), 1);
}
