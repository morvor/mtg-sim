//! Round-trip follow-up `roundtrip-tail-2` (cards E–L): renderer checks, and in-game
//! tests for compiler misreads the round trip found (each shows the behavior the
//! corrected compilation has and the old one didn't).

use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::oracle::render::compare::{normalize_unit, tokens_match};
use mtg_engine::oracle::render::{render_ability, FaceInfo};
use mtg_engine::testing::*;
use mtg_engine::*;

#[allow(dead_code)]
fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

fn same(a: &str, b: &str) -> bool {
    tokens_match(&normalize_unit(a), &normalize_unit(b))
}

#[test]
fn a_choice_made_on_resolution_doesnt_render_as_a_modal_choice() {
    cr!("700.2a", "608.2d");
    // "Choose one —" with a bulleted list is a modal spell: the mode is chosen as it's
    // cast (CR 700.2a). A choice the effect offers as it resolves (CR 608.2d) is a
    // different ability, so a parser that compiled one as the other must mismatch.
    let options = vec![
        (
            String::new(),
            Effect::Draw {
                who: PlayerRef::You,
                n: Value::Const(1),
            },
        ),
        (
            String::new(),
            Effect::GainLife {
                who: PlayerRef::You,
                n: Value::Const(3),
            },
        ),
    ];
    let a = AbilityDef::new(
        AbilityKind::Spell(SpellAbility {
            body: Body::effect(Effect::ChooseOne {
                who: PlayerRef::You,
                options,
            }),
        }),
        "",
    );
    let r = render_ability(&a, &FaceInfo::default()).expect("renders");
    assert!(
        !same("Choose one — • Draw a card. • You gain 3 life.", &r),
        "{r}"
    );
    // The resolution-time choice is still worded as one ("draw a card or gain 3 life").
    assert!(same("Draw a card or gain 3 life.", &r), "{r}");
}

#[test]
fn a_wish_reveals_the_card_it_puts_into_your_hand() {
    cr!("701.20a", "108.3b");
    // Golden Wish: "You may reveal an artifact or enchantment card you own from outside
    // the game and put it into your hand." The card was put into the hand without being
    // revealed.
    supported("Golden Wish");
    let mut t = TestGame::new(2);
    t.g.logging = true;
    let side = t.g.add_to_sideboard(P0, vec![card("Ornithopter")]);
    let wish = t.hand(P0, "Golden Wish");
    t.lands(P0, "Plains", 5);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.cast(P0, wish).go();
    t.resolve();
    assert!(t.in_hand(P0, "Ornithopter"));
    assert!(
        t.g.log.iter().any(|e| e.text.contains("reveals") && e.text.contains("Ornithopter")),
        "{:?}",
        t.g.log.iter().map(|e| &e.text).collect::<Vec<_>>()
    );
}

fn count_subtype(t: &TestGame, p: PlayerId, sub: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.chars.has_subtype(sub)
        })
        .count()
}

fn treasures(t: &TestGame, p: PlayerId) -> usize {
    count_subtype(t, p, "Treasure")
}

#[test]
fn the_attacking_player_creates_the_treasure_once_per_attack() {
    cr!("508.3e", "603.2c");
    // Jolene, the Plunder Queen: "Whenever a player attacks one or more of your
    // opponents, that attacking player creates a Treasure token." "That attacking player"
    // was the attacked player, and attacking two of your opponents made two Treasures.
    supported("Jolene, the Plunder Queen");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Jolene, the Plunder Queen");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.attack(
        &[(a, Entity::Player(P1)), (b, Entity::Player(P2))],
        &[],
    );
    t.resolve_all();
    // Jolene doubles nothing here: "one or more Treasure tokens ... plus an additional
    // Treasure token" makes the one Treasure two.
    assert_eq!(treasures(&t, P1), 0);
    assert_eq!(treasures(&t, P2), 0);
    assert_eq!(treasures(&t, P0), 2);
}

#[test]
fn that_attacking_player_is_the_player_who_attacked() {
    cr!("508.3e", "303.4b", "303.4e");
    // Curse of Shallow Graves: "Whenever a player attacks enchanted player with one or
    // more creatures, that attacking player may create a tapped 2/2 black Zombie creature
    // token." A companion check for the Jolene fix above (this wording was already
    // compiled correctly): the attacker gets the token, not the attacked player nor the
    // Curse's controller, and once however many creatures attack.
    supported("Curse of Shallow Graves");
    let mut t = TestGame::new(3);
    let curse = t.battlefield(P1, "Curse of Shallow Graves");
    assert!(t.g.attach(curse, Entity::Player(P2)));
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P2)), (b, Entity::Player(P2))], &[]);
    t.resolve_all();
    assert_eq!(count_subtype(&t, P1, "Zombie"), 0);
    assert_eq!(count_subtype(&t, P2, "Zombie"), 0);
    assert_eq!(count_subtype(&t, P0, "Zombie"), 1);
}

#[test]
fn a_batched_attack_trigger_refers_to_all_the_attacking_creatures() {
    cr!("603.2c", "508.3e");
    // "Whenever you attack one or more of your opponents, put a +1/+1 counter on each of
    // those creatures": one trigger for the declaration, and "those creatures" are the
    // creatures attacking any of them. A batch of events that are each about several
    // objects (the creatures attacking one player) kept none of them.
    use mtg_engine::object::{Characteristics, Zone};
    use mtg_engine::types::CardType;
    use std::sync::Arc;
    let trigger = TriggerCond::Batched {
        trigger: Box::new(TriggerCond::PlayerAttacksPlayer {
            attacker: PlayerRel::You,
            defender: PlayerRel::Opponent,
        }),
        per: BatchPer::Batch,
    };
    let body = Body::effect(Effect::AddCounters {
        what: Sel::TriggerObjects,
        kind: smol_str::SmolStr::new("+1/+1"),
        n: Value::c(1),
    });
    let mut chars = Characteristics {
        name: smol_str::SmolStr::new("Batched Attack Watcher"),
        rules_text: Arc::from(""),
        ..Default::default()
    };
    chars.card_types.insert(CardType::Enchantment);
    chars.abilities.push(AbilityDef::new(
        AbilityKind::Triggered(TriggeredAbility::new(trigger, body)),
        "triggered",
    ));
    let mut t = TestGame::new(3);
    t.custom(P0, mtg_engine::card::CardDef::custom(chars), Zone::Battlefield);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.attack(
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P2)),
        ],
        &[],
    );
    t.resolve_all();
    for x in [a, b, c] {
        assert_eq!(t.counters(t.g.current(x), "+1/+1"), 1);
    }
}
