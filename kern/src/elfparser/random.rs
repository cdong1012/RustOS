use rand_core::Error;
use rand_core::impls::fill_bytes_via_next;
use rand_core::RngCore;
use pi::timer::current_time;
#[derive(Default)]
pub struct PeterRand;
impl RngCore for PeterRand {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        let a = 1299709 as u64;
        let c = 12345 as u64;
        let m = ((2 >> 16) - 1) as u64;
        
        a.wrapping_mul(current_time().as_millis() as u64).wrapping_add(c).wrapping_rem(m)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}