//! Offers convenient kernel object abstractions, that create the necessary `create_*`-syscall
//! in the constructor by themselves and destroy the object when they are dropped.
//!
//! PD owns SM and EC objects. Global EC objects own their corresponding SC and local EC
//! objects own their corresponding PTs.

mod ec;
mod pd;
mod pt;
mod sc;
mod sm;

pub use ec::*;
pub use pd::*;
pub use pt::*;
pub use sc::*;
pub use sm::*;

#[cfg(test)]
mod tests {
    use crate::kobjects::{
        GlobalEcObject,
        LocalEcObject,
        PdObject,
        PtCtx,
        PtObject,
        ScObject,
    };
    use crate::process::consts::ROOTTASK_PROCESS_PID;
    use crate::service_ids::ServiceId;
    use libhedron::mtd::Mtd;

    #[test]
    fn test_pd_1() {
        let pd_sel = 0;
        let gl_ec_sel = 1;
        let local_ec_1_sel = 2;
        let local_ec_2_sel = 3;
        let pt_1_sel = 4;
        let sc_sel = 6;
        let pd_2_sel = 7;

        let pd = PdObject::new(ROOTTASK_PROCESS_PID, None, pd_sel);

        assert!(pd.global_ec().is_none());

        let gl_ec = GlobalEcObject::new(gl_ec_sel, &pd, 0xdeadbeef000, 0x1238);

        assert!(pd.global_ec().is_some());
        assert_eq!(
            pd.global_ec().as_ref().unwrap().pd().pid(),
            ROOTTASK_PROCESS_PID
        );

        let sc = ScObject::new(sc_sel, &gl_ec, None);
        assert_eq!(sc.gl_ec().ec_sel(), gl_ec_sel);

        // check if sc is correct with POV from PD
        assert_eq!(
            pd.global_ec()
                .as_ref()
                .unwrap()
                .sc()
                .as_ref()
                .unwrap()
                .cap_sel(),
            sc_sel
        );

        // now create local ec
        assert!(pd.local_ecs().is_empty());
        let local_ec_1 = LocalEcObject::new(local_ec_1_sel, &pd, 0xbadf00d, 0x1337000);
        assert_eq!(local_ec_1.pd().cap_sel(), pd_sel);

        assert_eq!(local_ec_1.portals().len(), 0);
        let pt1 = PtObject::new(
            pt_1_sel,
            &local_ec_1,
            Mtd::all(),
            0,
            PtCtx::Service(ServiceId::StdoutService),
        );
        assert_eq!(local_ec_1.portals().len(), 1);
        assert_eq!(pt1.local_ec().pd().cap_sel(), pd_sel);

        {
            // now attach 2 portals to the local EC
            let pd2 = PdObject::new(1, Some(&pd), pd_2_sel);
            assert_eq!(pd2.parent().unwrap().cap_sel(), pd_sel);
            let local_ec_2 = LocalEcObject::new(local_ec_2_sel, &pd2, 0xabcdef, 0x1000);
            assert_eq!(local_ec_2.pd().cap_sel(), pd_2_sel);
        }

        dbg!(pd);
    }

    /// Test creates PD0 and PD1. Creates PT0 in PD0 and delegates
    /// it to PD1, while PD0 still owns it.
    #[test]
    fn test_pd_pt_delegation() {
        let pd_0_sel = 0;
        let pd_1_sel = 1;
        let pt_0_sel = 2;
        let lec_0_sel = 3;

        let pd0 = PdObject::new(ROOTTASK_PROCESS_PID, None, pd_0_sel);
        let lec0 = LocalEcObject::new(lec_0_sel, &pd0, 0xd000, 0xf000);
        let pd1 = PdObject::new(1, None, pd_1_sel);

        let pt0 = PtObject::new(
            pt_0_sel,
            &lec0,
            Mtd::DEFAULT,
            1337,
            PtCtx::Service(ServiceId::StdoutService),
        );

        pd1.attach_delegated_pt(pt0.clone());
        pt0.attach_delegated_to_pd(&pd1);

        assert_eq!(pt0.delegated_to_pd().unwrap().pid(), 1);
        assert_eq!(pd1.delegated_pts().iter().next().unwrap().portal_id(), 1337);
    }
}
