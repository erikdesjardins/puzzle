use std::mem::transmute;

pub trait Ext {
    fn as_bytes(&self) -> &[u8; 16];
    fn as_mut_bytes(&mut self) -> &mut [u8; 16];
    fn all_bytes_nonzero(self) -> bool;
    fn swap_nibbles(self) -> Self;
    fn swap(self, i: usize, j: usize) -> Self;
}

impl Ext for u128 {
    fn as_bytes(&self) -> &[u8; 16] {
        unsafe { transmute(self) }
    }

    fn as_mut_bytes(&mut self) -> &mut [u8; 16] {
        unsafe { transmute(self) }
    }

    fn all_bytes_nonzero(self) -> bool {
        let discriminant = (self - 0x01010101010101010101010101010101) & !self & 0x80808080808080808080808080808080;
        discriminant == 0
    }

    fn swap_nibbles(self) -> Self {
        let high_to_low = (self >> 4) & 0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f;
        let low_to_high = (self << 4) & 0xf0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0;
        high_to_low | low_to_high
    }

    // doesn't touch memory, but many more instructions (slightly slower in practice)
    #[cfg(any())]
    fn swap(self, i: usize, j: usize) -> Self {
        let i_byte = (self >> (i * 8)) & 0xff;
        let j_byte = (self >> (j * 8)) & 0xff;

        let j_to_i = (self   & !(0xff << (i * 8))) | (j_byte << (i * 8));
        let i_to_j = (j_to_i & !(0xff << (j * 8))) | (i_byte << (j * 8));

        i_to_j
    }

    // uses memory, but much fewer instructions (slightly faster in practice)
    #[cfg(all())]
    fn swap(mut self, i: usize, j: usize) -> Self {
        let i_byte = self.as_bytes()[i];
        let j_byte = self.as_bytes()[j];
        {
            let this = self.as_mut_bytes();
            this[i] = j_byte;
            this[j] = i_byte;
        }
        self
    }
}
