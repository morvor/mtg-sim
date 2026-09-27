//! CR 702.188 Web-slinging (`src/kw/web_slinging.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const WEB: CastMethod = CastMethod::Keyword(KeywordKind::WebSlinging);
/// Spider-Man, Web-Slinger ({2}{W} 3/3): "Web-slinging {W}".
const SPIDEY: &str = "Spider-Man, Web-Slinger";

fn untapped_lands(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is(mtg_engine::types::CardType::Land) && !o.tapped)
        .count()
}

fn tapped(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize].tapped = true;
    id
}

#[test]
fn web_slinging_cards_compile() {
    assert_supported(&[
        SPIDEY,
        "Spiders-Man, Heroic Horde",
        "Amazing Spider-Girl",
        "Spider-Man, Brooklyn Visionary",
        "Silk, Web Weaver",
    ]);
}

#[test]
fn cast_for_the_cost_by_returning_a_tapped_creature() {
    cr!("702.188a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let bears = tapped(&mut t, P0, "Grizzly Bears");
    let c = t.hand(P0, SPIDEY);
    let spell = t.cast(P0, c).method(WEB).go();
    // {W} rather than {2}{W}, and the tapped Bears return to their owner's hand.
    assert_eq!(untapped_lands(&t), 2);
    assert!(!t.on_battlefield(bears));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.g.mana_value_of(spell), 3);
    t.resolve_all();
    assert_eq!(named(&t, SPIDEY).len(), 1);
}

#[test]
fn only_a_tapped_creature_you_control() {
    cr!("702.188a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    t.battlefield(P0, "Grizzly Bears");
    tapped(&mut t, P1, "Grizzly Bears");
    let c = t.hand(P0, SPIDEY);
    assert!(t.cast(P0, c).method(WEB).try_go().is_err());
    assert!(t.in_hand(P0, SPIDEY));
    assert_eq!(untapped_lands(&t), 3);
    // A creature card owned by the opponent returns to its owner's hand.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let stolen = tapped(&mut t, P1, "Hill Giant");
    t.g.objects[stolen.0 as usize].controller = P0;
    t.g.objects[stolen.0 as usize].base_controller = P0;
    t.recompute();
    let c = t.hand(P0, SPIDEY);
    t.cast(P0, c).method(WEB).go();
    assert!(t.in_hand(P1, "Hill Giant"));
}

#[test]
fn timing_rules_apply_and_it_is_an_alternative_cost() {
    cr!("702.188a");
    ruling!(
        "Scarlet Spider, Ben Reilly",
        "You must follow all normal timing rules when casting a spell with web-slinging."
    );
    ruling!(
        "Scarlet Spider, Ben Reilly",
        "you can't also choose to pay its web-slinging cost"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    tapped(&mut t, P0, "Grizzly Bears");
    let c = t.hand(P0, SPIDEY);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.cast(P0, c).method(WEB).try_go().is_err());
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P0, c).method(WEB).try_go().is_err());
    // Cast without paying its mana cost, no creature is returned.
    t.set_step(P0, Step::PrecombatMain);
    run(
        &mut t,
        P0,
        None,
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Target(0),
            free: true,
            optional: false,
        },
        &[Entity::Object(c)],
    );
    assert_eq!(t.stack_len(), 1);
    assert_eq!(named(&t, "Grizzly Bears").len(), 1);
    assert_eq!(untapped_lands(&t), 3);
}

#[test]
fn a_creature_tapped_for_mana_while_casting_can_be_returned() {
    cr!("702.188a");
    ruling!(
        "Scarlet Spider, Ben Reilly",
        "if you tap a creature with a mana ability to pay part of the spell’s web-slinging cost, you could also return that creature to your hand to pay the cost"
    );
    // Avacyn's Pilgrim: "{T}: Add {W}."
    let mut t = TestGame::new(2);
    let pilgrim = t.battlefield(P0, "Avacyn's Pilgrim");
    let c = t.hand(P0, SPIDEY);
    t.cast(P0, c).method(WEB).go();
    assert!(!t.on_battlefield(pilgrim));
    assert!(t.in_hand(P0, "Avacyn's Pilgrim"));
    t.resolve_all();
    assert_eq!(named(&t, SPIDEY).len(), 1);
}

#[test]
fn if_it_was_cast_using_web_slinging() {
    cr!("702.188a");
    // Spiders-Man, Heroic Horde ({1}{G}): "Web-slinging {4}{G}{G}. When Spiders-Man
    // enters, if they were cast using web-slinging, you gain 3 life and create two 2/1
    // green Spider creature tokens with reach."
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    tapped(&mut t, P0, "Grizzly Bears");
    let c = t.hand(P0, "Spiders-Man, Heroic Horde");
    t.cast(P0, c).method(WEB).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(named(&t, "Spider Token").len(), 2);
    // Cast for its mana cost: no life, no tokens.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let c = t.hand(P0, "Spiders-Man, Heroic Horde");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(named(&t, "Spider Token").is_empty());
    assert_eq!(t.zone(c), Zone::Battlefield);
}
