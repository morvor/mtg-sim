//! CR 702.181 Mobilize (`src/kw/mobilize.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn mobilize_cards_compile() {
    assert_supported(&[
        "Dalkovan Packbeasts",
        "Shock Brigade",
        "Avenger of the Fallen",
        "Voice of Victory",
        "Bone-Cairn Butcher",
        "Stadium Headliner",
    ]);
}

fn warriors(t: &TestGame) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Warrior"))
        .map(|o| o.id)
        .collect()
}

#[test]
fn attacking_creates_tapped_attacking_warriors_sacrificed_at_end_step() {
    cr!("702.181a");
    // "Vigilance. Mobilize 3" on a 0/4.
    let mut t = TestGame::new(2);
    let ox = t.battlefield(P0, "Dalkovan Packbeasts");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(ox, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    let ws = warriors(&t);
    assert_eq!(ws.len(), 3);
    for w in &ws {
        let o = t.obj_now(*w);
        assert!(o.tapped);
        assert_eq!((o.power(), o.toughness()), (1, 1));
        assert!(o.chars.colors.contains(Color::Red));
        assert!(o.is_creature());
        assert!(t.g.is_attacking(*w));
    }
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    // They're sacrificed at the beginning of the next end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(warriors(&t).is_empty());
    assert!(t.on_battlefield(ox));
}

#[test]
fn the_tokens_were_never_declared_as_attackers() {
    cr!("702.181a", "508.4");
    ruling!(
        "Dalkovan Packbeasts",
        "Abilities that trigger whenever a creature attacks won’t trigger when the tokens enter attacking."
    );
    let rally = custom_card(
        "Rally Horn",
        "{2}",
        "Artifact",
        None,
        "Whenever a creature you control attacks, you gain 1 life.",
    );
    let mut t = TestGame::new(2);
    put(&mut t, P0, rally, Zone::Battlefield);
    let brigade = t.battlefield(P0, "Shock Brigade");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(brigade, Entity::Player(P1))], &[]);
    // Only the Brigade attacked; its Warrior token entered attacking.
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn each_token_attacks_what_its_controller_chooses() {
    cr!("702.181a", "508.4");
    ruling!(
        "Dalkovan Packbeasts",
        "You choose the player, planeswalker, or battle each Warrior token is attacking."
    );
    let mut t = TestGame::new(3);
    let ox = t.battlefield(P0, "Voice of Victory");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(ox, Entity::Player(P1))]),
    );
    // The two tokens: one attacks P2, the other P1.
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Player(P2)]),
    );
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Player(P1)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    let ws = warriors(&t);
    assert_eq!(ws.len(), 2);
    let combat = t.g.combat.as_ref().unwrap();
    let mut targets: Vec<Option<Entity>> = ws.iter().map(|w| combat.attack_target(*w)).collect();
    targets.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(
        targets,
        vec![Some(Entity::Player(P1)), Some(Entity::Player(P2))]
    );
    t.advance_to(P0, Step::EndOfCombat);
    // Voice of Victory (1/3) and one Warrior hit P1; the other Warrior hit P2.
    assert_eq!((t.life(P1), t.life(P2)), (18, 19));
}

#[test]
fn mobilize_x_is_determined_as_the_ability_resolves() {
    cr!("702.181a");
    ruling!(
        "Avenger of the Fallen",
        "The value of X is calculated only once, as Avenger of the Fallen’s mobilize ability resolves."
    );
    let mut t = TestGame::new(2);
    let avenger = t.battlefield(P0, "Avenger of the Fallen");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(avenger, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A third creature card is put into the graveyard before it resolves.
    t.graveyard(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(warriors(&t).len(), 3);
    // Later changes don't matter.
    t.graveyard(P0, "Llanowar Elves");
    assert_eq!(warriors(&t).len(), 3);
}

#[test]
fn only_tokens_still_controlled_are_sacrificed() {
    cr!("702.181a", "701.21a");
    let mut t = TestGame::new(2);
    let brigade = t.battlefield(P0, "Shock Brigade");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(brigade, Entity::Player(P1))], &[]);
    let ws = warriors(&t);
    assert_eq!(ws.len(), 1);
    // The opponent gains control of the token before the end step.
    run(
        &mut t,
        P1,
        None,
        mtg_engine::ability::Effect::GainControl {
            what: mtg_engine::ability::Sel::Target(0),
            who: mtg_engine::ability::PlayerRef::You,
            duration: mtg_engine::ability::Duration::Permanent,
        },
        &[Entity::Object(ws[0])],
    );
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(ws[0]));
    assert_eq!(t.obj_now(ws[0]).controller, P1);
}
