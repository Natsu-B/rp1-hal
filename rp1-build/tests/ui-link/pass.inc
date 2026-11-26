    pub fn verify() {
        use rp1_proto::note::{Note, SHM_AREA, SHM_REGION};
        use sha2::{Digest, Sha256};

        let note = Note::parse(NOTE).unwrap();
        assert_eq!(AREA, ShmArea::parse(note.get(SHM_AREA).unwrap().value).unwrap());
        let expected = note
            .records()
            .map(Result::unwrap)
            .filter(|record| record.kind == SHM_REGION)
            .map(|record| ShmRegion::parse(record.value).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(REGIONS.as_slice(), expected);
        let digest: [u8; 32] = Sha256::digest(NOTE).into();
        assert_eq!(DIGEST, digest);
        assert_eq!(SRAM_RANGES.len(), expected.len());
        for (&range, region) in SRAM_RANGES.iter().zip(expected) {
            assert_eq!(range, (region.sram_offset, region.sram_len));
        }

        let [tx, rx, block] = REGIONS;
        assert!(tx.overwrite && tx.notify_irq);
        assert!(!rx.overwrite && rx.notify_irq);
        assert_eq!(rx.sram_offset, 0x18c0);
        assert_eq!(block.kind, ShmKind::Block { writer: generated::Writer::Linux });
        for region in [tx, rx, block] {
            assert_eq!(generated::ddr(region) as u32, rp1_proto::SHM_WINDOW + region.ddr_offset);
            assert_eq!(generated::sram(region) as u32, rp1_proto::LINK_START + u32::from(region.sram_offset));
        }
        let _: fn(Proc0Link) -> transport::RingTx<crate::Word, transport::notify::HostIrq> =
            |link| link.r#type;
        let _: fn(Proc1Link) -> transport::RingRx<crate::types::Word, crate::Doorbell1> =
            |link| link.down;
        let _: fn(Proc1Link) -> transport::Block<u32> = |link| link.trace;
    }
}
fn main() {
    link::verify();
}
