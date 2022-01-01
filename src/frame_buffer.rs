pub struct FrameBuffer([bool; 64 * 32]);

impl FrameBuffer {
    pub fn new() -> FrameBuffer {
        FrameBuffer([false; 64 * 32])
    }

    fn calculate_index_from_2d_cords(x: u8, y: u8, w: u8, h: u8) -> usize
    {
        // "wrap around" cords
        let x = x % w;
        let y = y % h;

        y as usize * w as usize + x as usize
    }

    pub fn get_pixel(&self, x_cord: u8, y_cord: u8) -> bool {
        let pixel_index = FrameBuffer::calculate_index_from_2d_cords(x_cord, y_cord, 64, 32);

        self.0[pixel_index]
    }

    pub fn flip_pixel(&mut self, x_cord: u8, y_cord: u8) {
        let pixel_index = FrameBuffer::calculate_index_from_2d_cords(x_cord, y_cord, 64, 32);

        self.0[pixel_index] = !self.0[pixel_index];
    }

    pub fn clear(&mut self) {
        for pixel in self.0.iter_mut() {
            *pixel = false;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_array_index_calculation() {
        assert_eq!(0, FrameBuffer::calculate_index_from_2d_cords(0, 0, 10, 10));
        assert_eq!(10, FrameBuffer::calculate_index_from_2d_cords(0, 1, 10, 10));
        assert_eq!(57, FrameBuffer::calculate_index_from_2d_cords(7, 5, 10, 10))
    }
}