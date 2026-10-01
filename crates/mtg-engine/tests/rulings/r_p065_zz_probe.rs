use mtg_engine::testing::*;
use crate::r_s06_common::activate_containing;
#[test]
fn p065_probe() {
    for (n, needle, land) in [("Butcher of the Horde", "Sacrifice", "Plains"), ("Shifting Ceratops", "{G}", "Forest"), ("Endling", "{1}", "Swamp")] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, n);
        t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, land, 3);
        let from = t.asked().len();
        let r = activate_containing(&mut t, P0, c, needle);
        println!("{n}: {r:?} activation asks {:#?}", &t.asked()[from..]);
        let from = t.asked().len();
        t.resolve_all();
        println!("{n}: resolution asks {:#?} pt {:?}", &t.asked()[from..], t.pt(c));
    }
}
