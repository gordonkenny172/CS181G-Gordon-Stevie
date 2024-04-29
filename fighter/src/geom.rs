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

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Shape {
    Circle(Circle),
    Rect(Rect),
}

impl Shape {
    pub fn overlap(&self, other: Shape) -> Option<Vec2> {
        match self {
            Shape::Circle(s_circle) => match other {
                Shape::Circle(o_circle) => s_circle.overlap(o_circle),
                Shape::Rect(o_rect) => {
                    let mut test_x: f32 = s_circle.x;
                    let mut test_y: f32 = s_circle.y;

                    if s_circle.x < o_rect.x {
                        test_x = o_rect.x;
                    } else if s_circle.x > o_rect.x + o_rect.w as f32 {
                        test_x = o_rect.x + o_rect.w as f32;
                    }

                    if s_circle.y < o_rect.y {
                        test_y = o_rect.y;
                    } else if s_circle.y > o_rect.y + o_rect.h as f32 {
                        test_y = o_rect.y + o_rect.h as f32;
                    }

                    let dist_x = s_circle.x - test_x;
                    let dist_y = s_circle.y - test_y;
                    let dist = f32::sqrt((dist_x * dist_x) + (dist_y * dist_y));

                    if dist <= s_circle.r {
                        Some(Vec2 {
                            x: s_circle.r - dist_x,
                            y: s_circle.r - dist_y,
                        })
                    } else {
                        None
                    }
                }
            },
            Shape::Rect(s_rect) => match other {
                // redundant code for now - same as above but flipped
                Shape::Circle(o_circle) => {
                    let mut test_x: f32 = o_circle.x;
                    let mut test_y: f32 = o_circle.y;

                    if o_circle.x < s_rect.x {
                        test_x = s_rect.x;
                    } else if o_circle.x > s_rect.x + s_rect.w as f32 {
                        test_x = s_rect.x + s_rect.w as f32;
                    }

                    if o_circle.y < s_rect.y {
                        test_y = s_rect.y;
                    } else if o_circle.y > s_rect.y + s_rect.h as f32 {
                        test_y = s_rect.y + s_rect.h as f32;
                    }

                    let dist_x = o_circle.x - test_x;
                    let dist_y = o_circle.y - test_y;
                    let dist = f32::sqrt((dist_x * dist_x) + (dist_y * dist_y));

                    if dist <= o_circle.r {
                        Some(Vec2 {
                            x: o_circle.r - dist_x,
                            y: o_circle.r - dist_y,
                        })
                    } else {
                        None
                    }
                },
                Shape::Rect(o_rect) => s_rect.overlap(o_rect),
            },
        }
    }
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
    pub fn new() -> Circle {
        Circle {
            x: 0.0,
            y: 0.0,
            r: 0.0,
        }
    }
    pub fn overlap(&self, other: Circle) -> Option<Vec2> {
        let distance_sq = f32::abs(
            (self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y),
        );
        let radius_sq = (self.r + other.r) * (self.r + other.r);

        if distance_sq < radius_sq {
            let midpoint_x = (self.x + other.x) / 2.0;
            let midpoint_y = (self.y + other.y) / 2.0;

            let x_overlap = midpoint_x - self.x;
            let y_overlap = midpoint_y - self.y;

            Some(Vec2 {
                x: x_overlap,
                y: y_overlap,
            })
        } else {
            None
        }
    }
    pub fn circ_to_pos(&self) -> Vec2 {
        Vec2 {
            x: self.x - self.r,
            y: self.y - self.r,
        }
    }
    pub fn origin(&self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
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
