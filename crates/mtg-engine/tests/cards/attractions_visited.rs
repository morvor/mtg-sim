//! Attractions (CR 701.51, 701.52, 717) compiled with
//! `oracle/patterns/un_stickers_attractions.rs`: "open two Attractions", Attractions
//! visited this turn, "whenever you visit an Attraction", "whenever you roll to visit your
//! Attractions, if you roll a 6".

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// Puts Attraction cards into `p`'s Attraction deck (face down in the command zone).
fn attraction_deck_of(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    let ids = names
        .iter()
        .map(|n| {
            let id = t.command(p, n);
            t.g.objects[id.0 as usize].face_down = true;
            id
        })
        .collect();
    t.g.recompute();
    ids
}

/// `p` rolls to visit their Attractions with the given die result.
fn roll_to_visit(t: &mut TestGame, p: PlayerId, result: u32) {
    t.g.dice.loaded.push_back(result);
    mtg_engine::variants::roll_to_visit(&mut t.g, p);
    t.g.flush_events();
    t.settle();
}

#[test]
fn open_two_attractions() {
    cr!("701.51a");
    compiles("Step Right Up");
    let mut t = TestGame::new(2);
    let deck = attraction_deck_of(&mut t, P0, &["Information Booth", "Kiddie Coaster", "Balloon Stand"]);
    t.lands(P0, "Swamp", 4);
    t.set_step(P0, Step::PrecombatMain);
    let spell = t.hand(P0, "Step Right Up");
    t.cast(P0, spell).go();
    t.resolve_all();
    let open = deck.iter().filter(|id| t.on_battlefield(**id)).count();
    assert_eq!(open, 2);
}

#[test]
fn as_long_as_you_have_visited_an_attraction_this_turn() {
    cr!("701.52a");
    compiles("Soul Swindler");
    // "As long as you've visited an Attraction this turn, this creature has
    // indestructible."
    let mut t = TestGame::new(2);
    let swindler = t.battlefield(P0, "Soul Swindler");
    t.battlefield(P0, "Information Booth");
    t.set_step(P0, Step::PrecombatMain);
    // Rolling to visit without a lit-up result visits nothing.
    roll_to_visit(&mut t, P0, 4);
    t.resolve_all();
    t.g.recompute();
    assert!(!t
        .obj_now(swindler)
        .has_keyword(mtg_engine::keywords::KeywordKind::Indestructible));
    roll_to_visit(&mut t, P0, 6);
    t.resolve_all();
    t.g.recompute();
    assert!(t
        .obj_now(swindler)
        .has_keyword(mtg_engine::keywords::KeywordKind::Indestructible));
}

#[test]
fn the_number_of_attractions_you_have_visited_this_turn() {
    cr!("701.52a");
    compiles("Black Hole");
    compiles("Squirrel Squatters");
    // "Whenever this creature attacks, create a 1/1 green Squirrel creature token that's
    // tapped and attacking for each Attraction you've visited this turn."
    let mut t = TestGame::new(2);
    let squatters = t.battlefield(P0, "Squirrel Squatters");
    t.battlefield(P0, "Information Booth");
    t.battlefield(P0, "Kiddie Coaster");
    t.set_step(P0, Step::PrecombatMain);
    // A 2 visits both (each visit counts); a 3 visits only Kiddie Coaster.
    roll_to_visit(&mut t, P0, 2);
    t.resolve_all();
    roll_to_visit(&mut t, P0, 3);
    t.resolve_all();
    // An opponent's visits aren't yours.
    t.battlefield(P1, "Balloon Stand");
    roll_to_visit(&mut t, P1, 6);
    t.resolve_all();
    t.attack(&[(squatters, Entity::Player(P1))], &[]);
    let squirrels = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Squirrel"))
        .count();
    assert_eq!(squirrels, 3);
}

#[test]
fn whenever_you_visit_an_attraction() {
    cr!("701.52a");
    compiles("Motion Sickness");
    // "Whenever you visit an Attraction, you may attach this Aura to target tapped
    // creature."
    let mut t = TestGame::new(2);
    let first = t.battlefield(P1, "Grizzly Bears");
    let second = t.battlefield(P1, "Hill Giant");
    t.g.objects[second.0 as usize].tapped = true;
    let aura = t.battlefield(P0, "Motion Sickness");
    t.g.attach(aura, Entity::Object(first));
    t.battlefield(P0, "Information Booth");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(second)]);
    t.answer_yes(P0, true);
    roll_to_visit(&mut t, P0, 6);
    t.resolve_all();
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(second)));
}

#[test]
fn whenever_you_roll_to_visit_if_you_roll_a_6() {
    cr!("701.52a");
    compiles("\"Lifetime\" Pass Holder");
    // "Whenever you roll to visit your Attractions, if you roll a 6, you may return this
    // card from your graveyard to the battlefield."
    let mut t = TestGame::new(2);
    let holder = t.graveyard(P0, "\"Lifetime\" Pass Holder");
    t.answer_yes(P0, true);
    roll_to_visit(&mut t, P0, 5);
    t.resolve_all();
    assert_eq!(t.zone(holder), Zone::Graveyard(P0));
    roll_to_visit(&mut t, P0, 6);
    t.resolve_all();
    assert!(t.on_battlefield(holder));
}
