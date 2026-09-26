//! CR 701.47: amass.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::kwa::amass::AMASSED_EVENT;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn armies(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Army"))
        .map(|o| o.id)
        .collect()
}

fn cast(t: &mut TestGame, name: &str, lands: &[(&str, usize)]) {
    for (l, n) in lands {
        t.lands(P0, l, *n);
    }
    let c = t.hand(P0, name);
    t.cast(P0, c).go();
    t.resolve_all();
}

/// Prevents counters from being put on creatures this turn.
fn no_counters(t: &mut TestGame) {
    run(
        t,
        P1,
        None,
        Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::PutCounters {
                    on_objects: Some(Filter::creature()),
                    on_players: None,
                    kind: None,
                },
                action: ReplacementAction::Prevent,
                self_replacement: false,
                optional: false,
            },
            duration: Duration::EndOfTurn,
            uses: None,
        },
        &[],
    );
}

#[test]
fn amass_creates_an_army_if_needed_and_puts_counters_on_an_army() {
    cr!("701.47a");
    ruling!(
        "Relentless Advance",
        "To amass Zombies N, if you don't control an Army creature, create a 0/0 black Zombie Army creature token."
    );
    supported("Relentless Advance");
    let mut t = TestGame::new(2);
    // "Amass Zombies 3."
    cast(&mut t, "Relentless Advance", &[("Island", 4)]);
    let army = armies(&t, P0);
    assert_eq!(army.len(), 1);
    let army = army[0];
    let o = t.obj(army);
    assert!(o.is_token());
    assert!(o.chars.has_subtype("Zombie") && o.chars.has_subtype("Army"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
    assert_eq!(t.counters(army, "+1/+1"), 3);
    assert_eq!(t.pt(army), (3, 3));
    // With an Army, no new token: that Army gets the counters.
    cast(&mut t, "Relentless Advance", &[("Island", 4)]);
    assert_eq!(armies(&t, P0), vec![army]);
    assert_eq!(t.pt(army), (6, 6));
}

#[test]
fn the_chosen_army_becomes_the_subtype_in_addition_to_its_other_types() {
    cr!("701.47a");
    ruling!(
        "Relentless Advance",
        "By combining cards with amass Orcs and amass Zombies, you can end up with an Orc Zombie Army."
    );
    ruling!(
        "Relentless Advance",
        "In the rare case that you control multiple Army creatures (perhaps because you played a creature with changeling) while you amass Zombies, you choose which of your Army creatures to put the +1/+1 counters on."
    );
    let mut t = TestGame::new(2);
    // An Orc Army ("Amass Orcs 3, then target player mills X cards").
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast(&mut t, "Surrounded by Orcs", &[("Island", 4)]);
    let orc = armies(&t, P0)[0];
    assert!(t.obj(orc).chars.has_subtype("Orc"));
    cast(&mut t, "Relentless Advance", &[("Island", 4)]);
    let o = t.obj(orc);
    assert!(o.chars.has_subtype("Orc") && o.chars.has_subtype("Zombie"));
    assert_eq!(t.pt(orc), (6, 6));
    // A changeling is an Army too: the player chooses which Army gets the counters.
    let changeling = t.battlefield(P0, "Changeling Outcast");
    choose(&mut t, P0, &[changeling]);
    cast(&mut t, "Relentless Advance", &[("Island", 4)]);
    assert_eq!(t.counters(changeling, "+1/+1"), 3);
    assert_eq!(t.pt(orc), (6, 6));
}

#[test]
fn a_player_amasses_even_if_the_actions_were_impossible() {
    cr!("701.47b");
    supported("Widespread Brutality");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    no_counters(&mut t);
    // "Amass Zombies 2, then the Army you amassed deals damage equal to its power to each
    // non-Army creature." No counters can be put on the new Army.
    cast(
        &mut t,
        "Widespread Brutality",
        &[("Mountain", 3), ("Swamp", 1)],
    );
    let ev = custom_events(&t, AMASSED_EVENT);
    assert_eq!(ev.len(), 1);
    assert_eq!(ev[0].0, Some(P0));
    // The 0/0 Army died; it dealt no damage.
    assert!(armies(&t, P0).is_empty());
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj(bears).damage, 0);
}

#[test]
fn the_amassed_army_is_the_chosen_creature_whether_or_not_it_got_counters() {
    cr!("701.47c");
    ruling!(
        "Relentless Advance",
        "Some cards refer to the \"amassed Army.\" That means the Army creature you chose to receive counters, even if no counters were placed on it for some reason."
    );
    supported("Surrounded by Orcs");
    let mut t = TestGame::new(2);
    // Two Armies: a 5/5 and a 1/1.
    let big = t.battlefield(P0, "Changeling Outcast");
    t.g.add_counters(Entity::Object(big), "+1/+1", 4, None);
    let small = t.battlefield(P0, "Changeling Outcast");
    no_counters(&mut t);
    // "Amass Orcs 3, then target player mills X cards, where X is the amassed Army's
    // power."
    choose(&mut t, P0, &[small]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let lib = t.library_size(P1);
    cast(&mut t, "Surrounded by Orcs", &[("Island", 4)]);
    assert_eq!(t.counters(small, "+1/+1"), 0);
    assert!(t.obj(small).chars.has_subtype("Orc"));
    assert_eq!(t.library_size(P1), lib - 1);
    let _ = big;
}

#[test]
fn older_amass_cards_amass_zombies() {
    cr!("701.47d");
    ruling!(
        "Relentless Advance",
        "Previous cards with amass have received errata to say \"amass Zombies N.\""
    );
    supported("Dreadhorde Invasion");
    let text = card("Dreadhorde Invasion").front().chars.rules_text.to_string();
    assert!(text.contains("amass Zombies 1"), "{text}");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dreadhorde Invasion");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let army = armies(&t, P0);
    assert_eq!(army.len(), 1);
    assert!(t.obj(army[0]).chars.has_subtype("Zombie"));
    assert_eq!(t.pt(army[0]), (1, 1));
    assert_eq!(t.life(P0), 19);
}
