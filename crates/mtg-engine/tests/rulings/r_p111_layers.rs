//! Rulings batch P111 — characteristics (CR 613): base power/toughness setting effects
//! (layer 7b) under modifications (7c) and switches (7d) no matter their timestamps;
//! lose-all-abilities and a static -1/-1 from another source together; a creature
//! targeting itself; "if it's a Citizen" checks the type however it got it; an ability
//! that names its source's name uses last known information (CR 608.2h, 113.7a).

use crate::r_p009_common::kill;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn mana(t: &mut TestGame, p: PlayerId, cost: &str) {
    for c in cost.chars() {
        let ty = match c {
            'W' => ManaType::W,
            'U' => ManaType::U,
            'B' => ManaType::B,
            'R' => ManaType::R,
            'G' => ManaType::G,
            d => {
                let n = d.to_digit(10).expect("a mana symbol");
                t.g.players[p.idx()].mana_pool.add_type(ManaType::C, n);
                continue;
            }
        };
        t.g.players[p.idx()].mana_pool.add_type(ty, 1);
    }
}

fn act(t: &mut TestGame, src: ObjectId, cost: &str, needle: &str, targets: &[ObjectId]) {
    mana(t, P0, cost);
    for x in targets {
        t.answer_targets(P0, &[Entity::Object(*x)]);
    }
    activate_containing(t, P0, src, needle).expect("activation");
    t.resolve_all();
}

fn pump(t: &mut TestGame, name: &str, target: ObjectId) {
    let c = t.hand(P0, name);
    mana(t, P0, "G2");
    t.cast(P0, c).target(target).go();
    t.resolve_all();
}

#[test]
fn toxrill_and_sludge_monster_together() {
    cr!("613.4b", "613.4c", "613.1f", "122.1");
    ruling!(
        "Toxrill, the Corrosive",
        "If you control both Toxrill and Sludge Monster, those non-Horror creatures are 2/2 creatures with no abilities that also get -1/-1 for each slime counter."
    );
    supported("Toxrill, the Corrosive");
    supported("Sludge Monster");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Toxrill, the Corrosive");
    t.battlefield(P0, "Sludge Monster");
    let angel = t.battlefield(P1, "Serra Angel");
    assert!(t.obj_now(angel).has_keyword(KeywordKind::Flying));
    t.g.objects[angel.0 as usize]
        .counters
        .insert("slime".into(), 1);
    t.g.recompute();
    assert_eq!(t.pt(angel), (1, 1));
    assert!(!t.obj_now(angel).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(angel).has_keyword(KeywordKind::Vigilance));
}

#[test]
fn modifications_apply_to_a_new_base_power_and_toughness_whenever_they_began() {
    cr!("613.4b", "613.4c", "613.4d", "613.7");
    ruling!(
        "Warden of the First Tree",
        "Effects that modify Warden of the First Tree's power or toughness, such as the one created by Titanic Growth, will apply to Warden of the First Tree no matter when they started to take effect. The same is true for any +1/+1 counters on Warden of the First Tree and effects that switch its power and toughness."
    );
    supported("Warden of the First Tree");
    supported("Titanic Growth");
    supported("Brute Strength");
    supported("Twisted Image");
    // "{1}{W/B}: This creature becomes a Human Warrior with base power and toughness 3/3."
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Warden of the First Tree");
    t.g.objects[warden.0 as usize]
        .counters
        .insert("+1/+1".into(), 1);
    pump(&mut t, "Titanic Growth", warden);
    // Brute Strength: +3/+1 (and first strike, trample). Twisted Image: "Switch target
    // creature's power and toughness until end of turn."
    let bs = t.hand(P0, "Brute Strength");
    mana(&mut t, P0, "R1");
    t.cast(P0, bs).target(warden).go();
    t.resolve_all();
    let ti = t.hand(P0, "Twisted Image");
    mana(&mut t, P0, "U");
    t.cast(P0, ti).target(warden).go();
    t.resolve_all();
    assert_eq!(t.pt(warden), (7, 9));
    act(&mut t, warden, "1W", "base power", &[]);
    assert!(t.obj_now(warden).chars.has_subtype("Warrior"));
    // Base 3/3, +1/+1 counter, +4/+4, +3/+1: 11/9, switched.
    assert_eq!(t.pt(warden), (9, 11));
}

#[test]
fn switching_is_applied_last() {
    cr!("613.4d", "613.4c");
    ruling!(
        "Phantasmal Fiend",
        "Effects that switch power and toughness are always applied last when determining a creature's power and toughness, even if other power/toughness-changing effects are created later in the turn."
    );
    // "{B}: +1/-1 until end of turn. {1}{U}: Switch this creature's power and toughness
    // until end of turn." 1/5 → 2/4 → 4/2 → 3/3.
    let mut t = TestGame::new(2);
    let fiend = t.battlefield(P0, "Phantasmal Fiend");
    act(&mut t, fiend, "B", "+1/-1", &[]);
    assert_eq!(t.pt(fiend), (2, 4));
    act(&mut t, fiend, "1U", "Switch", &[]);
    assert_eq!(t.pt(fiend), (4, 2));
    act(&mut t, fiend, "B", "+1/-1", &[]);
    assert_eq!(t.pt(fiend), (3, 3));
}

#[test]
fn fleshformer_targeting_itself_or_an_illegal_target() {
    cr!("608.2b", "613.4c", "115.1");
    ruling!(
        "Fleshformer",
        "Fleshformer can target itself with its ability. The result is that Fleshformer gains fear, since the +2/+2 and -2/-2 parts of the effect will balance each other out."
    );
    ruling!(
        "Fleshformer",
        "If that creature becomes an illegal target by the time Fleshformer's ability would resolve, the entire ability doesn't resolve. Fleshformer won't get +2/+2 and won't gain fear."
    );
    supported("Fleshformer");
    // "{W}{U}{B}{R}{G}: This creature gets +2/+2 and gains fear until end of turn. Target
    // creature gets -2/-2 until end of turn. Activate only during your turn."
    let mut t = TestGame::new(2);
    let ff = t.battlefield(P0, "Fleshformer");
    act(&mut t, ff, "WUBRG", "fear", &[ff]);
    assert_eq!(t.pt(ff), (2, 2));
    assert!(t.obj_now(ff).has_keyword(KeywordKind::Fear));
    // Control and the illegal target.
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let ff = t.battlefield(P0, "Fleshformer");
        let giant = t.battlefield(P1, "Hill Giant");
        mana(&mut t, P0, "WUBRG");
        t.answer_targets(P0, &[Entity::Object(giant)]);
        activate_containing(&mut t, P0, ff, "fear").unwrap();
        if gone {
            kill(&mut t, giant);
        }
        t.resolve_all();
        assert_eq!(t.obj_now(ff).has_keyword(KeywordKind::Fear), !gone);
        assert_eq!(t.pt(ff), if gone { (2, 2) } else { (4, 4) });
        if !gone {
            assert_eq!(t.pt(giant), (1, 1));
        }
    }
}

#[test]
fn frodo_checks_its_creature_types_however_it_got_them() {
    cr!("205.3m", "608.2c", "613.1d");
    ruling!(
        "Frodo, Sauron's Bane",
        "Frodo, Sauron's Bane's first ability checks whether it's a Citizen, and its second ability checks whether it's a Scout. It doesn't matter how it became the appropriate creature type."
    );
    supported("Frodo, Sauron's Bane");
    supported("Maskwood Nexus");
    // "{B}{B}{B}: If Frodo is a Scout, it becomes a Halfling Rogue with ..."
    let mut t = TestGame::new(2);
    let frodo = t.battlefield(P0, "Frodo, Sauron's Bane");
    act(&mut t, frodo, "BBB", "Rogue", &[]);
    assert!(!t.obj_now(frodo).chars.has_subtype("Rogue"), "control: not a Scout");
    // Maskwood Nexus: "Creatures you control are every creature type."
    t.battlefield(P0, "Maskwood Nexus");
    act(&mut t, frodo, "BBB", "Rogue", &[]);
    assert!(t.obj_now(frodo).chars.has_subtype("Rogue"));
    assert!(t
        .obj_now(frodo)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("loses the game")));
    // The first ability on a Frodo that's a Scout from its own ability: no longer a
    // Citizen (without the Nexus), nothing happens.
    let mut t = TestGame::new(2);
    let frodo = t.battlefield(P0, "Frodo, Sauron's Bane");
    act(&mut t, frodo, "WW", "Citizen", &[]);
    assert_eq!(t.pt(frodo), (2, 3));
    assert!(t.obj_now(frodo).chars.has_subtype("Scout"));
    assert!(!t.obj_now(frodo).chars.has_subtype("Citizen"));
    act(&mut t, frodo, "BBB", "Rogue", &[]);
    assert!(t.obj_now(frodo).chars.has_subtype("Rogue"));
}

#[test]
fn the_same_name_is_its_last_known_name() {
    cr!("608.2h", "113.7a", "201.2");
    ruling!(
        "Cylian Sunsinger",
        "If Cylian Sunsinger has left the battlefield by the time its ability resolves, its ability will affect each creature whose name is the same as the name Cylian Sunsinger had as it last existed on the battlefield."
    );
    supported("Cylian Sunsinger");
    // "{R}{G}{W}: This creature and each other creature with the same name as it get
    // +3/+3 until end of turn."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Cylian Sunsinger");
    let b = t.battlefield(P0, "Cylian Sunsinger");
    let theirs = t.battlefield(P1, "Cylian Sunsinger");
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, "RGW");
    activate_containing(&mut t, P0, a, "same name").unwrap();
    kill(&mut t, a);
    t.resolve_all();
    assert_eq!(t.pt(b), (5, 5));
    assert_eq!(t.pt(theirs), (5, 5));
    assert_eq!(t.pt(bears), (2, 2));
}
