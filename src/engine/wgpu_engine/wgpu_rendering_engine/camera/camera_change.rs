pub struct CameraChange {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
}

impl CameraChange {
    pub fn new() -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
        }
    }

    pub fn amount_left(&self) -> f32 {
        self.amount_left
    }

    pub fn with_amount_left(&mut self, amount_left: f32) {
        self.amount_left = amount_left;
    }

    pub fn amount_right(&self) -> f32 {
        self.amount_right
    }

    pub fn with_amount_right(&mut self, amount_right: f32) {
        self.amount_right = amount_right;
    }

    pub fn amount_forward(&self) -> f32 {
        self.amount_forward
    }

    pub fn with_amount_forward(&mut self, amount_forward: f32) {
        self.amount_forward = amount_forward;
    }

    pub fn amount_backward(&self) -> f32 {
        self.amount_backward
    }

    pub fn with_amount_backward(&mut self, amount_backward: f32) {
        self.amount_backward = amount_backward;
    }

    pub fn amount_up(&self) -> f32 {
        self.amount_up
    }

    pub fn with_amount_up(&mut self, amount_up: f32) {
        self.amount_up = amount_up;
    }

    pub fn amount_down(&self) -> f32 {
        self.amount_down
    }

    pub fn with_amount_down(&mut self, amount_down: f32) {
        self.amount_down = amount_down;
    }

    pub fn rotate_horizontal(&self) -> f32 {
        self.rotate_horizontal
    }

    pub fn with_rotate_horizontal(&mut self, rotate_horizontal: f32) {
        self.rotate_horizontal = rotate_horizontal;
    }

    pub fn rotate_vertical(&self) -> f32 {
        self.rotate_vertical
    }

    pub fn with_rotate_vertical(&mut self, rotate_vertical: f32) {
        self.rotate_vertical = rotate_vertical;
    }
}

impl Default for CameraChange {
    fn default() -> Self {
        Self::new()
    }
}
