//! Launch animation hook trait mirroring upstream `mindustry.world.blocks.LaunchAnimator`.

pub trait LaunchAnimator {
    fn draw_launch(&mut self);

    fn draw_launch_global_z(&mut self) {}

    fn begin_launch(&mut self, launching: bool);

    fn end_launch(&mut self);

    fn update_launch(&mut self);

    fn launch_duration(&self) -> f32;

    fn land_music(&self) -> Option<&'static str> {
        Some("land")
    }

    fn zoom_launch(&self) -> f32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct Animator {
        calls: Vec<&'static str>,
        launching: bool,
    }

    impl LaunchAnimator for Animator {
        fn draw_launch(&mut self) {
            self.calls.push("draw");
        }

        fn begin_launch(&mut self, launching: bool) {
            self.calls.push("begin");
            self.launching = launching;
        }

        fn end_launch(&mut self) {
            self.calls.push("end");
            self.launching = false;
        }

        fn update_launch(&mut self) {
            self.calls.push("update");
        }

        fn launch_duration(&self) -> f32 {
            120.0
        }

        fn zoom_launch(&self) -> f32 {
            4.0
        }
    }

    #[test]
    fn launch_animator_required_hooks_are_callable() {
        let mut animator = Animator::default();

        animator.begin_launch(true);
        animator.draw_launch();
        animator.update_launch();
        animator.end_launch();

        assert_eq!(animator.calls, vec!["begin", "draw", "update", "end"]);
        assert!(!animator.launching);
        assert_eq!(animator.launch_duration(), 120.0);
        assert_eq!(animator.zoom_launch(), 4.0);
    }

    #[test]
    fn default_global_z_and_land_music_match_java_defaults() {
        let mut animator = Animator::default();

        animator.draw_launch_global_z();

        assert!(animator.calls.is_empty());
        assert_eq!(animator.land_music(), Some("land"));
    }
}
