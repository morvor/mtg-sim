//! Zone-change replacement effects compiled by the replacement grammar
//! (`src/oracle/patterns/replacement_grammar_zones.rs`): "[object] would die / be put
//! into a graveyard / leave the battlefield / enter and it wasn't cast", with "exile it
//! [with counters] instead", "return it to its owner's hand instead", "instead
//! [instructions]" (CR 614.1a, 614.6).

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn zone_grammar_cards_compile() {
    compiles(&[
        "Malicious Malfunction",
        "Malicious Eclipse",
        "Pulmonic Sliver",
        "Dauthi Voidwalker",
        "Ravenloft Adventurer",
        "Ravenous Slime",
        "Kumano's Blessing",
        "Necromancer's Magemark",
        "The Darkness Crystal",
        "Kalitas, Traitor of Ghet",
        "Containment Priest",
        "Samurai of the Pale Curtain",
        "Void Maw",
        "Darigaaz Reincarnated",
        "Mistcaller",
        "Hallowed Moonlight",
        "Geth, Thane of Contracts",
        "Nexus of Fate",
    ]);
}

#[test]
fn malicious_malfunction_exiles_creatures_that_would_die_this_turn() {
    cr!("614.1a", "614.6");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Swamp", 3);
    let m = t.hand(P0, "Malicious Malfunction");
    t.cast(P0, m).go();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"), "{}", t.dump_log());
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    // Later this turn too.
    t.g.destroy(dreadmaw, None);
    t.settle();
    assert!(t.in_exile("Colossal Dreadmaw"));
    let _ = bears;
}

#[test]
fn dauthi_voidwalker_exiles_opponents_cards_with_a_void_counter() {
    cr!("614.1a", "614.6", "400.3");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dauthi Voidwalker");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    t.g.destroy(theirs, None);
    t.g.destroy(mine, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    let exiled = t
        .g
        .objects
        .iter()
        .find(|o| o.zone == Zone::Exile && o.chars.name == "Grizzly Bears")
        .map(|o| o.id)
        .expect("exiled");
    assert_eq!(t.counters(exiled, "void"), 1);
}

#[test]
fn ravenloft_adventurer_puts_a_hit_counter_on_the_exiled_card() {
    cr!("614.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ravenloft Adventurer");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(theirs, None);
    t.settle();
    let exiled = t
        .g
        .objects
        .iter()
        .find(|o| o.zone == Zone::Exile && o.chars.name == "Grizzly Bears")
        .map(|o| o.id)
        .expect("exiled");
    assert_eq!(t.counters(exiled, "hit"), 1);
}

#[test]
fn kalitas_exiles_nontoken_creatures_and_makes_zombies() {
    cr!("614.1a");
    ruling!(
        "Kalitas, Traitor of Ghet",
        "If Kalitas dies at the same time as creatures your opponents control, those creature cards will be exiled"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kalitas, Traitor of Ghet");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(theirs, None);
    t.settle();
    assert!(t.in_exile("Grizzly Bears"));
    let zombies = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.g.obj(**o).chars.has_subtype("Zombie"))
        .count();
    assert_eq!(zombies, 1);
}

#[test]
fn containment_priest_exiles_creatures_that_werent_cast() {
    cr!("614.1c", "601.1");
    ruling!(
        "Containment Priest",
        "Containment Priest's last ability won't affect any creatures that were cast"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Containment Priest");
    // Put onto the battlefield without being cast: exiled.
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.g.move_object(bears, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0));
    t.settle();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_exile("Grizzly Bears"));
    // Cast: it enters.
    t.lands(P0, "Forest", 2);
    let b2 = t.hand(P0, "Grizzly Bears");
    t.cast(P0, b2).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn darigaaz_is_exiled_with_three_egg_counters_instead_of_dying() {
    cr!("614.1a", "614.6");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Darigaaz Reincarnated");
    t.g.destroy(d, None);
    t.settle();
    let exiled = t
        .g
        .objects
        .iter()
        .find(|o| o.zone == Zone::Exile && o.chars.name == "Darigaaz Reincarnated")
        .map(|o| o.id)
        .expect("exiled");
    assert_eq!(t.counters(exiled, "egg"), 3);
}

#[test]
fn necromancers_magemark_returns_enchanted_creatures_to_hand() {
    cr!("614.1a", "614.6");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mark = t.battlefield(P0, "Necromancer's Magemark");
    t.g.obj_mut(mark).attached_to = Some(Entity::Object(bears));
    let plain = t.battlefield(P0, "Hill Giant");
    t.g.destroy(bears, None);
    t.g.destroy(plain, None);
    t.settle();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn geth_grants_exile_instead_of_leaving_the_battlefield() {
    cr!("614.1a", "614.6");
    let mut t = TestGame::new(2);
    let geth = t.battlefield(P0, "Geth, Thane of Contracts");
    let dead = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    t.activate(P0, geth, 0, &[Entity::Object(dead)]).unwrap();
    t.resolve();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    t.g.destroy(giant, None);
    t.settle();
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn nexus_of_fate_is_shuffled_into_its_library_as_it_resolves() {
    cr!("614.1a", "113.6b");
    ruling!(
        "Nexus of Fate",
        "Nexus of Fate’s last ability applies if it would be put into a graveyard in any way, including while it’s resolving."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    let n = t.hand(P0, "Nexus of Fate");
    let lib = t.library_size(P0);
    t.cast(P0, n).go();
    t.resolve();
    assert!(!t.in_graveyard(P0, "Nexus of Fate"));
    assert_eq!(t.library_size(P0), lib + 1);
}

#[test]
fn obstinate_baloth_enters_when_an_opponent_makes_you_discard_it() {
    cr!("701.9", "614.1a", "113.6");
    compiles(&["Obstinate Baloth", "Loxodon Smiter", "Dodecapod", "Nephalia Academy"]);
    let mut t = TestGame::new(2);
    t.hand(P0, "Obstinate Baloth");
    t.hand(P0, "Dodecapod");
    t.lands(P1, "Swamp", 3);
    let rot = t.hand(P1, "Mind Rot");
    t.g.turn.active = P1;
    t.cast(P1, rot).target(P0).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Obstinate Baloth").len(), 1, "{}", t.dump_log());
    let pod = t.named_on_battlefield("Dodecapod");
    assert_eq!(pod.len(), 1);
    assert_eq!(t.counters(pod[0], "+1/+1"), 2);
}

#[test]
fn obstinate_baloth_goes_to_the_graveyard_when_you_discard_it_yourself() {
    cr!("701.9");
    let mut t = TestGame::new(2);
    let b = t.hand(P0, "Obstinate Baloth");
    t.g.discard(P0, b, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Obstinate Baloth"));
}

#[test]
fn wheel_of_sun_and_moon_puts_cards_on_the_bottom_of_the_library() {
    cr!("614.1a", "614.6", "303.4");
    compiles(&[
        "Wheel of Sun and Moon",
        "Sanctifier en-Vec",
        "Anafenza, the Foremost",
    ]);
    let mut t = TestGame::new(2);
    let wheel = t.battlefield(P0, "Wheel of Sun and Moon");
    t.g.obj_mut(wheel).attached_to = Some(Entity::Player(P1));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lib = t.library_size(P1);
    t.g.destroy(bears, None);
    t.settle();
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.library_size(P1), lib + 1);
    // Not your cards.
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(mine, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn sanctifier_en_vec_exiles_black_and_red_cards_going_to_graveyards() {
    cr!("614.1a", "614.6");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanctifier en-Vec");
    let red = t.battlefield(P1, "Goblin Piker");
    let green = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(red, None);
    t.g.destroy(green, None);
    t.settle();
    assert!(t.in_exile("Goblin Piker"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn anafenza_exiles_opponents_creature_cards_from_anywhere() {
    cr!("614.1a", "614.6");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anafenza, the Foremost");
    let c = t.hand(P1, "Grizzly Bears");
    t.g.discard(P1, c, None);
    t.settle();
    assert!(t.in_exile("Grizzly Bears"));
    let land = t.hand(P1, "Forest");
    t.g.discard(P1, land, None);
    t.settle();
    assert!(t.in_graveyard(P1, "Forest"));
}
