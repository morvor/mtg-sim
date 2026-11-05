//! Rulings batch P124 — permanents that give other permanents indestructible: damage stays
//! marked on an indestructible creature (CR 120.6), so it's destroyed by state-based
//! actions (CR 704.5g) once it loses indestructible later in the turn (CR 702.12b), and
//! what indestructible doesn't stop.

use crate::r_p124_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::ability::Modification;
use mtg_engine::*;

/// How the protection is taken away.
#[derive(Clone, Copy)]
enum Removal {
    /// The protecting permanent leaves the battlefield.
    LeavesTheBattlefield,
    /// The protecting permanent loses all abilities.
    LosesAbilities,
    /// The protected creature untaps (Adept Watershaper protects tapped creatures).
    VictimUntaps,
}

struct Case {
    lord: &'static str,
    victim: &'static str,
    /// The damage marked on the victim, lethal without indestructible.
    damage: u32,
    removal: Removal,
}

const fn case(lord: &'static str, victim: &'static str, damage: u32, removal: Removal) -> Case {
    Case {
        lord,
        victim,
        damage,
        removal,
    }
}

#[test]
fn lethal_damage_stays_marked_until_the_indestructible_is_lost() {
    cr!("120.6", "702.12b", "704.5g");
    ruling!(
        "Call the Spirit Dragons",
        "nonlethal damage dealt to a Dragon you control may become lethal if Call the Spirit Dragons leaves the battlefield during that turn"
    );
    ruling!(
        "Darksteel Splicer",
        "nonlethal damage dealt to a Golem you control may become lethal if Darksteel Splicer leaves the battlefield during that turn"
    );
    ruling!(
        "Avacyn, Angel of Hope",
        "nonlethal damage dealt to a creature you control may become lethal if Avacyn leaves the battlefield during that turn"
    );
    ruling!(
        "The Walls of Ba Sing Se",
        "nonlethal damage dealt to a creature you control may become lethal if The Walls of Ba Sing Se leave the battlefield during that turn"
    );
    ruling!(
        "General's Enforcer",
        "nonlethal damage dealt to a legendary Human you control may become lethal if General's Enforcer leaves the battlefield during that turn"
    );
    ruling!(
        "Darksteel Forge",
        "nonlethal damage dealt to an artifact creature you control may become lethal if Darksteel Forge leaves the battlefield during that turn"
    );
    ruling!(
        "Adept Watershaper",
        "may become lethal if that creature becomes untapped or if Adept Watershaper leaves the battlefield during that turn"
    );
    ruling!(
        "Sliver Hivelord",
        "If it has lethal damage marked on it and it loses indestructible (perhaps because Sliver Hivelord leaves the battlefield), it will be destroyed."
    );
    ruling!(
        "Crested Sunmare",
        "If another Horse you control has been dealt lethal damage, and later in the turn Crested Sunmare leaves the battlefield, that Horse will be destroyed."
    );
    ruling!(
        "Knight Exemplar",
        "If, at some point later in that turn, you no longer control Knight Exemplar or it loses its abilities, the other Knight creature will lose indestructible and will be destroyed."
    );
    use Removal::*;
    let cases = [
        case("Call the Spirit Dragons", "Shivan Dragon", 5, LeavesTheBattlefield),
        case("Darksteel Splicer", "Stone Golem", 4, LeavesTheBattlefield),
        case("Avacyn, Angel of Hope", "Grizzly Bears", 2, LeavesTheBattlefield),
        case("The Walls of Ba Sing Se", "Grizzly Bears", 2, LeavesTheBattlefield),
        case(
            "General's Enforcer",
            "Thalia, Guardian of Thraben",
            1,
            LeavesTheBattlefield,
        ),
        case("Darksteel Forge", "Ornithopter", 2, LeavesTheBattlefield),
        case("Adept Watershaper", "Grizzly Bears", 2, LeavesTheBattlefield),
        case("Adept Watershaper", "Grizzly Bears", 2, VictimUntaps),
        case("Sliver Hivelord", "Metallic Sliver", 1, LeavesTheBattlefield),
        case("Crested Sunmare", "Armored Warhorse", 3, LeavesTheBattlefield),
        // Knight Exemplar's Knights are 3/3.
        case("Knight Exemplar", "Benalish Knight", 3, LeavesTheBattlefield),
        case("Knight Exemplar", "Benalish Knight", 3, LosesAbilities),
    ];
    for c in cases {
        supported(c.lord);
        supported(c.victim);
        let mut t = TestGame::new(2);
        let lord = t.battlefield(P0, c.lord);
        let victim = t.battlefield(P0, c.victim);
        if matches!(c.lord, "Adept Watershaper") {
            t.g.tap(victim);
        }
        assert!(indestructible(&mut t, victim), "{}", c.lord);
        damage(&mut t, victim, c.damage);
        assert!(t.on_battlefield(victim), "{}: indestructible", c.lord);
        match c.removal {
            LeavesTheBattlefield => exile_now(&mut t, lord),
            LosesAbilities => lose_abilities(&mut t, lord),
            VictimUntaps => {
                t.g.untap(victim);
                t.g.dirty = true;
                t.settle();
            }
        }
        assert!(!t.on_battlefield(victim), "{}: destroyed", c.lord);
    }
}

#[test]
fn svyelun_with_damage_is_destroyed_when_a_merfolk_leaves() {
    cr!("120.6", "702.12b", "704.5g", "611.3a");
    ruling!(
        "Svyelun of Sea and Sky",
        "If you control Svyelun of Sea and Sky with at least 4 damage and two other Merfolk and one of those Merfolk leaves the battlefield"
    );
    let mut t = TestGame::new(2);
    let svyelun = t.battlefield(P0, "Svyelun of Sea and Sky");
    let a = t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.battlefield(P0, "Merfolk of the Pearl Trident");
    damage(&mut t, svyelun, 4);
    assert!(t.on_battlefield(svyelun));
    exile_now(&mut t, a);
    assert!(!t.on_battlefield(svyelun));
}

#[test]
fn kindred_boon_divinity_counters_and_damage() {
    cr!("120.6", "702.12b", "704.5g", "122.1", "602.5");
    ruling!(
        "Kindred Boon",
        "If a creature with a divinity counter on it has been dealt lethal damage, and later in the turn Kindred Boon leaves the battlefield or the counter is removed from that creature, that creature will be destroyed."
    );
    ruling!(
        "Kindred Boon",
        "If a creature that isn't of the chosen type somehow gets a divinity counter on it, that creature will have indestructible."
    );
    ruling!(
        "Kindred Boon",
        "If you somehow control a Kindred Boon with no chosen creature type, its activated ability can't be activated."
    );
    supported("Kindred Boon");
    for remove_counter in [false, true] {
        let mut t = TestGame::new(2);
        choose_creature_type(&mut t, P0, "Bear");
        let boon = t.enter(P0, "Kindred Boon");
        t.settle();
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Plains", 2);
        t.answer_targets(P0, &[obj(bears)]);
        t.activate(P0, boon, 0, &[obj(bears)]).unwrap();
        t.resolve_all();
        assert_eq!(t.counters(bears, "divinity"), 1);
        damage(&mut t, bears, 2);
        assert!(t.on_battlefield(bears));
        if remove_counter {
            let b = t.g.current(bears);
            t.g.remove_counters(Entity::Object(b), "divinity", 1);
            t.g.dirty = true;
            t.settle();
        } else {
            exile_now(&mut t, boon);
        }
        assert!(!t.on_battlefield(bears), "remove counter: {remove_counter}");
    }
    // A Hill Giant (a Giant) with a divinity counter has indestructible too.
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Kindred Boon");
    t.settle();
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(!indestructible(&mut t, giant));
    put_counters(&mut t, giant, "divinity", 1);
    assert!(indestructible(&mut t, giant));
    // A Kindred Boon put onto the battlefield with no type chosen can't be activated.
    let mut t = TestGame::new(2);
    let boon = t.battlefield(P0, "Kindred Boon");
    assert!(t.obj(boon).choices.creature_type.is_none());
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    assert!(!can_activate(&mut t, P0, boon));
    let _ = bears;
}

#[test]
fn king_darien_leaving_takes_away_the_bonus_at_once() {
    cr!("611.3a", "704.5g", "602.2b", "117.5");
    ruling!(
        "King Darien XLVIII",
        "If King Darien XLVIII leaves the battlefield, other creatures you control immediately lose the +1/+1 bonus."
    );
    supported("King Darien XLVIII");
    // A Soldier token (2/2 with Darien) with 1 damage: sacrificing Darien for its last
    // ability makes it a 1/1 with lethal damage before the ability resolves.
    let mut t = TestGame::new(2);
    let darien = t.battlefield(P0, "King Darien XLVIII");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let token = create_token(&mut t, P0, "Soldier");
    t.g.recompute();
    assert_eq!(t.pt(token), (2, 2));
    assert_eq!(t.pt(bears), (3, 3));
    damage(&mut t, token, 1);
    damage(&mut t, bears, 2);
    t.activate(P0, darien, 1, &[]).unwrap();
    // Before the ability resolves, both have lethal damage.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(!t.on_battlefield(token));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn knight_exemplar_destroyed_with_another_knight_and_two_exemplars() {
    cr!("608.2b", "702.12b", "613.1f", "611.3a");
    ruling!(
        "Knight Exemplar",
        "If an effect would simultaneously destroy Knight Exemplar and another Knight creature you control, only Knight Exemplar is destroyed."
    );
    ruling!(
        "Knight Exemplar",
        "If you control two Knight Exemplars, each one causes the other to get +1/+1 and have indestructible."
    );
    // Wrath of God: "Destroy all creatures. They can't be regenerated."
    let mut t = TestGame::new(2);
    let exemplar = t.battlefield(P0, "Knight Exemplar");
    let knight = t.battlefield(P0, "Benalish Knight");
    cast_resolve(&mut t, P0, "Wrath of God", &[]);
    assert!(!t.on_battlefield(exemplar));
    assert!(t.on_battlefield(knight));
    // Two Exemplars: each is a 3/3 with indestructible; Wrath of God destroys neither.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Knight Exemplar");
    let b = t.battlefield(P0, "Knight Exemplar");
    t.g.recompute();
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(b), (3, 3));
    assert!(indestructible(&mut t, a) && indestructible(&mut t, b));
    cast_resolve(&mut t, P0, "Wrath of God", &[]);
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
}

#[test]
fn indestructible_doesnt_stop_sacrifice_legend_rule_or_zero_toughness() {
    cr!("702.12b", "704.5f", "704.5j", "701.21a");
    ruling!(
        "Knight Exemplar",
        "However, a creature with indestructible can be put into the graveyard for a number of reasons. The most likely reasons are if it's sacrificed, if it's legendary and another legendary creature with the same name is controlled by the same player, or if its toughness is 0 or less."
    );
    ruling!(
        "Terra Eternal",
        "If that land is also a creature, lethal damage won’t cause it to be put into its owner’s graveyard either."
    );
    // Knights: deathtouch damage and "destroy" do nothing; sacrifice, 0 toughness and the
    // legend rule still work.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Knight Exemplar");
    let a = t.battlefield(P0, "Benalish Knight");
    let b = t.battlefield(P0, "Benalish Knight");
    let c = t.battlefield(P0, "Benalish Knight");
    let viper = t.battlefield(P1, "Typhoid Rats");
    let rats = t.g.current(viper);
    t.g.deal_damage(rats, Entity::Object(a), 1, false);
    t.g.flush_events();
    t.settle();
    destroy(&mut t, a);
    assert!(t.on_battlefield(a));
    sacrifice(&mut t, b);
    assert!(!t.on_battlefield(b));
    modify_until_eot(
        &mut t,
        c,
        vec![Modification::ModifyPT(
            mtg_engine::ability::Value::Const(0),
            mtg_engine::ability::Value::Const(-3),
        )],
    );
    assert!(!t.on_battlefield(c));
    // Terra Eternal ("All lands have indestructible."): destroy and lethal damage to a land
    // creature do nothing; the legend rule and sacrifice still apply.
    supported("Terra Eternal");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Terra Eternal");
    let forest = t.battlefield(P0, "Forest");
    destroy(&mut t, forest);
    assert!(t.on_battlefield(forest));
    let dryad = t.battlefield(P0, "Dryad Arbor");
    damage(&mut t, dryad, 5);
    assert!(t.on_battlefield(dryad));
    let a = t.battlefield(P0, "Okina, Temple to the Grandfathers");
    let b = t.battlefield(P0, "Okina, Temple to the Grandfathers");
    t.answer_choose(P0, &[obj(b)]);
    t.g.dirty = true;
    t.settle();
    assert_eq!(
        [a, b].iter().filter(|x| t.on_battlefield(**x)).count(),
        1,
        "legend rule"
    );
    sacrifice(&mut t, forest);
    assert!(!t.on_battlefield(forest));
}

#[test]
fn indestructible_lords_edge_cases() {
    cr!("702.12b", "611.3a", "613.1f");
    ruling!(
        "Timber Protector",
        "If Timber Protector somehow becomes a Forest, it doesn’t give itself indestructible."
    );
    ruling!(
        "Greater Auramancy",
        "If you have two on the battlefield, they’ll each grant the other one shroud."
    );
    ruling!(
        "Dawnglade Regent",
        "Dawnglade Regent's second ability applies to itself while you're the monarch."
    );
    ruling!(
        "General's Enforcer",
        "Only creatures are Humans. Planeswalkers such as Lukka and Narset who represent human characters aren't affected by General's Enforcer."
    );
    // Timber Protector made a Forest (Forest-type change) still lacks indestructible; its
    // Forests have it.
    let mut t = TestGame::new(2);
    let protector = t.battlefield(P0, "Timber Protector");
    let forest = t.battlefield(P0, "Forest");
    assert!(indestructible(&mut t, forest));
    modify_until_eot(
        &mut t,
        protector,
        vec![
            Modification::AddTypes(vec![mtg_engine::types::CardType::Land]),
            Modification::AddSubtypes(vec!["Forest".into()]),
        ],
    );
    assert!(t.obj_now(protector).chars.has_subtype("Forest"));
    assert!(!indestructible(&mut t, protector));
    // Two Greater Auramancies give each other shroud.
    supported("Greater Auramancy");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Greater Auramancy");
    let b = t.battlefield(P0, "Greater Auramancy");
    assert!(has_kw(&mut t, a, KeywordKind::Shroud));
    assert!(has_kw(&mut t, b, KeywordKind::Shroud));
    // Dawnglade Regent: hexproof for itself while P0 is the monarch.
    supported("Dawnglade Regent");
    let mut t = TestGame::new(2);
    let regent = t.enter(P0, "Dawnglade Regent");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    assert!(has_kw(&mut t, regent, KeywordKind::Hexproof));
    // General's Enforcer: legendary Human creatures only.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "General's Enforcer");
    let thalia = t.battlefield(P0, "Thalia, Guardian of Thraben");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let enforcer = t.named_on_battlefield("General's Enforcer")[0];
    assert!(indestructible(&mut t, thalia));
    assert!(!indestructible(&mut t, bears));
    // The Enforcer is a nonlegendary Human: not itself.
    assert!(!indestructible(&mut t, enforcer));
    // A legendary planeswalker isn't a Human.
    let narset = t.battlefield(P0, "Narset, Parter of Veils");
    assert!(!indestructible(&mut t, narset));
}
