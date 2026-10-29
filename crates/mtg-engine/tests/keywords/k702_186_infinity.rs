//! CR 702.186 ∞ (Infinity) ("∞ — [ability]" in `oracle/patterns/a701_action_triggers.rs`;
//! harness is CR 701.64, `kwa/harness.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn harness(t: &mut TestGame, id: ObjectId) {
    run(
        t,
        P0,
        Some(id),
        Effect::KeywordAction {
            action: KeywordAction::Harness,
            who: PlayerRef::You,
            what: Sel::This,
            n: Value::c(1),
        },
        &[],
    );
}

fn has_upkeep_trigger(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Triggered(tr)
            if matches!(tr.trigger, TriggerCond::BeginningOf { step: TriggerStep::Upkeep, .. }))
    })
}

#[test]
fn infinity_and_its_ability_text_are_one_static_ability() {
    cr!("702.186a");
    // The Mind Stone: "∞ — At the beginning of your end step, exile up to one other target
    // nonland permanent you control, then return that card to the battlefield under its
    // owner's control."
    assert_supported(&["The Mind Stone"]);
    let stone = mtg_engine::card::card("The Mind Stone");
    let inf: Vec<&AbilityDef> = stone
        .front()
        .chars
        .abilities
        .iter()
        .map(|a| &**a)
        .filter(|a| a.text.starts_with("∞"))
        .collect();
    assert_eq!(inf.len(), 1);
    assert!(matches!(inf[0].kind, AbilityKind::Static(_)));
}

#[test]
fn it_has_the_ability_only_while_harnessed() {
    cr!("702.186b");
    ruling!(
        "The Soul Stone",
        "Until it is harnessed, The Soul Stone doesn't have the ability listed after the infinity symbol. It also doesn't have that ability in zones other than the battlefield."
    );
    // The Soul Stone: "∞ — At the beginning of your upkeep, return target creature card
    // from your graveyard to the battlefield."
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "The Soul Stone");
    assert!(!has_upkeep_trigger(&t, in_hand));
    let stone = t.battlefield(P0, "The Soul Stone");
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(!has_upkeep_trigger(&t, stone));
    // Not harnessed: nothing at the upkeep.
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    harness(&mut t, stone);
    assert!(has_upkeep_trigger(&t, stone));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn a_copy_isnt_harnessed() {
    cr!("702.186b");
    ruling!(
        "The Soul Stone",
        "Being harnessed isn't copiable. If something else becomes a copy of The Soul Stone, it must be harnessed separately."
    );
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "The Soul Stone");
    harness(&mut t, stone);
    let copy = run(
        &mut t,
        P0,
        None,
        Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &[Entity::Object(stone)],
    )
    .var_objects(vars::CREATED);
    assert!(has_upkeep_trigger(&t, stone));
    assert!(!has_upkeep_trigger(&t, copy[0]));
    harness(&mut t, copy[0]);
    assert!(has_upkeep_trigger(&t, copy[0]));
}
