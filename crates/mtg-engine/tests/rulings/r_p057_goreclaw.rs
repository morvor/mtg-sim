//! Rulings batch P057 — Goreclaw, Terror of Qal Sisma: "Creature spells you cast with
//! power 4 or greater cost {2} less to cast." looks at the spell's power on the stack (CR
//! 601.2f, 208.3), and "Whenever Goreclaw attacks, each creature you control with power 4
//! or greater gets +1/+1 and gains trample until end of turn." locks in the affected set
//! as it resolves (CR 611.2c); its controller orders simultaneous attack triggers (CR
//! 603.3b).

use crate::r_p057_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::has_kw;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

const GORECLAW: &str = "Goreclaw, Terror of Qal Sisma";
const BIG: &str = "Rumbling Baloth"; // {2}{G}{G} vanilla 4/4

fn at_p1(id: ObjectId) -> (ObjectId, Entity) {
    (id, Entity::Player(P1))
}

#[test]
fn goreclaw_reduces_by_the_spells_own_power_not_entering_counters_or_later_effects() {
    cr!("601.2f", "208.3", "614.1c");
    ruling!(
        "Goreclaw, Terror of Qal Sisma",
        "If you cast a creature spell that will enter the battlefield with a number of +1/+1 counters, such as Hungering Hydra, those counters aren't considered when determining whether Goreclaw reduces that spell's cost."
    );
    supported(GORECLAW);
    supported("Hungering Hydra");
    supported("Glorious Anthem");
    // A 4-power creature spell: {2}{G}{G} costs {G}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, GORECLAW);
    t.lands(P0, "Forest", 2);
    let baloth = t.hand(P0, BIG);
    t.cast(P0, baloth).try_go().expect("{G}{G} is enough");
    // Hungering Hydra ({X}{G}, 0/0, enters with X +1/+1 counters) with X = 4: no
    // reduction, {4}{G} is needed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, GORECLAW);
    t.lands(P0, "Forest", 3);
    let hydra = t.hand(P0, "Hungering Hydra");
    assert!(t.cast(P0, hydra).x(4).try_go().is_err());
    t.lands(P0, "Forest", 2);
    t.cast(P0, hydra).x(4).try_go().expect("{4}{G}");
    // Grizzly Bears with two Glorious Anthems would be a 4/4 on the battlefield, but the
    // spell is a 2/2: {1}{G} isn't reduced.
    let mut t = TestGame::new(2);
    t.battlefield(P0, GORECLAW);
    t.battlefield(P0, "Glorious Anthem");
    t.battlefield(P0, "Glorious Anthem");
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    t.lands(P0, "Forest", 1);
    let spell = t.cast(P0, bears).try_go().expect("{1}{G}");
    t.resolve_all();
    assert_eq!(t.pt(t.g.current(spell)), (4, 4));
}

#[test]
fn goreclaw_attack_trigger_locks_in_the_affected_creatures() {
    cr!("611.2c", "603.2");
    ruling!(
        "Goreclaw, Terror of Qal Sisma",
        "Goreclaw's last ability affects only creatures you control with the appropriate power at the time it resolves."
    );
    supported(GORECLAW);
    let mut t = TestGame::new(2);
    let gc = t.battlefield(P0, GORECLAW); // 4/3
    let baloth = t.battlefield(P0, BIG);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[at_p1(gc), at_p1(bears)]);
    assert_eq!(t.stack_len(), 1);
    // Raised before it resolves: the Bears count.
    pump(&mut t, bears, 2, 0);
    t.resolve_all();
    assert_eq!(t.pt(gc), (5, 4));
    assert!(has_kw(&t, gc, KeywordKind::Trample));
    assert_eq!(t.pt(baloth), (5, 5), "non-attacking creatures too");
    assert!(has_kw(&t, baloth, KeywordKind::Trample));
    assert_eq!(t.pt(bears), (5, 3));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
    // A creature whose power drops afterward keeps both bonuses.
    pump(&mut t, baloth, -3, 0);
    assert_eq!(t.pt(baloth), (2, 5));
    assert!(has_kw(&t, baloth, KeywordKind::Trample));
    // A creature that gets power 4 later, or enters later, gets neither.
    let other = t.battlefield(P0, "Hill Giant");
    pump(&mut t, other, 1, 0);
    let late = t.battlefield(P0, BIG);
    assert_eq!(t.pt(other), (4, 3));
    assert!(!has_kw(&t, other, KeywordKind::Trample));
    assert_eq!(t.pt(late), (4, 4));
    assert!(!has_kw(&t, late, KeywordKind::Trample));
}

#[test]
fn goreclaw_brawl_bash_ogre_trigger_can_resolve_first() {
    cr!("603.3b", "611.2c");
    ruling!(
        "Goreclaw, Terror of Qal Sisma",
        "If another creature has an ability that changes its power when it attacks, such as Brawl-Bash Ogre, you may have that ability resolve before Goreclaw's last ability."
    );
    supported(GORECLAW);
    supported("Brawl-Bash Ogre");
    // Brawl-Bash Ogre (3/3): "Whenever this creature attacks, you may sacrifice another
    // creature. If you do, this creature gets +2/+2 until end of turn."
    let mut outcomes = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let gc = t.battlefield(P0, GORECLAW);
        let ogre = t.battlefield(P0, "Brawl-Bash Ogre");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(bears)]);
        attack_with(&mut t, &[at_p1(gc), at_p1(ogre)]);
        assert_eq!(t.stack_len(), 2);
        t.resolve_all();
        assert!(!t.on_battlefield(bears), "sacrificed");
        outcomes.push((t.pt(ogre), has_kw(&t, ogre, KeywordKind::Trample)));
    }
    outcomes.sort();
    // Goreclaw's first: the Ogre is 3/3 then, so only +2/+2. Ogre's first: 5/5, then
    // +1/+1 and trample.
    assert_eq!(outcomes, vec![((5, 5), false), ((6, 6), true)]);
}
