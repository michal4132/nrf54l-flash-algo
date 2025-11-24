#![no_std]
#![no_main]

use core::{
    mem::MaybeUninit,
    slice,
    sync::atomic::{compiler_fence, Ordering},
};

use flash_algorithm::*;
use nrf_pac::rramc::vals;
use nrf_pac::RRAMC_S as RRAMC;

struct Algorithm;

algorithm!(Algorithm, {
    flash_address: 0x0,
    flash_size: 0x1FD000,
    page_size: 4096,
    empty_value: 0xFF,
    sectors: [{
        size: 4096,
        address: 0x0,
    }]
});

impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        RRAMC.config().write(|_| {});
        Ok(Self)
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        RRAMC.erase().eraseall().write(|w| {
            w.set_erase(vals::Erase::ERASE);
        });
        while !RRAMC.ready().read().ready() {}
        Ok(())
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        Ok(())
    }

    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        // Enable writes
        RRAMC.config().write(|w| {
            w.set_wen(true);
            w.set_writebufsize(vals::Writebufsize::from_bits(32));
        });
        while !RRAMC.ready().read().ready() {}

        compiler_fence(Ordering::SeqCst);
        let dest = unsafe { slice::from_raw_parts_mut(addr as *mut u8, data.len()) };
        dest.copy_from_slice(data);
        compiler_fence(Ordering::SeqCst);

        RRAMC.tasks_commitwritebuf().write_value(1);
        while !RRAMC.ready().read().ready() {}
        RRAMC.config().write(|_| {});
        while !RRAMC.ready().read().ready() {}

        Ok(())
    }
}

impl Drop for Algorithm {
    fn drop(&mut self) {
        RRAMC.config().write(|_| {});
    }
}
