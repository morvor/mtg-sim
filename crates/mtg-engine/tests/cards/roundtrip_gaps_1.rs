//! Round-trip follow-up `roundtrip-gaps-1` (renderer gaps in filters, conditions,
//! replacements and triggers): in-game tests for the compiler misreads the round trip
//! found (each shows the behavior the corrected compilation has and the old one didn't),
//! and renderer checks for the newly worded nodes.

use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::oracle::render::compare::check_card;
use mtg_engine::testing::*;
use mtg_engine::events::Event;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

fn assert_round_trips(name: &str) {
    supported(name);
    let r = check_card(&card(name));
    assert!(
        r.pass,
        "{name} doesn't round-trip:\n  oracle: {:?}\n  rendered: {:?}\n  gaps: {:?}",
        r.unmatched_oracle, r.unmatched_rendered, r.gaps
    );
}

#[test]
fn target_opponent_blights_has_one_target() {
    cr!("115.1c", "602.2b");
    // Champion of the Weird: "Pay 1 life, Blight 2: Target opponent blights 2." Trying the
    // subject with each player keyword action's verb left a "target opponent" behind for
    // each verb that didn't fit, so the ability had six targets.
    supported("Champion of the Weird");
    let c = card("Champion of the Weird");
    let targets: Vec<usize> = c.faces[0]
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Activated(x) if !x.is_mana_ability => Some(x.body.targets.len()),
            _ => None,
        })
        .collect();
    assert_eq!(targets, [1]);
    // In a game: the ability on the stack has one target, the opponent.
    let mut t = TestGame::new(2);
    let champion = t.battlefield(P0, "Champion of the Weird");
    t.battlefield(P0, "Grizzly Bears");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ab = t
        .activate(P0, champion, 0, &[Entity::Player(P1)])
        .expect("activate")
        .expect("on the stack");
    let si = t.obj_now(ab).stack.clone().expect("stack info");
    let slots = &si.chosen[0].targets;
    assert_eq!(slots.len(), 1, "{slots:?}");
    // The opponent blights 2: two -1/-1 counters on their Grizzly Bears.
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.resolve();
    assert!(!t.on_battlefield(bears), "the opponent's 2/2 got two -1/-1 counters");
}

/// Sauron attacks with a creature card in your graveyard; the token copy is created.
fn sauron_attacks(ring_bearer: bool) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let sauron = t.battlefield(P0, "Sauron, the Necromancer");
    let card = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.attack(&[(sauron, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1, "the token copy");
    if ring_bearer {
        t.g.player_mut(P0).ring_bearer = Some(sauron);
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    (t, sauron)
}

#[test]
fn sauron_exiles_the_token_at_the_end_step_unless_it_is_the_ring_bearer() {
    cr!("603.7a", "608.2c");
    // Sauron, the Necromancer: "Create a tapped and attacking token that's a copy of that
    // card, except ... At the beginning of the next end step, exile that token unless ~
    // is your Ring-bearer." "That token" was read as the exiled card (so the token
    // stayed), and the Ring-bearer check was made when the token was created rather than
    // at the end step.
    supported("Sauron, the Necromancer");
    let (t, _) = sauron_attacks(false);
    assert!(
        t.named_on_battlefield("Hill Giant").is_empty(),
        "the token is exiled at the end step"
    );
    // Sauron became the Ring-bearer after the token was created: checked at the end step.
    let (t, _) = sauron_attacks(true);
    assert_eq!(
        t.named_on_battlefield("Hill Giant").len(),
        1,
        "the token stays while ~ is the Ring-bearer"
    );
}

#[test]
fn newly_worded_nodes_round_trip() {
    for name in [
        // Filters: "can be attached to it", loyalty abilities, crafted cards, ...
        "Nahiri, Heir of the Ancients",
        "Keral Keep Disciples",
        "Ajani Unrelenting",
        "Saheeli's Lattice // Mastercraft Raptor",
        "Urza's Saga",
        "Overgrown Pest",
        "Catch // Release",
        "Witch-king of Angmar",
        "Kitesail Skirmisher",
        // Conditions.
        "Spara's Adjudicators",
        "Karakyk Guardian",
        "Seraphic Greatsword",
        "Zariel, Archduke of Avernus",
        "Adrestia",
        // Replacements.
        "Loxodon Smiter",
        "Forethought Amulet",
        "Containment Priest",
        "Uphill Battle",
        "Callous Giant",
        "Fasting",
        "Scion of Halaster",
        "Nephalia Academy",
        "Equal Treatment",
        // Triggers.
        "Reidane, God of the Worthy // Valkmira, Protector's Shield",
        "Leyline of Combustion",
        "Mistbind Clique",
        "Aboleth Spawn",
        // Targets.
        "Disarm",
        "Keeper of the Light",
        "Oath of Lieges",
        "Raft Security Officer",
        "Belt of Giant Strength",
        "Plasm Capture",
        // The triggering object.
        "Garruk, Curse Breaker",
        "Reflect Damage",
        "Locket of Yesterdays",
        // Parser fixes above.
        "Champion of the Weird",
        "Sauron, the Necromancer",
    ] {
        assert_round_trips(name);
    }
}

#[test]
fn nephalia_academy_reveals_the_discarded_card_and_puts_it_on_top() {
    cr!("701.20a", "701.20b", "614.1a");
    ruling!("Nephalia Academy", "the card was still discarded");
    // Nephalia Academy: "If a spell or ability an opponent controls causes you to discard
    // a card, you may reveal that card and put it on top of your library instead of
    // putting it anywhere else." The reveal was dropped; the card is revealed (where it
    // is, in your hand) and then put on top, and it's still discarded.
    supported("Nephalia Academy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nephalia Academy");
    t.battlefield(P1, "Megrim");
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P1, "Swamp", 3);
    let rot = t.hand(P1, "Mind Rot");
    t.g.turn.active = P1;
    t.cast(P1, rot).target(P0).go();
    t.resolve_all();
    let top = *t.g.player(P0).library.last().expect("a card on top");
    assert_eq!(t.g.obj(top).name(), "Grizzly Bears", "{}", t.dump_log());
    assert!(
        t.g.turn_events.iter().any(|e| matches!(e, Event::Custom { name, obj: Some(o), .. }
            if name == mtg_engine::reveal::REVEALED && *o == bears)),
        "the card was revealed: {}",
        t.dump_log()
    );
    // Discarded all the same: Megrim's trigger dealt 2 damage.
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn equal_treatment_makes_one_damage_two() {
    cr!("614.1a");
    // Equal Treatment: "If any source would deal 1 or more damage to a permanent or player
    // this turn, it deals 2 damage to that permanent or player instead." Compiled as
    // subtracting (the amount minus 2), which was clamped at 0: 1 damage stayed 1.
    supported("Equal Treatment");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let et = t.hand(P0, "Equal Treatment");
    let src = t.battlefield(P0, "Grizzly Bears");
    t.cast(P0, et).go();
    t.resolve_all();
    t.g.deal_damage(src, Entity::Player(P1), 1, false);
    t.settle();
    assert_eq!(t.life(P1), 18, "1 damage became 2: {}", t.dump_log());
    t.g.deal_damage(src, Entity::Player(P1), 5, false);
    t.settle();
    assert_eq!(t.life(P1), 16, "5 damage became 2");
}
