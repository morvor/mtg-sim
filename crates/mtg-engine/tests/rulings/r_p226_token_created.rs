//! Rulings batch P226 — "Whenever you create a [Blood] token" (CR 111.1, 111.2: tokens you
//! create, which you own): Voldaren
//! Bloodcaster (with its intervening "if" clause, CR 603.4) and Rosie Cotton of South Lane.

use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s17_common::name_of;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const BLOODCASTER: &str = "Voldaren Bloodcaster // Bloodbat Summoner";

/// `p` creates a token (as an effect would), then triggers are put on the stack.
fn make(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = create_token(t, p, name);
    t.g.flush_events();
    t.settle();
    id
}

fn blood(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    with_subtype(t, p, "Blood")
}

#[test]
fn voldaren_bloodcaster_checks_five_blood_tokens_as_it_triggers_and_resolves() {
    cr!("603.4", "111.2", "701.27a");
    ruling!(
        "Voldaren Bloodcaster // Bloodbat Summoner",
        "Voldaren Bloodcaster’s last ability checks how many Blood tokens you have both as it triggers and as it resolves. If you don’t control at least five Blood tokens at both of those times, it won’t transform."
    );
    supported(BLOODCASTER);
    // "Whenever you create a Blood token, if you control five or more Blood tokens,
    // transform this creature." The fourth Blood token: no trigger.
    let mut t = TestGame::new(2);
    let caster = t.battlefield(P0, BLOODCASTER);
    for _ in 0..4 {
        make(&mut t, P0, "Blood");
    }
    assert_eq!(t.stack_len(), 0);
    // The fifth: it triggers; one is sacrificed before it resolves: no transformation.
    make(&mut t, P0, "Blood");
    assert_eq!(t.stack_len(), 1);
    let one = blood(&t, P0)[0];
    t.g.sacrifice(one, P0);
    t.resolve_all();
    assert_eq!(name_of(&t, caster), "Voldaren Bloodcaster");
    // Five again, still five as it resolves: it transforms.
    make(&mut t, P0, "Blood");
    t.resolve_all();
    assert_eq!(name_of(&t, caster), "Bloodbat Summoner");
    // Blood tokens another player creates don't trigger it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, BLOODCASTER);
    for _ in 0..5 {
        make(&mut t, P1, "Blood");
    }
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn voldaren_bloodcaster_makes_blood_when_it_or_another_nontoken_creature_dies() {
    cr!("603.10a", "702.9b");
    // Flying. "Whenever this creature or another nontoken creature you control dies,
    // create a Blood token."
    let mut t = TestGame::new(2);
    let caster = t.battlefield(P0, BLOODCASTER);
    assert!(t.obj_now(caster).has_keyword(KeywordKind::Flying));
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(blood(&t, P0).len(), 1);
    // A token, or an opponent's creature: nothing.
    let soldier = create_token(&mut t, P0, "Soldier");
    destroy(&mut t, soldier);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, theirs);
    t.resolve_all();
    assert_eq!(blood(&t, P0).len(), 1);
    destroy(&mut t, caster);
    t.resolve_all();
    assert_eq!(blood(&t, P0).len(), 2);
}

#[test]
fn bloodbat_summoner_animates_a_blood_token_at_the_beginning_of_combat() {
    cr!("611.2c", "205.1b");
    // Bloodbat Summoner (3/3 flying): "At the beginning of combat on your turn, up to one
    // target Blood token you control becomes a 2/2 black Bat creature with flying and
    // haste in addition to its other types."
    let mut t = TestGame::new(2);
    let summoner = crate::r_s17_common::enter_transformed(&mut t, P0, BLOODCASTER);
    assert_eq!(t.pt(summoner), (3, 3));
    assert!(t.obj_now(summoner).has_keyword(KeywordKind::Flying));
    let token = make(&mut t, P0, "Blood");
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(token)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let o = t.obj_now(token);
    assert!(o.is(CardType::Creature) && o.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Bat") && o.chars.has_subtype("Blood"));
    assert!(o.has_keyword(KeywordKind::Flying) && o.has_keyword(KeywordKind::Haste));
    assert_eq!(t.pt(token), (2, 2));
    assert!(o.chars.colors.contains(Color::Black));
}

#[test]
fn rosie_cotton_grows_another_creature_whenever_you_create_a_token() {
    cr!("603.2", "603.6a", "111.2");
    supported("Rosie Cotton of South Lane");
    // "When Rosie Cotton enters, create a Food token." / "Whenever you create a token,
    // put a +1/+1 counter on target creature you control other than Rosie Cotton."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let rosie = t.enter(P0, "Rosie Cotton of South Lane");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Food").len(), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // Rosie herself can't be the target.
    let from = t.asked().len();
    make(&mut t, P0, "Treasure");
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.counters(rosie, counters::PLUS1), 0);
    let candidates = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(candidates
        .iter()
        .all(|c| !c.contains(&Entity::Object(rosie))));
    // An opponent's token: nothing.
    make(&mut t, P1, "Treasure");
    assert_eq!(t.stack_len(), 0);
}
