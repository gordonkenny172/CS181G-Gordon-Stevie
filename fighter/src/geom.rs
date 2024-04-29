#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: u16,
    pub h: u16,
}

pub struct Circle {
    pub x: f32,
    pub y: f32, 
    pub r: f32,
}

impl Rect {
    pub fn overlap(&self, other: Rect) -> Option<Vec2> {
        let x_overlap =
            (self.x + self.w as f32).min(other.x + other.w as f32) - self.x.max(other.x);
        let y_overlap =
            (self.y + self.h as f32).min(other.y + other.h as f32) - self.y.max(other.y);
        if x_overlap >= 0.0 && y_overlap >= 0.0 {
            // This will return the magnitude of overlap in each axis.
            Some(Vec2 {
                x: x_overlap,
                y: y_overlap,
            })
        } else {
            None
        }
    }
    //converts from rectangle coordinates to center coordinates to match position
    pub fn rect_to_pos(&self) -> Vec2 {
        Vec2 {
            x: self.x + (self.w / 2) as f32,
            y: self.y + (self.h / 2) as f32,
        }
    }
    pub fn origin(&self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }
}

impl Circle {
    pub fn overlap(&self, other: Circle) -> Option<Vec2> {
        let distance_sq = f32::abs((self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)); 
        let radius_sq = (self.r + other.r) * (self.r + other.r);

        if distance_sq < radius_sq {
            let midpoint_x = (self.x + other.x) / 2.0;
            let midpoint_y = (self.y + other.y) / 2.0;

            let x_overlap = midpoint_x - self.x;
            let y_overlap = midpoint_y - self.y;

            Some (Vec2 {
                x: x_overlap,
                y: y_overlap
            })
        }
        else {
            None
        }
    }

    pub fn circ_to_pos(&self) -> Vec2 {
        Vec2 {
            x: self.x - self.r,
            y: self.y - self.r
        }
    }

    pub fn origin(&self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y
        }
    }

    pub fn is_empty(&self) -> bool {
        self.r == 0.0
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl Vec2 {
    pub fn mag_sq(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}
