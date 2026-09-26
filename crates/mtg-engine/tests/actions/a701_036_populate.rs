//! CR 701.36: populate.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn tokens_named(t: &TestGame, name: &str) -> usize {
    t.named_on_battlefield(name).len()
}

#[test]
fn populate_copies_a_creature_token_you_control() {
    cr!("701.36a");
    ruling!(
        "Wake the Reflections",
        "If a spell or ability causes you to create a creature token and then instructs you to populate, you may choose to copy the token you just created"
    );
    supported("Coursers' Accord");
    // "Create a 3/3 green Centaur creature token, then populate."
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Plains", 2);
    let accord = t.hand(P0, "Coursers' Accord");
    t.cast(P0, accord).go();
    t.resolve_all();
    let centaurs = t.named_on_battlefield("Centaur Token");
    assert_eq!(centaurs.len(), 2);
    for c in &centaurs {
        assert_eq!(t.pt(*c), (3, 3));
        assert!(t.obj(*c).is_token());
        assert_eq!(t.obj(*c).controller, P0);
    }
}

#[test]
fn the_populating_player_chooses_which_token_to_copy_and_only_its_copiable_values_are_copied() {
    cr!("701.36a");
    ruling!(
        "Wake the Reflections",
        "The new token doesn't copy whether the original token is tapped or untapped, whether it has any counters on it"
    );
    supported("Wake the Reflections");
    let mut t = TestGame::new(2);
    let spirit = run(
        &mut t,
        P0,
        None,
        Effect::CreateToken {
            spec: TokenSpec {
                name: Default::default(),
                colors: ColorSet::single(Color::White),
                supertypes: vec![],
                card_types: vec![CardType::Creature],
                subtypes: vec!["Spirit".into()],
                power: Some(1),
                toughness: Some(1),
                abilities: vec![],
                scryfall_name: None,
            },
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: true,
            attacking: false,
        },
        &[],
    )
    .var_objects(vars::CREATED)[0];
    t.g.add_counters(Entity::Object(spirit), "+1/+1", 2, None);
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    run(
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
        &[Entity::Object(their_bears)],
    );
    let bear_token = t
        .named_on_battlefield("Grizzly Bears")
        .into_iter()
        .find(|o| t.obj(*o).is_token())
        .unwrap();
    // Choose the Spirit, not the Grizzly Bears token.
    choose(&mut t, P0, &[spirit]);
    t.lands(P0, "Plains", 1);
    let wake = t.hand(P0, "Wake the Reflections");
    t.cast(P0, wake).go();
    t.resolve_all();
    let spirits = t.named_on_battlefield("Spirit Token");
    assert_eq!(spirits.len(), 2);
    let new = spirits.into_iter().find(|s| *s != spirit).unwrap();
    assert!(!t.obj(new).tapped);
    assert_eq!(t.counters(new, "+1/+1"), 0);
    assert_eq!(t.pt(new), (1, 1));
    assert_eq!(tokens_named(&t, "Grizzly Bears"), 2);
    let _ = bear_token;
}

#[test]
fn populating_without_a_creature_token_does_nothing() {
    cr!("701.36b");
    ruling!(
        "Wake the Reflections",
        "If you control no creature tokens when you populate, nothing will happen."
    );
    supported("Wake the Reflections");
    let mut t = TestGame::new(2);
    // A nontoken creature and a noncreature token.
    t.battlefield(P0, "Grizzly Bears");
    run(
        &mut t,
        P0,
        None,
        Effect::KeywordAction {
            action: KeywordAction::Investigate,
            who: PlayerRef::You,
            what: Sel::None,
            n: Value::c(1),
        },
        &[],
    );
    // An opponent's creature token isn't yours to populate.
    let giant = t.battlefield(P1, "Hill Giant");
    run(
        &mut t,
        P1,
        None,
        Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &[Entity::Object(giant)],
    );
    let before = t.g.battlefield.len();
    t.lands(P0, "Plains", 1);
    let wake = t.hand(P0, "Wake the Reflections");
    t.cast(P0, wake).go();
    t.resolve_all();
    assert_eq!(t.g.battlefield.len(), before + 1); // just the Plains
    assert!(t.in_graveyard(P0, "Wake the Reflections"));
}
