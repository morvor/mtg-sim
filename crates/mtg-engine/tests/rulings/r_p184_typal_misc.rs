//! Rulings batch P184 — Elemental, Faerie, Fungus, Giant and Goat typal cards: counts and
//! conditions checked as an ability resolves (CR 608.2h), effects that affect only the
//! objects there as they resolve (CR 611.2c), Twinflame Travelers's extra triggers (CR
//! 603.2d), control-changing Auras (CR 613.1b), casting a spell during an ability's
//! resolution (CR 608.2g), and a mana ability with a sacrifice-X cost (CR 605).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::has_kw;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts `name` onto the battlefield for P0 through a real zone change and puts its
/// triggers on the stack.
fn enter_and_settle(t: &mut TestGame, name: &str) -> ObjectId {
    let id = t.enter(P0, name);
    t.g.flush_events();
    t.settle();
    id
}

#[test]
fn lavakin_brawler_counts_itself_and_fixes_the_bonus_on_resolution() {
    cr!("608.2h", "611.2c", "508.1m");
    ruling!(
        "Lavakin Brawler",
        "Because Lavakin Brawler is itself an Elemental, its ability will usually give it at least +1/+0."
    );
    ruling!(
        "Lavakin Brawler",
        "The size of the bonus is determined as Lavakin Brawler’s ability begins to resolve; it won’t change later in the turn if the number of Elementals you control changes."
    );
    supported("Lavakin Brawler");
    // Alone: +1/+0.
    let mut t = TestGame::new(2);
    let brawler = t.battlefield(P0, "Lavakin Brawler");
    attack_with(&mut t, &[(brawler, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(brawler), (3, 4));
    // With another Elemental: +2/+0, which stays after that Elemental leaves.
    let mut t = TestGame::new(2);
    let brawler = t.battlefield(P0, "Lavakin Brawler");
    let other = t.battlefield(P0, "Fire Elemental");
    attack_with(&mut t, &[(brawler, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(brawler), (4, 4));
    destroy(&mut t, other);
    t.battlefield(P0, "Fire Elemental");
    t.battlefield(P0, "Fire Elemental");
    assert_eq!(t.pt(brawler), (4, 4));
}

#[test]
fn scampering_scorcher_hastes_only_the_elementals_there_as_it_resolves() {
    cr!("611.2c", "302.6", "702.10b");
    ruling!(
        "Scampering Scorcher",
        "Scampering Scorcher’s ability affects only Elementals you control as the ability resolves (after creating the Elemental tokens). This means that those tokens and Scampering Scorcher itself will gain haste. Elementals you begin to control later in the turn won’t gain haste."
    );
    supported("Scampering Scorcher");
    let mut t = TestGame::new(2);
    let scorcher = enter_and_settle(&mut t, "Scampering Scorcher");
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    assert!(has_kw(&t, scorcher, KeywordKind::Haste));
    for tok in toks {
        assert!(has_kw(&t, tok, KeywordKind::Haste));
    }
    let later = t.battlefield_sick(P0, "Fire Elemental");
    assert!(!has_kw(&t, later, KeywordKind::Haste));
}

#[test]
fn twinflame_travelers_doubles_triggers_but_not_replacement_effects() {
    cr!("603.2d", "603.3d", "614.1c");
    ruling!(
        "Twinflame Travelers",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities, such as mobilize (from the Tarkir: Dragonstorm release), are triggered abilities and will have \"when,\" \"whenever,\" or \"at\" in their reminder text. Replacement effects are unaffected by Twinflame Travelers's ability. For example, a 1/1 Elemental creature that enters under your control \"with a +1/+1 counter on it\" won't receive an additional +1/+1 counter."
    );
    ruling!(
        "Twinflame Travelers",
        "Twinflame Travelers's ability doesn't copy the triggered ability; it just causes the ability to trigger an additional time. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    supported("Twinflame Travelers");
    supported("Festercreep");
    supported("Shriekmaw");
    // A replacement effect: Festercreep gets one +1/+1 counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twinflame Travelers");
    let creep = enter_and_settle(&mut t, "Festercreep");
    assert_eq!(t.counters(creep, counters::PLUS1), 1);
    assert_eq!(t.stack_len(), 0);
    // Shriekmaw's enters trigger triggers twice, with a target chosen for each.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twinflame Travelers");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Gray Ogre");
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    enter_and_settle(&mut t, "Shriekmaw");
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn twinflame_travelers_doesnt_affect_as_enters_abilities() {
    cr!("614.12", "603.2d");
    ruling!(
        "Twinflame Travelers",
        "Abilities that apply \"as [this creature] enters\" or \"as [this creature] is turned face up\" are also unaffected by Twinflame Travelers's ability."
    );
    supported("Nyxathid");
    // Nyxathid's "as this creature enters, choose an opponent" is made once.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Twinflame Travelers");
    let from = t.asked().len();
    enter_and_settle(&mut t, "Nyxathid");
    let choices = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
        .count();
    assert_eq!(choices, 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn peppersmoke_checks_for_a_faerie_only_as_it_resolves() {
    cr!("608.2h", "608.2c");
    ruling!(
        "Peppersmoke",
        "Peppersmoke checks if you control a Faerie only when it resolves."
    );
    supported("Peppersmoke");
    for keep in [true, false] {
        let mut t = TestGame::new(2);
        let faerie = t.battlefield(P0, "Pixie Queen");
        let ogre = t.battlefield(P1, "Gray Ogre");
        cast_new(&mut t, P0, "Peppersmoke", &[Entity::Object(ogre)]);
        if !keep {
            destroy(&mut t, faerie);
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.pt(ogre), (1, 1));
        assert_eq!(t.hand_size(P0), hand + keep as usize, "keep {keep}");
    }
    // A Faerie arriving before it resolves counts too.
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Gray Ogre");
    cast_new(&mut t, P0, "Peppersmoke", &[Entity::Object(ogre)]);
    t.battlefield(P0, "Pixie Queen");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn sporesower_thallid_puts_spore_counters_on_every_fungus() {
    cr!("122.1", "503.1a");
    ruling!(
        "Sporesower Thallid",
        "Sporesower Thallid's triggered ability will put a spore counter on each Fungus creature you control, even if that Fungus doesn't have any abilities that makes use of them."
    );
    supported("Sporesower Thallid");
    let mut t = TestGame::new(2);
    let thallid = t.battlefield(P0, "Sporesower Thallid");
    let beast = t.battlefield(P0, "Fungus Beast");
    let theirs = t.battlefield(P1, "Fungus Beast");
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(thallid, "spore"), 1);
    assert_eq!(t.counters(beast, "spore"), 1);
    assert_eq!(t.counters(theirs, "spore"), 0);
}

/// P0 casts Giant's Grasp on P0's Hill Giant, targeting `target` with its trigger;
/// returns the Aura spell.
fn grasp(t: &mut TestGame, target: ObjectId) -> ObjectId {
    let giant = t.named_on_battlefield("Hill Giant")[0];
    let spell = cast_new(t, P0, "Giant's Grasp", &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.resolve();
    spell
}

#[test]
fn giants_grasp_takes_only_the_permanent_and_only_while_it_remains() {
    cr!("613.1b", "303.4e", "611.2b");
    ruling!(
        "Giant's Grasp",
        "Gaining control of a nonland permanent doesn't cause you to gain control of any Auras or Equipment attached to it."
    );
    ruling!(
        "Giant's Grasp",
        "If Giant's Grasp leaves the battlefield before its triggered ability resolves, you won't gain control of the target nonland permanent at all."
    );
    supported("Giant's Grasp");
    supported("Holy Strength");
    // The Aura on the stolen creature stays with its controller.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P1, "Holy Strength", bears);
    assert_eq!(t.pt(bears), (3, 4));
    grasp(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    let s = t.named_on_battlefield("Holy Strength")[0];
    assert_eq!(t.obj_now(s).controller, P1);
    // The Aura leaves before its trigger resolves: no control change.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    grasp(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    let aura = t.named_on_battlefield("Giant's Grasp")[0];
    destroy(&mut t, aura);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn surtland_elementalist_casts_the_spell_while_its_ability_resolves() {
    cr!("608.2g", "601.2", "307.1");
    ruling!(
        "Surtland Elementalist",
        "If you cast a spell using Surtland Elementalist’s triggered ability, you do so as part of the resolution of that ability. You can’t wait to cast the spell later in the turn. Timing permissions based on the card’s type are ignored, and the spell will resolve before blockers are declared."
    );
    supported("Surtland Elementalist");
    let mut t = TestGame::new(2);
    let surt = t.battlefield(P0, "Surtland Elementalist");
    let div = t.hand(P0, "Divination");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(div)]);
    attack_with(&mut t, &[(surt, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    t.resolve();
    // The sorcery is on the stack during the declare attackers step.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.zone(t.g.current(div)), Zone::Stack);
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    t.resolve();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn thundercloud_shaman_counts_giants_as_it_resolves() {
    cr!("608.2h", "603.3");
    ruling!(
        "Thundercloud Shaman",
        "The amount of damage dealt to each non-Giant creature is equal to the number of Giants you control when the ability resolves."
    );
    supported("Thundercloud Shaman");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    enter_and_settle(&mut t, "Thundercloud Shaman");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 1);
}

/// Activates Springjack Pasture's last ability sacrificing `goats`, choosing red.
fn pasture_mana(t: &mut TestGame, pasture: ObjectId, goats: &[ObjectId]) -> Option<ObjectId> {
    let ents: Vec<Entity> = goats.iter().map(|g| Entity::Object(*g)).collect();
    t.answer(
        P0,
        DecisionKind::X,
        mtg_engine::decision::Answer::Number(goats.len() as i64),
    );
    t.answer_choose(P0, &ents);
    crate::r_s06_common::activate_containing(t, P0, pasture, "Sacrifice X Goats").unwrap()
}

#[test]
fn springjack_pasture_sacrifices_any_goats_as_a_mana_ability() {
    cr!("605.1a", "605.3a", "107.3a");
    ruling!(
        "Springjack Pasture",
        "Springjack Pasture's last ability is a mana ability. The whole ability — including the life gain — doesn't use the stack and therefore can't be responded to."
    );
    ruling!(
        "Springjack Pasture",
        "You can sacrifice any Goats you control to activate the third ability, not just Goats created by Springjack Pasture."
    );
    ruling!(
        "Springjack Pasture",
        "If you sacrifice no Goats, you'll produce no mana and you'll gain no life."
    );
    supported("Springjack Pasture");
    let mut t = TestGame::new(2);
    let pasture = t.battlefield(P0, "Springjack Pasture");
    let colos = t.battlefield(P0, "Wild Colos");
    let colos2 = t.battlefield(P0, "Wild Colos");
    let r = pasture_mana(&mut t, pasture, &[colos, colos2]);
    assert!(r.is_none());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.players[0].mana_pool.total(), 2);
    assert_eq!(t.life(P0), 22);
    assert!(!t.on_battlefield(colos));
    // X = 0: no mana, no life.
    let mut t = TestGame::new(2);
    let pasture = t.battlefield(P0, "Springjack Pasture");
    pasture_mana(&mut t, pasture, &[]);
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn surtland_elementalist_reveals_only_a_card_with_the_giant_type() {
    cr!("601.2b", "601.2f", "205.3");
    ruling!(
        "Surtland Elementalist",
        "A Giant card is a creature card with the creature type Giant. Presumably, it will be the same size as all your other cards."
    );
    // With seven lands, it can be cast only by revealing a Giant card instead of paying {2}.
    let castable_with = |other: &str| {
        let mut t = TestGame::new(2);
        t.hand(P0, other);
        let surt = t.hand(P0, "Surtland Elementalist");
        t.lands(P0, "Island", 7);
        let ok = t.cast(P0, surt).try_go().is_ok();
        t.clear_answers();
        ok
    };
    assert!(castable_with("Hill Giant"));
    assert!(!castable_with("Giant Growth"));
}
