//! Bullet movement callback mirroring upstream `mindustry.entities.Mover`.

pub trait Mover<Bullet> {
    fn move_bullet(&mut self, bullet: &mut Bullet);
}

impl<Bullet, F> Mover<Bullet> for F
where
    F: FnMut(&mut Bullet),
{
    fn move_bullet(&mut self, bullet: &mut Bullet) {
        self(bullet);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, PartialEq)]
    struct Bullet {
        x: f32,
        y: f32,
    }

    #[test]
    fn mover_callback_can_mutate_bullet_position() {
        let mut bullet = Bullet::default();
        let mut mover = |bullet: &mut Bullet| {
            bullet.x += 3.0;
            bullet.y -= 2.0;
        };

        mover.move_bullet(&mut bullet);

        assert_eq!(bullet, Bullet { x: 3.0, y: -2.0 });
    }
}
